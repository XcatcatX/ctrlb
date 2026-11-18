// Closing and opening the terminal window, and reading a line from it.
// Windows does the real work. On other systems these functions do almost nothing.

#[cfg(windows)]
use std::ffi::c_void;

#[cfg(windows)]
const OWN_CONSOLE_FLAG: &str = "--own-console";
#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
#[cfg(windows)]
const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
#[cfg(windows)]
const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5;
#[cfg(windows)]
const STD_ERROR_HANDLE: u32 = 0xFFFF_FFF4;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn FreeConsole() -> i32;
    fn AllocConsole() -> i32;
    fn GetStdHandle(handle_id: u32) -> *mut c_void;
    fn SetStdHandle(handle_id: u32, handle: *mut c_void) -> i32;
    fn GetConsoleProcessList(process_list: *mut u32, process_count: u32) -> u32;
    fn ReadConsoleW(
        console_input: *mut c_void,
        buffer: *mut c_void,
        chars_to_read: u32,
        chars_read: *mut u32,
        input_control: *const c_void,
    ) -> i32;
}

/// Reads one line from the terminal. Returns None when the terminal is gone.
#[cfg(windows)]
pub fn read_line() -> Option<String> {
    let mut buffer = [0u16; 1024];
    let mut count: u32 = 0;
    // SAFETY: the buffer is valid for the number of characters we pass.
    let ok = unsafe {
        ReadConsoleW(
            GetStdHandle(STD_INPUT_HANDLE),
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
            &mut count,
            std::ptr::null(),
        )
    };
    if ok == 0 || count == 0 {
        return None;
    }
    let text = String::from_utf16_lossy(&buffer[..count as usize]);
    Some(text.trim_end_matches(|c| c == '\r' || c == '\n').to_string())
}

#[cfg(not(windows))]
pub fn read_line() -> Option<String> {
    let mut text = String::new();
    match std::io::stdin().read_line(&mut text) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(text.trim_end_matches(|c| c == '\r' || c == '\n').to_string()),
    }
}

/// Closes the terminal window. The program keeps running.
#[cfg(windows)]
pub fn close_console() {
    // SAFETY: FreeConsole has no arguments.
    unsafe {
        FreeConsole();
    }
}

#[cfg(not(windows))]
pub fn close_console() {}

/// Opens a new terminal window for the running program.
#[cfg(windows)]
pub fn open_console() {
    // SAFETY: AllocConsole has no arguments.
    unsafe {
        AllocConsole();
    }
    bind_standard_handles();
}

#[cfg(not(windows))]
pub fn open_console() {}

/// If the program was started inside another terminal (for example PowerShell),
/// it opens itself in its own terminal window and returns true.
/// Only a program with its own terminal window can close that window.
#[cfg(windows)]
pub fn relaunch_in_own_console() -> bool {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    if std::env::args().any(|argument| argument == OWN_CONSOLE_FLAG) {
        bind_standard_handles();
        return false;
    }
    if attached_process_count() <= 1 {
        return false;
    }
    let Ok(program) = std::env::current_exe() else {
        return false;
    };
    Command::new(program)
        .arg(OWN_CONSOLE_FLAG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .is_ok()
}

#[cfg(not(windows))]
pub fn relaunch_in_own_console() -> bool {
    false
}

/// Number of programs that use the current terminal window.
#[cfg(windows)]
fn attached_process_count() -> u32 {
    let mut list = [0u32; 8];
    // SAFETY: the list is valid for the number of entries we pass.
    unsafe { GetConsoleProcessList(list.as_mut_ptr(), list.len() as u32) }
}

/// Connects input and output of the program to the current terminal window.
#[cfg(windows)]
fn bind_standard_handles() {
    use std::fs::OpenOptions;
    use std::os::windows::io::IntoRawHandle;

    if let Ok(output) = OpenOptions::new().read(true).write(true).open("CONOUT$") {
        let handle = output.into_raw_handle();
        // SAFETY: the handle is valid and stays open for the whole run.
        unsafe {
            SetStdHandle(STD_OUTPUT_HANDLE, handle);
            SetStdHandle(STD_ERROR_HANDLE, handle);
        }
    }
    if let Ok(input) = OpenOptions::new().read(true).write(true).open("CONIN$") {
        // SAFETY: the handle is valid and stays open for the whole run.
        unsafe {
            SetStdHandle(STD_INPUT_HANDLE, input.into_raw_handle());
        }
    }
}
