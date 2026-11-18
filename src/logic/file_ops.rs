use super::safe_paths::is_safe_name;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub fn create_folder(parent: &Path, name: &str) -> Result<(), String> {
    let name = name.trim();
    if !is_safe_name(name) {
        return Err("This folder name is not allowed.".to_string());
    }
    match fs::create_dir(parent.join(name)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            Err("A file or folder with this name already exists.".to_string())
        }
        Err(error) => Err(format!("Could not create the folder: {error}")),
    }
}

/// Deletes one item. For a link or shortcut only the link is removed,
/// never the content it points to.
pub fn delete_entry(parent: &Path, name: &str) -> io::Result<()> {
    let direct = parent.join(name);
    match fs::symlink_metadata(&direct) {
        Ok(metadata) if metadata.file_type().is_symlink() => remove_link(&direct),
        Ok(_) => delete_path(&direct),
        Err(_) => fs::remove_file(parent.join(format!("{name}.lnk"))),
    }
}

fn delete_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn remove_link(path: &Path) -> io::Result<()> {
    fs::remove_file(path).or_else(|_| fs::remove_dir(path))
}
