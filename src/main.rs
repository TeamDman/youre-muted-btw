// disable windows console

use clap::CommandFactory;
use clap::FromArgMatches;
use tracing::debug;
use tracing::info;
use ymb_args::Args;
use ymb_args::Command;
use ymb_console::hide_console_window;
use ymb_console::is_inheriting_console;
use ymb_lifecycle::GLOBAL_ARGS;
use ymb_logs::DualWriter;
use ymb_logs::setup_tracing;
use ymb_windy::WindyResult;

fn main() -> WindyResult<()> {
    color_eyre::install()?;

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

            let ran_from_console = is_inheriting_console();
            info!("Running from console: {}", ran_from_console);

            // Hide the console window at startup
            if ran_from_console {
                debug!("Already running from terminal, no need to hide console window");
            } else {
                debug!("Not launched from a console, hiding the default one");
                // hide_console_window();
            }

            info!("Starting tray icon application");
            ymb_tray::main(args.global, log_buffer)?;
        }
        Some(Command::WelcomeGui) => {
            setup_tracing(&args.global, std::io::stderr)?;
            info!("Running GUI application");
            ymb_welcome_gui::run(&args.global)?;
        }
    }

    info!("Application finished successfully");
    // wait for newline
    // println!("Press Enter to exit...");
    // let _ = std::io::stdin().read_line(&mut String::new());
    Ok(())
}
