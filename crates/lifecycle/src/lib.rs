use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;

pub static OUR_HWND: AtomicUsize = AtomicUsize::new(0);
pub static SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION: AtomicBool = AtomicBool::new(false);
