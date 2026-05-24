use crate::group::{group_skills, SkillListItem};
use crate::skill::build_skill_entry;
use crate::walk::walk_filtered;
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanComplete {
    pub skills: Vec<SkillListItem>,
    pub dirs_visited: u64,
    pub elapsed_ms: u64,
    pub errors: Vec<String>,
    pub cancelled: bool,
}

fn finish_scan(
    raw_skills: Vec<crate::skill::SkillEntry>,
    dirs_visited: u64,
    start: std::time::Instant,
    mut errors: Vec<String>,
    cancelled: bool,
) -> ScanComplete {
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
        cancelled,
    }
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

    for entry in walk_filtered(root, ignore_patterns) {
        if cancel.load(Ordering::Relaxed) {
            return finish_scan(raw_skills, dirs_visited, start, errors, true);
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

    finish_scan(raw_skills, dirs_visited, start, errors, false)
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
    fn cancelled_scan_keeps_collected_skills() {
        use crate::skill::build_skill_entry;

        let tmp = TempDir::new().unwrap();
        write_skill(tmp.path(), "partial-skill", "Body");
        let entry = build_skill_entry(&tmp.path().join("partial-skill")).unwrap();

        let result = finish_scan(vec![entry], 42, std::time::Instant::now(), vec![], true);

        assert!(result.cancelled);
        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].folder_name, "partial-skill");
        assert_eq!(result.dirs_visited, 42);
    }

    #[test]
    fn is_recycle_bin_dir_matches() {
        assert!(crate::skill::is_recycle_bin_dir("$Recycle.Bin"));
        assert!(crate::skill::is_recycle_bin_dir("$recycle.bin"));
        assert!(crate::skill::is_recycle_bin_dir(".Trash"));
        assert!(!crate::skill::is_recycle_bin_dir("skills"));
    }
}
