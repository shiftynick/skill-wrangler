use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

const CONTEXT_DIRS: &[&str] = &[".claude", ".agents", ".cursor"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillEntry {
    pub id: String,
    pub folder_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub skill_md_path: String,
    pub parent_context: String,
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
}

pub fn skill_id(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    hash.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..16]
        .to_string()
}

pub fn parse_skill_md(content: &str, folder_name: &str) -> (Option<String>, Option<String>) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return fallback_metadata(content, folder_name);
    }

    let rest = trimmed.strip_prefix("---").unwrap_or(trimmed).trim_start();
    let Some((yaml_block, body)) = rest.split_once("\n---") else {
        return fallback_metadata(content, folder_name);
    };

    let yaml_block = yaml_block.trim();
    match serde_yaml::from_str::<SkillFrontmatter>(yaml_block) {
        Ok(fm) => {
            let name = fm.name.filter(|n| !n.is_empty());
            let description = fm.description.filter(|d| !d.is_empty());
            if name.is_some() || description.is_some() {
                (name, description)
            } else {
                fallback_metadata(body, folder_name)
            }
        }
        Err(_) => fallback_metadata(body, folder_name),
    }
}

fn fallback_metadata(body: &str, folder_name: &str) -> (Option<String>, Option<String>) {
    let description = body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let stripped = line.trim_start_matches('#').trim();
            if stripped.len() > 120 {
                format!("{}...", &stripped[..117])
            } else {
                stripped.to_string()
            }
        });

    (Some(folder_name.to_string()), description)
}

pub fn parent_context(skill_path: &Path) -> String {
    let mut components: Vec<String> = Vec::new();
    let mut found_context: Option<String> = None;

    for ancestor in skill_path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
            if CONTEXT_DIRS.contains(&name) {
                found_context = Some(name.to_string());
                break;
            }
            if !name.is_empty() {
                components.push(name.to_string());
            }
        }
    }

    components.reverse();

    if let Some(ctx) = found_context {
        if components.is_empty() {
            ctx
        } else {
            format!("{}/{}", ctx, components.join("/"))
        }
    } else if components.is_empty() {
        skill_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        components.join("/")
    }
}

pub fn build_skill_entry(skill_dir: &Path) -> std::io::Result<SkillEntry> {
    let skill_md_path = skill_dir.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md_path)?;
    let folder_name = skill_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let (name, description) = parse_skill_md(&content, &folder_name);
    let abs_path = skill_dir.canonicalize().unwrap_or_else(|_| skill_dir.to_path_buf());

    Ok(SkillEntry {
        id: skill_id(&abs_path),
        folder_name,
        name,
        description,
        path: abs_path.to_string_lossy().to_string(),
        skill_md_path: skill_md_path.to_string_lossy().to_string(),
        parent_context: parent_context(&abs_path),
    })
}

pub fn should_ignore_dir(dir_name: &str, ignore_patterns: &[String]) -> bool {
    if is_recycle_bin_dir(dir_name) {
        return true;
    }
    ignore_patterns.iter().any(|pattern| {
        if pattern.contains('*') {
            glob_match(pattern, dir_name)
        } else {
            dir_name == pattern
        }
    })
}

/// Windows/macOS/Linux recycle bin folder names (matched case-insensitively).
pub fn is_recycle_bin_dir(dir_name: &str) -> bool {
    const NAMES: &[&str] = &[
        "$Recycle.Bin",
        "$RECYCLE.BIN",
        "RECYCLER",
        "RECYCLED",
        ".Trash",
        ".trash",
    ];
    let lower = dir_name.to_ascii_lowercase();
    NAMES.iter().any(|name| lower == name.to_ascii_lowercase())
}

fn glob_match(pattern: &str, text: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        text.starts_with(prefix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        text.ends_with(suffix)
    } else {
        pattern == text
    }
}

pub fn default_ignore_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
        "dist".to_string(),
        ".pnpm-store".to_string(),
        "plugins/cache".to_string(),
        "skills-cursor".to_string(),
        "$Recycle.Bin".to_string(),
        "RECYCLER".to_string(),
        ".Trash".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parses_valid_frontmatter() {
        let content = r#"---
name: find-skills
description: Helps users discover skills
---

# Find Skills
"#;
        let (name, desc) = parse_skill_md(content, "find-skills");
        assert_eq!(name.as_deref(), Some("find-skills"));
        assert_eq!(desc.as_deref(), Some("Helps users discover skills"));
    }

    #[test]
    fn falls_back_without_frontmatter() {
        let content = "# My Skill\n\nDoes something useful.";
        let (name, desc) = parse_skill_md(content, "my-skill");
        assert_eq!(name.as_deref(), Some("my-skill"));
        assert_eq!(desc.as_deref(), Some("Does something useful."));
    }

    #[test]
    fn ignores_malformed_frontmatter() {
        let content = r#"---
name: [invalid yaml
---
Body line one.
"#;
        let (name, desc) = parse_skill_md(content, "broken");
        assert_eq!(name.as_deref(), Some("broken"));
        assert_eq!(desc.as_deref(), Some("Body line one."));
    }

    #[test]
    fn parent_context_finds_agents() {
        let path = PathBuf::from(r"C:\Users\test\.agents\skills\find-skills");
        let ctx = parent_context(&path);
        assert_eq!(ctx, ".agents/skills/find-skills");
    }
}
