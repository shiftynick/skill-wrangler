use fs_extra::dir::{copy as copy_dir, CopyOptions};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::skill::{is_known_agent_skills_dir, should_ignore_dir};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    Skip,
    Overwrite,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CopyStatus {
    Success,
    Skipped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyResult {
    pub source: String,
    pub destination: String,
    pub status: CopyStatus,
    pub message: Option<String>,
}

pub fn copy_skills(
    sources: &[PathBuf],
    dest_root: &Path,
    policy: ConflictPolicy,
    copy_to_all_skill_folders: bool,
) -> Vec<CopyResult> {
    let dest_roots = resolve_copy_destinations(dest_root, copy_to_all_skill_folders);
    let mut results = Vec::new();

    for source in sources {
        for dest in &dest_roots {
            results.push(copy_one_skill(source, dest, policy));
        }
    }

    results
}

/// Known agent skill directories under `root`, e.g. `.agents/skills`, `.claude/skills`.
pub fn find_skill_containers(root: &Path) -> Vec<PathBuf> {
    let mut containers = Vec::new();

    if !root.is_dir() {
        return containers;
    }

    if is_known_agent_skills_dir(root) {
        containers.push(normalize_path(root));
    }

    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            if let Some(name) = entry.file_name().to_str() {
                !should_ignore_dir(name, &[])
            } else {
                true
            }
        });

    for entry in walker.flatten() {
        if !entry.file_type().is_dir() || entry.depth() == 0 {
            continue;
        }
        let path = entry.path();
        if is_known_agent_skills_dir(path) {
            containers.push(normalize_path(path));
        }
    }

    containers.sort();
    containers.dedup();
    containers
}

fn resolve_copy_destinations(root: &Path, copy_to_all_skill_folders: bool) -> Vec<PathBuf> {
    if !copy_to_all_skill_folders {
        return vec![root.to_path_buf()];
    }

    let containers = find_skill_containers(root);
    if containers.is_empty() {
        vec![root.to_path_buf()]
    } else {
        containers
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn copy_one_skill(source: &Path, dest_root: &Path, policy: ConflictPolicy) -> CopyResult {
    let folder_name = match source.file_name().and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            return CopyResult {
                source: source.to_string_lossy().to_string(),
                destination: String::new(),
                status: CopyStatus::Error,
                message: Some("Invalid source folder name".to_string()),
            };
        }
    };

    if !source.join("SKILL.md").is_file() {
        return CopyResult {
            source: source.to_string_lossy().to_string(),
            destination: String::new(),
            status: CopyStatus::Error,
            message: Some("Source folder does not contain SKILL.md".to_string()),
        };
    }

    if !dest_root.is_dir() {
        return CopyResult {
            source: source.to_string_lossy().to_string(),
            destination: dest_root.to_string_lossy().to_string(),
            status: CopyStatus::Error,
            message: Some("Destination is not a directory".to_string()),
        };
    }

    let dest_path = resolve_destination(dest_root, &folder_name, policy);

    if dest_path.is_none() {
        let skipped_dest = dest_root.join(&folder_name);
        return CopyResult {
            source: source.to_string_lossy().to_string(),
            destination: skipped_dest.to_string_lossy().to_string(),
            status: CopyStatus::Skipped,
            message: Some("Destination already exists".to_string()),
        };
    }

    let dest_path = dest_path.unwrap();

    if policy == ConflictPolicy::Overwrite && dest_path.exists() {
        if let Err(e) = fs_extra::dir::remove(&dest_path) {
            return CopyResult {
                source: source.to_string_lossy().to_string(),
                destination: dest_path.to_string_lossy().to_string(),
                status: CopyStatus::Error,
                message: Some(format!("Failed to remove existing folder: {e}")),
            };
        }
    }

    let mut options = CopyOptions::new();
    options.overwrite = policy == ConflictPolicy::Overwrite;
    options.copy_inside = true;

    match copy_dir(source, &dest_path, &options) {
        Ok(_) => {
            if !dest_path.join("SKILL.md").is_file() {
                return CopyResult {
                    source: source.to_string_lossy().to_string(),
                    destination: dest_path.to_string_lossy().to_string(),
                    status: CopyStatus::Error,
                    message: Some("Copy completed but SKILL.md missing at destination".to_string()),
                };
            }
            CopyResult {
                source: source.to_string_lossy().to_string(),
                destination: dest_path.to_string_lossy().to_string(),
                status: CopyStatus::Success,
                message: None,
            }
        }
        Err(e) => CopyResult {
            source: source.to_string_lossy().to_string(),
            destination: dest_path.to_string_lossy().to_string(),
            status: CopyStatus::Error,
            message: Some(format!("Copy failed: {e}")),
        },
    }
}

fn resolve_destination(
    dest_root: &Path,
    folder_name: &str,
    policy: ConflictPolicy,
) -> Option<PathBuf> {
    let base = dest_root.join(folder_name);

    match policy {
        ConflictPolicy::Skip => {
            if base.exists() {
                None
            } else {
                Some(base)
            }
        }
        ConflictPolicy::Overwrite => Some(base),
        ConflictPolicy::Rename => {
            if !base.exists() {
                return Some(base);
            }
            let mut n = 2;
            loop {
                let candidate = dest_root.join(format!("{folder_name}-{n}"));
                if !candidate.exists() {
                    return Some(candidate);
                }
                n += 1;
                if n > 1000 {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_skill(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("SKILL.md"), "---\nname: test\n---\n").unwrap();
        path
    }

    #[test]
    fn copies_skill_folder() {
        let tmp = TempDir::new().unwrap();
        let src = make_skill(tmp.path(), "my-skill");
        let dest_root = tmp.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let results = copy_skills(&[src.clone()], &dest_root, ConflictPolicy::Rename, false);
        assert_eq!(results[0].status, CopyStatus::Success);
        assert!(PathBuf::from(&results[0].destination).join("SKILL.md").is_file());
    }

    #[test]
    fn rename_on_conflict() {
        let tmp = TempDir::new().unwrap();
        let src = make_skill(tmp.path(), "my-skill");
        let dest_root = tmp.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();
        make_skill(&dest_root, "my-skill");

        let results = copy_skills(&[src], &dest_root, ConflictPolicy::Rename, false);
        assert_eq!(results[0].status, CopyStatus::Success);
        assert!(results[0].destination.ends_with("my-skill-2"));
    }

    #[test]
    fn skips_on_conflict() {
        let tmp = TempDir::new().unwrap();
        let src = make_skill(tmp.path(), "my-skill");
        let dest_root = tmp.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();
        make_skill(&dest_root, "my-skill");

        let results = copy_skills(&[src], &dest_root, ConflictPolicy::Skip, false);
        assert_eq!(results[0].status, CopyStatus::Skipped);
    }

    #[test]
    fn finds_skill_containers_under_repo() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("some-repo");
        let agents = repo.join(".agents").join("skills");
        let claude = repo.join(".claude").join("skills");
        make_skill(&agents, "existing-a");
        make_skill(&claude, "existing-b");

        let containers = find_skill_containers(&repo);
        assert_eq!(containers.len(), 2);
    }

    #[test]
    fn ignores_non_agent_skills_folders() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("some-repo");
        let generic = repo.join("custom").join("skills");
        make_skill(&generic, "existing");

        let containers = find_skill_containers(&repo);
        assert!(containers.is_empty());
    }

    #[test]
    fn finds_cursor_skills_folder() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("some-repo");
        let cursor = repo.join(".cursor").join("skills");
        fs::create_dir_all(&cursor).unwrap();

        let containers = find_skill_containers(&repo);
        assert_eq!(containers.len(), 1);
    }

    #[test]
    fn copies_to_all_skill_containers() {
        let tmp = TempDir::new().unwrap();
        let src = make_skill(tmp.path(), "new-skill");
        let repo = tmp.path().join("some-repo");
        let agents = repo.join(".agents").join("skills");
        let claude = repo.join(".claude").join("skills");
        fs::create_dir_all(&agents).unwrap();
        fs::create_dir_all(&claude).unwrap();
        make_skill(&agents, "existing-a");
        make_skill(&claude, "existing-b");

        let results = copy_skills(&[src], &repo, ConflictPolicy::Rename, true);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == CopyStatus::Success));
        assert!(agents.join("new-skill").join("SKILL.md").is_file());
        assert!(claude.join("new-skill").join("SKILL.md").is_file());
    }
}
