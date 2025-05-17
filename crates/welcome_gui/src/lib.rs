//! Shows how to display a window in transparent mode.
//!
//! This feature works as expected depending on the platform. Please check the
//! [documentation](https://docs.rs/bevy/latest/bevy/prelude/struct.Window.html#structfield.transparent)
//! for more details.

use std::env::current_exe;
use std::process::Command;
use std::thread;

use bevy::log::LogPlugin;
use bevy::prelude::*;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use bevy::window::CompositeAlphaMode;
use ymb_args::Args;
use ymb_args::GlobalArgs;

pub fn spawn(global_args: GlobalArgs) -> eyre::Result<()> {
    info!("Ahoy from GUI!");
    thread::spawn(move || {
        let result: eyre::Result<()> = (|| {
            let exe = current_exe()?;
            let output = Command::new(exe)
                .args(
                    Args {
                        global: global_args.clone(),
                        command: Some(ymb_args::Command::WelcomeGui),
                    }
                    .as_args(),
                )
                .spawn()?
                .wait_with_output()?;
            info!("Got: {output:#?}");
            Ok(())
        })();
        if let Err(e) = result {
            error!("Error running GUI: {e}");
            std::process::exit(1);
        };
    });
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
        // ClearColor must have 0 alpha, otherwise some color will bleed through
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .run();
    Ok(())
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("branding/icon.png")));
}
