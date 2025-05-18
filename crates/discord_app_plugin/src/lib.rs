use bevy::prelude::*;

pub struct DiscordAppPlugin;

impl Plugin for DiscordAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_host_cursor_position);
        app.init_resource::<DiscordAppPluginPluginConfig>();
    }
}

#[derive(Resource)]
pub struct DiscordAppPluginPluginConfig {
    pub refresh_interval: Timer,
}
impl Default for DiscordAppPluginPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::from_seconds(0.01, TimerMode::Repeating),
        }
    }
}

fn update_host_cursor_position(mut config: ResMut<DiscordAppPluginPluginConfig>, time: Res<Time>) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    info!("Checking discord app position");
}
