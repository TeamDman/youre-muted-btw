use crate::mic_icon::get_mic_icon;
use crate::wcslen;
use bevy::reflect::Reflect;
use image::RgbaImage;
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
use ymb_windy::WindyResult;

#[derive(Debug, Clone, Reflect)]
pub struct MicInfo {
    pub is_default: bool,
    pub name: String,
    #[reflect(ignore)]
    pub icon: Option<RgbaImage>,
}

pub fn enumerate_mics_win() -> WindyResult<Vec<MicInfo>> {
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

            let icon = get_mic_icon(&props, &name)?;

            mics.push(MicInfo {
                is_default,
                name,
                icon,
            });
        }
        Ok(mics)
    }
}

#[cfg(test)]
mod test {
    use ymb_args::GlobalArgs;
    use ymb_logs::setup_tracing;
    use ymb_windy::WindyResult;

    #[test]
    fn it_works() -> WindyResult {
        setup_tracing(&GlobalArgs { debug: true }, std::io::stdout)?;
        let mics = super::enumerate_mics_win()?;
        for mic in mics {
            println!("Mic: {}, Default: {}", mic.name, mic.is_default);
            if let Some(icon) = &mic.icon {
                println!("Icon size: {}x{}", icon.width(), icon.height());
            } else {
                println!("No icon available");
            }
        }
        Ok(())
    }
}
