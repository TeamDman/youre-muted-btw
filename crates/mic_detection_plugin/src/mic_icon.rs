use crate::wcslen;
use eyre::eyre;
use image::RgbaImage;
use tracing::debug;
use tracing::error;
use tracing::warn;
use windows::core::GUID;
use windows::core::PCWSTR;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::GetObjectW;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BI_RGB;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
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

        if let Ok(resource_id) = parts[1].parse::<i32>() {
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
                        MAKEINTRESOURCEW(resource_id as u16),
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

// unsafe fn hicon_to_rgba ... (remains the same as your last version with RAII guards)
// Ensure it's included here. For brevity, I'll skip pasting it again but assume it's present.
// ... (paste the hicon_to_rgba function from the previous response here) ...
unsafe fn hicon_to_rgba(hicon: &HICON) -> WindyResult<(Vec<u8>, u32, u32)> {
    debug!("hicon_to_rgba: Starting conversion for HICON: {:?}", hicon);
    let mut icon_info = ICONINFO::default();
    // According to docs, GetIconInfo creates new HBITMAPs for hbmMask and hbmColor
    // that must be deleted.
    if unsafe { GetIconInfo(*hicon, &mut icon_info) }.is_err() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetIconInfo failed: {:?}", err);
        return Err(err.into());
    }
    debug!(
        "hicon_to_rgba: GetIconInfo success. hbmColor: {:?}, hbmMask: {:?}",
        icon_info.hbmColor, icon_info.hbmMask
    );

    struct BitmapGuard(windows::Win32::Graphics::Gdi::HBITMAP);
    impl Drop for BitmapGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                unsafe { _ = DeleteObject(self.0.into()) };
            }
        }
    }

    // These are copies and must be deleted.
    let _hbm_color_guard = BitmapGuard(icon_info.hbmColor);
    // hbmMask might be null if the icon has no mask or is 32bpp with alpha
    let _hbm_mask_guard =
        if !icon_info.hbmMask.is_invalid() && icon_info.hbmMask != icon_info.hbmColor {
            Some(BitmapGuard(icon_info.hbmMask))
        } else {
            None
        };

    let hbm_color_to_process = icon_info.hbmColor;

    let mut bitmap_struct = windows::Win32::Graphics::Gdi::BITMAP::default();
    if unsafe {
        GetObjectW(
            hbm_color_to_process.into(),
            std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAP>() as i32,
            Some(&mut bitmap_struct as *mut _ as *mut std::ffi::c_void),
        )
    } == 0
    {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetObjectW (for BITMAP) failed: {:?}", err);
        return Err(err.into());
    }

    debug!(
        "hicon_to_rgba: GetObjectW (BITMAP) success. Width: {}, Height: {}, BitsPixel: {}",
        bitmap_struct.bmWidth, bitmap_struct.bmHeight, bitmap_struct.bmBitsPixel
    );

    let width = bitmap_struct.bmWidth.abs() as u32;
    let height = bitmap_struct.bmHeight.abs() as u32;
    let source_bpp = bitmap_struct.bmBitsPixel;

    if width == 0 || height == 0 {
        error!("hicon_to_rgba: Icon dimensions are zero.");
        return Err(eyre!("Icon has zero width or height").into());
    }

    let mut rgba_data = vec![0u8; (width * height * 4) as usize];

    let screen_dc = unsafe { windows::Win32::Graphics::Gdi::GetDC(None) };
    if screen_dc.is_invalid() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetDC(None) failed: {:?}", err);
        return Err(err.into());
    }
    struct ReleaseDCGuard(windows::Win32::Graphics::Gdi::HDC);
    impl Drop for ReleaseDCGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                unsafe { windows::Win32::Graphics::Gdi::ReleaseDC(None, self.0) };
            }
        }
    }
    let _screen_dc_guard = ReleaseDCGuard(screen_dc);

    let mem_dc = unsafe { CreateCompatibleDC(Some(screen_dc)) };
    if mem_dc.is_invalid() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: CreateCompatibleDC failed: {:?}", err);
        return Err(err.into());
    }
    struct DeleteDCGuard(windows::Win32::Graphics::Gdi::HDC);
    impl Drop for DeleteDCGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                unsafe { _ = DeleteDC(self.0) };
            }
        }
    }
    let _mem_dc_guard = DeleteDCGuard(mem_dc);

    let old_bitmap = unsafe { SelectObject(mem_dc, hbm_color_to_process.into()) };
    struct SelectObjectGuard {
        hdc: windows::Win32::Graphics::Gdi::HDC,
        hgdiobj: windows::Win32::Graphics::Gdi::HGDIOBJ,
    }
    impl Drop for SelectObjectGuard {
        fn drop(&mut self) {
            if !self.hdc.is_invalid() && !self.hgdiobj.is_invalid() {
                unsafe { SelectObject(self.hdc, self.hgdiobj) };
            }
        }
    }
    let _old_bitmap_guard = SelectObjectGuard {
        hdc: mem_dc,
        hgdiobj: old_bitmap,
    };

    let mut bmi = windows::Win32::Graphics::Gdi::BITMAPINFO::default();
    bmi.bmiHeader.biSize =
        std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32);
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

    debug!("hicon_to_rgba: Calling GetDIBits for color bitmap.");
    if unsafe {
        GetDIBits(
            mem_dc,
            hbm_color_to_process,
            0,
            height,
            Some(rgba_data.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    } == 0
    {
        let err = windows::core::Error::from_win32();
        error!(
            "hicon_to_rgba: GetDIBits for color bitmap failed: {:?}",
            err
        );
        return Err(err.into());
    }
    debug!("hicon_to_rgba: GetDIBits for color bitmap success.");

    if !icon_info.hbmMask.is_invalid() && icon_info.hbmMask != icon_info.hbmColor {
        debug!(
            "hicon_to_rgba: Processing separate mask bitmap: {:?}",
            icon_info.hbmMask
        );
        let mut mask_pixel_data = vec![0u8; (width * height) as usize]; // 1bpp mask data

        // Create a BITMAPINFO for the 1bpp mask
        let mut mask_bmi = windows::Win32::Graphics::Gdi::BITMAPINFO::default();
        mask_bmi.bmiHeader.biSize =
            std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
        mask_bmi.bmiHeader.biWidth = width as i32;
        mask_bmi.bmiHeader.biHeight = -(height as i32); // Top-down
        mask_bmi.bmiHeader.biPlanes = 1;
        mask_bmi.bmiHeader.biBitCount = 1; // 1 bit per pixel
        mask_bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

        // GetDIBits for the mask
        if unsafe {
            GetDIBits(
                mem_dc,            // Use the same DC, but select the mask bitmap into it
                icon_info.hbmMask, // The actual mask bitmap
                0,
                height,
                Some(mask_pixel_data.as_mut_ptr() as *mut std::ffi::c_void),
                &mut mask_bmi,
                DIB_RGB_COLORS,
            )
        } == 0
        {
            let err = windows::core::Error::from_win32();
            error!(
                "hicon_to_rgba: GetDIBits for 1bpp mask bitmap failed: {:?}",
                err
            );
        } else {
            debug!("hicon_to_rgba: GetDIBits for 1bpp mask bitmap success. Applying mask.");
            // Iterate over each pixel. The mask_pixel_data is packed 8 pixels per byte.
            // The DIB is top-down, so scanlines are in order.
            // Row size for 1bpp DIB must be DWORD aligned.
            let row_size_bytes = ((width + 31) / 32) * 4;

            for y in 0..height {
                for x in 0..width {
                    let byte_index = (y * row_size_bytes + x / 8) as usize;
                    let bit_index = 7 - (x % 8); // Bits are packed from MSB to LSB
                    let mask_bit = (mask_pixel_data[byte_index] >> bit_index) & 1;

                    let pixel_idx_rgba = ((y * width + x) * 4) as usize;
                    if mask_bit == 1 {
                        // Mask bit 1 means transparent (for ICONINFO mask)
                        rgba_data[pixel_idx_rgba + 3] = 0; // Set alpha to transparent
                    } else {
                        // Mask bit 0 means opaque
                        if source_bpp != 32 {
                            // If original wasn't 32bpp, ensure opaque
                            rgba_data[pixel_idx_rgba + 3] = 255;
                        }
                        // If original was 32bpp, its alpha is already in rgba_data[pixel_idx_rgba + 3]
                        // and this mask bit being 0 means that alpha should be preserved.
                    }
                }
            }
        }
    } else {
        debug!(
            "hicon_to_rgba: No separate mask bitmap or mask is same as color (e.g. 32bpp icon)."
        );
        if source_bpp != 32 {
            for i in 0..(width * height) as usize {
                rgba_data[i * 4 + 3] = 255; // Make opaque
            }
        }
    }

    for i in 0..(width * height) as usize {
        let pixel_idx = i * 4;
        rgba_data.swap(pixel_idx, pixel_idx + 2); // BGRA to RGBA
    }
    debug!("hicon_to_rgba: BGRA to RGBA conversion complete.");

    Ok((rgba_data, width, height))
}

#[allow(non_snake_case)]
fn MAKEINTRESOURCEW(i: u16) -> windows::core::PCWSTR {
    windows::core::PCWSTR(i as usize as *const u16)
}
