use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tracing::error;
use tracing::info;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::GetConsoleProcessList;
use windows::Win32::System::Console::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::BOOL;
use ymb_lifecycle::OUR_HWND;
use ymb_lifecycle::SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION;
use ymb_logs::LogBuffer;

// todo: rename to detach
pub fn hide_console_window() {
    unsafe {
        info!(
            "Detaching from this console, ctrl+c will no longer work and you will have to use the system tray icon to close the program"
        );
        if let Err(e) = FreeConsole() {
            error!("Failed to free console: {}", e);
        }
    }
}

pub unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> BOOL {
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

// todo: rename to attach
pub fn show_console_window(log_buffer: LogBuffer) {
    if let Err(e) = unsafe { AllocConsole() } {
        error!("Failed to allocate console: {}", e);
    } else {
        SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION.store(true, Ordering::SeqCst);
        // Set up console control handler for the new console
        if let Err(e) = unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), true) } {
            error!("Failed to set console control handler: {}", e);
        }

        // Replay buffered logs
        if let Ok(buffer) = log_buffer.lock() {
            if let Ok(logs) = String::from_utf8(buffer.clone()) {
                println!("=== Previous logs ===");
                println!("{logs}");
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

pub fn is_inheriting_console() -> bool {
    // https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist
    let mut buffer = [0u32; 1];
    let rtn = unsafe { GetConsoleProcessList(&mut buffer) };
    println!("GetConsoleProcessList returned: {rtn}");

    rtn != 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_opened_from_console() {
        assert!(is_inheriting_console());
    }
}
