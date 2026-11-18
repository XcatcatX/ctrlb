use std::path::{Path, PathBuf};

// Reads the target of a Windows shortcut (.lnk file).
// Only the parts needed to find a local path are read.

const HEADER_SIZE: usize = 0x4C;
const FLAG_HAS_TARGET_LIST: u32 = 0x01;
const FLAG_HAS_LINK_INFO: u32 = 0x02;
const INFO_HAS_LOCAL_PATH: u32 = 0x01;
const INFO_HEADER_WITH_UNICODE: usize = 0x24;
const MAX_FILE_SIZE: u64 = 65536;

pub fn read_shortcut_target(file: &Path) -> Option<PathBuf> {
    let metadata = std::fs::metadata(file).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_SIZE {
        return None;
    }
    let bytes = std::fs::read(file).ok()?;
    parse_target(&bytes)
}

fn parse_target(bytes: &[u8]) -> Option<PathBuf> {
    if read_u32(bytes, 0)? as usize != HEADER_SIZE {
        return None;
    }
    let flags = read_u32(bytes, 20)?;
    if flags & FLAG_HAS_LINK_INFO == 0 {
        return None;
    }
    let mut position = HEADER_SIZE;
    if flags & FLAG_HAS_TARGET_LIST != 0 {
        position += 2 + read_u16(bytes, position)? as usize;
    }
    parse_link_info(bytes.get(position..)?)
}

fn parse_link_info(info: &[u8]) -> Option<PathBuf> {
    let header_size = read_u32(info, 4)? as usize;
    if read_u32(info, 8)? & INFO_HAS_LOCAL_PATH == 0 {
        return None;
    }
    let text = read_unicode_path(info, header_size).or_else(|| read_ansi_path(info))?;
    Some(PathBuf::from(text))
}

fn read_unicode_path(info: &[u8], header_size: usize) -> Option<String> {
    if header_size < INFO_HEADER_WITH_UNICODE {
        return None;
    }
    let base_offset = read_u32(info, 28)? as usize;
    if base_offset == 0 {
        return None;
    }
    let suffix_offset = read_u32(info, 32)? as usize;
    let base = read_utf16(info, base_offset)?;
    let suffix = read_utf16(info, suffix_offset).unwrap_or_default();
    Some(base + &suffix)
}

fn read_ansi_path(info: &[u8]) -> Option<String> {
    let base = read_ansi(info, read_u32(info, 16)? as usize)?;
    let suffix = read_ansi(info, read_u32(info, 24)? as usize).unwrap_or_default();
    Some(base + &suffix)
}

// Old Windows code page text. Letters like ä, ö, ü are read as Latin-1.
fn read_ansi(info: &[u8], offset: usize) -> Option<String> {
    let rest = info.get(offset..)?;
    let end = rest.iter().position(|&byte| byte == 0)?;
    Some(rest[..end].iter().map(|&byte| char::from(byte)).collect())
}

fn read_utf16(info: &[u8], offset: usize) -> Option<String> {
    let rest = info.get(offset..)?;
    let mut units = Vec::new();
    for pair in rest.chunks_exact(2) {
        let unit = u16::from_le_bytes([pair[0], pair[1]]);
        if unit == 0 {
            return Some(String::from_utf16_lossy(&units));
        }
        units.push(unit);
    }
    None
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let part: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(part))
}

fn read_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let part: [u8; 2] = bytes.get(at..at + 2)?.try_into().ok()?;
    Some(u16::from_le_bytes(part))
}
