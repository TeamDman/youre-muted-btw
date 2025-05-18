use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;

use ymb_args::GlobalArgs;

pub static OUR_HWND: AtomicUsize = AtomicUsize::new(0);
pub static SHOULD_SHOW_HIDE_LOGS_TRAY_ACTION: AtomicBool = AtomicBool::new(false);
pub static GLOBAL_ARGS: OnceLock<GlobalArgs> = OnceLock::new();
