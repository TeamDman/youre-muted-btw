use clap::CommandFactory;
use clap::Parser;
use std::env;
use std::os::windows::process::CommandExt;
use std::process::Command as StdCommand;
use tracing::info;
use ymb_args::Args;
use ymb_args::Command;
use ymb_console::is_inheriting_console;
use ymb_logs::DualLogWriter;
use ymb_logs::setup_tracing;
use ymb_windy::error::WindyResult;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() -> WindyResult<()> {
    eprintln!("Ahoy from fn main!");
    let args = Args::try_parse();

    let is_tray_mode = args.as_ref().map_or(false, |a| a.tray_mode_active);
    eprintln!("Is tray mode active: {}", is_tray_mode);

    let inheriting_console = is_inheriting_console();
    eprintln!("Is inheriting console: {}", inheriting_console);

    if !is_tray_mode && !inheriting_console {
        eprintln!("Relaunch in detached tray mode");
        eprintln!(
            "Initial launch without inherited console detected. Re-launching in detached tray mode..."
        );
        let current_exe = env::current_exe().expect("Failed to get current executable path.");
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
        return Ok(());
    } else {
        eprintln!(
            "No need to relaunch, either tray mode is active or inheriting console is detected."
        );
    }

    // Parse args or print error/help and exit
    let args = match args {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Argument parsing error: {}", e);
            Args::command().get_matches();
            return Err(e.into());
        }
    };

    if !args.tray_mode_active && inheriting_console {
        if ymb_console::maybe_attach_or_hide_console() {
            info!("Ran from inherited console, console attached successfully.");
        } else {
            eprintln!(
                "Warning: Detected inherited console, but maybe_attach_or_hide_console returned false after attempt."
            );
        }
    }

    if let Err(e) = color_eyre::install() {
        eprintln!("Failed to install color_eyre: {:?}", e);
    }

    eprintln!("Installing main panic hook for tracing");
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let msg = format!(
            "Panic encountered at {}",
            panic_info
                .location()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "unknown location".to_string())
        );
        eprintln!("{}", msg);
        tracing::error!("{}", msg);
        panic_hook(panic_info);
    }));

    eprintln!("Setting up tracing");
    let log_writer = DualLogWriter::new();
    setup_tracing(&args.global, log_writer.clone())?;
    info!("Tracing setup complete!");

    info!("Handling command line arguments");
    match args.command {
        None | Some(Command::Tray) => {
            info!("Starting tray icon process...");
            ymb_tray::main(args.global, log_writer)?;
        }
        Some(Command::WelcomeGui) => {
            info!("Starting GUI process...");
            ymb_welcome_gui::run(&args.global)?;
        }
    }

    info!("Application finished successfully.");
    Ok(())
}
