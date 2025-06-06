use eyre::eyre;
use tracing::debug;
use tracing::error;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::GetObjectW;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BI_RGB;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFO;
use ymb_windy::error::WindyResult;

// unsafe fn hicon_to_rgba ... (remains the same as your last version with RAII guards)
// Ensure it's included here. For brevity, I'll skip pasting it again but assume it's present.
// ... (paste the hicon_to_rgba function from the previous response here) ...
pub unsafe fn hicon_to_rgba(hicon: &HICON) -> WindyResult<(Vec<u8>, u32, u32)> {
    let hicon = *hicon;
    debug!("hicon_to_rgba: Starting conversion for HICON: {:?}", hicon);
    let mut icon_info = ICONINFO::default();
    // According to docs, GetIconInfo creates new HBITMAPs for hbmMask and hbmColor
    // that must be deleted.
    if unsafe { GetIconInfo(hicon, &mut icon_info) }.is_err() {
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
