use super::link_resolver;
use serde::Serialize;
use std::fs::Metadata;
use std::io;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct EntryInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_link: bool,
    pub size: u64,
}

/// Lists one folder. Folders come first, then files, both sorted by name.
pub fn list_folder(folder: &Path, folder_relative: &str) -> io::Result<Vec<EntryInfo>> {
    let mut entries: Vec<EntryInfo> = link_resolver::visible_children(folder)?
        .into_iter()
        .filter_map(|child| {
            let metadata = std::fs::metadata(&child.path).ok()?;
            let path = join_relative(folder_relative, &child.name);
            Some(build_entry(path, child.name, &metadata, child.is_link))
        })
        .collect();
    entries.sort_by_cached_key(|entry| (!entry.is_dir, entry.name.to_lowercase()));
    Ok(entries)
}

/// Reads info about one file or folder. Returns None if it does not exist.
pub fn entry_for(path: &Path, relative: &str) -> Option<EntryInfo> {
    let metadata = std::fs::metadata(path).ok()?;
    let name = relative.rsplit('/').next()?.to_string();
    Some(build_entry(relative.to_string(), name, &metadata, false))
}

fn join_relative(folder_relative: &str, name: &str) -> String {
    if folder_relative.is_empty() {
        name.to_string()
    } else {
        format!("{folder_relative}/{name}")
    }
}

fn build_entry(path: String, name: String, metadata: &Metadata, is_link: bool) -> EntryInfo {
    let is_dir = metadata.is_dir();
    EntryInfo {
        name,
        path,
        is_dir,
        is_link,
        size: if is_dir { 0 } else { metadata.len() },
    }
}
