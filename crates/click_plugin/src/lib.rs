use bevy::prelude::*;
use ymb_app_under_cursor_plugin::UnderCursor;
use ymb_assets::Sound;
use ymb_discord_app_plugin::MuteButtonIdentified;
use ymb_host_cursor_position_plugin::HostCursorPosition;
use ymb_windows_app_plugin::WindowsApp;
use ymb_windows_app_plugin::WindowsAppKind;

pub struct ClickPlugin;

impl Plugin for ClickPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_click);
    }
}

fn handle_click(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    apps: Query<(Entity, &WindowsApp, Option<&UnderCursor>)>,
    mouse_state: Res<ButtonInput<MouseButton>>,
    mut click_events: EventWriter<MuteButtonIdentified>,
    host_cursor_position: Res<HostCursorPosition>,
) {
    if !mouse_state.just_pressed(MouseButton::Left) {
        return;
    }
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
    if let Some(discord) = discord {
        click_events.write(MuteButtonIdentified {
            discord_app: discord,
            pos: host_cursor_position.position,
        });
    }
}
