use std::ffi::OsStr;
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Creates a command for a program that runs in the background.
/// On Windows it gets no window and is not connected to our terminal,
/// so the terminal can be closed while the program keeps running.
pub fn background_command(program: impl AsRef<OsStr>) -> Command {
    #[allow(unused_mut)]
    let mut command = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut command = Command::from(command);
    command.kill_on_drop(true);
    command
}
