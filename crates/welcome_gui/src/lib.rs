mod spawn;
use bevy::log::LogPlugin;
use bevy::prelude::*;
pub use spawn::*;
use ymb_app_under_cursor_plugin::AppUnderCursorPlugin;
use ymb_args::GlobalArgs;
use ymb_click_plugin::ClickPlugin;
use ymb_discord_app_plugin::DiscordAppPlugin;
use ymb_egui_plugin::YMBEguiPlugin;
use ymb_exit_on_esc_plugin::ExitOnEscPlugin;
use ymb_host_cursor_position_plugin::HostCursorPositionPlugin;
use ymb_inspector_plugin::Inspector;
use ymb_inspector_plugin::InspectorPlugin;
use ymb_position_window_plugin::WindowPositionPlugin;
use ymb_targetting_window_plugin::TargettingWindowPlugin;
use ymb_tree_window_plugin::TreeWindowPlugin;
use ymb_windows_app_plugin::WindowsAppPlugin;
use ymb_world_inspector_plugin::YMBWorldInspectorPlugin;

pub fn run(_global_args: &GlobalArgs) -> eyre::Result<()> {
    App::new()
        // bevy
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .disable::<LogPlugin>(),
        )
        .insert_resource(ClearColor(Color::NONE))
        // ours
        .add_plugins(ExitOnEscPlugin)
        .add_plugins(HostCursorPositionPlugin)
        .add_plugins(
            WindowPositionPlugin::new().run_if(|inspector: Res<Inspector>| !inspector.enabled),
        )
        .add_plugins(InspectorPlugin)
        .add_plugins(WindowsAppPlugin)
        .add_plugins(TargettingWindowPlugin)
        .add_plugins(AppUnderCursorPlugin)
        .add_plugins(ClickPlugin)
        .add_plugins(DiscordAppPlugin)
        .add_plugins(TreeWindowPlugin)
        .add_plugins(YMBEguiPlugin)
        .add_plugins(YMBWorldInspectorPlugin)
        .run();
    Ok(())
}
