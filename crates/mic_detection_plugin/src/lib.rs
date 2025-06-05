pub mod mic_icon;
pub mod mic_list;
pub mod windows_macros;
pub mod hicon_to_image;
pub mod icon_path;
pub mod load_icon_from_dll;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::reflect::Reflect;
use image::DynamicImage;
use mic_list::enumerate_mics_win;
use mic_list::MicInfo;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

#[derive(Debug, Clone, Event, Reflect)]
pub enum MicDetectionThreadboundMessage {
    EnumerateMics,
}


#[derive(Debug, Clone, Event, Reflect)]
pub enum MicDetectionGameboundMessage {
    MicsEnumerated(Vec<MicInfo>),
}

#[derive(Component, Debug, Clone)]
pub struct Mic {
    pub is_default: bool,
    pub name: String,
    pub icon_handle: Option<Handle<Image>>,
}

#[derive(Default)]
pub struct MicDetectionState;

pub struct MicDetectionPlugin;

impl Plugin for MicDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MicDetectionThreadboundMessage>();
        app.register_type::<MicDetectionGameboundMessage>();
        app.register_type::<MicInfo>();
        app.add_plugins(WorkerPlugin {
            config: WorkerConfig::<
                MicDetectionThreadboundMessage,
                MicDetectionGameboundMessage,
                MicDetectionState,
            > {
                name: "MicDetectionWorker".to_string(),
                handle_threadbound_message,
                ..default()
            },
        });
        app.add_systems(Startup, trigger_enumerate_mics);
        app.add_systems(Update, handle_mics_enumerated);
    }
}

fn trigger_enumerate_mics(mut writer: EventWriter<MicDetectionThreadboundMessage>) {
    writer.write(MicDetectionThreadboundMessage::EnumerateMics);
}

fn handle_threadbound_message(
    msg: &MicDetectionThreadboundMessage,
    reply_tx: &Sender<MicDetectionGameboundMessage>,
    _state: &mut MicDetectionState,
) -> Result {
    match msg {
        MicDetectionThreadboundMessage::EnumerateMics => {
            let mics = enumerate_mics_win()?;
            reply_tx.send(MicDetectionGameboundMessage::MicsEnumerated(mics))?;
        }
    }
    Ok(())
}

fn handle_mics_enumerated(
    mut events: EventReader<MicDetectionGameboundMessage>,
    mut commands: Commands,
    mut query: Query<&mut Mic>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        let MicDetectionGameboundMessage::MicsEnumerated(mics) = event;
        for mic in mics {
            let mut found = false;
            for mut existing in query.iter_mut() {
                if existing.name == mic.name {
                    existing.is_default = mic.is_default;
                    found = true;
                    break;
                }
            }
            if !found {
                let image = if let Some(icon) = &mic.icon {
                    textures.add(Image::from_dynamic(
                        DynamicImage::ImageRgba8(icon.clone()),
                        true,
                        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                    ))
                } else {
                    textures.add(Image::default())
                };
                commands.spawn((
                    Mic {
                        is_default: mic.is_default,
                        name: mic.name.clone(),
                        icon_handle: None, // Icon handling can be added later
                    },
                    Sprite { image, ..default() },
                    Name::new(mic.name.clone()),
                ));
            }
        }
    }
}

pub unsafe fn wcslen(mut ptr: *const u16) -> usize {
    let mut len = 0;
    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }
    len
}
