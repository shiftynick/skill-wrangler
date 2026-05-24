use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::skill::default_ignore_patterns;
use crate::walk::walk_filtered;

const BINARY_PROBE_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderFileInfo {
    pub relative_path: String,
    pub absolute_path: String,
    pub size_bytes: u64,
    pub is_binary: bool,
}

pub fn list_folder_files(dir: &Path) -> io::Result<Vec<FolderFileInfo>> {
    let mut files = Vec::new();
    let ignores = default_ignore_patterns();

    for entry in walk_filtered(dir, &ignores).filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(dir)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let meta = entry.metadata()?;
        let probe = read_probe(entry.path())?;
        files.push(FolderFileInfo {
            relative_path: rel_str,
            absolute_path: entry.path().to_string_lossy().to_string(),
            size_bytes: meta.len(),
            is_binary: is_probably_binary(&probe),
        });
    }

    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

pub fn read_folder_file(path: &Path, max_bytes: usize) -> io::Result<(String, bool, bool)> {
    let meta = fs::metadata(path)?;
    let truncated = meta.len() as usize > max_bytes;
    let bytes = if truncated {
        let mut file = fs::File::open(path)?;
        let mut buf = vec![0u8; max_bytes];
        let read = file.read(&mut buf)?;
        buf.truncate(read);
        buf
    } else {
        fs::read(path)?
    };

    let binary = is_probably_binary(&bytes);
    if binary {
        Ok((
            format!(
                "[Binary file, {} bytes{}]",
                meta.len(),
                if truncated { " (preview truncated)" } else { "" }
            ),
            true,
            truncated,
        ))
    } else {
        Ok((
            String::from_utf8_lossy(&bytes).into_owned(),
            false,
            truncated,
        ))
    }
}

fn read_probe(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut buf = vec![0u8; BINARY_PROBE_BYTES];
    let read = file.read(&mut buf)?;
    buf.truncate(read);
    Ok(buf)
}

fn is_probably_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    if bytes.contains(&0) {
        return true;
    }
    let non_text = bytes
        .iter()
        .filter(|&&b| b != b'\n' && b != b'\r' && b != b'\t' && (b < 0x20 || b == 0x7F))
        .count();
    non_text * 10 > bytes.len()
}
