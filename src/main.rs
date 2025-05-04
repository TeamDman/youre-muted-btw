pub mod windy_error;

use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tracing::error;
use tracing::info;
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
const ID_QUIT: u32 = 3;

struct TrayWindow {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl TrayWindow {
    fn handle(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        match message {
            WM_TRAYICON => {
                if lparam.0 as u32 == WM_RBUTTONUP {
                    unsafe {
                        let hmenu = CreatePopupMenu().unwrap();
                        let hello_text = w!("Hello!");
                        let quit_text = w!("Quit");

                        AppendMenuW(hmenu, MF_STRING, ID_HELLO as usize, hello_text).unwrap();
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
                } else {
                    false
                }
            }
            WM_COMMAND => match wparam.0 as u32 {
                ID_HELLO => {
                    unsafe {
                        MessageBoxW(Some(self.hwnd), w!("Hello from tray!"), w!("Hello"), MB_OK);
                    }
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
                // If we have our window, ask it to close itself:
                if let Err(e) = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }
                {
                    error!("Failed to post close message: {}", e);
                    FALSE
                } else {
                    TRUE
                }
            } else {
                error!("No window handle available for cleanup");
                FALSE
            }
        }
        _ => FALSE,
    }
}

static OUR_HWND: AtomicUsize = AtomicUsize::new(0);

fn main() -> WindyResult<()> {
    color_eyre::install()?;

    let mut cmd = Args::command();
    cmd = cmd.version(env!("CARGO_PKG_VERSION"));
    let args = Args::from_arg_matches(&cmd.get_matches())?;

    let debug = args.global.debug;
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_target(false)
        .with_max_level(match debug {
            true => tracing::level_filters::LevelFilter::DEBUG,
            false => tracing::level_filters::LevelFilter::INFO,
        })
        .init();

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
        let window = Box::new(TrayWindow { hwnd, nid });
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
