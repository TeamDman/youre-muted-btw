use bevy::prelude::*;
use ymb_app_under_cursor_plugin::UnderCursor;
use ymb_assets::Sound;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_ui_automation_plugin::UIWorkerThreadboundMessage;
use ymb_windows_app_plugin::WindowsApp;
use ymb_windows_app_plugin::WindowsAppKind;

use crate::TargetingCircleClicked;
use crate::TargetingCircleWindow;
use crate::TargetingState;

pub(super) struct ClickHandlingPlugin;

impl Plugin for ClickHandlingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_click);
        app.add_systems(Update, forward_click_events);
        app.add_systems(Update, play_sound);
    }
}

fn handle_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut targeting_state: ResMut<TargetingState>,
    host_cursor_position: Res<HostCursorPosition>,
    mut clicked_event_writer: EventWriter<TargetingCircleClicked>,
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
        }
        TargetingState::Paused => {
            *targeting_state = TargetingState::FollowingMouse;
            info!("Targeting circle clicked (now FollowingMouse).");
        }
    }
}

fn play_sound(
    mut commands: Commands,
    mut receiver: EventReader<TargetingCircleClicked>,
    asset_server: Res<AssetServer>,
    apps: Query<(Entity, &WindowsApp, Option<&UnderCursor>)>,
) {
    for _ in receiver.read() {
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
}

fn forward_click_events(
    mut receiver: EventReader<TargetingCircleClicked>,
    mut writer: EventWriter<UIWorkerThreadboundMessage>,
) {
    for event in receiver.read() {
        writer.write(UIWorkerThreadboundMessage::UpdateFromPos {
            host_cursor_position: event.position,
        });
        info!("Forwarded TargetingCircleClicked event: {:?}", event);
    }
}
