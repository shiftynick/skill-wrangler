use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

use crate::skill::SkillEntry;

#[derive(Debug, Clone)]
pub struct FolderFileEntry {
    pub relative_path: String,
    pub absolute_path: String,
    pub size_bytes: u64,
    pub is_binary: bool,
}

/// Hash of all files in a folder (relative path + per-file content hash), order-independent.
pub fn compute_folder_content_hash(dir: &Path) -> io::Result<String> {
    let mut file_hashes: Vec<(String, String)> = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
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

pub fn list_folder_files(dir: &Path) -> io::Result<Vec<FolderFileEntry>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
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
        files.push(FolderFileEntry {
            relative_path: rel_str,
            absolute_path: entry.path().to_string_lossy().to_string(),
            size_bytes: meta.len(),
            is_binary: is_probably_binary(&bytes),
        });
    }

    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

pub fn read_folder_file(path: &Path, max_bytes: usize) -> io::Result<(String, bool, bool)> {
    let meta = fs::metadata(path)?;
    let truncated = meta.len() as usize > max_bytes;
    let bytes = if truncated {
        let file = fs::File::open(path)?;
        use std::io::Read;
        let mut handle = file;
        let mut buf = vec![0u8; max_bytes];
        let read = handle.read(&mut buf)?;
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

fn hash_bytes(bytes: &[u8]) -> String {
    short_hash(&Sha256::digest(bytes))
}

fn short_hash(hash: &[u8]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect::<String>()[..16].to_string()
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillListItem {
    pub id: String,
    pub folder_name: String,
    pub description: Option<String>,
    pub parent_context: String,
    pub path: String,
    pub skill_md_path: String,
    pub content_hash: String,
    pub identical_copy_count: u32,
    pub all_paths: Vec<String>,
    pub variant_label: Option<String>,
}

struct ScoredEntry {
    entry: SkillEntry,
    content_hash: String,
}

pub fn group_skills(entries: Vec<SkillEntry>) -> Result<Vec<SkillListItem>, String> {
    use std::collections::HashMap;

    let mut by_name: HashMap<String, Vec<ScoredEntry>> = HashMap::new();

    for entry in entries {
        let hash = compute_folder_content_hash(Path::new(&entry.path))
            .map_err(|e| format!("Failed to hash {}: {e}", entry.path))?;
        let key = entry.folder_name.to_ascii_lowercase();
        by_name
            .entry(key)
            .or_default()
            .push(ScoredEntry { entry, content_hash: hash });
    }

    let mut items = Vec::new();

    for (_key, mut group) in by_name {
        group.sort_by(|a, b| a.entry.path.cmp(&b.entry.path));

        let folder_name = group[0].entry.folder_name.clone();
        let mut by_hash: HashMap<String, Vec<&ScoredEntry>> = HashMap::new();
        for item in &group {
            by_hash
                .entry(item.content_hash.clone())
                .or_default()
                .push(item);
        }

        let mut hash_groups: Vec<_> = by_hash.into_iter().collect();
        hash_groups.sort_by(|a, b| a.1[0].entry.path.cmp(&b.1[0].entry.path));

        let variant_count = hash_groups.len();

        for (variant_idx, (content_hash, instances)) in hash_groups.into_iter().enumerate() {
            let primary = instances[0].entry.clone();
            let all_paths: Vec<String> = instances.iter().map(|i| i.entry.path.clone()).collect();
            let identical_copy_count = all_paths.len() as u32;

            let variant_label = if variant_count > 1 {
                Some(format!(
                    "variant {} of {} — {}",
                    variant_idx + 1,
                    variant_count,
                    primary.parent_context
                ))
            } else if identical_copy_count > 1 {
                Some(format!("{identical_copy_count} identical copies"))
            } else {
                None
            };

            let id = format!("{folder_name}:{content_hash}");

            items.push(SkillListItem {
                id,
                folder_name: folder_name.clone(),
                description: primary.description.clone(),
                parent_context: primary.parent_context.clone(),
                path: primary.path.clone(),
                skill_md_path: primary.skill_md_path.clone(),
                content_hash,
                identical_copy_count,
                all_paths,
                variant_label,
            });
        }
    }

    items.sort_by(|a, b| {
        a.folder_name
            .to_ascii_lowercase()
            .cmp(&b.folder_name.to_ascii_lowercase())
            .then_with(|| {
                a.variant_label
                    .as_deref()
                    .unwrap_or("")
                    .cmp(b.variant_label.as_deref().unwrap_or(""))
            })
    });

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::build_skill_entry;
    use std::fs;
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

    #[test]
    fn groups_identical_and_variants() {
        let tmp = TempDir::new().unwrap();
        let path1 = tmp.path().join("path1");
        let path2 = tmp.path().join("path2");
        write_skill(&path1, "my-skill", "body", Some(("x.txt", "1")));
        write_skill(&path2, "my-skill", "body", Some(("x.txt", "2")));

        let entries = vec![
            build_skill_entry(&path1.join("my-skill")).unwrap(),
            build_skill_entry(&path2.join("my-skill")).unwrap(),
        ];

        let grouped = group_skills(entries).unwrap();
        assert_eq!(grouped.len(), 2);
        assert!(grouped.iter().all(|g| g.folder_name == "my-skill"));
    }
}
