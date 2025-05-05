use windows::Win32::System::Console::GetConsoleProcessList;

pub fn is_inheriting_console() -> bool {
    // https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist
    let mut buffer = [0u32; 1];
    let rtn = unsafe { GetConsoleProcessList(&mut buffer) };
    println!("GetConsoleProcessList returned: {}", rtn);
    let is_standalone = rtn != 1;
    return is_standalone;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_opened_from_console() {
        assert!(is_inheriting_console());
    }
}
