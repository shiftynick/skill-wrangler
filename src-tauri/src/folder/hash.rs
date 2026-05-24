use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

use crate::skill::default_ignore_patterns;
use crate::walk::walk_filtered;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileManifestEntry {
    pub relative_path: String,
    pub content_hash: String,
    pub size_bytes: u64,
    pub is_binary: bool,
}

/// Per-file manifest for a skill folder (relative path, hash, size, binary flag).
pub fn build_folder_manifest(dir: &Path) -> io::Result<Vec<FileManifestEntry>> {
    let mut entries = Vec::new();
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
        let bytes = fs::read(entry.path())?;
        entries.push(FileManifestEntry {
            relative_path: rel_str,
            content_hash: hash_bytes(&bytes),
            size_bytes: meta.len(),
            is_binary: is_probably_binary_from_bytes(&bytes),
        });
    }

    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(entries)
}

fn is_probably_binary_from_bytes(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    if bytes.contains(&0) {
        return true;
    }
    let sample = &bytes[..bytes.len().min(8 * 1024)];
    let non_text = sample
        .iter()
        .filter(|&&b| b != b'\n' && b != b'\r' && b != b'\t' && (b < 0x20 || b == 0x7F))
        .count();
    non_text * 10 > sample.len()
}

/// Hash of all files in a folder (relative path + per-file content hash), order-independent.
pub fn compute_folder_content_hash(dir: &Path) -> io::Result<String> {
    let manifest = build_folder_manifest(dir)?;
    let mut hasher = Sha256::new();
    for entry in &manifest {
        hasher.update(entry.relative_path.as_bytes());
        hasher.update(b"\0");
        hasher.update(entry.content_hash.as_bytes());
        hasher.update(b"\n");
    }
    Ok(short_hash(&hasher.finalize()))
}

fn hash_bytes(bytes: &[u8]) -> String {
    short_hash(&Sha256::digest(bytes))
}

fn short_hash(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..16]
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn write_skill(base: &Path, name: &str, skill_body: &str, extra: Option<(&str, &str)>) {
        let dir = base.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            format!("---\ndescription: Test\n---\n\n{skill_body}\n"),
        )
        .unwrap();
        if let Some((fname, content)) = extra {
            fs::write(dir.join(fname), content).unwrap();
        }
    }

    #[test]
    fn identical_folders_share_hash() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "a", "same", Some(("helper.txt", "hello")));
        write_skill(tmp.path(), "b", "same", Some(("helper.txt", "hello")));
        let ha = compute_folder_content_hash(&tmp.path().join("a")).unwrap();
        let hb = compute_folder_content_hash(&tmp.path().join("b")).unwrap();
        assert_eq!(ha, hb);
    }

    #[test]
    fn different_contents_differ_hash() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "a", "same", Some(("helper.txt", "hello")));
        write_skill(tmp.path(), "b", "same", Some(("helper.txt", "world")));
        let ha = compute_folder_content_hash(&tmp.path().join("a")).unwrap();
        let hb = compute_folder_content_hash(&tmp.path().join("b")).unwrap();
        assert_ne!(ha, hb);
    }
}
