mod ansi_support;
pub use ansi_support::*;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tracing::debug;
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
use windows::Win32::System::Console::{AttachConsole, FreeConsole};
use windows::Win32::Foundation::{GetLastError, ERROR_ACCESS_DENIED};

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

pub fn attach_console_window(log_buffer: LogBuffer) {
    // Create new console
    if let Err(e) = unsafe { AllocConsole() } {
        error!("Failed to allocate console: {}", e);
        return;
    }

    // Update the flag for the tray action
    SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION.store(true, Ordering::SeqCst);

    // Attach ctrl+c handler
    if let Err(e) = unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), true) } {
        error!("Failed to set console control handler: {}", e);
    }

    // Enable ANSI support
    if let Err(e) = enable_ansi_support() {
        error!("Failed to enable ANSI support: {:?}", e);
    }

    // Replay buffered logs
    if let Ok(buffer) = log_buffer.lock() {
        if let Ok(logs) = String::from_utf8(buffer.clone()) {
            println!("=== Previous logs ===");
            println!("{logs}");
            println!("=== End of previous logs ===");
        }
    }

    // Tell the user whats up
    info!("Console allocated, new logs will be visible here");
    info!("Closing this window will exit the program");
    info!(
        "The system tray icon now has a 'Hide logs' option if you want to close this window without exiting the program"
    );

    // Diagnostic message to show that the console is working
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        info!("Ahoy, there!");
    });
}

pub fn is_inheriting_console() -> bool {
    // https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist
    let mut buffer = [0u32; 1];
    let rtn = unsafe { GetConsoleProcessList(&mut buffer) };
    debug!(
        "GetConsoleProcessList returned: {rtn}, inheriting console: {}",
        rtn != 1
    );

    rtn != 1
}

/// Handles console attachment/detachment logic for windows subsystem apps.
/// Returns true if logs should go to the terminal, false if logs should go to buffer.
pub fn maybe_attach_or_hide_console() -> bool {
    let mut ran_from_inherited_console = is_inheriting_console();
    if ran_from_inherited_console {
        unsafe {
            // Try to attach to parent console (ATTACH_PARENT_PROCESS = u32::MAX)
            if !AttachConsole(u32::MAX).is_ok() {
                let error = GetLastError();
                if error != ERROR_ACCESS_DENIED {
                    // Only treat as failure if not "already attached"
                    eprintln!(
                        "Warning: is_inheriting_console was true, but AttachConsole failed with error: {:?}",
                        error
                    );
                    ran_from_inherited_console = false;
                }
            }
        }
    } else {
        // Not running from a terminal, so hide/detach the auto console if present
        unsafe {
            info!("Attempting to detach from console (if any is attached)");
            if let Err(e) = FreeConsole() {
                debug!("FreeConsole failed (likely no console was attached): {}", e);
            }
        }
    }
    ran_from_inherited_console
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_opened_from_console() {
        assert!(is_inheriting_console());
    }
}
