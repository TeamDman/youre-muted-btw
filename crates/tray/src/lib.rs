use chrono;
use interprocess::os::windows::named_pipe::DuplexPipeStream;
use interprocess::os::windows::named_pipe::pipe_mode;
use std::io::Write;
use std::sync::atomic::Ordering;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use ymb_args::GlobalArgs;
use ymb_console::ctrl_handler;
use ymb_console::hide_console_window;
use ymb_console::show_console_window;
use ymb_lifecycle::GLOBAL_ARGS;
use ymb_lifecycle::OUR_HWND;
use ymb_lifecycle::SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION;
use ymb_logs::LogBuffer;
use ymb_windy::WindyResult;

const WM_TRAYICON: u32 = WM_USER + 1;
const ID_TRAYICON: u32 = 1;
const ID_HELLO: u32 = 2;
const ID_SHOW_LOGS: u32 = 3;
const ID_HIDE_LOGS: u32 = 5;
const ID_QUIT: u32 = 4;
const ID_OPEN: u32 = 6;
const ID_DEBUG_MSG: u32 = 7;

struct TrayWindow {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    log_buffer: LogBuffer,
    global_args: GlobalArgs,
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
                        let debug_msg_text = w!("Debug Msg");
                        let quit_text = w!("Quit");
                        AppendMenuW(hmenu, MF_STRING, ID_OPEN as usize, open_text).unwrap();
                        AppendMenuW(hmenu, MF_STRING, ID_HELLO as usize, hello_text).unwrap();
                        AppendMenuW(hmenu, MF_STRING, ID_SHOW_LOGS as usize, show_logs_text)
                            .unwrap();
                        if SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION.load(Ordering::SeqCst) {
                            AppendMenuW(hmenu, MF_STRING, ID_HIDE_LOGS as usize, hide_logs_text)
                                .unwrap();
                        }
                        AppendMenuW(hmenu, MF_STRING, ID_DEBUG_MSG as usize, debug_msg_text)
                            .unwrap();
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
                    ymb_welcome_gui::spawn(self.global_args.clone()).unwrap();

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
                    hide_console_window();
                    show_console_window(self.log_buffer.clone());
                    true
                }
                ID_HIDE_LOGS => {
                    hide_console_window();
                    SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION.store(false, Ordering::SeqCst);
                    info!("Console hidden");
                    true
                }
                ID_DEBUG_MSG => {
                    info!("Debug Msg menu item clicked");
                    // Get the pipe name from the welcome_gui spawn module
                    let pipe_name = ymb_welcome_gui::spawn::get_pipe_name_for_tray();
                    match pipe_name {
                        Some(pipe_name) => {
                            info!(
                                "Tray: Attempting to send debug message to IPC pipe: {}",
                                pipe_name
                            );
                            let message_to_send = format!(
                                "Debug message from tray at {}!\n",
                                chrono::Local::now().format("%H:%M:%S")
                            );
                            std::thread::spawn(move || {
                                match DuplexPipeStream::<pipe_mode::Bytes>::connect_by_path(
                                    &*pipe_name,
                                ) {
                                    Ok(mut stream) => {
                                        info!("Tray (IPC Thread): Connected to IPC pipe.");
                                        if let Err(e) = stream.write_all(message_to_send.as_bytes())
                                        {
                                            error!(
                                                "Tray (IPC Thread): Failed to send debug message: {}",
                                                e
                                            );
                                        } else {
                                            info!(
                                                "Tray (IPC Thread): Sent debug message: '{}'",
                                                message_to_send.trim()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "Tray (IPC Thread): Failed to connect to IPC pipe: {}. Is GUI running and its IPC server ready?",
                                            e
                                        );
                                    }
                                }
                            });
                            true
                        }
                        None => {
                            warn!("Tray: IPC pipe name not set. Is GUI running?");
                            unsafe {
                                MessageBoxW(
                                    Some(self.hwnd),
                                    w!(
                                        "GUI IPC pipe name not found. Is the GUI application running and its IPC server initialized?"
                                    ),
                                    w!("IPC Error"),
                                    MB_OK,
                                );
                            }
                            true
                        }
                    }
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
                        debug!("Failed to delete tray icon, this always happens :P {}", e);
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
            log_buffer: Default::default(),
            global_args: GLOBAL_ARGS.get().unwrap().clone(),
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

pub fn main(global_args: GlobalArgs, log_buffer: LogBuffer) -> WindyResult<()> {
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
            global_args,
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
                debug!("Failed to delete tray icon, this always happens :P {}", e);
            }
        }
        DestroyIcon(icon)?;
    }

    Ok(())
}
