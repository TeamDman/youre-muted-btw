use crate::mic_icon::get_mic_icon;
use crate::wcslen;
use bevy::reflect::Reflect;
use image::RgbaImage;
use tracing::{debug, error, info}; // Added info
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
use windows::Win32::System::Com::CoUninitialize; // Added CoUninitialize
use windows::Win32::System::Com::CLSCTX_ALL;
use windows::Win32::System::Com::COINIT_MULTITHREADED;
use windows::Win32::System::Com::STGM_READ;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use ymb_windy::WindyResult;

#[derive(Debug, Clone, Reflect)]
pub struct MicInfo {
    pub is_default: bool,
    pub name: String,
    #[reflect(ignore)]
    pub icon: Option<RgbaImage>,
}

pub fn enumerate_mics_win() -> WindyResult<Vec<MicInfo>> {
    info!("Starting microphone enumeration (enumerate_mics_win)");
    unsafe {
        // CoInitializeEx should ideally be handled by the worker thread's setup.
        // If this function is only called from such a worker, this can be removed.
        // For now, keeping it for self-containment of the function.
        if let Err(e) = CoInitializeEx(None, COINIT_MULTITHREADED).ok() {
            error!("enumerate_mics_win: CoInitializeEx failed: {:?}", e);
            return Err(e.into());
        }
        debug!("enumerate_mics_win: CoInitializeEx successful.");
    }

    let result = unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        debug!("enumerate_mics_win: IMMDeviceEnumerator created.");

        let default_device = enumerator.GetDefaultAudioEndpoint(eCapture, ERole(1))?; // ERole(1) is eConsole
        debug!("enumerate_mics_win: Got default audio endpoint.");
        let default_id = default_device.GetId()?;
        debug!(
            "enumerate_mics_win: Got default device ID: {:?}",
            default_id.to_string()
        );

        let collection: IMMDeviceCollection =
            enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;
        let count = collection.GetCount()?;
        info!(
            "enumerate_mics_win: Found {} active capture devices.",
            count
        );

        let mut mics = Vec::new();
        for i in 0..count {
            debug!("enumerate_mics_win: Processing device index {}.", i);
            let device: IMMDevice = collection.Item(i)?;
            let id = device.GetId()?;
            let is_default = id == default_id;
            let props: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;

            let get_value_result = props.GetValue(
                &Properties::DEVPKEY_Device_FriendlyName as *const _
                    as *const windows::Win32::Foundation::PROPERTYKEY,
            );
            let name = if let Ok(propvar) = get_value_result {
                let pwstr = propvar.Anonymous.Anonymous.Anonymous.pwszVal;
                if !pwstr.is_null() {
                    let len = wcslen(pwstr.0);
                    String::from_utf16_lossy(std::slice::from_raw_parts(pwstr.0, len))
                } else {
                    "(Unknown Name)".to_string()
                }
            } else {
                "(Error Getting Name)".to_string()
            };
            debug!(
                "enumerate_mics_win: Device {}: Name='{}', Default={}",
                i, name, is_default
            );

            match get_mic_icon(&props, &name) {
                Ok(icon_option) => {
                    if icon_option.is_some() {
                        debug!("enumerate_mics_win: Successfully got icon for '{}'.", name);
                    } else {
                        debug!("enumerate_mics_win: No icon available for '{}'.", name);
                    }
                    mics.push(MicInfo {
                        is_default,
                        name,
                        icon: icon_option,
                    });
                }
                Err(e) => {
                    error!(
                        "enumerate_mics_win: Error getting icon for device '{}': {:?}",
                        name, e
                    );
                    mics.push(MicInfo {
                        // Push even if icon fails, so device list is complete
                        is_default,
                        name,
                        icon: None,
                    });
                }
            }
        }
        Ok(mics)
    };

    unsafe {
        CoUninitialize(); // Balance CoInitializeEx
        debug!("enumerate_mics_win: CoUninitialize called.");
    }
    info!(
        "Finished microphone enumeration. Found {} mics.",
        result.as_ref().map_or(0, |v| v.len())
    );
    result
}

#[cfg(test)]
mod test {
    use super::*;
    use ymb_args::GlobalArgs;
    use ymb_logs::setup_tracing;
    // Ensure MicInfo and enumerate_mics_win are in scope
    use ymb_windy::WindyResult; // Ensure WindyResult is in scope

    #[test]
    fn it_works() -> WindyResult<()> {
        // Setup tracing for tests
        setup_tracing(&GlobalArgs { debug: true }, std::io::stdout)?;

        let mics = enumerate_mics_win()?;
        assert!(!mics.is_empty(), "No microphones were enumerated.");
        for mic in mics {
            println!("Mic: '{}', Default: {}", mic.name, mic.is_default);
            if let Some(icon) = &mic.icon {
                println!(
                    "  Icon dimensions: {}x{}, First pixel RGBA: {:?}",
                    icon.width(),
                    icon.height(),
                    icon.get_pixel(0, 0) // Print first pixel to check data
                );
                // Optionally save the image to inspect it
                // icon.save(format!("{}.png", mic.name.replace("/", "_").replace("\\", "_"))).unwrap();
            } else {
                println!("  No icon available for this mic.");
            }
        }
        Ok(())
    }
}
