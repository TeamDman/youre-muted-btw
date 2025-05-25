use bevy::math::IRect;
use bevy::math::IVec2;
use bevy::platform::collections::HashMap;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::EnumWindows;
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
use windows::core::BOOL;
use ymb_windy::WindyResult;

use crate::WindowId;
use crate::WindowsApp;
use crate::WindowsAppKind;

pub fn list_apps() -> WindyResult<Vec<WindowsApp>> {
    let mut apps = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut apps as *mut _ as isize),
        )?;
    }
    Ok(apps)
}

pub fn get_apps() -> WindyResult<HashMap<WindowId, WindowsApp>> {
    Ok(list_apps()?.into_iter().map(|app| (app.id, app)).collect())
}

unsafe extern "system" fn enum_windows_callback(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: LPARAM,
) -> BOOL {
    if unsafe { IsWindowVisible(hwnd).as_bool() } {
        // Get window title
        let mut text_buf = [0u16; 1024];
        let text_len = unsafe { GetWindowTextW(hwnd, &mut text_buf) };
        let title = String::from_utf16_lossy(&text_buf[..text_len as usize]);

        // Determine app kind
        let kind = if title.contains("Discord") {
            WindowsAppKind::Discord
        } else {
            WindowsAppKind::NotDiscord
        };

        // Get window rect
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut rect).is_ok() } {
            let bounds = IRect {
                min: IVec2::new(rect.left, rect.top),
                max: IVec2::new(rect.right, rect.bottom),
            };

            let apps = unsafe { &mut *(lparam.0 as *mut Vec<WindowsApp>) };
            apps.push(WindowsApp {
                id: WindowId::from(hwnd),
                kind,
                bounds,
                title,
            });
        }
    }
    true.into()
}
#[cfg(test)]
mod test {
    use super::list_apps;
    use crate::get_apps;
    use ymb_windy::WindyResult;

    #[test]
    fn it_works() -> WindyResult<()> {
        let apps = list_apps()?;
        for app in apps {
            println!("{app:#?}");
        }

        Ok(())
    }
    #[test]
    fn it_works2() -> WindyResult<()> {
        let apps = get_apps()?;
        println!("{apps:#?}");
        Ok(())
    }
}
