use anyhow::{Result, anyhow};
use std::{fs::read_to_string, path::PathBuf};

pub fn path_to_content(path: PathBuf) -> Result<String> {
    read_to_string(path.as_path()).map_err(|e| anyhow!(e))
}

pub fn find_file_in_ancestors(current_dir: PathBuf, file_names: Vec<&str>) -> Option<PathBuf> {
    for path in current_dir.ancestors() {
        for entry in PathBuf::from(path).read_dir().unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            if file_names.contains(&file_name.as_str()) {
                return Some(entry.path());
            }
        }
    }
    None
}
