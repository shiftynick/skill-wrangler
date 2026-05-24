use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

use crate::walk::walk_filtered;
use crate::skill::default_ignore_patterns;

/// Hash of all files in a folder (relative path + per-file content hash), order-independent.
pub fn compute_folder_content_hash(dir: &Path) -> io::Result<String> {
    let mut file_hashes: Vec<(String, String)> = Vec::new();
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
        let bytes = fs::read(entry.path())?;
        file_hashes.push((rel_str, hash_bytes(&bytes)));
    }

    file_hashes.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (path, file_hash) in &file_hashes {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(file_hash.as_bytes());
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
