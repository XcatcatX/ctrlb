use super::help_text;
use crate::runtime::Runtime;
use crate::server::state::AppState;
use crate::settings::{self, SETTING_LIST};
use crate::tunnel;

#[derive(PartialEq)]
pub enum Next {
    Continue,
    Tray,
    Quit,
}

/// Runs one line typed by the user.
pub async fn run_command(line: &str, runtime: &mut Runtime) -> Next {
    let (command, rest) = split_first_word(line);
    match command.to_lowercase().as_str() {
        "" => {}
        "help" => print!("{}", help_text::build_help(&runtime.state().settings_copy())),
        "settings" => show_settings(runtime.state()),
        "set" => set_setting(rest, runtime.state()),
        "status" => show_status(runtime),
        "link" => show_link(runtime.state()),
        "restart" => runtime.restart().await,
        "tray" => return Next::Tray,
        "stop" => {
            println!("Stopping...");
            runtime.stop().await;
            return Next::Quit;
        }
        _ => println!("Unknown command. Type \"help\" to see all commands."),
    }
    Next::Continue
}

fn split_first_word(text: &str) -> (&str, &str) {
    let text = text.trim();
    match text.split_once(char::is_whitespace) {
        Some((first, rest)) => (first, rest.trim()),
        None => (text, ""),
    }
}

fn show_settings(state: &AppState) {
    let settings = state.settings_copy();
    for info in SETTING_LIST {
        println!("{} = {}", info.name, settings.display_value(info.name));
    }
}

fn set_setting(arguments: &str, state: &AppState) {
    let (name, value) = split_first_word(arguments);
    if name.is_empty() {
        println!("Usage: set <name> <value>");
        println!("Example: set port 9000");
        return;
    }
    let name = name.to_lowercase();
    match state.change_setting(&name, value) {
        Ok(()) if settings::needs_restart(&name) => {
            println!("Saved. Type \"restart\" to apply the change.")
        }
        Ok(()) => println!("Saved."),
        Err(message) => println!("{message}"),
    }
}

fn show_status(runtime: &mut Runtime) {
    let server_running = runtime.server_running();
    let tunnel_running = runtime.tunnel_running();
    let state = runtime.state();
    let settings = state.settings_copy();
    if server_running {
        println!("Server: running on port {}", settings.port);
    } else {
        println!("Server: not running");
    }
    println!("Tunnel: {}", if tunnel_running { "running" } else { "not running" });
    println!("Link: {}", state.tunnel_link().unwrap_or_else(|| "none".to_string()));
    println!("Password: {}", if settings.password.is_empty() { "not set" } else { "set" });
    println!("Storage folder: {}", state.storage_root().display());
}

fn show_link(state: &AppState) {
    match state.tunnel_link() {
        Some(link) => tunnel::print_link(&link),
        None => println!("No link yet. The tunnel is not running or is still starting."),
    }
}
