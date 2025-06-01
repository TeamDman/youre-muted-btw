#![windows_subsystem = "windows"]
use clap::CommandFactory;
use clap::FromArgMatches;
use tracing::debug;
use tracing::info;
use ymb_args::Args;
use ymb_args::Command;
use ymb_console::maybe_attach_or_hide_console;
use ymb_lifecycle::GLOBAL_ARGS;
use ymb_logs::DualWriter;
use ymb_logs::setup_tracing;
use ymb_windy::WindyResult;

fn main() -> WindyResult<()> {
    // Handle console attach/detach logic before any logging or error handling is set up
    let ran_from_inherited_console = maybe_attach_or_hide_console();

    if let Err(e) = color_eyre::install() {
        eprintln!("Failed to install color_eyre: {:?}", e);
    }

    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(
            "Panic encountered at {}",
            info.location()
                .map(|x| x.to_string())
                .unwrap_or("unknown location".to_string())
        );
        panic_hook(info);
    }));

    let mut cmd = Args::command();
    cmd = cmd.version(env!("CARGO_PKG_VERSION"));
    let args = Args::from_arg_matches(&cmd.get_matches())?;
    GLOBAL_ARGS.set(args.global.clone()).unwrap();

    match args.command {
        None | Some(Command::Tray) => {
            let writer = DualWriter::new();
            let log_buffer = writer.buffer.clone();
            setup_tracing(&args.global, writer)?;
            info!("Running tray icon application");
            info!("Running from console: {}", ran_from_inherited_console);

            // No need to call hide_console_window here; handled in maybe_attach_or_hide_console

            info!("Starting tray icon application");
            ymb_tray::main(args.global, log_buffer)?;
        }
        Some(Command::WelcomeGui) => {
            setup_tracing(&args.global, std::io::stderr)?;
            info!("Running GUI application");
            info!("Running from console: {}", ran_from_inherited_console);
            ymb_welcome_gui::run(&args.global)?;
        }
    }

    info!("Application finished successfully");
    // wait for newline
    // println!("Press Enter to exit...");
    // let _ = std::io::stdin().read_line(&mut String::new());
    Ok(())
}
