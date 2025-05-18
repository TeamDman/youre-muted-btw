use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_global.run_if(input_just_pressed(KeyCode::Backquote)),
        );
        app.register_type::<Inspector>();
        app.init_resource::<Inspector>();
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Inspector {
    pub enabled: bool,
}

fn toggle_global(
    mut inspector: ResMut<Inspector>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    inspector.enabled ^= true;
    for mut window in &mut window {
        window.cursor_options.visible = inspector.enabled;
        window.decorations = inspector.enabled;
    }
}
