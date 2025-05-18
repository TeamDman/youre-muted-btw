use bevy::prelude::*;
use holda::Holda;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use ymb_windy::WindyResult;

pub struct DiscordAppPluginPlugin;

impl Plugin for DiscordAppPluginPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_host_cursor_position);
        app.init_resource::<DiscordAppPlugin>();
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

#[derive(Resource, Holda, Default)]
#[holda(NoSerde)]
#[holda(NoOrd)]
pub struct DiscordAppPlugin {
    pub position: IVec2,
}

fn update_host_cursor_position(
    mut host_cursor_position: ResMut<DiscordAppPlugin>,
    mut config: ResMut<DiscordAppPluginPluginConfig>,
    time: Res<Time>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    host_cursor_position.position = get_host_cursor_position().unwrap();
}

pub fn get_host_cursor_position() -> WindyResult<IVec2> {
    unsafe {
        let mut point = POINT::default();
        GetCursorPos(&mut point)?;
        Ok(IVec2::new(point.x, point.y))
    }
}
