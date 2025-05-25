use bevy::prelude::*;
use bevy_egui::EguiGlobalSettings;
use bevy_egui::EguiPlugin;

pub struct YMBEguiPlugin;

impl Plugin for YMBEguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        });
        app.add_systems(Startup, |mut config: ResMut<EguiGlobalSettings>| {
            config.enable_absorb_bevy_input_system = true
        });
    }
}
