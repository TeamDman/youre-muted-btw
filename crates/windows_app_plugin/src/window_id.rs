use bevy::reflect::Reflect;
use std::ops::Deref;
use windows::Win32::Foundation::HWND;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[repr(transparent)]
pub struct WindowId(usize);
impl From<HWND> for WindowId {
    fn from(hwnd: HWND) -> Self {
        Self(hwnd.0 as usize)
    }
}
impl Deref for WindowId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
