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
use windows::Win32::Graphics::Gdi::ReleaseDC;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BI_RGB;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT; // Added for type clarity
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
// If you have Win32_Foundation feature enabled for windows crate, you could use:
// use windows::Win32::Foundation::MAKEINTRESOURCEW;
use ymb_windy::WindyResult;

// DEVPKEY_Device_IconPath
const PKEY_DEVICE_ICON: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x259abffc_507a_4ce8_8c10_9640b8a1c907),
    pid: 10,
};

// DEVPKEY_DeviceClass_IconPath
const PKEY_DEVICE_CLASS_ICON: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0x259abffc_507a_4ce8_8c10_9640b8a1c907), // Same fmtid
    pid: 12,                                                        // Different pid
};

// Helper to extract icon path string from PROPVARIANT
fn get_path_from_propvariant(
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

pub fn get_mic_icon(props: &IPropertyStore, name: &str) -> WindyResult<Option<RgbaImage>> {
    debug!("Attempting to get icon for device: {}", name);

    let mut icon_path_str: Option<String> = None;

    // Attempt 1: PKEY_DEVICE_ICON (Specific device icon)
    match unsafe { props.GetValue(&PKEY_DEVICE_ICON) } {
        Ok(propvar) => {
            icon_path_str = get_path_from_propvariant(&propvar, name, "PKEY_DEVICE_ICON");
        }
        Err(e) => {
            debug!(
                "Failed to get PKEY_DEVICE_ICON property for '{}': {:?}",
                name, e
            );
        }
    }

    // Attempt 2: PKEY_DEVICE_CLASS_ICON (Fallback to class icon)
    if icon_path_str.is_none() {
        debug!(
            "PKEY_DEVICE_ICON did not yield a path for '{}'. Trying PKEY_DEVICE_CLASS_ICON.",
            name
        );
        match unsafe { props.GetValue(&PKEY_DEVICE_CLASS_ICON) } {
            Ok(propvar) => {
                icon_path_str = get_path_from_propvariant(&propvar, name, "PKEY_DEVICE_CLASS_ICON");
            }
            Err(e) => {
                debug!(
                    "Failed to get PKEY_DEVICE_CLASS_ICON property for '{}': {:?}",
                    name, e
                );
            }
        }
    }

    if let Some(path_str) = icon_path_str {
        let mut icon_rgba_data: Option<Vec<u8>> = None;
        let mut icon_width = 0;
        let mut icon_height = 0;

        let parts: Vec<&str> = path_str.split(",-").collect();
        if parts.len() == 2 {
            let dll_path_str = parts[0].replace(
                "%SystemRoot%",
                &std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()),
            );
            debug!("Parsed DLL path for {}: '{}'", name, dll_path_str);

            if let Ok(resource_id) = parts[1].parse::<i32>() {
                debug!("Parsed resource ID for {}: {}", name, resource_id);
                let mut dll_path_u16: Vec<u16> = dll_path_str.encode_utf16().collect();
                dll_path_u16.push(0); // Null terminate

                unsafe {
                    match GetModuleHandleW(PCWSTR(dll_path_u16.as_ptr())) {
                        Ok(h_module) if !h_module.is_invalid() => {
                            // HMODULE is type alias for HINSTANCE, direct use is fine.
                            let hicon_handle = LoadImageW(
                                Some(h_module.into()), // Use h_module directly
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
                                            error!(
                                                "Failed to convert HICON (resource) to RGBA for {}: {:?}",
                                                name, e
                                            );
                                        }
                                    }
                                }
                                Ok(invalid_hicon) => {
                                    warn!("LoadImageW from resource for {} returned an invalid HICON: {:?}. OS Error: {:?}", name, invalid_hicon, windows::core::Error::from_win32());
                                }
                                Err(e) => {
                                    warn!("LoadImageW from resource for {} failed: {:?}", name, e);
                                }
                            }
                        }
                        Ok(invalid_handle) => {
                            warn!("GetModuleHandleW for DLL '{}' for device {} returned an invalid handle: {:?}", dll_path_str, name, invalid_handle);
                        }
                        Err(e) => {
                            warn!(
                                "GetModuleHandleW for DLL '{}' for device {} failed: {:?}",
                                dll_path_str, name, e
                            );
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
                            name, hicon
                        );
                        match hicon_to_rgba(&HICON(hicon.0)) {
                            Ok((rgba, w, h)) => {
                                icon_rgba_data = Some(rgba);
                                icon_width = w;
                                icon_height = h;
                                debug!("Converted HICON to RGBA for {}: {}x{}", name, w, h);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to convert HICON (file) to RGBA for {}: {:?}",
                                    name, e
                                );
                            }
                        }
                    }
                    Ok(invalid_hicon) => {
                        warn!("LoadImageW from file for {} returned an invalid HICON: {:?}. OS Error: {:?}", name, invalid_hicon, windows::core::Error::from_win32());
                    }
                    Err(e) => {
                        warn!("LoadImageW from file for {} failed: {:?}", name, e);
                    }
                }
            }
        } else {
            warn!("Unrecognized icon path format for {}: '{}'", name, path_str);
        }

        if let Some(rgba) = icon_rgba_data {
            if icon_width > 0 && icon_height > 0 {
                match RgbaImage::from_raw(icon_width, icon_height, rgba) {
                    Some(image) => {
                        debug!("Successfully created RgbaImage for {}", name);
                        return Ok(Some(image));
                    }
                    None => {
                        error!(
                            "RgbaImage::from_raw failed for {} ({}x{}) despite having data.",
                            name, icon_width, icon_height
                        );
                    }
                }
            } else {
                warn!(
                    "Icon for {} had data but invalid dimensions: {}x{}",
                    name, icon_width, icon_height
                );
            }
        }
    } else {
        debug!(
            "No icon path found for {} after trying device and class properties.",
            name
        );
    }

    Ok(None) // Default: no icon found or error occurred
}

// hicon_to_rgba function remains largely the same.
// One minor improvement: use icon_info.hbmColor and icon_info.hbmMask directly
// for GetObjectW and DeleteObject, as GetIconInfo creates copies.
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

    // These are the bitmaps we need to manage and delete.
    // GetIconInfo creates copies of the icon's color and mask bitmaps.
    let hbm_color = icon_info.hbmColor; // This is an HBITMAP
    let hbm_mask = icon_info.hbmMask; // This is an HBITMAP

    // RAII guard for cleaning up GDI objects
    struct GdiObjectGuard(windows::Win32::Graphics::Gdi::HGDIOBJ);
    impl Drop for GdiObjectGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                unsafe { _ = DeleteObject(self.0) };
            }
        }
    }
    // Wrap hbm_color and hbm_mask for automatic cleanup
    // Note: hbm_mask might be the same as hbm_color or null, handle carefully.
    let _hbm_color_guard = GdiObjectGuard(hbm_color.into());
    let _hbm_mask_guard = if hbm_mask != hbm_color && !hbm_mask.is_invalid() {
        Some(GdiObjectGuard(hbm_mask.into()))
    } else {
        None
    };

    let mut bitmap_struct = windows::Win32::Graphics::Gdi::BITMAP::default();
    if GetObjectW(
        hbm_color.into(), // Use HBITMAP directly
        std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAP>() as i32,
        Some(&mut bitmap_struct as *mut _ as *mut std::ffi::c_void),
    ) == 0
    {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetObjectW (for BITMAP) failed: {:?}", err);
        // Guards will handle cleanup
        return Err(err.into());
    }

    debug!(
        "hicon_to_rgba: GetObjectW (BITMAP) success. Width: {}, Height: {}, BitsPixel: {}",
        bitmap_struct.bmWidth, bitmap_struct.bmHeight, bitmap_struct.bmBitsPixel
    );

    let width = bitmap_struct.bmWidth.abs() as u32;
    let height = bitmap_struct.bmHeight.abs() as u32; // Height can be negative for top-down DDBs
    let source_bpp = bitmap_struct.bmBitsPixel;

    if width == 0 || height == 0 {
        error!("hicon_to_rgba: Icon dimensions are zero.");
        return Err(eyre!("Icon has zero width or height").into());
    }

    let mut rgba_data = vec![0u8; (width * height * 4) as usize];

    let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
    if screen_dc.is_invalid() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: GetDC(None) failed: {:?}", err);
        return Err(err.into());
    }
    // RAII for screen_dc
    struct ReleaseDCGuard(windows::Win32::Graphics::Gdi::HDC);
    impl Drop for ReleaseDCGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                unsafe { ReleaseDC(None, self.0) };
            }
        }
    }
    let _screen_dc_guard = ReleaseDCGuard(screen_dc);

    let mem_dc = CreateCompatibleDC(Some(screen_dc));
    if mem_dc.is_invalid() {
        let err = windows::core::Error::from_win32();
        error!("hicon_to_rgba: CreateCompatibleDC failed: {:?}", err);
        return Err(err.into());
    }

    struct HDCObjectGuard(windows::Win32::Graphics::Gdi::HDC);
    impl Drop for HDCObjectGuard {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                let _ = unsafe {
                    _ = DeleteDC(self.0);
                };
            }
        }
    }
    let _mem_dc_guard = HDCObjectGuard(mem_dc); // DeleteDC on drop

    let old_bitmap = SelectObject(mem_dc, hbm_color.into());
    // RAII for restoring old bitmap
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
    bmi.bmiHeader.biHeight = -(height as i32); // Request top-down DIB
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32; // We want 32-bit RGBA
    bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

    debug!("hicon_to_rgba: Calling GetDIBits for color bitmap.");
    if GetDIBits(
        mem_dc,
        hbm_color, // Source HBITMAP
        0,
        height,
        Some(rgba_data.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bmi,
        DIB_RGB_COLORS,
    ) == 0
    {
        let err = windows::core::Error::from_win32();
        error!(
            "hicon_to_rgba: GetDIBits for color bitmap failed: {:?}",
            err
        );
        return Err(err.into());
    }
    debug!("hicon_to_rgba: GetDIBits for color bitmap success.");

    // Alpha channel handling
    // If the original icon was 32bpp, its alpha channel should be in rgba_data already.
    // If it wasn't (e.g., 24bpp), alpha bytes might be 0 or garbage.
    // We need to apply the mask if it exists and is separate.

    if !hbm_mask.is_invalid() && hbm_mask != hbm_color {
        debug!(
            "hicon_to_rgba: Processing separate mask bitmap: {:?}",
            hbm_mask
        );
        // The mask bitmap is 1bpp. We need to get its bits.
        // For simplicity, we can draw the icon with DrawIconEx and capture,
        // or get mask bits and apply manually.
        // The current approach gets mask as 32bpp then checks if it's black.

        let mut mask_pixel_data = vec![0u8; (width * height * 4) as usize]; // Get as 32bpp

        // Select mask into DC
        let old_mask_bitmap_in_dc = SelectObject(mem_dc, hbm_mask.into());
        let _old_mask_bitmap_guard = SelectObjectGuard {
            hdc: mem_dc,
            hgdiobj: old_mask_bitmap_in_dc,
        };

        // bmi is already set up for 32bpp top-down, which is fine for getting mask data too
        if GetDIBits(
            mem_dc,
            hbm_mask,
            0,
            height,
            Some(mask_pixel_data.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bmi, // Use same bmi to get it as 32bpp BGRA
            DIB_RGB_COLORS,
        ) == 0
        {
            let err = windows::core::Error::from_win32();
            error!("hicon_to_rgba: GetDIBits for mask bitmap failed: {:?}", err);
            // Continue without mask, or return error? For now, log and continue.
        } else {
            debug!("hicon_to_rgba: GetDIBits for mask bitmap success. Applying mask.");
            for i in 0..(width * height) as usize {
                let pixel_idx = i * 4;
                // Mask convention: Black on mask means transparent in image. White means opaque.
                // If GetDIBits on a 1bpp mask returns it as 32bpp, black (0) means transparent.
                if mask_pixel_data[pixel_idx] == 0 // Blue channel of mask pixel
                    && mask_pixel_data[pixel_idx + 1] == 0 // Green
                    && mask_pixel_data[pixel_idx + 2] == 0
                // Red
                {
                    rgba_data[pixel_idx + 3] = 0; // Set alpha to transparent
                } else {
                    // If not masked out, and original wasn't 32bpp (so alpha wasn't native)
                    // ensure it's opaque. If original was 32bpp, its alpha is already there.
                    if source_bpp != 32 {
                        rgba_data[pixel_idx + 3] = 255; // Set alpha to opaque
                    }
                }
            }
        }
    } else {
        // No separate mask, or mask is same as color (e.g., 32-bit icon with its own alpha)
        debug!("hicon_to_rgba: No separate mask bitmap or mask is same as color.");
        if source_bpp != 32 {
            // If the source image wasn't 32bpp, it has no alpha channel. Make it opaque.
            for i in 0..(width * height) as usize {
                rgba_data[i * 4 + 3] = 255;
            }
        }
        // If source_bpp was 32, its alpha is already in rgba_data[pixel_idx + 3]
    }

    // BGRA to RGBA conversion (Windows DIBs are often BGRA)
    for i in 0..(width * height) as usize {
        let pixel_idx = i * 4;
        // Swap B (at index 0) and R (at index 2)
        rgba_data.swap(pixel_idx, pixel_idx + 2);
    }
    debug!("hicon_to_rgba: BGRA to RGBA conversion complete.");

    // GDI objects are cleaned up by RAII guards when they go out of scope.
    Ok((rgba_data, width, height))
}

// Your local MAKEINTRESOURCEW is fine if you don't have Win32_Foundation enabled
// or prefer to keep it self-contained.
#[allow(non_snake_case)]
fn MAKEINTRESOURCEW(i: u16) -> windows::core::PCWSTR {
    windows::core::PCWSTR(i as usize as *const u16)
}
