use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::folder::compute_folder_content_hash;
use crate::skill::SkillEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub variant_group_id: Option<String>,
    pub sibling_ids: Vec<String>,
}

struct ScoredEntry {
    entry: SkillEntry,
    content_hash: String,
}

pub fn group_skills(entries: Vec<SkillEntry>) -> Result<Vec<SkillListItem>, String> {
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
        let mut group_items: Vec<SkillListItem> = Vec::new();

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

            group_items.push(SkillListItem {
                id,
                folder_name: folder_name.clone(),
                description: primary.description,
                parent_context: primary.parent_context,
                path: primary.path,
                skill_md_path: primary.skill_md_path,
                content_hash,
                identical_copy_count,
                all_paths,
                variant_label,
                variant_group_id: None,
                sibling_ids: Vec::new(),
            });
        }

        if group_items.len() > 1 {
            let variant_group_id = folder_name.to_ascii_lowercase();
            let all_ids: Vec<String> = group_items.iter().map(|i| i.id.clone()).collect();
            for item in &mut group_items {
                item.variant_group_id = Some(variant_group_id.clone());
                item.sibling_ids = all_ids
                    .iter()
                    .filter(|id| *id != &item.id)
                    .cloned()
                    .collect();
            }
        }

        items.extend(group_items);
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
        assert!(grouped.iter().all(|g| g.variant_group_id.as_deref() == Some("my-skill")));
        assert!(grouped[0].sibling_ids == vec![grouped[1].id.clone()]);
        assert!(grouped[1].sibling_ids == vec![grouped[0].id.clone()]);
    }
}
