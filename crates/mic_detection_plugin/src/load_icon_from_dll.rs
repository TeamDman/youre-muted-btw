use crate::windows_macros::MAKEINTRESOURCEW;
use eyre::eyre;
use eyre::Context;
use std::ffi::OsString;
use std::path::PathBuf;
use windows::core::Owned;
use windows::core::PCWSTR;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::Win32::UI::WindowsAndMessaging::LoadImageW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::IMAGE_ICON;
use windows::Win32::UI::WindowsAndMessaging::LR_DEFAULTSIZE;
use windows::Win32::UI::WindowsAndMessaging::LR_SHARED;
use ymb_windy::bail;
use ymb_windy::error::WindyResult;
use ymb_windy::string_replacement::Replacey;

pub fn expand_system_root(path: PathBuf) -> WindyResult<PathBuf> {
    // this is a convention but not something we use in our code.
    // if dll_path.starts_with('@') {
    //     dll_path.remove(0);
    // }
    let system_root = match std::env::var("SystemRoot") {
        Ok(x) => OsString::from(x),
        Err(e) => match e {
            std::env::VarError::NotPresent => {
                bail!("Environment variable 'SystemRoot' is not set.")
            }
            std::env::VarError::NotUnicode(os_string) => os_string,
        },
    };
    let path = path.replace("%SystemRoot%", &system_root)?;
    let path = PathBuf::from(path);

    Ok(path)
}

pub unsafe fn try_load_icon_from_dll(
    dll_path: PathBuf,
    resource_id: u16,
) -> eyre::Result<Owned<HICON>> {
    let dll_path = widestring::U16CString::from_os_str_truncate(dll_path.as_os_str());
    unsafe {
        let module = LoadLibraryW(PCWSTR::from_raw(dll_path.as_ptr())).wrap_err_with(|| {
            eyre!(
                "LoadLibraryW failed on to load DLL when loading icon {resource_id}: {}",
                dll_path.to_string_lossy()
            )
        })?;
        let hicon_handle = LoadImageW(
            Some(module.into()),
            MAKEINTRESOURCEW(resource_id),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        )
        .wrap_err_with(|| {
            eyre!(
                "LoadImageW failed to load icon {resource_id} from DLL: {}",
                dll_path.display()
            )
        })?;
        Ok(Owned::new(HICON(hicon_handle.0)))
    }
}

#[cfg(test)]
mod test {
    use super::try_load_icon_from_dll;
    use crate::hicon_to_image::hicon_to_rgba;
    use std::ffi::OsString;
    use std::ops::Deref;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;
    use widestring::U16CString;
    use windows::core::PCWSTR;
    use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
    use windows::Win32::UI::Shell::PathUnExpandEnvStringsW;
    use ymb_windy::bail;
    use ymb_windy::error::WindyResult;
    use ymb_windy::string_conversion::EasyPCWSTR;

    #[test]
    fn it_works() -> WindyResult {
        unsafe {
            let icon = try_load_icon_from_dll("C:\\Windows\\system32\\mmres.dll".into(), 3012)?;
            let _image = hicon_to_rgba(icon.deref())?;
        }
        Ok(())
    }
    #[test]
    fn it_works_root() -> WindyResult {
        unsafe {
            let icon = try_load_icon_from_dll("%SystemRoot%\\system32\\mmres.dll".into(), 3012)?;
            let _image = hicon_to_rgba(icon.deref())?;
        }
        Ok(())
    }
    #[test]
    fn it_works_at_root() -> WindyResult {
        unsafe {
            let icon = try_load_icon_from_dll("@%SystemRoot%\\system32\\mmres.dll".into(), 3012);
            assert!(icon.is_err(), "We don't support @ prefix in icon paths");
        }
        Ok(())
    }

    #[test]
    fn contract() -> WindyResult {
        let path: PathBuf = "C:\\Windows\\system32\\mmres.dll".into();
        println!("Was: {}", path.to_string_lossy());
        let mut dest = [0u16; 8096];
        unsafe { PathUnExpandEnvStringsW(&path.easy_pcwstr()?, &mut dest) }.ok()?;
        let expanded = OsString::from_wide(&dest);
        println!("Now: {}", expanded.to_string_lossy());
        Ok(())
    }
    #[test]
    fn expand() -> WindyResult {
        let path = PathBuf::from("%SystemRoot%\\system32\\mmres.dll");
        let mut dest = [0u16; 256];
        // todo: retry when size is larger than dest
        let size = unsafe { ExpandEnvironmentStringsW(&path.easy_pcwstr()?, Some(&mut dest)) } as usize;
        println!("ExpandEnvironmentStringsW returned size: {}", size);
        if size == 0 {
            let err = windows::core::Error::from_win32();
            return Err(err.into());
        }
        if size > dest.len() {
            bail!(
                "ExpandEnvironmentStringsW returned a size larger than the destination buffer: {} > {}",
                size,
                dest.len()
            );
        }
        let expanded = OsString::from_wide(&dest);
        let expanded = U16CString::from_os_str_truncate(&expanded);
        let expanded = PathBuf::from(expanded.to_os_string());
        println!("Expanded path: {}", expanded.to_string_lossy());
        assert_eq!(expanded, PathBuf::from("C:\\Windows\\system32\\mmres.dll"));
        Ok(())
    }
}
