use clap::CommandFactory;
use clap::FromArgMatches;
use ymb_args::Args;
use ymb_console::hide_console_window;
use ymb_console::is_inheriting_console;
use ymb_logs::setup_tracing;
use ymb_windy::WindyResult;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::util::SubscriberInitExt;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::BOOL;
use windows::core::w;

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

    let debug = args.global.debug;
    let ran_from_console = is_inheriting_console();
    let log_buffer = setup_tracing(debug);
    info!("Running from console: {}", ran_from_console);

    // Hide the console window at startup
    if ran_from_console {
        debug!("Already running from terminal, no need to hide console window");
    } else {
        debug!("Not launched from a console, hiding the default one");
        hide_console_window();
    }

    info!("Starting tray icon application");
    ymb_tray::main(log_buffer)?;
    Ok(())
}