use crate::WindowsApp;
use crate::get_apps;
use bevy::prelude::*;

pub struct WindowsAppPlugin;

impl Plugin for WindowsAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_apps);
        app.init_resource::<WindowsAppListPluginConfig>();
    }
}

#[derive(Resource)]
pub struct WindowsAppListPluginConfig {
    pub refresh_interval: Timer,
}
impl Default for WindowsAppListPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

fn update_apps(
    mut commands: Commands,
    mut apps: Query<(Entity, &WindowsApp)>,
    mut config: ResMut<WindowsAppListPluginConfig>,
    time: Res<Time>,
) -> Result {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return Ok(());
    }
    let fresh_apps = get_apps()?;
    for (entity, app) in apps.iter_mut() {
        if let Some(fresh_app) = fresh_apps.get(&app.id) {
            commands.entity(entity).insert(fresh_app.clone());
        } else {
            commands.entity(entity).despawn();
        }
    }
    Ok(())
}
