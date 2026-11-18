use crate::logic::pin_store::PinStore;
use crate::settings::{self, Settings};
use rand::Rng;
use rand::distributions::Alphanumeric;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

pub struct SharedState {
    settings: RwLock<Settings>,
    storage_root: RwLock<PathBuf>,
    session_token: RwLock<String>,
    pub pins: Mutex<PinStore>,
    pub tunnel_link: Arc<Mutex<Option<String>>>,
}

pub type AppState = Arc<SharedState>;

impl SharedState {
    pub fn new(initial: Settings) -> AppState {
        let storage_root = initial.storage_root();
        let pins_file = settings::exe_folder().join("pins.json");
        Arc::new(SharedState {
            settings: RwLock::new(initial),
            storage_root: RwLock::new(storage_root),
            session_token: RwLock::new(new_token()),
            pins: Mutex::new(PinStore::load(pins_file)),
            tunnel_link: Arc::new(Mutex::new(None)),
        })
    }

    pub fn settings_copy(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn port(&self) -> u16 {
        self.settings.read().unwrap().port
    }

    pub fn password(&self) -> String {
        self.settings.read().unwrap().password.clone()
    }

    /// Changes one setting and saves it. A new password logs everybody out.
    pub fn change_setting(&self, name: &str, value: &str) -> Result<(), String> {
        {
            let mut settings = self.settings.write().unwrap();
            settings.set_value(name, value)?;
            settings.save();
        }
        if name == "password" {
            self.reset_sessions();
        }
        Ok(())
    }

    pub fn storage_root(&self) -> PathBuf {
        self.storage_root.read().unwrap().clone()
    }

    /// Takes the storage folder from the settings. Called when the server starts.
    pub fn refresh_storage_root(&self) {
        let root = self.settings.read().unwrap().storage_root();
        *self.storage_root.write().unwrap() = root;
    }

    pub fn session_token(&self) -> String {
        self.session_token.read().unwrap().clone()
    }

    pub fn is_valid_token(&self, token: &str) -> bool {
        *self.session_token.read().unwrap() == token
    }

    pub fn reset_sessions(&self) {
        *self.session_token.write().unwrap() = new_token();
    }

    pub fn tunnel_link(&self) -> Option<String> {
        self.tunnel_link.lock().unwrap().clone()
    }
}

fn new_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}
