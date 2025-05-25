mod click_handling;
mod window_management;

use bevy::prelude::*;
use click_handling::ClickHandlingPlugin;
use window_management::WindowManagementPlugin;

pub struct TargetingCirclePlugin;

impl Plugin for TargetingCirclePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WindowManagementPlugin, ClickHandlingPlugin))
            .register_type::<TargetingCircleClicked>()
            .add_event::<TargetingCircleClicked>()
            .register_type::<TargetingCircleWindow>()
            .register_type::<TargetingCircleSprite>()
            .register_type::<TargetingState>();
    }
}

/// Event emitted when the targeting circle is clicked to select a position.
#[derive(Event, Debug, Clone, Copy, Reflect)]
pub struct TargetingCircleClicked {
    pub position: IVec2,
}

/// Component to mark the targeting circle window entity.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct TargetingCircleWindow;

/// Component to mark the targeting circle sprite entity.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct TargetingCircleSprite;

/// Component to mark the targeting circle text entity.
#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct TargetingCircleText;

/// Resource to hold the current state of the targeting circle.
#[derive(Resource, Debug, Reflect, Default)]
#[reflect(Resource)]
pub enum TargetingState {
    #[default]
    FollowingMouse, // Circle is white, window focused, follows mouse
    Paused, // Circle is red, window unfocused, fixed position
}
