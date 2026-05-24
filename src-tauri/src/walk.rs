use std::path::Path;

use walkdir::{DirEntry, WalkDir};

use crate::skill::should_ignore_dir;

/// Directory walk with consistent ignore rules (recycle bins, `ignore_patterns`, etc.).
pub fn walk_filtered<'a>(
    root: &'a Path,
    ignore_patterns: &'a [String],
) -> impl Iterator<Item = Result<DirEntry, walkdir::Error>> + use<'a> {
    WalkDir::new(root)
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
        })
}
