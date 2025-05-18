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
    }
}

#[derive(Resource)]
pub struct HostCursorPositionPluginConfig {
    pub timer: Timer,
}
impl Default for HostCursorPositionPluginConfig {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.01, TimerMode::Repeating),
        }
    }
}

#[derive(Resource, Holda, Default)]
#[holda(NoSerde)]
#[holda(NoOrd)]
pub struct HostCursorPosition {
    pub position: IVec2,
}

fn update_host_cursor_position(
    mut host_cursor_position: ResMut<HostCursorPosition>,
    mut config: ResMut<HostCursorPositionPluginConfig>,
    time: Res<Time>,
) {
    config.timer.tick(time.delta());
    if !config.timer.just_finished() {
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
