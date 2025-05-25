use bevy::prelude::*;
use ymb_app_under_cursor_plugin::UnderCursor;
use ymb_assets::Sound;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_windows_app_plugin::WindowsApp;
use ymb_windows_app_plugin::WindowsAppKind;

use crate::TargetingCircleClicked;
use crate::TargetingState;

pub(super) struct ClickHandlingPlugin;

impl Plugin for ClickHandlingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_mouse_click_system);
    }
}

fn handle_mouse_click_system(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut targeting_state: ResMut<TargetingState>,
    host_cursor_position: Res<HostCursorPosition>,
    mut clicked_event_writer: EventWriter<TargetingCircleClicked>,
    apps: Query<(Entity, &WindowsApp, Option<&UnderCursor>)>,
    asset_server: Res<AssetServer>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }
    match *targeting_state {
        TargetingState::FollowingMouse => {
            *targeting_state = TargetingState::Paused;
            clicked_event_writer.write(TargetingCircleClicked {
                position: host_cursor_position.position,
            });
            info!(
                "Targeting circle clicked (now Paused). Position: {:?}",
                host_cursor_position.position
            );

            let discord = {
                let mut rtn = None;
                for (entity, app, under_cursor) in apps.iter() {
                    if app.kind == WindowsAppKind::Discord && under_cursor.is_some() {
                        rtn = Some(entity);
                        break;
                    }
                }
                rtn
            };
            commands.spawn((
                AudioPlayer::new(asset_server.load(if discord.is_some() {
                    Sound::SimpleTone
                } else {
                    Sound::ShortSoft
                })),
                PlaybackSettings::DESPAWN,
                Name::new("Click Sound"),
            ));
        }
        TargetingState::Paused => {
            *targeting_state = TargetingState::FollowingMouse;
            info!("Targeting circle clicked (now FollowingMouse).");
            // Per prompt, event is emitted when it *enters* paused state.
        }
    }
}
