use std::convert::Infallible;
use std::ffi::OsString;
use std::path::PathBuf;

use widestring::U16CString;
use windows::core::PCWSTR;
use windows::core::Param;

use crate::error::WindyError;
use crate::error::WindyResult;

pub struct PCWSTRGuard {
    string: U16CString,
}
impl PCWSTRGuard {
    pub fn new(string: U16CString) -> Self {
        Self { string }
    }
}
// MUST NOT implement this for `PCWSTRGuard` itself, only for `&PCWSTRGuard`, to ensure the data the PCWSTR points to is valid for the lifetime of the parameter.
impl Param<PCWSTR> for &PCWSTRGuard {
    unsafe fn param(self) -> windows::core::ParamValue<PCWSTR> {
        windows::core::ParamValue::Borrowed(PCWSTR(self.string.as_ptr()))
    }
}
pub trait EasyPCWSTR {
    type Error;
    fn easy_pcwstr(self) -> WindyResult<PCWSTRGuard, Self::Error>;

}
impl EasyPCWSTR for U16CString {
    type Error = Infallible;

    fn easy_pcwstr(self) -> WindyResult<PCWSTRGuard, Self::Error> {
        Ok(PCWSTRGuard::new(self))
    }
}
impl EasyPCWSTR for &str {
    type Error = WindyError;

    fn easy_pcwstr(self) -> WindyResult<PCWSTRGuard, Self::Error> {
        Ok(PCWSTRGuard::new(U16CString::from_str(self)
            .map_err(|_| air!("Failed to convert string to U16CString: {}", self))?))
    }
}
impl EasyPCWSTR for String {
    type Error = WindyError;

    fn easy_pcwstr(self) -> WindyResult<PCWSTRGuard, Self::Error> {
        Ok(PCWSTRGuard::new(U16CString::from_str(&self)
            .map_err(|_| air!("Failed to convert string to U16CString: {}", self))?))
    }
}
impl EasyPCWSTR for OsString {
    type Error = WindyError;

    fn easy_pcwstr(self) -> WindyResult<PCWSTRGuard, Self::Error> {
        Ok(PCWSTRGuard::new(U16CString::from_os_str_truncate(&self)))
    }
}
impl EasyPCWSTR for PathBuf {
    type Error = WindyError;

    fn easy_pcwstr(self) -> WindyResult<PCWSTRGuard, Self::Error> {
        Ok(PCWSTRGuard::new(U16CString::from_os_str_truncate(self.as_os_str())))
    }
}

#[cfg(test)]
mod test {
    use std::ffi::OsString;

    use super::EasyPCWSTR;

    #[test]
    fn it_works() -> eyre::Result<()> {
        "Hello, World!".easy_pcwstr()?;
        OsString::from("asd").easy_pcwstr()?;
        "asd".to_string().easy_pcwstr()?;
        Ok(())
    }
}
