use super::shortcut_reader::read_shortcut_target;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One visible item of a folder. For a link, `path` is the real target.
pub struct Child {
    pub name: String,
    pub path: PathBuf,
    pub is_link: bool,
}

/// Finds the real path of an item inside a folder.
/// A Windows shortcut "Name.lnk" counts as the item "Name".
/// Folder links (mklink) are followed by the system.
pub fn resolve_child(folder: &Path, name: &str) -> Option<PathBuf> {
    let direct = folder.join(name);
    if fs::symlink_metadata(&direct).is_ok() {
        return Some(direct);
    }
    shortcut_target(folder, name, &folder.join(format!("{name}.lnk")))
}

/// Lists the items of a folder. Shortcuts are shown under their own name.
/// Broken links are left out.
pub fn visible_children(folder: &Path) -> io::Result<Vec<Child>> {
    let mut children = Vec::new();
    for item in fs::read_dir(folder)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().into_owned();
        if let Some(child) = make_child(folder, name, item.path()) {
            children.push(child);
        }
    }
    Ok(children)
}

fn make_child(folder: &Path, name: String, path: PathBuf) -> Option<Child> {
    if let Some(stem) = strip_shortcut_ending(&name) {
        if let Some(target) = shortcut_target(folder, stem, &path) {
            return Some(Child { name: stem.to_string(), path: target, is_link: true });
        }
    }
    let is_link = fs::symlink_metadata(&path).ok()?.file_type().is_symlink();
    if is_link && fs::metadata(&path).is_err() {
        return None;
    }
    Some(Child { name, path, is_link })
}

fn shortcut_target(folder: &Path, stem: &str, shortcut_file: &Path) -> Option<PathBuf> {
    if fs::symlink_metadata(folder.join(stem)).is_ok() {
        return None;
    }
    let target = read_shortcut_target(shortcut_file)?;
    target.exists().then_some(target)
}

fn strip_shortcut_ending(name: &str) -> Option<&str> {
    let split = name.len().checked_sub(4)?;
    let stem = name.get(..split)?;
    let ending = name.get(split..)?;
    (ending.eq_ignore_ascii_case(".lnk") && !stem.is_empty()).then_some(stem)
}
