use clap::CommandFactory;
use clap::Parser;
use std::env;
use std::os::windows::process::CommandExt;
use std::process::Command as StdCommand;
use tracing::info;
use ymb_args::Args;
use ymb_args::Command;
use ymb_console::is_inheriting_console;
use ymb_lifecycle::GLOBAL_ARGS;
use ymb_logs::DualLogWriter;
use ymb_logs::setup_tracing;
use ymb_windy::WindyResult;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() -> WindyResult<()> {
    eprintln!("Ahoy from fn main!");
    // --- Early argument parsing to check for --tray-mode-active and console state ---
    let preliminary_args = Args::try_parse();
    let (_parsed_args_ok, is_tray_mode_arg, is_inheriting_console_arg) = match &preliminary_args {
        Ok(args) => (true, args.tray_mode_active, is_inheriting_console()),
        Err(_) => (false, false, is_inheriting_console()),
    };

    // Re-launch logic:
    // If NOT launched with --tray-mode-active AND NOT inheriting an existing console (e.g., double-clicked)
    if !is_tray_mode_arg && !is_inheriting_console_arg {
        eprintln!(
            "Initial launch without inherited console detected. Re-launching in detached tray mode..."
        );
        let current_exe = env::current_exe().expect("Failed to get current executable path.");
        // Collect all original arguments EXCEPT the executable itself
        let mut cmd_args: Vec<String> = env::args_os()
            .skip(1)
            .map(|s| s.into_string().unwrap())
            .collect();
        cmd_args.push("--tray-mode-active".to_string());
        StdCommand::new(current_exe)
            .args(&cmd_args)
            .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
            .spawn()
            .expect("Failed to re-launch application in detached mode.");
        return Ok(()); // Initial process exits
    }

    // --- Proceed with normal execution (either inherited console or we are the detached tray instance) ---
    let args = match preliminary_args {
        Ok(args) => args,
        Err(e) => {
            // If preliminary parsing failed but we didn't re-launch (e.g., ran from terminal with invalid args)
            // then try a full parse to show clap's error message.
            // This error will go to the console if `is_inheriting_console_arg` was true.
            eprintln!("Argument parsing error: {}", e); // Show clap's error
            Args::command().get_matches(); // This will print help and exit via clap
            return Err(e.into()); // Or convert clap::Error to WindyResult
        }
    };
    GLOBAL_ARGS.set(args.global.clone()).unwrap();

    let mut ran_from_inherited_console_successfully = false;
    if !args.tray_mode_active && is_inheriting_console_arg {
        // We are NOT the re-launched tray_mode instance, AND we initially detected an inherited console.
        // Try to properly attach to it.
        if ymb_console::maybe_attach_or_hide_console() {
            ran_from_inherited_console_successfully = true;
        } else {
            eprintln!(
                "[main] Warning: Detected inherited console, but maybe_attach_or_hide_console returned false after attempt."
            );
        }
    }
    // If args.tray_mode_active is true, ran_from_inherited_console_successfully remains false.
    // This means the re-launched instance does not attempt to attach to any console.

    // Install panic hook and color_eyre *after* console decisions.
    if let Err(e) = color_eyre::install() {
        eprintln!("Failed to install color_eyre: {:?}", e);
    }

    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info_arg| {
        let msg = format!(
            "Panic encountered at {}",
            panic_info_arg
                .location()
                .map(|x| x.to_string())
                .unwrap_or("unknown location".to_string())
        );
        eprintln!("{}", msg); // Fallback for very early panics in detached mode
        tracing::error!("{}", msg); // If tracing is up and has a sink
        panic_hook(panic_info_arg);
    }));

    // --- Logging Setup ---
    // The DualWriter will write to stderr (if connected) and an internal buffer.
    // If ran_from_inherited_console_successfully = true, stderr is the terminal.
    // If false (detached tray mode), stderr is likely disconnected initially.
    let log_writer = DualLogWriter::new();

    // Pass the writer to setup_tracing.
    setup_tracing(&args.global, log_writer.clone())?;

    info!(
        "Application starting. Inherited console: {}, Tray mode active: {}",
        ran_from_inherited_console_successfully, args.tray_mode_active
    );

    match args.command {
        None | Some(Command::Tray) => {
            info!("Starting tray icon process...");
            ymb_tray::main(args.global, log_writer)?; // Pass the cloned buffer
        }
        Some(Command::WelcomeGui) => {
            info!("Starting GUI process...");
            ymb_welcome_gui::run(&args.global)?;
        }
    }

    info!("Application finished successfully.");
    Ok(())
}
