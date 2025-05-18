use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::text::TextBounds;
use bevy::window::CursorOptions;
use bevy::window::WindowLevel;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use ymb_discord_app_plugin::DiscordAppPlugin;
use std::env::current_exe;
use std::process::Child;
use std::process::Command;
use std::thread;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::JobObjects::AssignProcessToJobObject;
use windows::Win32::System::JobObjects::CreateJobObjectW;
use windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
use windows::Win32::System::JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
use windows::Win32::System::JobObjects::JobObjectExtendedLimitInformation;
use windows::Win32::System::JobObjects::SetInformationJobObject;
use ymb_app_under_cursor_plugin::AppUnderCursorPlugin;
use ymb_args::Args;
use ymb_args::GlobalArgs;
use ymb_click_plugin::ClickPlugin;
use ymb_exit_on_esc_plugin::ExitOnEscPlugin;
use ymb_host_cursor_position_plugin::HostCursorPositionPlugin;
use ymb_inspector_plugin::Inspector;
use ymb_inspector_plugin::InspectorPlugin;
use ymb_position_window_plugin::WindowPositionPlugin;
use ymb_windows_app_plugin::WindowsAppPlugin;

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
        // bevy
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Setting `transparent` allows the `ClearColor`'s alpha value to take effect
                        transparent: true,
                        // Disabling window decorations to make it feel more like a widget than a window
                        decorations: false,
                        window_level: WindowLevel::AlwaysOnTop,
                        cursor_options: CursorOptions {
                            visible: false,
                            // visible: true,
                            // hit_test: false,
                            hit_test: true,
                            ..default()
                        },
                        // resolution: WindowResolution::new(1920.,1080.),
                        ..default()
                    }),
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        // bevy-inspector-egui
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(
            WorldInspectorPlugin::new().run_if(|inspector: Res<Inspector>| inspector.enabled),
        )
        // ours
        .add_plugins(ExitOnEscPlugin)
        .add_plugins(HostCursorPositionPlugin)
        .add_plugins(
            WindowPositionPlugin::new().run_if(|inspector: Res<Inspector>| !inspector.enabled),
        )
        .add_plugins(InspectorPlugin)
        .add_plugins(WindowsAppPlugin)
        .add_plugins(AppUnderCursorPlugin)
        .add_plugins(ClickPlugin)
        .add_plugins(DiscordAppPlugin)
        .run();
    Ok(())
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(
        asset_server.load(ymb_assets::Texture::TargettingCircle),
    ));

    let font = asset_server.load(ymb_assets::Font::FixederSys2x);
    let slightly_smaller_text_font = TextFont {
        font,
        font_size: 35.0,
        ..default()
    };
    let box_size = Vec2::new(300.0, 200.0);
    let box_position = Vec2::new(-300.0, 250.0);
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.25, 0.25, 0.55), box_size),
            Transform::from_translation(box_position.extend(0.0)),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text2d::new("Where is the Discord mute button? Esc to cancel."),
                slightly_smaller_text_font.clone(),
                TextLayout::new(JustifyText::Left, LineBreak::WordBoundary),
                // Wrap text in the rectangle
                TextBounds::from(box_size),
                // Ensure the text is drawn on top of the box
                Transform::from_translation(Vec3::Z),
            ));
        });
}
