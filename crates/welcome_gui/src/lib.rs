use bevy::log::LogPlugin;
use bevy::prelude::*;
use windows::Win32::System::JobObjects::AssignProcessToJobObject;
use windows::Win32::System::JobObjects::CreateJobObjectW;
use windows::Win32::System::JobObjects::JobObjectExtendedLimitInformation;
use windows::Win32::System::JobObjects::SetInformationJobObject;
use windows::Win32::System::JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
use windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
use std::env::current_exe;
use std::process::Child;
use std::process::Command;
use std::thread;
use windows::Win32::Foundation::HANDLE;
use ymb_args::Args;
use ymb_args::GlobalArgs;

pub fn spawn(global_args: GlobalArgs) -> eyre::Result<()> {
    info!("Ahoy from GUI!");
    thread::spawn(move || {
        let result = spawn_gui_with_job(global_args);
        if let Err(e) = result {
            error!("Error running GUI: {e}");
            std::process::exit(1);
        }
    });
    Ok(())
}

fn spawn_gui_with_job(global_args: GlobalArgs) -> eyre::Result<()> {
    let exe = current_exe()?;

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
            }
            .as_args(),
        )
        .spawn()?;

    // Attach child process to the job
    attach_to_job(job_handle, &mut child)?;

    // Wait for the GUI process to finish
    let output = child.wait_with_output()?;
    info!("Got: {output:#?}");
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

pub fn run(_global_args: &GlobalArgs) -> eyre::Result<()> {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Setting `transparent` allows the `ClearColor`'s alpha value to take effect
                        // transparent: true,
                        // Disabling window decorations to make it feel more like a widget than a window
                        // decorations: false,
                        ..default()
                    }),
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .run();
    Ok(())
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("branding/icon.png")));
}
