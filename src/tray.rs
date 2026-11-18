use crate::background_command::background_command;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

const TRAY_SCRIPT: &str = include_str!("tray_script.ps1");

// Must be the same text as in tray_script.ps1.
const EXIT_MARKER: &str = "::exit::";

pub enum TrayCommand {
    ShowTerminal,
    Quit,
}

/// The tray icon. It runs as a small PowerShell script in the background.
pub struct Tray {
    child: Child,
    commands: UnboundedReceiver<TrayCommand>,
    script_file: PathBuf,
    link_file: PathBuf,
    last_link: String,
}

impl Tray {
    pub fn start() -> Result<Tray, String> {
        let id = std::process::id();
        let folder = std::env::temp_dir();
        let script_file = folder.join(format!("datei-browser-tray-{id}.ps1"));
        let link_file = folder.join(format!("datei-browser-tray-{id}.txt"));
        std::fs::write(&script_file, TRAY_SCRIPT)
            .map_err(|error| format!("Could not prepare the tray: {error}"))?;
        std::fs::write(&link_file, "")
            .map_err(|error| format!("Could not prepare the tray: {error}"))?;

        let mut command = background_command("powershell");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-STA",
                "-WindowStyle",
                "Hidden",
                "-File",
            ])
            .arg(&script_file)
            .arg("-ParentId")
            .arg(id.to_string())
            .arg("-LinkFile")
            .arg(&link_file)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command.spawn().map_err(describe_spawn_error)?;
        let output = child
            .stdout
            .take()
            .ok_or_else(|| "Could not read from the tray.".to_string())?;
        let (sender, commands) = mpsc::unbounded_channel();
        spawn_reader(output, sender);
        Ok(Tray {
            child,
            commands,
            script_file,
            link_file,
            last_link: String::new(),
        })
    }

    /// Waits for a click in the tray. None means the tray program ended.
    pub async fn next_command(&mut self) -> Option<TrayCommand> {
        self.commands.recv().await
    }

    /// Shows the link in the tray menu. Only writes when the link changed.
    pub fn show_link(&mut self, link: Option<String>) {
        let text = link.unwrap_or_default();
        if text == self.last_link {
            return;
        }
        if std::fs::write(&self.link_file, &text).is_ok() {
            self.last_link = text;
        }
    }

    /// Removes the tray icon and ends the tray program.
    pub async fn stop(mut self) {
        let _ = std::fs::write(&self.link_file, EXIT_MARKER);
        let closed = tokio::time::timeout(Duration::from_secs(3), self.child.wait())
            .await
            .is_ok();
        if !closed {
            let _ = self.child.kill().await;
        }
        let _ = std::fs::remove_file(&self.script_file);
        let _ = std::fs::remove_file(&self.link_file);
    }
}

fn describe_spawn_error(error: std::io::Error) -> String {
    if error.kind() == ErrorKind::NotFound {
        "The tray needs Windows PowerShell. It is only available on Windows.".to_string()
    } else {
        format!("Could not start the tray: {error}")
    }
}

fn spawn_reader(output: ChildStdout, sender: UnboundedSender<TrayCommand>) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(output).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let command = match line.trim() {
                "show" => TrayCommand::ShowTerminal,
                "quit" => TrayCommand::Quit,
                _ => continue,
            };
            if sender.send(command).is_err() {
                break;
            }
        }
    });
}
