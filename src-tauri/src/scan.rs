use crate::folder::{group_skills, SkillListItem};
use crate::skill::{build_skill_entry, should_ignore_dir};
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanComplete {
    pub skills: Vec<SkillListItem>,
    pub dirs_visited: u64,
    pub elapsed_ms: u64,
    pub errors: Vec<String>,
    pub cancelled: bool,
}

pub fn scan_skills(
    root: &Path,
    ignore_patterns: &[String],
    cancel: Arc<AtomicBool>,
) -> ScanComplete {
    let start = std::time::Instant::now();
    let mut raw_skills = Vec::new();
    let mut errors = Vec::new();
    let mut dirs_visited: u64 = 0;

    let walker = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            if let Some(name) = entry.file_name().to_str() {
                !should_ignore_dir(name, ignore_patterns)
            } else {
                true
            }
        });

    for entry in walker {
        if cancel.load(Ordering::Relaxed) {
            return ScanComplete {
                skills: Vec::new(),
                dirs_visited,
                elapsed_ms: start.elapsed().as_millis() as u64,
                errors,
                cancelled: true,
            };
        }

        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Walk error: {e}"));
                continue;
            }
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        dirs_visited += 1;

        let skill_md = entry.path().join("SKILL.md");
        if skill_md.is_file() {
            match build_skill_entry(entry.path()) {
                Ok(skill) => raw_skills.push(skill),
                Err(e) => errors.push(format!(
                    "Failed to parse {}: {e}",
                    entry.path().display()
                )),
            }
        }
    }

    let skills = match group_skills(raw_skills) {
        Ok(grouped) => grouped,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };

    ScanComplete {
        skills,
        dirs_visited,
        elapsed_ms: start.elapsed().as_millis() as u64,
        errors,
        cancelled: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::AtomicBool;
    use tempfile::TempDir;

    fn write_skill(dir: &Path, name: &str, body: &str) {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: Test skill\n---\n\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn finds_nested_skills() {
        let tmp = TempDir::new().unwrap();
        let agents = tmp.path().join(".agents").join("skills");
        write_skill(&agents, "foo-skill", "Foo body");
        write_skill(&agents, "bar-skill", "Bar body");

        let cancel = Arc::new(AtomicBool::new(false));
        let result = scan_skills(tmp.path(), &[], cancel);

        assert_eq!(result.skills.len(), 2);
        assert!(!result.cancelled);
    }

    #[test]
    fn respects_ignore_patterns() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "visible-skill", "Visible");
        let hidden = tmp.path().join("node_modules").join("hidden-skill");
        fs::create_dir_all(&hidden).unwrap();
        fs::write(
            hidden.join("SKILL.md"),
            "---\nname: hidden\n---\n",
        )
        .unwrap();

        let cancel = Arc::new(AtomicBool::new(false));
        let ignores = vec!["node_modules".to_string()];
        let result = scan_skills(tmp.path(), &ignores, cancel);

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].folder_name, "visible-skill");
    }

    #[test]
    fn ignores_recycle_bin() {
        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "visible-skill", "Visible");
        let recycle = tmp.path().join("$Recycle.Bin").join("hidden-skill");
        fs::create_dir_all(&recycle).unwrap();
        fs::write(recycle.join("SKILL.md"), "---\nname: hidden\n---\n").unwrap();

        let cancel = Arc::new(AtomicBool::new(false));
        let result = scan_skills(tmp.path(), &[], cancel);

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].folder_name, "visible-skill");
    }

    #[test]
    fn is_recycle_bin_dir_matches() {
        assert!(crate::skill::is_recycle_bin_dir("$Recycle.Bin"));
        assert!(crate::skill::is_recycle_bin_dir("$recycle.bin"));
        assert!(crate::skill::is_recycle_bin_dir(".Trash"));
        assert!(!crate::skill::is_recycle_bin_dir("skills"));
    }

    #[test]
    #[ignore = "manual diagnostic against user profile"]
    fn diagnostic_scan_user_profile() {
        use crate::skill::default_ignore_patterns;
        let home = std::env::var("USERPROFILE").expect("USERPROFILE");
        let cancel = Arc::new(AtomicBool::new(false));
        let result = scan_skills(std::path::Path::new(&home), &default_ignore_patterns(), cancel);
        eprintln!("skills_found={}", result.skills.len());
        eprintln!("dirs_visited={}", result.dirs_visited);
        eprintln!("errors={}", result.errors.len());
    }
}
