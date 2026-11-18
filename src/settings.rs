use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SETTINGS_FILE_NAME: &str = "settings.json";

/// Describes one setting. The "help" command is built from this list.
pub struct SettingInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub needs_restart: bool,
}

pub const SETTING_LIST: &[SettingInfo] = &[
    SettingInfo {
        name: "port",
        description: "Local port of the server (1 to 65535).",
        example: "set port 9000",
        needs_restart: true,
    },
    SettingInfo {
        name: "password",
        description: "Password for the website. Empty means no password. Use \"set password\" alone to remove it.",
        example: "set password secret123",
        needs_restart: false,
    },
    SettingInfo {
        name: "storage_folder",
        description: "Folder where the files are stored. A relative path starts next to the program.",
        example: "set storage_folder D:\\Files",
        needs_restart: true,
    },
];

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub port: u16,
    pub password: String,
    pub storage_folder: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: 8080,
            password: String::new(),
            storage_folder: "storage".to_string(),
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        let Ok(text) = std::fs::read_to_string(settings_file_path()) else {
            return Settings::default();
        };
        serde_json::from_str(&text).unwrap_or_else(|_| {
            println!("Could not read settings.json. Using default settings.");
            Settings::default()
        })
    }

    pub fn save(&self) {
        let Ok(text) = serde_json::to_string_pretty(self) else {
            return;
        };
        if let Err(error) = std::fs::write(settings_file_path(), text) {
            println!("Could not save settings: {error}");
        }
    }

    pub fn storage_root(&self) -> PathBuf {
        exe_folder().join(&self.storage_folder)
    }

    pub fn value_of(&self, name: &str) -> Option<String> {
        match name {
            "port" => Some(self.port.to_string()),
            "password" => Some(self.password.clone()),
            "storage_folder" => Some(self.storage_folder.clone()),
            _ => None,
        }
    }

    pub fn display_value(&self, name: &str) -> String {
        match self.value_of(name) {
            Some(value) if !value.is_empty() => value,
            _ => "(empty)".to_string(),
        }
    }

    pub fn set_value(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            "port" => self.port = parse_port(value)?,
            "password" => self.password = value.to_string(),
            "storage_folder" => self.storage_folder = parse_folder(value)?,
            _ => {
                return Err(format!(
                    "Unknown setting \"{name}\". Type \"settings\" to see all settings."
                ));
            }
        }
        Ok(())
    }
}

pub fn needs_restart(name: &str) -> bool {
    SETTING_LIST
        .iter()
        .any(|info| info.name == name && info.needs_restart)
}

pub fn exe_folder() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|folder| folder.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn settings_file_path() -> PathBuf {
    exe_folder().join(SETTINGS_FILE_NAME)
}

fn parse_port(value: &str) -> Result<u16, String> {
    match value.parse::<u16>() {
        Ok(port) if port > 0 => Ok(port),
        _ => Err("The port must be a number from 1 to 65535.".to_string()),
    }
}

fn parse_folder(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err("The folder cannot be empty. Example: set storage_folder files".to_string());
    }
    Ok(value.to_string())
}
