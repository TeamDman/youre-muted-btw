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

pub fn run(_global_args: &GlobalArgs) -> eyre::Result<()> {
    App::new()
        // bevy
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    close_when_requested: false, // see below
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
        .add_plugins(IpcPlugin)
        .run();
    Ok(())
}

/*

We have disabled the default exit behavior of the `WindowPlugin` because we want to have the app continue running when no windows are open.



/// Exit the application when the primary window has been closed
///
/// This system is added by the [`WindowPlugin`]
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_primary_closed(
    mut app_exit_events: EventWriter<AppExit>,
    windows: Query<(), (With<Window>, With<PrimaryWindow>)>,
) {
    if windows.is_empty() {
        log::info!("Primary window was closed, exiting");
        app_exit_events.write(AppExit::Success);
    }
}

*/
