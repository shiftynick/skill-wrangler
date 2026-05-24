use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::folder::build_folder_manifest;

pub const DIFF_MAX_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FileDiffStatus {
    Added,
    Removed,
    Unchanged,
    Modified,
    BinaryChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiffEntry {
    pub relative_path: String,
    pub status: FileDiffStatus,
    pub left_size_bytes: Option<u64>,
    pub right_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFolderDiff {
    pub left_path: String,
    pub right_path: String,
    pub files: Vec<FileDiffEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub tag: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFileDiff {
    pub relative_path: String,
    pub truncated: bool,
    pub lines: Vec<DiffLine>,
}

pub fn diff_skill_folders(left: &Path, right: &Path) -> Result<SkillFolderDiff, String> {
    let left_manifest = build_folder_manifest(left).map_err(|e| e.to_string())?;
    let right_manifest = build_folder_manifest(right).map_err(|e| e.to_string())?;

    let left_map: BTreeMap<_, _> = left_manifest
        .into_iter()
        .map(|e| (e.relative_path.clone(), e))
        .collect();
    let right_map: BTreeMap<_, _> = right_manifest
        .into_iter()
        .map(|e| (e.relative_path.clone(), e))
        .collect();

    let all_paths: BTreeSet<_> = left_map.keys().chain(right_map.keys()).cloned().collect();
    let mut files = Vec::new();

    for rel in all_paths {
        match (left_map.get(&rel), right_map.get(&rel)) {
            (None, Some(r)) => files.push(FileDiffEntry {
                relative_path: rel,
                status: FileDiffStatus::Added,
                left_size_bytes: None,
                right_size_bytes: Some(r.size_bytes),
            }),
            (Some(l), None) => files.push(FileDiffEntry {
                relative_path: rel,
                status: FileDiffStatus::Removed,
                left_size_bytes: Some(l.size_bytes),
                right_size_bytes: None,
            }),
            (Some(l), Some(r)) if l.content_hash == r.content_hash => files.push(FileDiffEntry {
                relative_path: rel,
                status: FileDiffStatus::Unchanged,
                left_size_bytes: Some(l.size_bytes),
                right_size_bytes: Some(r.size_bytes),
            }),
            (Some(l), Some(r)) => {
                let status = if l.is_binary || r.is_binary {
                    FileDiffStatus::BinaryChanged
                } else {
                    FileDiffStatus::Modified
                };
                files.push(FileDiffEntry {
                    relative_path: rel,
                    status,
                    left_size_bytes: Some(l.size_bytes),
                    right_size_bytes: Some(r.size_bytes),
                });
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(SkillFolderDiff {
        left_path: left.to_string_lossy().into_owned(),
        right_path: right.to_string_lossy().into_owned(),
        files,
    })
}

pub fn diff_skill_file(
    left_dir: &Path,
    right_dir: &Path,
    relative_path: &str,
) -> Result<TextFileDiff, String> {
    let left_path = left_dir.join(relative_path);
    let right_path = right_dir.join(relative_path);

    if !left_path.is_file() && !right_path.is_file() {
        return Err(format!("File not found: {relative_path}"));
    }

    let (left_text, left_trunc) = read_text_for_diff(&left_path)?;
    let (right_text, right_trunc) = read_text_for_diff(&right_path)?;
    let truncated = left_trunc || right_trunc;

    let diff = TextDiff::from_lines(&left_text, &right_text);
    let mut lines = Vec::new();
    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Equal => "equal",
            ChangeTag::Insert => "insert",
            ChangeTag::Delete => "delete",
        };
        let content = change.value().trim_end_matches(|c| c == '\n' || c == '\r').to_string();
        lines.push(DiffLine { tag: tag.to_string(), content });
    }

    Ok(TextFileDiff {
        relative_path: relative_path.to_string(),
        truncated,
        lines,
    })
}

fn read_text_for_diff(path: &Path) -> Result<(String, bool), String> {
    if !path.is_file() {
        return Ok((String::new(), false));
    }
    let meta = fs::metadata(path).map_err(|e| e.to_string())?;
    let truncated = meta.len() as usize > DIFF_MAX_BYTES;
    let bytes = if truncated {
        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; DIFF_MAX_BYTES];
        let read = file.read(&mut buf).map_err(|e| e.to_string())?;
        buf.truncate(read);
        buf
    } else {
        fs::read(path).map_err(|e| e.to_string())?
    };

    if bytes.contains(&0) {
        return Err("Cannot diff binary file".to_string());
    }

    Ok((String::from_utf8_lossy(&bytes).into_owned(), truncated))
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
    fn identical_folders_all_unchanged() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "a", "same", None);
        write_skill(tmp.path(), "b", "same", None);
        let diff = diff_skill_folders(&tmp.path().join("a"), &tmp.path().join("b")).unwrap();
        assert!(diff.files.iter().all(|f| f.status == FileDiffStatus::Unchanged));
    }

    #[test]
    fn detects_added_removed_and_modified() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "left", "v1", Some(("only-left.txt", "x")));
        write_skill(tmp.path(), "right", "v2", Some(("only-right.txt", "y")));

        let diff = diff_skill_folders(&tmp.path().join("left"), &tmp.path().join("right")).unwrap();
        let skill_md = diff
            .files
            .iter()
            .find(|f| f.relative_path == "SKILL.md")
            .unwrap();
        assert_eq!(skill_md.status, FileDiffStatus::Modified);

        assert!(
            diff.files
                .iter()
                .any(|f| f.relative_path == "only-left.txt" && f.status == FileDiffStatus::Removed)
        );
        assert!(
            diff.files
                .iter()
                .any(|f| f.relative_path == "only-right.txt" && f.status == FileDiffStatus::Added)
        );
    }

    #[test]
    fn text_file_diff_has_insert_delete() {
        let tmp = TempDir::new().unwrap();
        let left = tmp.path().join("left");
        let right = tmp.path().join("right");
        fs::create_dir_all(&left).unwrap();
        fs::create_dir_all(&right).unwrap();
        fs::write(left.join("SKILL.md"), "line one\nline two\n").unwrap();
        fs::write(right.join("SKILL.md"), "line one\nline three\n").unwrap();

        let diff = diff_skill_file(&left, &right, "SKILL.md").unwrap();
        assert!(diff.lines.iter().any(|l| l.tag == "delete" && l.content == "line two"));
        assert!(diff.lines.iter().any(|l| l.tag == "insert" && l.content == "line three"));
    }

    #[test]
    fn binary_change_has_no_line_diff() {
        let tmp = TempDir::new().unwrap();
        let left = tmp.path().join("left");
        let right = tmp.path().join("right");
        fs::create_dir_all(&left).unwrap();
        fs::create_dir_all(&right).unwrap();
        fs::write(left.join("data.bin"), vec![0u8, 1, 2, 3]).unwrap();
        fs::write(right.join("data.bin"), vec![0u8, 9, 8, 7]).unwrap();

        let folder_diff = diff_skill_folders(&left, &right).unwrap();
        let entry = folder_diff.files.iter().find(|f| f.relative_path == "data.bin").unwrap();
        assert_eq!(entry.status, FileDiffStatus::BinaryChanged);

        assert!(diff_skill_file(&left, &right, "data.bin").is_err());
    }
}
