use crate::background_command::background_command;
use crate::settings;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Child;

pub type LinkSlot = Arc<Mutex<Option<String>>>;

/// Starts cloudflared as a child process. Returns None if it cannot be started.
pub fn start(port: u16, link_slot: LinkSlot) -> Option<Child> {
    let mut command = background_command(find_cloudflared());
    command
        .arg("tunnel")
        .arg("--url")
        .arg(format!("http://localhost:{port}"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            print_start_error(&error);
            return None;
        }
    };
    if let Some(output) = child.stdout.take() {
        spawn_reader(output, link_slot.clone());
    }
    if let Some(output) = child.stderr.take() {
        spawn_reader(output, link_slot);
    }
    println!("Starting tunnel...");
    Some(child)
}

/// Stops cloudflared and forgets the link.
pub async fn stop(mut child: Child, link_slot: &LinkSlot) {
    let _ = child.kill().await;
    *link_slot.lock().unwrap() = None;
}

pub fn print_link(link: &str) {
    println!();
    println!("--------------------------------------------------");
    println!("Open this link: {link}");
    println!("--------------------------------------------------");
}

/// Looks next to the program first, then in the PATH.
fn find_cloudflared() -> PathBuf {
    let name = if cfg!(windows) { "cloudflared.exe" } else { "cloudflared" };
    let next_to_program = settings::exe_folder().join(name);
    if next_to_program.is_file() {
        next_to_program
    } else {
        PathBuf::from("cloudflared")
    }
}

fn print_start_error(error: &std::io::Error) {
    if error.kind() == ErrorKind::NotFound {
        println!("cloudflared was not found, so there is no public link.");
        println!("Put cloudflared next to this program or install it in the PATH.");
        println!("Then type \"restart\". The server runs on this computer only.");
    } else {
        println!("Could not start cloudflared: {error}");
    }
}

/// Reads the output of cloudflared line by line until it ends.
fn spawn_reader<R>(output: R, link_slot: LinkSlot)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(output).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(link) = find_link(&line) {
                remember_link(&link_slot, link);
            }
        }
    });
}

fn remember_link(link_slot: &LinkSlot, link: String) {
    let mut slot = link_slot.lock().unwrap();
    if slot.as_deref() == Some(link.as_str()) {
        return;
    }
    print_link(&link);
    *slot = Some(link);
}

fn find_link(line: &str) -> Option<String> {
    let start = line.find("https://")?;
    let text = &line[start..];
    let end = text
        .find(|c: char| c.is_whitespace() || c == '|')
        .unwrap_or(text.len());
    let link = &text[..end];
    let is_tunnel_link = link.ends_with(".trycloudflare.com") && !link.starts_with("https://api.");
    is_tunnel_link.then(|| link.to_string())
}
