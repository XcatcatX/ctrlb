mod background_command;
mod logic;
mod runtime;
mod server;
mod settings;
mod terminal;
mod tray;
mod tunnel;

use runtime::Runtime;
use server::state::SharedState;
use settings::Settings;

#[tokio::main]
async fn main() {
    if terminal::console_control::relaunch_in_own_console() {
        println!("Datei-Browser is open in its own terminal window.");
        return;
    }
    let state = SharedState::new(Settings::load());
    terminal::messages::print_banner(&state);
    let mut runtime = Runtime::new(state);
    runtime.start().await;
    terminal::session::run_session(&mut runtime).await;
    println!("Stopped.");
}
