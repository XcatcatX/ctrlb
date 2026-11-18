use super::console_control;
use tokio::sync::mpsc::{self, UnboundedReceiver};

/// Reads lines from the terminal in its own thread, so the server never waits.
pub fn start_input_thread() -> UnboundedReceiver<String> {
    let (sender, receiver) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        while let Some(line) = console_control::read_line() {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}
