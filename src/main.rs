pub mod console_check;
pub mod windy_error;
pub mod gui;

use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use console_check::is_inheriting_console;
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
use windy_error::WindyResult;

#[derive(Debug, Parser)]
#[command(name = "youre-muted-btw", bin_name = "youre-muted-btw")]
pub struct Args {
    #[command(flatten)]
    pub global: GlobalArgs,
}

#[derive(Debug, Parser)]
pub struct GlobalArgs {
    /// Enable debug logging
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,
}

const WM_TRAYICON: u32 = WM_USER + 1;
const ID_TRAYICON: u32 = 1;
const ID_HELLO: u32 = 2;
const ID_SHOW_LOGS: u32 = 3;
const ID_HIDE_LOGS: u32 = 5;
const ID_QUIT: u32 = 4;
const ID_OPEN: u32 = 6;

struct TrayWindow {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    /// Logs are stored in a buffer to be displayed in the console when the user clicks show logs
    log_buffer: Arc<Mutex<Vec<u8>>>,
    /// Has the user clicked show logs button at least once
    console_shown: bool,
}

impl TrayWindow {
    fn handle(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        match message {
            WM_TRAYICON => {
                if lparam.0 as u32 == WM_RBUTTONUP {
                    unsafe {
                        let hmenu = CreatePopupMenu().unwrap();
                        let open_text = w!("Open");
                        let hello_text = w!("Hello!");
                        let show_logs_text = w!("Show logs");
                        let hide_logs_text = w!("Hide logs");
                        let quit_text = w!("Quit");

                        AppendMenuW(hmenu, MF_STRING, ID_OPEN as usize, open_text).unwrap();
                        AppendMenuW(hmenu, MF_STRING, ID_HELLO as usize, hello_text).unwrap();
                        AppendMenuW(hmenu, MF_STRING, ID_SHOW_LOGS as usize, show_logs_text)
                            .unwrap();
                        if self.console_shown {
                            AppendMenuW(hmenu, MF_STRING, ID_HIDE_LOGS as usize, hide_logs_text)
                                .unwrap();
                        }
                        AppendMenuW(hmenu, MF_STRING, ID_QUIT as usize, quit_text).unwrap();

                        let mut pt = POINT { x: 0, y: 0 };
                        GetCursorPos(&mut pt).unwrap();
                        SetForegroundWindow(self.hwnd).unwrap();
                        TrackPopupMenu(
                            hmenu,
                            TPM_RIGHTBUTTON,
                            pt.x,
                            pt.y,
                            Default::default(),
                            self.hwnd,
                            None,
                        )
                        .unwrap();
                        DestroyMenu(hmenu).unwrap();
                    }
                    true
                } else if lparam.0 as u32 == WM_LBUTTONUP {
                    info!("Hello from tray icon click!");
                    thread::spawn(|| {
                        gui::main().unwrap();
                    });
                    true
                } else {
                    false
                }
            }
            WM_COMMAND => match wparam.0 as u32 {
                ID_OPEN => {
                    info!("Open menu item clicked");
                    true
                }
                ID_HELLO => {
                    unsafe {
                        MessageBoxW(Some(self.hwnd), w!("Hello from tray!"), w!("Hello"), MB_OK);
                    }
                    true
                }
                ID_SHOW_LOGS => {
                    unsafe {
                        hide_console_window();
                        if let Err(e) = AllocConsole() {
                            error!("Failed to allocate console: {}", e);
                        } else {
                            self.console_shown = true;
                            // Set up console control handler for the new console
                            if let Err(e) = SetConsoleCtrlHandler(Some(ctrl_handler), true) {
                                error!("Failed to set console control handler: {}", e);
                            }

                            // Replay buffered logs
                            if let Ok(buffer) = self.log_buffer.lock() {
                                if let Ok(logs) = String::from_utf8(buffer.clone()) {
                                    println!("=== Previous logs ===");
                                    println!("{}", logs);
                                    println!("=== End of previous logs ===");
                                }
                            }
                            info!("Console allocated, new logs will be visible here");
                            info!("Closing this window will exit the program");
                            info!(
                                "The system tray icon now has a 'Hide logs' option if you want to close this window without exiting the program"
                            );
                            thread::spawn(move || {
                                thread::sleep(Duration::from_secs(5));
                                info!("Ahoy, there!");
                            });
                        }
                    }
                    true
                }
                ID_HIDE_LOGS => {
                    hide_console_window();
                    self.console_shown = false;
                    info!("Console hidden");
                    true
                }
                ID_QUIT => {
                    unsafe {
                        // Clean up the tray icon before quitting
                        if let Err(e) = Shell_NotifyIconW(NIM_DELETE, &self.nid).ok() {
                            error!("Failed to delete tray icon: {}", e);
                        }
                        DestroyWindow(self.hwnd).ok();
                    }
                    true
                }
                _ => false,
            },
            WM_CLOSE => {
                unsafe {
                    // Clean up the tray icon before closing
                    if let Err(e) = Shell_NotifyIconW(NIM_DELETE, &self.nid).ok() {
                        error!("Failed to delete tray icon: {}", e);
                    }
                    DestroyWindow(self.hwnd).ok();
                }
                true
            }
            WM_DESTROY => {
                unsafe {
                    // Clean up the tray icon before quitting
                    if let Err(e) = Shell_NotifyIconW(NIM_DELETE, &self.nid).ok() {
                        error!("Failed to delete tray icon: {}", e);
                    }
                    PostQuitMessage(0);
                }
                true
            }
            _ => false,
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_CREATE {
        let window = Box::new(TrayWindow {
            hwnd,
            nid: Default::default(),
            log_buffer: Arc::new(Mutex::new(Vec::new())),
            console_shown: false,
        });
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(window) as _) };
        return LRESULT(0);
    }

    let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if user_data == 0 {
        return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
    }

    let window = unsafe { &mut *(user_data as *mut TrayWindow) };
    if window.handle(message, wparam, lparam) {
        LRESULT(0)
    } else {
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }
}

unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT
        | CTRL_SHUTDOWN_EVENT => {
            info!("Received shutdown signal, cleaning up...");
            let hwnd_val = OUR_HWND.load(Ordering::SeqCst);
            if hwnd_val != 0 {
                let hwnd = HWND(hwnd_val as *mut _);
                // SendMessageW will synchronously pump the message and wait for it to finish
                let _result = unsafe { SendMessageW(hwnd, WM_CLOSE, None, None) };
                TRUE
            } else {
                error!("No window handle available for cleanup");
                FALSE
            }
        }
        _ => FALSE,
    }
}

static OUR_HWND: AtomicUsize = AtomicUsize::new(0);

fn hide_console_window() {
    unsafe {
        info!("Detaching from this console, ctrl+c will no longer work and you will have to use the system tray icon to close the program");
        if let Err(e) = FreeConsole() {
            error!("Failed to free console: {}", e);
        }
    }
}

struct DualWriter {
    stdout: std::io::Stdout,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to stdout
        self.stdout.write(buf)?;

        // Write to buffer
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}

impl<'a> MakeWriter<'a> for DualWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        DualWriter {
            stdout: std::io::stdout(),
            buffer: self.buffer.clone(),
        }
    }
}

fn setup_tracing(debug: bool) -> Arc<Mutex<Vec<u8>>> {
    let buffer = Arc::new(Mutex::new(Vec::new()));

    let subscriber = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(false)
        .with_max_level(match debug {
            true => tracing::level_filters::LevelFilter::DEBUG,
            false => tracing::level_filters::LevelFilter::INFO,
        })
        .with_ansi(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_span_events(FmtSpan::NONE)
        .with_timer(SystemTime::default())
        .with_writer(DualWriter {
            stdout: std::io::stdout(),
            buffer: buffer.clone(),
        })
        .finish();

    subscriber.init();

    buffer
}

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

    unsafe {
        let instance = {
            let mut out = Default::default();
            GetModuleHandleExW(Default::default(), None, &mut out)?;
            out
        };
        let class_name = w!("TrayIconWindow");

        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };
        let atom = RegisterClassExW(&window_class);
        std::debug_assert_ne!(atom, 0);

        let window_title = w!("Tray Icon");
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            window_title,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;
        OUR_HWND.store(hwnd.0 as usize, Ordering::SeqCst);

        info!("Window handle set");

        // Set up Ctrl+C handler
        SetConsoleCtrlHandler(Some(ctrl_handler), true)?;

        // Load the icon
        let icon_path = w!("favicon.ico");
        let icon = LoadImageW(
            Some(instance.into()),
            icon_path,
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE,
        );
        let icon = match icon {
            Ok(icon) => HICON(icon.0),
            Err(_) => LoadIconW(None, IDI_APPLICATION)?,
        };

        // Create tray icon
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: ID_TRAYICON,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: icon,
            szTip: [0; 128],
            ..Default::default()
        };

        // Set tooltip
        let tooltip =
            w!("You're Muted Btw - This app warns when you're trying to talk while muted");
        let tooltip_bytes = tooltip.as_wide();
        nid.szTip[..tooltip_bytes.len()].copy_from_slice(tooltip_bytes);

        Shell_NotifyIconW(NIM_ADD, &nid).ok()?;

        // Store the nid in the window
        let window = Box::new(TrayWindow {
            hwnd,
            nid,
            log_buffer,
            console_shown: false,
        });
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(window) as _);

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Final cleanup
        if let Some(window) = (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TrayWindow).as_mut() {
            if let Err(e) = Shell_NotifyIconW(NIM_DELETE, &window.nid).ok() {
                error!("Failed to delete tray icon: {}", e);
            }
        }
        DestroyIcon(icon)?;
    }

    Ok(())
}
