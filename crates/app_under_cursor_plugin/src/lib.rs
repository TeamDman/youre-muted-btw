use bevy::prelude::*;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_windows_app::WindowsApp;

pub struct AppUnderCursorPlugin;

impl Plugin for AppUnderCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_app_under_cursor);
    }
}

#[derive(Component)]
pub struct UnderCursor;

fn update_app_under_cursor(
    mut commands: Commands,
    apps: Query<(Entity, &WindowsApp)>,
    host_cursor_position: Res<HostCursorPosition>,
) {
    for (entity, app) in apps.iter() {
        if app.bounds.contains(**host_cursor_position) {
            commands.entity(entity).insert(UnderCursor);
        } else {
            commands.entity(entity).remove::<UnderCursor>();
        }
    }
}
