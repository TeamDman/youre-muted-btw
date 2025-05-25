use bevy::prelude::*;
use holda::Holda;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
use ymb_windy::WindyResult;

pub struct HostCursorPositionPlugin;

impl Plugin for HostCursorPositionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_host_cursor_position);
        app.init_resource::<HostCursorPosition>();
        app.init_resource::<HostCursorPositionPluginConfig>();
        app.register_type::<HostCursorPosition>();
        app.register_type::<HostCursorPositionPluginConfig>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct HostCursorPositionPluginConfig {
    pub refresh_interval: Timer,
}
impl Default for HostCursorPositionPluginConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Timer::from_seconds(0.01, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Holda, Default, Reflect)]
#[holda(NoSerde)]
#[holda(NoOrd)]
#[reflect(Resource)]
pub struct HostCursorPosition {
    pub position: IVec2,
}

fn update_host_cursor_position(
    mut host_cursor_position: ResMut<HostCursorPosition>,
    mut config: ResMut<HostCursorPositionPluginConfig>,
    time: Res<Time>,
) {
    config.refresh_interval.tick(time.delta());
    if !config.refresh_interval.just_finished() {
        return;
    }
    if let Ok(position) = get_host_cursor_position() {
        host_cursor_position.position = position;
    }
}

/// This may fail if, for example, the cursor is hovering over an Admin-level task manager window
pub fn get_host_cursor_position() -> WindyResult<IVec2> {
    unsafe {
        let mut point = POINT::default();
        GetCursorPos(&mut point)?;
        Ok(IVec2::new(point.x, point.y))
    }
}
