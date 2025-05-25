use std::sync::Mutex;
use bevy::ecs::schedule::BoxedCondition;
use bevy::prelude::*;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_targetting_window_plugin::TargettingWindow;

#[derive(Default)]
pub struct WindowPositionPlugin {
    condition: Mutex<Option<BoxedCondition>>,
}
impl WindowPositionPlugin {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn run_if<M>(mut self, condition: impl Condition<M>) -> Self {
        let condition_system = IntoSystem::into_system(condition);
        self.condition = Mutex::new(Some(Box::new(condition_system) as BoxedCondition));
        self
    }
}

impl Plugin for WindowPositionPlugin {
    fn build(&self, app: &mut App) {
        let condition = self.condition.lock().unwrap().take();
        let mut system = position_window.into_configs();
        if let Some(condition) = condition {
            system.run_if_dyn(condition);
        }
        app.add_systems(Update, system);
    }
}
fn position_window(
    mut windows: Query<&mut Window, With<TargettingWindow>>,
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
