use bevy::prelude::*;
use bevy::reflect::Reflect;
use serde::Deserialize;
use serde::Serialize;
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

#[cfg(target_os = "windows")]
fn enumerate_mics_win() -> Result<Vec<MicInfo>, bevy::prelude::BevyError> {
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
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err(bevy::prelude::BevyError::from("CoInitializeEx failed"));
        }
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(|e| {
                bevy::prelude::BevyError::from(format!("CoCreateInstance failed: {e}"))
            })?;
        let default_device = enumerator
            .GetDefaultAudioEndpoint(eCapture, ERole(1))
            .map_err(|e| {
                bevy::prelude::BevyError::from(format!("GetDefaultAudioEndpoint failed: {e}"))
            })?;
        let default_id = default_device
            .GetId()
            .map_err(|e| bevy::prelude::BevyError::from(format!("GetId failed: {e}")))?;
        let collection: IMMDeviceCollection = enumerator
            .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
            .map_err(|e| {
                bevy::prelude::BevyError::from(format!("EnumAudioEndpoints failed: {e}"))
            })?;
        let count = collection
            .GetCount()
            .map_err(|e| bevy::prelude::BevyError::from(format!("GetCount failed: {e}")))?;
        let mut mics = Vec::new();
        for i in 0..count {
            let device: IMMDevice = collection
                .Item(i)
                .map_err(|e| bevy::prelude::BevyError::from(format!("Item failed: {e}")))?;
            let id = device
                .GetId()
                .map_err(|e| bevy::prelude::BevyError::from(format!("GetId failed: {e}")))?;
            let is_default = id == default_id;
            let props: IPropertyStore = device.OpenPropertyStore(STGM_READ).map_err(|e| {
                bevy::prelude::BevyError::from(format!("OpenPropertyStore failed: {e}"))
            })?;
            // Use the correct type for the property key: DEVPROPKEY is compatible with PROPERTYKEY (same layout)
            let key = &Properties::DEVPKEY_Device_FriendlyName as *const _
                as *const windows::Win32::Foundation::PROPERTYKEY;
            let get_value_result = props.GetValue(key);
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

#[cfg(not(target_os = "windows"))]
fn enumerate_mics_win() -> Result<Vec<MicInfo>, bevy::prelude::BevyError> {
    Ok(vec![])
}

unsafe fn wcslen(mut ptr: *const u16) -> usize {
    let mut len = 0;
    while *ptr != 0 {
        len += 1;
        ptr = ptr.add(1);
    }
    len
}
