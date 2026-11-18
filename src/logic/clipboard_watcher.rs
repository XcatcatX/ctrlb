use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use arboard::Clipboard;

use crate::logic::history_file;
use crate::logic::history_list::SharedHistory;
use crate::theme;

// The text we saw last in the clipboard. Used to ignore texts we already know.
pub type SharedLastText = Arc<Mutex<String>>;

struct WatcherContext {
    history: SharedHistory,
    last_text: SharedLastText,
    file_path: PathBuf,
}

// Starts a thread that checks the clipboard and saves every new text
pub fn start(history: SharedHistory, last_text: SharedLastText, file_path: PathBuf) {
    let context = WatcherContext {
        history,
        last_text,
        file_path,
    };
    thread::spawn(move || run_loop(context));
}

// Puts a text into the clipboard without it counting as a new copy
pub fn put_text_in_clipboard(last_text: &SharedLastText, text: &str) {
    let mut last = last_text.lock().unwrap();
    match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text)) {
        Ok(()) => *last = text.to_string(),
        Err(error) => eprintln!("Cannot write to clipboard: {error}"),
    }
}

fn run_loop(context: WatcherContext) {
    let mut clipboard = match Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(error) => {
            eprintln!("Cannot open clipboard: {error}");
            return;
        }
    };
    loop {
        // Errors happen when the clipboard has no text (e.g. an image). Ignore them.
        if let Ok(text) = clipboard.get_text() {
            handle_text(&context, text);
        }
        thread::sleep(theme::CLIPBOARD_POLL_INTERVAL);
    }
}

fn handle_text(context: &WatcherContext, text: String) {
    if text.trim().is_empty() || !remember_if_new(&context.last_text, &text) {
        return;
    }
    add_to_history(context, text);
}

fn remember_if_new(last_text: &SharedLastText, text: &str) -> bool {
    let mut last = last_text.lock().unwrap();
    if *last == text {
        return false;
    }
    *last = text.to_string();
    true
}

fn add_to_history(context: &WatcherContext, text: String) {
    let mut history = context.history.lock().unwrap();
    history.add(text);
    history_file::save(&context.file_path, history.items());
}
