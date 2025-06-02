use bevy::prelude::*;
use std::env::current_exe;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::OnceLock;
use std::thread;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::AssignProcessToJobObject;
use windows::Win32::System::JobObjects::CreateJobObjectW;
use windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
use windows::Win32::System::JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
use windows::Win32::System::JobObjects::JobObjectExtendedLimitInformation;
use windows::Win32::System::JobObjects::SetInformationJobObject;
use ymb_args::Args;
use ymb_args::GlobalArgs;
use ymb_logs::DualLogWriter;

const DETACHED_PROCESS: u32 = 0x00000008;
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn spawn(global_args: GlobalArgs, log_writer: DualLogWriter) -> eyre::Result<()> {
    info!("Ahoy from GUI!");
    thread::spawn(move || {
        let result = spawn_gui_with_job(global_args, log_writer);
        if let Err(e) = result {
            error!("Error running GUI: {e}");
            std::process::exit(1);
        }
    });
    Ok(())
}

fn spawn_gui_with_job(global_args: GlobalArgs, log_writer: DualLogWriter) -> eyre::Result<()> {
    let exe = current_exe()?;

    // Generate a unique pipe name
    let pipe_guid = uuid::Uuid::new_v4();
    // Use the same format as the Bevy IPC plugin expects (\\.\pipe\ymb-gui-ipc-<pid>-<guid>)
    let gui_pid = std::process::id();
    let pipe_name = format!(
        r"\\.\pipe\ymb-gui-ipc-{}-{}",
        gui_pid,
        pipe_guid.as_simple()
    );

    // Create a job object that kills processes when the handle is closed
    let job_handle = unsafe { CreateJobObjectW(None, None)? };
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    unsafe {
        SetInformationJobObject(
            job_handle,
            JobObjectExtendedLimitInformation,
            &info as *const _ as _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )?;
    }

    let mut child = Command::new(exe)
        .args(
            Args {
                global: global_args,
                command: Some(ymb_args::Command::WelcomeGui),
                tray_mode_active: true,
            }
            .as_args(),
        )
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .env("YMB_IPC_PIPE_NAME", &pipe_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Attach child process to the job
    attach_to_job(job_handle, &mut child)?;

    // Print for debug: ensure the tray and GUI are using the same pipe name
    info!("[SPAWN] Pipe name for tray and GUI: {}", pipe_name);

    // Store the pipe name somewhere accessible to the tray (e.g., in a static/global)
    set_pipe_name_for_tray(pipe_name);

    // Pipe child stdout/stderr to the tray's log buffer
    if let Some(stdout) = child.stdout.take() {
        info!("Capturing child stdout");
        let mut log_writer = log_writer.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                // info!("Got stdout line: {line:?}");
                if let Ok(line) = line {
                    // let mut buffer = log_buffer.lock().unwrap();
                    // buffer.extend_from_slice(line.as_bytes());
                    // buffer.push(b'\n');
                    write!(log_writer, "{line}\n").unwrap_or_else(|e| {
                        error!("Failed to write to log writer: {e}");
                    });
                }
            }
        });
    } else {
        error!("Failed to capture child stdout");
    }
    if let Some(stderr) = child.stderr.take() {
        info!("Capturing child stderr");
        let mut log_writer = log_writer.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                // info!("Got stderr line: {line:?}");
                if let Ok(line) = line {
                    // let mut buffer = log_buffer.lock().unwrap();
                    // buffer.extend_from_slice(line.as_bytes());
                    // buffer.push(b'\n');
                    write!(log_writer, "{line}\n").unwrap_or_else(|e| {
                        error!("Failed to write to log writer: {e}");
                    });
                }
            }
        });
    } else {
        error!("Failed to capture child stderr");
    }

    // Wait for the GUI process to finish
    child.wait_with_output()?.exit_ok()?;
    Ok(())
}

fn attach_to_job(job_handle: HANDLE, child: &mut Child) -> eyre::Result<()> {
    use std::os::windows::io::AsRawHandle;
    let proc_handle = HANDLE(child.as_raw_handle());
    unsafe {
        AssignProcessToJobObject(job_handle, proc_handle)?;
    }

    // Leak the job handle so it stays valid until this process exits,
    // ensuring the GUI is killed if the tray process terminates.
    Box::leak(Box::new(job_handle));

    Ok(())
}

// Store the pipe name for the tray to use
static PIPE_NAME_FOR_TRAY: OnceLock<String> = OnceLock::new();
fn set_pipe_name_for_tray(name: String) {
    let _ = PIPE_NAME_FOR_TRAY.set(name);
}
pub fn get_pipe_name_for_tray() -> Option<String> {
    PIPE_NAME_FOR_TRAY.get().cloned()
}
