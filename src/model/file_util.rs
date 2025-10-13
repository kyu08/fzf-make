use anyhow::{Result, anyhow};
use std::{
    fs::{OpenOptions, read_to_string},
    io::Write,
    path::PathBuf,
};

pub fn path_to_content(path: PathBuf) -> Result<String> {
    read_to_string(path.as_path()).map_err(|e| anyhow!(e))
}

pub fn find_file_in_ancestors(current_dir: PathBuf, file_names: Vec<&str>) -> Option<PathBuf> {
    for path in current_dir.ancestors() {
        match PathBuf::from(path).read_dir() {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            let file_name = entry.file_name().to_string_lossy().to_lowercase();
                            if file_names.contains(&file_name.as_str()) {
                                return Some(entry.path());
                            }
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[find_file_in_ancestors] Failed to read entry in {:?}: {}", path, e);
                        }
                    }
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("[find_file_in_ancestors] Failed to read directory {:?}: {}", path, e);
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn write_debug_info_to_file(content: &str) -> std::io::Result<()> {
    // Open a file in append mode. If the file does not exist, create it.
    let mut file = OpenOptions::new().append(true).create(true).open("debug_info.txt")?;

    // Write the file_name_string to the file
    writeln!(file, "{}", content)?;

    Ok(())
}
