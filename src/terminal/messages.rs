use crate::server::state::AppState;
use crate::tunnel;

pub fn print_banner(state: &AppState) {
    println!("Datei-Browser");
    println!("Type \"help\" to see all commands.");
    if state.password().is_empty() {
        println!();
        println!("WARNING: No password is set.");
        println!("Anyone with the link can see, change and delete all files.");
        println!("Set a password with: set password <text>");
    }
    println!();
}

/// Shown when the terminal is opened again from the tray.
pub fn print_welcome_back(state: &AppState) {
    print_banner(state);
    match state.tunnel_link() {
        Some(link) => tunnel::print_link(&link),
        None => println!("No link yet. The tunnel is not running or is still starting."),
    }
}
