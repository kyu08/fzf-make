use anyhow::{Result, anyhow};
use std::{fs::read_to_string, path::PathBuf};

pub fn path_to_content(path: PathBuf) -> Result<String> {
    read_to_string(path.as_path()).map_err(|e| anyhow!(e))
}

pub fn find_file_in_ancestors(current_dir: PathBuf, file_names: Vec<&str>) -> Option<PathBuf> {
    for path in current_dir.ancestors() {
        if let Ok(entries) = PathBuf::from(path).read_dir() {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_names.contains(&file_name.as_str()) {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

pub fn find_file_in_ancestors_with_priority(current_dir: PathBuf, file_names: Vec<&str>) -> Option<PathBuf> {
    for path in current_dir.ancestors() {
        if let Ok(entries) = PathBuf::from(path).read_dir() {
            // Collect entries first to avoid borrowing issues
            let entries: Vec<_> = entries.flatten().collect();
            
            // Check files in priority order (first match wins)
            for target_name in &file_names {
                for entry in &entries {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name == *target_name {
                        return Some(entry.path());
                    }
                }
            }
        }
    }
    None
}
