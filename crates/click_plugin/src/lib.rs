use bevy::prelude::*;
use ymb_app_under_cursor_plugin::UnderCursor;
use ymb_assets::Sound;
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
    apps: Query<(&WindowsApp, Option<&UnderCursor>)>,
) {
    let is_discord_under_cursor = {
        let mut rtn = false;
        for (app, under_cursor) in apps.iter() {
            if app.kind == WindowsAppKind::Discord && under_cursor.is_some() {
                rtn = true;
                break;
            }
        }
        rtn
    };
    commands.spawn(AudioPlayer::new(asset_server.load(
        if is_discord_under_cursor {
            Sound::SimpleTone
        } else {
            Sound::ShortSoft
        },
    )));
}
