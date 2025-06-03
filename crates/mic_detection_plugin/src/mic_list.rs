use crate::mic_icon; // Import the module
use crate::wcslen;
use bevy::reflect::Reflect;
use image::RgbaImage;
use tracing::{debug, error, info, warn}; // Added warn
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
use windows::Win32::System::Com::CoUninitialize;
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

// Define the generic microphone icon path (mmres.dll,-3012 is a common one)
const GENERIC_MIC_ICON_PATH: &str = "%SystemRoot%\\system32\\mmres.dll,-3012";

pub fn enumerate_mics_win() -> WindyResult<Vec<MicInfo>> {
    info!("Starting microphone enumeration (enumerate_mics_win)");
    unsafe {
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

        let default_device = enumerator.GetDefaultAudioEndpoint(eCapture, ERole(1))?;
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

            let mut final_icon_option: Option<RgbaImage> = None;

            // 1. Try getting icon path from properties
            match mic_icon::get_icon_path_from_properties(&props, &name) {
                Ok(Some(path_str)) => {
                    debug!(
                        "Found icon path from properties for '{}': {}",
                        name, path_str
                    );
                    match mic_icon::load_image_from_icon_path_string(&path_str, &name) {
                        Ok(img_opt) => {
                            if img_opt.is_some() {
                                debug!("Successfully loaded icon from property path for '{}'", name);
                            } else {
                                warn!("Property path '{}' did not yield an image for '{}'. Will try generic.", path_str, name);
                            }
                            final_icon_option = img_opt;
                        }
                        Err(e) => {
                            error!(
                                "Error loading icon from property path '{}' for '{}': {:?}. Will try generic.",
                                path_str, name, e
                            );
                        }
                    }
                }
                Ok(None) => {
                    debug!(
                        "No icon path found from properties for '{}'. Will try generic mmres.dll icon.",
                        name
                    );
                    // final_icon_option remains None, will proceed to generic below
                }
                Err(e) => {
                    error!(
                        "Error getting icon path from properties for '{}': {:?}. Will try generic.",
                        name, e
                    );
                    // final_icon_option remains None, will proceed to generic below
                }
            }

            // 2. If no icon from properties, try generic mmres.dll icon
            if final_icon_option.is_none() {
                debug!(
                    "Attempting to load generic mmres.dll icon for '{}' using path: {}",
                    name, GENERIC_MIC_ICON_PATH
                );
                match mic_icon::load_image_from_icon_path_string(GENERIC_MIC_ICON_PATH, &name)
                {
                    Ok(img_opt) => {
                        if img_opt.is_some() {
                            info!(
                                "Successfully loaded generic mmres.dll icon for '{}'",
                                name
                            );
                        } else {
                            warn!(
                                "Generic mmres.dll icon path did not yield an image for '{}'. No icon will be used.",
                                name
                            );
                        }
                        final_icon_option = img_opt;
                    }
                    Err(e) => {
                        error!(
                            "Error loading generic mmres.dll icon for '{}': {:?}",
                            name, e
                        );
                        // final_icon_option remains None
                    }
                }
            }

            if final_icon_option.is_some() {
                debug!("enumerate_mics_win: Successfully got an icon for '{}'.", name);
            } else {
                warn!("enumerate_mics_win: No icon could be loaded for '{}'.", name);
            }

            mics.push(MicInfo {
                is_default,
                name,
                icon: final_icon_option,
            });
        }
        Ok(mics)
    };

    unsafe {
        CoUninitialize();
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
    use ymb_windy::WindyResult;

    #[test]
    fn it_works() -> WindyResult<()> {
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
                    icon.get_pixel(0, 0)
                );
                 // Optionally save the image to inspect it
                 // let safe_name = mic.name.replace(|c: char| !c.is_alphanumeric(), "_");
                 // icon.save(format!("{}.png", safe_name)).unwrap();
            } else {
                println!("  No icon available for this mic (after all fallbacks).");
            }
        }
        Ok(())
    }
}