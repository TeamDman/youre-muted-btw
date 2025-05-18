use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use ymb_host_cursor_position_plugin::HostCursorPosition;

pub struct PositionWindowPlugin;
impl Plugin for PositionWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, position_window);
    }
}
fn position_window(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut tick: Local<usize>,
    host_cursor_position: Res<HostCursorPosition>,
) -> bevy::ecs::error::Result<()> {
    *tick += 1;
    for mut window in &mut windows {
        let window_size = window.size();
        let offset = 0.0; //((*tick as f32) * 0.1).sin() * 100.0;
        let window_center = window_size / 2.0;
        let new_window_position = host_cursor_position.as_vec2() - window_center + offset;
        window.position = WindowPosition::At(new_window_position.as_ivec2());
    }
    Ok(())
}
