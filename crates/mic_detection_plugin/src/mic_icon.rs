use crate::wcslen;
use eyre::eyre;
use image::RgbaImage;
use tracing::{debug, error, warn}; // Added tracing
use windows::core::{GUID, PCWSTR}; // Removed unused PWSTR
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Graphics::Gdi::BI_RGB;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;
use windows::Win32::UI::WindowsAndMessaging::LoadImageW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFO;
use windows::Win32::UI::WindowsAndMessaging::IMAGE_ICON;
use windows::Win32::UI::WindowsAndMessaging::LR_DEFAULTSIZE;
use windows::Win32::UI::WindowsAndMessaging::LR_LOADFROMFILE;
use windows::Win32::UI::WindowsAndMessaging::LR_SHARED;
use ymb_windy::WindyResult;

const PKEY_DEVICE_ICON: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x259abffc_507a_4ce8_8c10_9640b8a1c907),
    pid: 10,
};

pub fn get_mic_icon(props: &IPropertyStore, name: &str) -> WindyResult<Option<RgbaImage>> {
    debug!("Attempting to get icon for device: {}", name);
    let mut icon_rgba_data: Option<Vec<u8>> = None;
    let mut icon_width = 0;
    let mut icon_height = 0;

    let get_icon_prop_result =
        unsafe { props.GetValue(&PKEY_DEVICE_ICON as *const _ as *const PROPERTYKEY) };

    match get_icon_prop_result {
        Ok(propvar) => {
            debug!(
                "Successfully retrieved PKEY_DEVICE_ICON property for {}",
                name
            );
            // Ensure it's a string type (VT_LPWSTR)
            if unsafe { propvar.Anonymous.Anonymous.vt } == VT_LPWSTR {
                let icon_path_pwstr = unsafe { propvar.Anonymous.Anonymous.Anonymous.pwszVal };
                if !icon_path_pwstr.is_null() {
                    let len = unsafe { wcslen(icon_path_pwstr.0) };
                    let path_str = unsafe {
                        String::from_utf16_lossy(std::slice::from_raw_parts(icon_path_pwstr.0, len))
                    };
                    debug!("Icon path string for {}: '{}'", name, path_str);

                    let parts: Vec<&str> = path_str.split(",-").collect();
                    if parts.len() == 2 {
                        let dll_path_str = parts[0].replace(
                            "%SystemRoot%",
                            &std::env::var("SystemRoot")
                                .unwrap_or_else(|_| "C:\\Windows".to_string()),
                        );
                        debug!("Parsed DLL path for {}: '{}'", name, dll_path_str);

                        if let Ok(resource_id) = parts[1].parse::<i32>() {
                            debug!("Parsed resource ID for {}: {}", name, resource_id);

                            let mut dll_path_u16: Vec<u16> = dll_path_str.encode_utf16().collect();
                            dll_path_u16.push(0); // Null terminate

                            unsafe {
                                // Fix use of moved value: clone h_module before unwrap
                                let h_module_result =
                                    GetModuleHandleW(PCWSTR(dll_path_u16.as_ptr()));
                                if h_module_result.is_err()
                                    || h_module_result.as_ref().unwrap().is_invalid()
                                {
                                    warn!(
                                        "Failed to get module handle for DLL '{}' for device {}: {:?}",
                                        dll_path_str, name, h_module_result.err()
                                    );
                                } else {
                                    let h_module_ok = h_module_result.unwrap();
                                    let hicon_handle = LoadImageW(
                                        Some(std::mem::transmute(h_module_ok)),
                                        MAKEINTRESOURCEW(resource_id as u16),
                                        IMAGE_ICON,
                                        0,
                                        0,
                                        LR_DEFAULTSIZE | LR_SHARED,
                                    );

                                    match hicon_handle {
                                        Ok(hicon) if !hicon.is_invalid() => {
                                            debug!("Successfully loaded HICON from resource for {}: {:?}", name, hicon);
                                            match hicon_to_rgba(&HICON(hicon.0)) {
                                                Ok((rgba, w, h)) => {
                                                    icon_rgba_data = Some(rgba);
                                                    icon_width = w;
                                                    icon_height = h;
                                                    debug!(
                                                        "Converted HICON to RGBA for {}: {}x{}",
                                                        name, w, h
                                                    );
                                                }
                                                Err(e) => {
                                                    error!("Failed to convert HICON (resource) to RGBA for {}: {:?}", name, e);
                                                }
                                            }
                                        }
                                        Ok(invalid_hicon) => {
                                            // Valid handle but is_invalid() is true
                                            warn!("LoadImageW from resource for {} returned an invalid HICON: {:?}. Error: {:?}", name, invalid_hicon, windows::core::Error::from_win32());
                                        }
                                        Err(e) => {
                                            warn!(
                                                "LoadImageW from resource for {} failed: {:?}",
                                                name, e
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            warn!(
                                "Failed to parse resource ID from '{}' for device {}",
                                parts[1], name
                            );
                        }
                    } else if path_str.to_lowercase().ends_with(".ico") {
                        debug!(
                            "Icon path for {} appears to be a direct .ico file: '{}'",
                            name, path_str
                        );
                        let mut path_u16: Vec<u16> = path_str.encode_utf16().collect();
                        path_u16.push(0); // Null terminate

                        unsafe {
                            let hicon_handle = LoadImageW(
                                None, // HINSTANCE is None for LR_LOADFROMFILE
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
                                        name, hicon
                                    );
                                    match hicon_to_rgba(&HICON(hicon.0)) {
                                        Ok((rgba, w, h)) => {
                                            icon_rgba_data = Some(rgba);
                                            icon_width = w;
                                            icon_height = h;
                                            debug!(
                                                "Converted HICON to RGBA for {}: {}x{}",
                                                name, w, h
                                            );
                                        }
                                        Err(e) => {
                                            error!("Failed to convert HICON (file) to RGBA for {}: {:?}", name, e);
                                        }
                                    }
                                }
                                Ok(invalid_hicon) => {
                                    warn!("LoadImageW from file for {} returned an invalid HICON: {:?}. Error: {:?}", name, invalid_hicon, windows::core::Error::from_win32());
                                }
                                Err(e) => {
                                    warn!("LoadImageW from file for {} failed: {:?}", name, e);
                                }
                            }
                        }
                    } else {
                        warn!("Unrecognized icon path format for {}: '{}'", name, path_str);
                    }
                } else {
                    debug!("Icon path string for {} is null", name);
                }
            } else {
                debug!(
                    "PKEY_DEVICE_ICON for {} is not a string (VT_LPWSTR). Actual type: {:?}",
                    name,
                    unsafe { propvar.Anonymous.Anonymous.vt }
                );
            }
        }
        Err(e) => {
            debug!(
                "Failed to get PKEY_DEVICE_ICON property for {}: {:?}",
                name, e
            );
        }
    }

    if let Some(rgba) = icon_rgba_data {
        if icon_width > 0 && icon_height > 0 {
            match RgbaImage::from_raw(icon_width, icon_height, rgba) {
                Some(image) => {
                    debug!("Successfully created RgbaImage for {}", name);
                    Ok(Some(image))
                }
                None => {
                    error!(
                        "RgbaImage::from_raw failed for {} ({}x{}) despite having data.",
                        name, icon_width, icon_height
                    );
                    Ok(None)
                }
            }
        } else {
            warn!(
                "Icon for {} had data but invalid dimensions: {}x{}",
                name, icon_width, icon_height
            );
            Ok(None)
        }
    } else {
        debug!("No icon RGBA data retrieved for {}", name);
        Ok(None)
    }
}

unsafe fn hicon_to_rgba(hicon: &HICON) -> WindyResult<(Vec<u8>, u32, u32)> {
    debug!("hicon_to_rgba: Starting conversion for HICON: {:?}", hicon);
    let mut icon_info = ICONINFO::default();
    if GetIconInfo(*hicon, &mut icon_info).is_err() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetIconInfo failed: {:?}", err);
        return Err(err.into());
    }
    debug!(
        "hicon_to_rgba: GetIconInfo success. hbmColor: {:?}, hbmMask: {:?}",
        icon_info.hbmColor, icon_info.hbmMask
    );

    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;

    let mut bitmap_info_header = BITMAPINFOHEADER::default();
    if windows::Win32::Graphics::Gdi::GetObjectW(
        hbm_color.into(),
        std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAP>() as i32,
        Some(&mut bitmap_info_header as *mut _ as *mut std::ffi::c_void),
    ) == 0
    {
        let mut bitmap_struct = windows::Win32::Graphics::Gdi::BITMAP::default();
        if windows::Win32::Graphics::Gdi::GetObjectW(
            hbm_color.into(),
            std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAP>() as i32,
            Some(&mut bitmap_struct as *mut _ as *mut std::ffi::c_void),
        ) == 0
        {
            let err = windows::core::Error::from_win32();
            error!("hicon_to_rgba: GetObjectW (for BITMAP) failed: {:?}", err);
            let _ = DeleteObject(hbm_color.into());
            if hbm_mask.0 != std::ptr::null_mut() {
                let _ = DeleteObject(hbm_mask.into());
            }
            return Err(err.into());
        }
        debug!(
            "hicon_to_rgba: GetObjectW (BITMAP) success. Width: {}, Height: {}, BitsPixel: {}",
            bitmap_struct.bmWidth, bitmap_struct.bmHeight, bitmap_struct.bmBitsPixel
        );
        bitmap_info_header.biWidth = bitmap_struct.bmWidth;
        bitmap_info_header.biHeight = bitmap_struct.bmHeight;
        bitmap_info_header.biBitCount = bitmap_struct.bmBitsPixel;
    } else {
        debug!("hicon_to_rgba: GetObjectW with BITMAPINFOHEADER (unusual path). Width: {}, Height: {}, BitsPixel: {}", bitmap_info_header.biWidth, bitmap_info_header.biHeight, bitmap_info_header.biBitCount);
    }

    let width = bitmap_info_header.biWidth.abs() as u32;
    let height = bitmap_info_header.biHeight.abs() as u32;
    let bpp = bitmap_info_header.biBitCount;

    debug!(
        "hicon_to_rgba: Determined dimensions: {}x{} @ {}bpp",
        width, height, bpp
    );

    if width == 0 || height == 0 {
        error!("hicon_to_rgba: Icon dimensions are zero.");
        let _ = DeleteObject(hbm_color.into());
        if hbm_mask.0 != std::ptr::null_mut() {
            let _ = DeleteObject(hbm_mask.into());
        }
        return Err(eyre!("Icon has zero width or height").into());
    }

    let mut rgba_data = vec![0u8; (width * height * 4) as usize];

    let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
    if screen_dc.is_invalid() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetDC(None) failed: {:?}", err);
        let _ = DeleteObject(hbm_color.into());
        if hbm_mask.0 != std::ptr::null_mut() {
            let _ = DeleteObject(hbm_mask.into());
        }
        return Err(err.into());
    }
    let mem_dc = CreateCompatibleDC(Some(screen_dc));
    if mem_dc.is_invalid() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: CreateCompatibleDC failed: {:?}", err);
        let _ = windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
        let _ = DeleteObject(hbm_color.into());
        if hbm_mask.0 != std::ptr::null_mut() {
            let _ = DeleteObject(hbm_mask.into());
        }
        return Err(err.into());
    }

    let old_bitmap = SelectObject(mem_dc, hbm_color.into());

    let mut bmi = windows::Win32::Graphics::Gdi::BITMAPINFO::default();
    bmi.bmiHeader.biSize =
        std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32);
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

    debug!("hicon_to_rgba: Calling GetDIBits for color bitmap.");
    if GetDIBits(
        mem_dc,
        hbm_color,
        0,
        height,
        Some(rgba_data.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bmi,
        DIB_RGB_COLORS,
    ) == 0
    {
        let err = windows::core::Error::from_win32();
        error!(
            "hicon_to_rgba: GetDIBbits for color bitmap failed: {:?}",
            err
        );
        SelectObject(mem_dc, old_bitmap);
        let _ = DeleteDC(mem_dc);
        let _ = windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
        let _ = DeleteObject(hbm_color.into());
        if hbm_mask.0 != std::ptr::null_mut() {
            let _ = DeleteObject(hbm_mask.into());
        }
        return Err(err.into());
    }
    debug!("hicon_to_rgba: GetDIBits for color bitmap success.");

    SelectObject(mem_dc, old_bitmap);

    if hbm_mask.0 != std::ptr::null_mut() && hbm_mask.0 != hbm_color.0 {
        debug!(
            "hicon_to_rgba: Processing separate mask bitmap: {:?}",
            hbm_mask
        );
        let mut mask_pixel_data = vec![0u8; (width * height * 4) as usize];

        let old_mask_bitmap = SelectObject(mem_dc, hbm_mask.into());

        bmi.bmiHeader.biHeight = -(height as i32);

        debug!("hicon_to_rgba: Calling GetDIBits for mask bitmap.");
        if GetDIBits(
            mem_dc,
            hbm_mask,
            0,
            height,
            Some(mask_pixel_data.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bmi,
            DIB_RGB_COLORS,
        ) == 0
        {
            let err = windows::core::Error::from_win32();
            error!("hicon_to_rgba: GetDIBits for mask bitmap failed: {:?}", err);
        } else {
            debug!("hicon_to_rgba: GetDIBits for mask bitmap success. Applying mask.");
            for i in 0..(width * height) as usize {
                let pixel_idx = i * 4;
                if mask_pixel_data[pixel_idx] == 0
                    && mask_pixel_data[pixel_idx + 1] == 0
                    && mask_pixel_data[pixel_idx + 2] == 0
                {
                    rgba_data[pixel_idx + 3] = 0;
                } else if bpp != 32 && rgba_data[pixel_idx + 3] == 0 {
                    rgba_data[pixel_idx + 3] = 255;
                }
            }
        }
        SelectObject(mem_dc, old_mask_bitmap);
        let _ = DeleteObject(hbm_mask.into());
    } else {
        debug!("hicon_to_rgba: No separate mask bitmap or mask is same as color.");
        if bpp != 32 {
            for i in 0..(width * height) as usize {
                rgba_data[i * 4 + 3] = 255;
            }
        }
    }

    let _ = DeleteDC(mem_dc);
    let _ = windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);
    let _ = DeleteObject(hbm_color.into());

    for i in 0..(width * height) as usize {
        let pixel_idx = i * 4;
        let b = rgba_data[pixel_idx];
        let r = rgba_data[pixel_idx + 2];
        rgba_data[pixel_idx] = r;
        rgba_data[pixel_idx + 2] = b;
    }
    debug!("hicon_to_rgba: BGRA to RGBA conversion complete.");

    Ok((rgba_data, width, height))
}

// Fix MAKEINTRESOURCEW: define it if not present
#[allow(non_snake_case)]
fn MAKEINTRESOURCEW(i: u16) -> windows::core::PCWSTR {
    windows::core::PCWSTR(i as usize as *const u16)
}
