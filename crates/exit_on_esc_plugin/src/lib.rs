use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct ExitOnEscPlugin;

impl Plugin for ExitOnEscPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, exit_on_esc);
    }
}
fn exit_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        info!("Exiting on ESC");
        for window in &window {
            commands.entity(window).despawn();
        }
    }
}
