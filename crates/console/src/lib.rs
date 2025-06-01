mod ansi_support;
pub use ansi_support::*;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tracing::debug;
use tracing::error;
use tracing::info;
use windows::Win32::Foundation::ERROR_ACCESS_DENIED;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::AttachConsole;
use windows::Win32::System::Console::FreeConsole;
use windows::Win32::System::Console::GetConsoleProcessList;
use windows::Win32::System::Console::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::BOOL;
use windows::core::w;
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
    let mut pids = [0u32; 2]; // Buffer for at least two PIDs
    // https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist
    let count = unsafe { GetConsoleProcessList(pids.as_mut_slice()) };

    // count == 0: GetConsoleProcessList failed or no console is attached.
    // count == 1: Only our process is attached (e.g., we called AllocConsole,
    //             or a console app started in its own new window and we are that app).
    // count > 1: More than one process, implies we inherited from a parent (e.g., shell).
    let inheriting = count > 1;

    #[cfg(debug_assertions)]
    {
        let msg = format!("GetConsoleProcessList count: {count}, inheriting: {inheriting}",);
        let msg = msg
            .encode_utf16()
            .chain("\0".encode_utf16())
            .collect::<Vec<u16>>();
        let wstring = windows::core::PCWSTR(msg.as_ptr());
        unsafe {
            MessageBoxW(
                None,
                wstring,
                w!("Console Check"),
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }

    // For very early diagnostics before tracing is set up:
    // eprintln!("[is_inheriting_console] GetConsoleProcessList count: {count}, inheriting: {inheriting}");
    inheriting
}

/// Handles console attachment/detachment logic.
/// Call this VERY EARLY, before logging or color_eyre is set up.
/// Returns true if successfully attached to an inherited console (logs should go to terminal),
/// false otherwise (e.g., double-clicked, logs to buffer).
pub fn maybe_attach_or_hide_console() -> bool {
    let is_inherited = is_inheriting_console();

    if is_inherited {
        unsafe {
            let mut attached_to_parent = false;
            // ATTACH_PARENT_PROCESS is u32::MAX
            if AttachConsole(u32::MAX).is_ok() {
                attached_to_parent = true;
                // eprintln!("[console] Attached to parent console.");
            } else {
                let error = GetLastError();
                if error == ERROR_ACCESS_DENIED {
                    // Already attached, which is fine.
                    attached_to_parent = true;
                    // eprintln!("[console] Already attached to a console (ERROR_ACCESS_DENIED).");
                } else {
                    eprintln!(
                        "[console] Warning: Detected inherited console, but AttachConsole failed (error: {:?}). Logs might not go to terminal.",
                        error
                    );
                    // Fall through, ran_from_inherited_console will be false.
                }
            }

            if attached_to_parent {
                // Crucial: Re-evaluate standard handles after AttachConsole
                // This isn't explicitly done here but Windows should repoint them.
                // Enable ANSI support for the attached console.
                if let Err(e) = enable_ansi_support() {
                    eprintln!(
                        "[console] Warning: Failed to enable ANSI support for attached console: {:?}",
                        e
                    );
                }
                return true; // Successfully using inherited console
            }
        }
    } else {
        // Not inheriting a console (e.g., double-clicked a "windows" subsystem app,
        // or a "console" subsystem app that got its own new console).
        // If we are a "windows" subsystem app, no console was made for us by the OS. FreeConsole is a no-op.
        // If we are a "console" subsystem app (OS made one), FreeConsole hides it.
        // This FreeConsole call is primarily for the "console subsystem app, double-clicked" scenario.
        // eprintln!("[console] Not inheriting console. Attempting FreeConsole.");
        unsafe {
            if FreeConsole().is_err() {
                // This is expected if no console was attached (e.g. windows subsystem app)
                // let error = GetLastError();
                // eprintln!("[console] FreeConsole failed or was no-op (error: {:?}).", error);
            }
        }
    }
    false // Not using an inherited console
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_opened_from_console() {
        assert!(is_inheriting_console());
    }
}
