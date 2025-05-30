use bevy::prelude::*;
use bevy::reflect::Reflect;
use serde::Deserialize;
use serde::Serialize;
use ymb_windy::WindyResult;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

#[derive(Debug, Clone, Serialize, Deserialize, Event, Reflect)]
pub enum MicDetectionThreadboundMessage {
    EnumerateMics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event, Reflect)]
pub struct MicInfo {
    pub is_default: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Event, Reflect)]
pub enum MicDetectionGameboundMessage {
    MicsEnumerated(Vec<MicInfo>),
}

#[derive(Component, Debug, Clone)]
pub struct Mic {
    pub is_default: bool,
    pub name: String,
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
                commands.spawn((
                    Mic {
                        is_default: mic.is_default,
                        name: mic.name.clone(),
                    },
                    Name::new(mic.name.clone()),
                ));
            }
        }
    }
}

fn enumerate_mics_win() -> WindyResult<Vec<MicInfo>> {
    use windows::Win32::Devices::Properties;
    use windows::Win32::Media::Audio::eCapture;
    use windows::Win32::Media::Audio::ERole;
    use windows::Win32::Media::Audio::IMMDevice;
    use windows::Win32::Media::Audio::IMMDeviceCollection;
    use windows::Win32::Media::Audio::IMMDeviceEnumerator;
    use windows::Win32::Media::Audio::MMDeviceEnumerator;
    use windows::Win32::Media::Audio::DEVICE_STATE_ACTIVE;
    use windows::Win32::System::Com::CoCreateInstance;
    use windows::Win32::System::Com::CoInitializeEx;
    use windows::Win32::System::Com::CLSCTX_ALL;
    use windows::Win32::System::Com::COINIT_MULTITHREADED;
    use windows::Win32::System::Com::STGM_READ;
    use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let default_device = enumerator.GetDefaultAudioEndpoint(eCapture, ERole(1))?;
        let default_id = default_device.GetId()?;
        let collection: IMMDeviceCollection =
            enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
        let count = collection.GetCount()?;
        let mut mics = Vec::new();
        for i in 0..count {
            let device: IMMDevice = collection.Item(i)?;
            let id = device.GetId()?;
            let is_default = id == default_id;
            let props: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
            // Use the correct type for the property key: DEVPROPKEY is compatible with PROPERTYKEY (same layout)
            let get_value_result = props.GetValue(
                &Properties::DEVPKEY_Device_FriendlyName as *const _
                    as *const windows::Win32::Foundation::PROPERTYKEY,
            );
            let name = if let Ok(propvar) = get_value_result {
                // propvar.Anonymous.Anonymous.pwszVal is a PWSTR, use .0 to get *const u16
                let pwstr = propvar.Anonymous.Anonymous.Anonymous.pwszVal.0;
                if !pwstr.is_null() {
                    let len = wcslen(pwstr);
                    String::from_utf16_lossy(std::slice::from_raw_parts(pwstr, len))
                } else {
                    "(Unknown)".to_string()
                }
            } else {
                "(Unknown)".to_string()
            };
            mics.push(MicInfo { is_default, name });
        }
        Ok(mics)
    }
}

unsafe fn wcslen(mut ptr: *const u16) -> usize {
    let mut len = 0;
    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }
    len
}
