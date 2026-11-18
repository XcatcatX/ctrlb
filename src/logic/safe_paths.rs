use super::link_resolver;
use std::path::{Path, PathBuf};

/// Checks one file or folder name (no slashes allowed).
pub fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains(|c: char| matches!(c, '/' | '\\' | ':' | '\0'))
}

/// Cleans a web path like "docs//a.txt". Returns None if it is unsafe.
pub fn clean_relative(relative: &str) -> Option<String> {
    let mut parts = Vec::new();
    for part in relative.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if !is_safe_name(part) {
            return None;
        }
        parts.push(part);
    }
    Some(parts.join("/"))
}

/// Returns the clean web path and the real path.
/// Links and shortcuts inside the storage folder are followed.
pub fn locate(root: &Path, relative: &str) -> Option<(String, PathBuf)> {
    let clean = clean_relative(relative)?;
    let mut full = root.to_path_buf();
    if !clean.is_empty() {
        for part in clean.split('/') {
            full = link_resolver::resolve_child(&full, part).unwrap_or_else(|| full.join(part));
        }
    }
    Some((clean, full))
}

pub fn last_segment(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
