use crate::hicon_to_image::hicon_to_rgba;
use crate::wcslen;
use crate::windows_macros::MAKEINTRESOURCEW;
use image::RgbaImage;
use tracing::debug;
use tracing::error;
use tracing::warn;
use windows::core::GUID;
use windows::core::PCWSTR;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::WindowsAndMessaging::LoadImageW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::IMAGE_ICON;
use windows::Win32::UI::WindowsAndMessaging::LR_DEFAULTSIZE;
use windows::Win32::UI::WindowsAndMessaging::LR_LOADFROMFILE;
use windows::Win32::UI::WindowsAndMessaging::LR_SHARED;
use ymb_windy::error::WindyResult;

// DEVPKEY_Device_IconPath
const PKEY_DEVICE_ICON: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x259abffc_507a_4ce8_8c10_9640b8a1c907),
    pid: 10,
};

// DEVPKEY_DeviceClass_IconPath
const PKEY_DEVICE_CLASS_ICON: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x259abffc_507a_4ce8_8c10_9640b8a1c907),
    pid: 12,
};

// Helper to extract icon path string from PROPVARIANT
fn get_path_from_propvariant_internal(
    propvar: &PROPVARIANT,
    device_name: &str,
    prop_key_name: &str,
) -> Option<String> {
    unsafe {
        if propvar.Anonymous.Anonymous.vt == VT_LPWSTR {
            let path_pwstr = propvar.Anonymous.Anonymous.Anonymous.pwszVal;
            if !path_pwstr.is_null() {
                let len = wcslen(path_pwstr.0);
                let path_str =
                    String::from_utf16_lossy(std::slice::from_raw_parts(path_pwstr.0, len));
                debug!(
                    "Icon path string from {} for '{}': '{}'",
                    prop_key_name, device_name, path_str
                );
                Some(path_str)
            } else {
                debug!(
                    "Icon path string from {} for '{}' is null.",
                    prop_key_name, device_name
                );
                None
            }
        } else {
            debug!(
                "Property {} for '{}' is not VT_LPWSTR. Actual type: {:?}",
                prop_key_name, device_name, propvar.Anonymous.Anonymous.vt
            );
            None
        }
    }
}

/// Attempts to get an icon path string from device properties.
/// Tries PKEY_DEVICE_ICON first, then PKEY_DEVICE_CLASS_ICON as a fallback.
pub fn get_icon_path_from_properties(
    props: &IPropertyStore,
    device_name: &str,
) -> WindyResult<Option<String>> {
    debug!(
        "Attempting to get icon path from properties for device: {}",
        device_name
    );

    match unsafe { props.GetValue(&PKEY_DEVICE_ICON) } {
        Ok(propvar) => {
            if let Some(path_str) =
                get_path_from_propvariant_internal(&propvar, device_name, "PKEY_DEVICE_ICON")
            {
                return Ok(Some(path_str));
            }
        }
        Err(e) => {
            debug!(
                "Failed to get PKEY_DEVICE_ICON property for '{}': {:?}",
                device_name, e
            );
            // Continue to try class icon
        }
    }

    debug!(
        "PKEY_DEVICE_ICON did not yield a path for '{}'. Trying PKEY_DEVICE_CLASS_ICON.",
        device_name
    );
    match unsafe { props.GetValue(&PKEY_DEVICE_CLASS_ICON) } {
        Ok(propvar) => {
            if let Some(path_str) =
                get_path_from_propvariant_internal(&propvar, device_name, "PKEY_DEVICE_CLASS_ICON")
            {
                return Ok(Some(path_str));
            }
        }
        Err(e) => {
            debug!(
                "Failed to get PKEY_DEVICE_CLASS_ICON property for '{}': {:?}",
                device_name, e
            );
        }
    }

    debug!(
        "No icon path found from properties for '{}' after trying device and class keys.",
        device_name
    );
    Ok(None)
}

/// Loads an RgbaImage from a given icon path string (e.g., "dll,-resourceId" or "file.ico").
pub fn load_image_from_icon_path_string(
    icon_path_str: &str,
    device_name_for_logging: &str,
) -> WindyResult<Option<RgbaImage>> {
    debug!(
        "Attempting to load icon from path string for '{}': {}",
        device_name_for_logging, icon_path_str
    );

    let mut icon_rgba_data: Option<Vec<u8>> = None;
    let mut icon_width = 0;
    let mut icon_height = 0;

    let parts: Vec<&str> = icon_path_str.split(",-").collect();
    if parts.len() == 2 {
        let mut dll_path_str = parts[0].replace(
            "%SystemRoot%",
            &std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()),
        );
        // Handle cases like "@%SystemRoot%\System32\drivers\usbaudio.sys"
        if dll_path_str.starts_with('@') {
            dll_path_str.remove(0);
        }

        debug!(
            "Parsed DLL path for {}: '{}'",
            device_name_for_logging, dll_path_str
        );

        if let Ok(resource_id) = parts[1].parse::<u16>() {
            debug!(
                "Parsed resource ID for {}: {}",
                device_name_for_logging, resource_id
            );
            let mut dll_path_u16: Vec<u16> = dll_path_str.encode_utf16().collect();
            dll_path_u16.push(0); // Null terminate

            unsafe {
                // GetModuleHandleW only works if the DLL is already loaded.
                // LoadLibraryW ensures it's loaded and increments ref count.
                // We should use LoadLibraryEx with LOAD_LIBRARY_AS_DATAFILE for icons.
                let h_module = LoadLibraryW(PCWSTR(dll_path_u16.as_ptr()));

                if h_module.is_ok() && !h_module.as_ref().unwrap().is_invalid() {
                    let h_module_ok = h_module.unwrap();
                    let hicon_handle = LoadImageW(
                        Some(h_module_ok.into()),
                        MAKEINTRESOURCEW(resource_id),
                        IMAGE_ICON,
                        0,
                        0,
                        LR_DEFAULTSIZE | LR_SHARED,
                    );

                    match hicon_handle {
                        Ok(hicon) if !hicon.is_invalid() => {
                            debug!(
                                "Successfully loaded HICON from resource for {}: {:?}",
                                device_name_for_logging, hicon
                            );
                            match hicon_to_rgba(&HICON(hicon.0)) {
                                Ok((rgba, w, h)) => {
                                    icon_rgba_data = Some(rgba);
                                    icon_width = w;
                                    icon_height = h;
                                    debug!(
                                        "Converted HICON to RGBA for {}: {}x{}",
                                        device_name_for_logging, w, h
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to convert HICON (resource) to RGBA for {}: {:?}",
                                        device_name_for_logging, e
                                    );
                                }
                            }
                            // HICON from LoadImageW with LR_SHARED should not be destroyed by us
                            // if it's a shared icon. If not LR_SHARED, DestroyIcon would be needed.
                            // For icons loaded from resources, they are often shared.
                        }
                        Ok(invalid_hicon) => {
                            warn!("LoadImageW from resource for {} returned an invalid HICON: {:?}. OS Error: {:?}", device_name_for_logging, invalid_hicon, windows::core::Error::from_win32());
                        }
                        Err(e) => {
                            warn!(
                                "LoadImageW from resource for {} failed: {:?}",
                                device_name_for_logging, e
                            );
                        }
                    }
                    // We loaded it, we should free it.
                    // windows::Win32::System::LibraryLoader::FreeLibrary(h_module_ok);
                    // Actually, for icons, often LOAD_LIBRARY_AS_DATAFILE is better,
                    // and then FreeLibrary. Or rely on LR_SHARED and system managing it.
                    // Given LoadImageW might share, let's be careful.
                    // If LoadImageW uses LR_SHARED, the system manages the icon.
                    // The module itself, if we LoadLibrary'd it, we should FreeLibrary.
                    // For now, let's assume mmres.dll is persistent.
                    // If loading arbitrary DLLs, FreeLibrary(h_module_ok) would be important.
                } else {
                    let err = windows::core::Error::from_win32();
                    warn!(
                        "LoadLibraryW for DLL '{}' for device {} failed or returned invalid handle. Error: {:?}",
                        dll_path_str, device_name_for_logging, err
                    );
                }
            }
        } else {
            warn!(
                "Failed to parse resource ID from '{}' for device {}",
                parts[1], device_name_for_logging
            );
        }
    } else if icon_path_str.to_lowercase().ends_with(".ico") {
        debug!(
            "Icon path for {} appears to be a direct .ico file: '{}'",
            device_name_for_logging, icon_path_str
        );
        let mut path_u16: Vec<u16> = icon_path_str.encode_utf16().collect();
        path_u16.push(0);

        unsafe {
            let hicon_handle = LoadImageW(
                None,
                PCWSTR(path_u16.as_ptr()),
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTSIZE | LR_SHARED | LR_LOADFROMFILE,
            );
            match hicon_handle {
                Ok(hicon) if !hicon.is_invalid() => {
                    debug!(
                        "Successfully loaded HICON from file for {}: {:?}",
                        device_name_for_logging, hicon
                    );
                    match hicon_to_rgba(&HICON(hicon.0)) {
                        Ok((rgba, w, h)) => {
                            icon_rgba_data = Some(rgba);
                            icon_width = w;
                            icon_height = h;
                        }
                        Err(e) => {
                            error!(
                                "Failed to convert HICON (file) to RGBA for {}: {:?}",
                                device_name_for_logging, e
                            );
                        }
                    }
                    // If LR_SHARED is used, system manages it. Otherwise DestroyIcon(hicon).
                    // For LR_LOADFROMFILE | LR_SHARED, it's usually fine.
                }
                Ok(invalid_hicon) => {
                    warn!("LoadImageW from file for {} returned an invalid HICON: {:?}. OS Error: {:?}", device_name_for_logging, invalid_hicon, windows::core::Error::from_win32());
                }
                Err(e) => {
                    warn!(
                        "LoadImageW from file for {} failed: {:?}",
                        device_name_for_logging, e
                    );
                }
            }
        }
    } else {
        warn!(
            "Unrecognized icon path format for {}: '{}'",
            device_name_for_logging, icon_path_str
        );
    }

    if let Some(rgba) = icon_rgba_data {
        if icon_width > 0 && icon_height > 0 {
            match RgbaImage::from_raw(icon_width, icon_height, rgba) {
                Some(image) => {
                    debug!(
                        "Successfully created RgbaImage for {}",
                        device_name_for_logging
                    );
                    return Ok(Some(image));
                }
                None => {
                    error!(
                        "RgbaImage::from_raw failed for {} ({}x{}) despite having data.",
                        device_name_for_logging, icon_width, icon_height
                    );
                }
            }
        } else {
            warn!(
                "Icon for {} had data but invalid dimensions: {}x{}",
                device_name_for_logging, icon_width, icon_height
            );
        }
    }
    Ok(None)
}
