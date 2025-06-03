use bevy::prelude::*;
use bevy::reflect::Reflect;
use image::RgbaImage;
use windows::core::PWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;
use windows::Win32::UI::WindowsAndMessaging::LoadImageW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFO;
use windows::Win32::UI::WindowsAndMessaging::IMAGE_ICON;
use windows::Win32::UI::WindowsAndMessaging::LR_DEFAULTSIZE;
use windows::Win32::UI::WindowsAndMessaging::LR_LOADFROMFILE;
use windows::Win32::UI::WindowsAndMessaging::LR_SHARED;
use ymb_windy::WindyResult;
use ymb_worker_plugin::Sender;
use ymb_worker_plugin::WorkerConfig;
use ymb_worker_plugin::WorkerPlugin;

#[derive(Debug, Clone, Event, Reflect)]
pub enum MicDetectionThreadboundMessage {
    EnumerateMics,
}

#[derive(Debug, Clone, Event, Reflect)]
pub struct MicInfo {
    pub is_default: bool,
    pub name: String,
    #[reflect(ignore)]
    pub icon: Option<RgbaImage>,
}

#[derive(Debug, Clone, Event, Reflect)]
pub enum MicDetectionGameboundMessage {
    MicsEnumerated(Vec<MicInfo>),
}

#[derive(Component, Debug, Clone)]
pub struct Mic {
    pub is_default: bool,
    pub name: String,
    pub icon_handle: Option<Handle<Image>>,
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
                        icon_handle: None, // Icon handling can be added later
                    },
                    Name::new(mic.name.clone()),
                ));
            }
        }
    }
}

const PKEY_DEVICE_ICON: windows::Win32::Foundation::PROPERTYKEY =
    windows::Win32::Foundation::PROPERTYKEY {
        fmtid: windows::core::GUID::from_u128(0x259abffc_507a_4ce8_8c10_9640b8a1c907),
        pid: 10,
    };

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

            // Attempt to get the icon property
            let mut icon_rgba: Option<Vec<u8>> = None;
            let mut icon_width = 0;
            let mut icon_height = 0;
            let get_icon_result = props.GetValue(
                &PKEY_DEVICE_ICON as *const _ as *const windows::Win32::Foundation::PROPERTYKEY,
            );

            if let Ok(propvar) = get_icon_result {
                // Ensure it's a string type
                if propvar.Anonymous.Anonymous.vt == VT_LPWSTR {
                    let icon_path_str = propvar.Anonymous.Anonymous.Anonymous.pwszVal.0;
                    if !icon_path_str.is_null() {
                        let len = wcslen(icon_path_str);
                        let path = String::from_utf16_lossy(std::slice::from_raw_parts(
                            icon_path_str,
                            len,
                        ));

                        // Parse the path, e.g., "%SystemRoot%\System32\mmres.dll,-3012"
                        // This parsing is a bit naive and can be improved with regex or more robust string ops.
                        let parts: Vec<&str> = path.split(",-").collect();
                        if parts.len() == 2 {
                            let dll_path = parts[0].replace(
                                "%SystemRoot%",
                                &std::env::var("SystemRoot")
                                    .unwrap_or_else(|_| "C:\\Windows".to_string()),
                            );
                            if let Ok(resource_id) = parts[1].parse::<i32>() {
                                // Load the icon. Use LR_SHARED for caching, LR_DEFAULTSIZE for native size.
                                // We are requesting ICON_SMALL (16x16) or ICON_BIG (32x32) first.
                                // The system often provides multiple sizes for icons.
                                let hicon = LoadImageW(
                                    Some(
                                        GetModuleHandleW(PWSTR::from_raw(
                                            dll_path.as_ptr() as *mut u16
                                        ))?
                                        .into(),
                                    ), // Load from DLL specified
                                    PWSTR::from_raw(resource_id as *mut u16), // Resource ID
                                    IMAGE_ICON,
                                    0,                          // Desired width, 0 for default
                                    0,                          // Desired height, 0 for default
                                    LR_DEFAULTSIZE | LR_SHARED, // Flags: load default size and share
                                )?;

                                if hicon.is_invalid() {
                                    tracing::warn!("Failed to load icon for device '{}' from '{}', resource_id: {}. HICON was invalid.", name, dll_path, resource_id);
                                } else {
                                    // Convert HICON to RGBA data
                                    match hicon_to_rgba(&HICON(hicon.0)) {
                                        Ok((rgba, w, h)) => {
                                            icon_rgba = Some(rgba);
                                            icon_width = w;
                                            icon_height = h;
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to convert HICON to RGBA for device '{}': {:?}", name, e);
                                        }
                                    }
                                    // Destroy the icon if not LR_SHARED
                                    // DestroyIcon(HICON(hicon.0)); // Don't destroy if LR_SHARED
                                }
                            }
                        } else {
                            tracing::warn!(
                                "Unrecognized icon path format for device '{}': '{}'",
                                name,
                                path
                            );
                            // If it's a direct .ico file path, LoadImageW with LR_LOADFROMFILE could be used.
                            // e.g., "C:\path\to\icon.ico"
                            if path.to_lowercase().ends_with(".ico") {
                                let hicon = LoadImageW(
                                    None,
                                    PWSTR::from_raw(path.as_ptr() as *mut u16),
                                    IMAGE_ICON,
                                    0,
                                    0,
                                    LR_DEFAULTSIZE | LR_SHARED | LR_LOADFROMFILE,
                                )?;
                                if hicon.is_invalid() {
                                    tracing::warn!("Failed to load icon from file path for device '{}': '{}'. HICON was invalid.", name, path);
                                } else {
                                    match hicon_to_rgba(&HICON(hicon.0)) {
                                        Ok((rgba, w, h)) => {
                                            icon_rgba = Some(rgba);
                                            icon_width = w;
                                            icon_height = h;
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to convert HICON (from file) to RGBA for device '{}': {:?}", name, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                tracing::debug!("No PKEY_DEVICE_ICON found for device: '{}'", name);
            }

            let icon = icon_rgba.and_then(|rgba| {
                RgbaImage::from_raw(icon_width, icon_height, rgba)
            });

            mics.push(MicInfo {
                is_default,
                name,
                icon,
            });
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

// Function to convert HICON to RGBA data
// This function needs to be added to `ymb_mic_detection_plugin/src/lib.rs`
unsafe fn hicon_to_rgba(hicon: &HICON) -> WindyResult<(Vec<u8>, u32, u32)> {
    // Get icon information
    let mut icon_info = ICONINFO::default();
    if GetIconInfo(*hicon, &mut icon_info).is_err() {
        return Err(windows::core::Error::from_win32().into());
    }

    // icon_info.hbmColor is the color bitmap
    // icon_info.hbmMask is the monochrome mask bitmap (for transparency)
    // If hbmMask is NULL (or equal to hbmColor), it's a 32-bit icon with alpha channel
    // If hbmMask is separate, it's a 24-bit icon and mask needs to be applied.

    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;

    // Get bitmap info to determine width, height, and bit depth
    let mut bitmap_info = windows::Win32::Graphics::Gdi::BITMAP::default();
    if windows::Win32::Graphics::Gdi::GetObjectW(
        hbm_color,
        std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAP>() as i32,
        Some(&mut bitmap_info as *mut _ as *mut std::ffi::c_void),
    )
    .is_err()
    {
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);
        return Err(windows::core::Error::from_win32().into());
    }

    let width = bitmap_info.bmWidth as u32;
    let height = bitmap_info.bmHeight as u32;
    let bpp = bitmap_info.bmBitsPixel;

    let mut rgba_data = vec![0u8; (width * height * 4) as usize];

    // Create a compatible DC
    let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
    let mem_dc = CreateCompatibleDC(Some(screen_dc));
    let old_bitmap = SelectObject(mem_dc, hbm_color);

    // Get the bits. DIB_RGB_COLORS for palette colors.
    // DIB_RGB_COLORS should be specified if the biCompression member is BI_RGB and the biBitCount member is less than 16.
    // For 32-bit (BI_RGB, 32-bit color), we expect BGRA directly.
    let mut bmi = windows::Win32::Graphics::Gdi::BITMAPINFO::default();
    bmi.bmiHeader.biSize =
        std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32); // Negative height for top-down DIB
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32; // Request 32-bit RGBA for consistency
    bmi.bmiHeader.biCompression = windows::Win32::Graphics::Gdi::BI_RGB;

    let bytes_per_row = (width * 4 + 3) & !3; // Ensure 4-byte alignment

    if GetDIBits(
        mem_dc,
        hbm_color,
        0,
        height,
        Some(rgba_data.as_mut_ptr() as *mut std::ffi::c_void),
        &mut bmi,
        windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
    )
    .is_err()
    {
        SelectObject(mem_dc, old_bitmap);
        DeleteDC(mem_dc);
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);
        return Err(windows::core::Error::from_win32().into());
    }

    SelectObject(mem_dc, old_bitmap);
    DeleteDC(mem_dc);
    DeleteObject(hbm_color);

    // If mask is separate, apply it for transparency.
    // This is often needed for 24-bit icons or icons that don't have an explicit alpha channel.
    // For 32-bit icons, hbmMask is often equal to hbmColor or NULL if the alpha channel is embedded.
    if hbm_mask.0 != 0 && hbm_mask.0 != hbm_color.0 {
        let mut mask_rgba = vec![0u8; (width * height * 4) as usize];
        let mem_dc_mask = CreateCompatibleDC(Some(screen_dc));
        let old_mask_bitmap = SelectObject(mem_dc_mask, hbm_mask);

        let mut mask_bmi = windows::Win32::Graphics::Gdi::BITMAPINFO::default();
        mask_bmi.bmiHeader.biSize =
            std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32;
        mask_bmi.bmiHeader.biWidth = width as i32;
        mask_bmi.bmiHeader.biHeight = -(height as i32);
        mask_bmi.bmiHeader.biPlanes = 1;
        mask_bmi.bmiHeader.biBitCount = 32;
        mask_bmi.bmiHeader.biCompression = windows::Win32::Graphics::Gdi::BI_RGB;

        if GetDIBits(
            mem_dc_mask,
            hbm_mask,
            0,
            height,
            Some(mask_rgba.as_mut_ptr() as *mut std::ffi::c_void),
            &mut mask_bmi,
            windows::Win32::Graphics::Gdi::DIB_RGB_COLORS,
        )
        .is_err()
        {
            SelectObject(mem_dc_mask, old_mask_bitmap);
            DeleteDC(mem_dc_mask);
            DeleteObject(hbm_mask);
            // We can still return the image without mask if this fails
            tracing::error!("Failed to get mask bits. Icon transparency may be incorrect.");
        } else {
            // Apply mask to color data (BGRA to RGBA conversion also happens here)
            for y in 0..height {
                for x in 0..width {
                    let pixel_idx = ((y * width + x) * 4) as usize;
                    let mask_idx = ((y * width + x) * 4) as usize;

                    // BGRA to RGBA (Windows gives BGRA)
                    let b = rgba_data[pixel_idx];
                    let g = rgba_data[pixel_idx + 1];
                    let r = rgba_data[pixel_idx + 2];
                    // Alpha is typically rgba_data[pixel_idx + 3] for 32-bit icons

                    // The mask bitmap is monochrome: black (0) for transparent, white (255) for opaque.
                    // The mask bitmap's color channels will all be the same (0 or 255).
                    let mask_alpha = mask_rgba[mask_idx]; // Or mask_rgba[mask_idx+1], etc.

                    // If the original icon was 32-bit with alpha, it's already there.
                    // If it was 24-bit, its alpha might be 0, and we use the mask for alpha.
                    // Combine original alpha with mask alpha.
                    // Max of existing alpha and mask alpha (inverted, as 0 means transparent in mask)
                    let final_alpha = if bpp == 32 {
                        // For 32-bit icons, the alpha channel is in rgba_data[pixel_idx + 3]
                        // And the hbmMask might actually be the inverse or combined with alpha
                        // A common pattern is hbmMask is the alpha channel itself.
                        // For simplicity, we'll assume BGRA and use the mask as the alpha if present and not fully opaque/transparent.
                        // If it's a true 32-bit icon, rgba_data[pixel_idx + 3] already contains alpha.
                        // We use `!mask_alpha` because black in mask means transparent.
                        let original_alpha = rgba_data[pixel_idx + 3];
                        if mask_alpha == 0 {
                            // black in mask means transparent
                            0 // fully transparent if mask is black
                        } else {
                            original_alpha // use original alpha if mask is not black (opaque)
                        }
                    } else {
                        // For 24-bit icons, the mask provides the alpha.
                        // Black (0) in the mask means the pixel is transparent.
                        // White (255) in the mask means the pixel is opaque.
                        !mask_alpha // Invert mask for alpha: 0 (black) -> 255 (opaque), 255 (white) -> 0 (transparent)
                    };

                    rgba_data[pixel_idx] = r; // R
                    rgba_data[pixel_idx + 1] = g; // G
                    rgba_data[pixel_idx + 2] = b; // B
                    rgba_data[pixel_idx + 3] = final_alpha; // A
                }
            }
        }
        SelectObject(mem_dc_mask, old_mask_bitmap);
        DeleteDC(mem_dc_mask);
        DeleteObject(hbm_mask);
    } else {
        // If no separate mask, or mask is same as color bitmap, assume alpha is embedded for 32-bit
        // or no transparency for less than 32-bit.
        // Convert BGRA to RGBA if necessary.
        if bpp == 32 {
            for i in 0..(width * height) as usize {
                let pixel_idx = i * 4;
                let b = rgba_data[pixel_idx];
                let r = rgba_data[pixel_idx + 2];
                rgba_data[pixel_idx] = r;
                rgba_data[pixel_idx + 2] = b;
                // Alpha is already rgba_data[pixel_idx + 3]
            }
        } else {
            // For 24-bit or less, assume opaque if no mask was found.
            // We've already read as 32-bit, so add full alpha if original was < 32-bit.
            for i in 0..(width * height) as usize {
                let pixel_idx = i * 4;
                let b = rgba_data[pixel_idx];
                let r = rgba_data[pixel_idx + 2];
                rgba_data[pixel_idx] = r;
                rgba_data[pixel_idx + 2] = b;
                rgba_data[pixel_idx + 3] = 255; // Fully opaque
            }
        }
    }

    // Release the DC for the screen, as it's no longer needed for GDI operations.
    // Note: GetDC(HWND::default()) gets the DC for the entire screen, not a specific window.
    // It should be released when no longer needed.
    // There is no explicit `ReleaseDC` needed when `GetDC(HWND::default())` was used.
    // `DeleteDC` is for memory DCs created by `CreateCompatibleDC`.

    Ok((rgba_data, width, height))
}
