use std::io;
use std::path::PathBuf;

/// Keeps the list of pinned paths and saves it in a file.
pub struct PinStore {
    file: PathBuf,
    items: Vec<String>,
}

impl PinStore {
    pub fn load(file: PathBuf) -> Self {
        let items = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self { file, items }
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }

    pub fn set_pinned(&mut self, path: &str, pinned: bool) -> io::Result<()> {
        self.items.retain(|item| item != path);
        if pinned {
            self.items.push(path.to_string());
        }
        self.save()
    }

    fn save(&self) -> io::Result<()> {
        let text = serde_json::to_string_pretty(&self.items).map_err(io::Error::other)?;
        std::fs::write(&self.file, text)
    }
}
