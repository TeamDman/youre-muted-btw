#[allow(non_snake_case)]
pub fn MAKEINTRESOURCEW(i: u16) -> windows::core::PCWSTR {
    windows::core::PCWSTR(i as usize as *const u16)
}
