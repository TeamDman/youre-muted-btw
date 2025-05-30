pub mod spawn;
use bevy::log::LogPlugin;
use bevy::prelude::*;
pub use spawn::*;
use ymb_args::GlobalArgs;
use ymb_egui_plugin::YMBEguiPlugin;
use ymb_exit_on_esc_plugin::ExitOnEscPlugin;
use ymb_ipc_plugin::IpcPlugin;
use ymb_ui_automation_plugin::UIAutomationPlugin;
use ymb_windows_app_plugin::WindowsAppPlugin;
use ymb_world_inspector_plugin::YMBWorldInspectorPlugin;
use ymb_mute_status_window_plugin::YMBMuteStatusWindowPlugin;

pub fn run(_global_args: &GlobalArgs) -> eyre::Result<()> {
    App::new()
        // bevy
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit, // we want to control the exit behavior ourselves
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .insert_resource(ClearColor(Color::NONE))
        // ours
        .add_plugins(ExitOnEscPlugin)
        .add_plugins(WindowsAppPlugin)
        .add_plugins(YMBEguiPlugin)
        .add_plugins(UIAutomationPlugin)
        .add_plugins(YMBWorldInspectorPlugin)
        .add_plugins(YMBMuteStatusWindowPlugin)
        .add_plugins(IpcPlugin)
        .run();
    Ok(())
}