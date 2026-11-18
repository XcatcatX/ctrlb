use super::link_resolver;
use super::safe_paths;
use rand::Rng;
use rand::distributions::Alphanumeric;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

type Writer = ZipWriter<BufWriter<File>>;

// Protects against links that point back to a parent folder.
const MAX_DEPTH: usize = 32;

/// Returns a new file name in the temp folder for a ZIP file.
pub fn new_temp_zip_path() -> PathBuf {
    let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    std::env::temp_dir().join(format!("datei-browser-{id}.zip"))
}

/// Turns web paths into (real path, name in the ZIP) pairs.
/// The main folder itself is not added, only its content.
pub fn items_for_paths(root: &Path, relatives: &[String]) -> Vec<(PathBuf, String)> {
    let mut items = Vec::new();
    for relative in relatives {
        let Some((clean, full)) = safe_paths::locate(root, relative) else {
            continue;
        };
        if clean.is_empty() {
            add_children(&full, &mut items);
        } else if full.exists() {
            items.push((full, safe_paths::last_segment(&clean).to_string()));
        }
    }
    items
}

pub fn build_zip(items: &[(PathBuf, String)], zip_path: &Path) -> Result<(), String> {
    write_zip(items, zip_path).map_err(|error| error.to_string())
}

fn write_zip(items: &[(PathBuf, String)], zip_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = ZipWriter::new(BufWriter::new(File::create(zip_path)?));
    for (path, name) in items {
        add_item(&mut writer, path, name, 0)?;
    }
    writer.finish()?;
    Ok(())
}

fn add_children(folder: &Path, items: &mut Vec<(PathBuf, String)>) {
    if let Ok(children) = link_resolver::visible_children(folder) {
        items.extend(children.into_iter().map(|child| (child.path, child.name)));
    }
}

fn add_item(
    writer: &mut Writer,
    path: &Path,
    name: &str,
    depth: usize,
) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        add_folder(writer, path, name, depth)
    } else {
        add_file(writer, path, name)
    }
}

fn add_folder(
    writer: &mut Writer,
    path: &Path,
    name: &str,
    depth: usize,
) -> Result<(), Box<dyn Error>> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    writer.add_directory(format!("{name}/"), SimpleFileOptions::default())?;
    for child in link_resolver::visible_children(path)? {
        let child_name = format!("{name}/{}", child.name);
        add_item(writer, &child.path, &child_name, depth + 1)?;
    }
    Ok(())
}

fn add_file(writer: &mut Writer, path: &Path, name: &str) -> Result<(), Box<dyn Error>> {
    let size = std::fs::metadata(path)?.len();
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .large_file(size >= u32::MAX as u64);
    writer.start_file(name, options)?;
    let mut input = File::open(path)?;
    io::copy(&mut input, writer)?;
    Ok(())
}
