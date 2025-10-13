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

#[allow(dead_code)]
pub fn write_debug_info_to_file(content: &str) -> std::io::Result<()> {
    use std::path::Path;

    let file_path = "debug_info.txt";
    let file_exists = Path::new(file_path).exists();

    // Open a file in append mode. If the file does not exist, create it.
    let mut file = OpenOptions::new().append(true).create(true).open(file_path)?;

    // If file already exists, add a newline before new content
    if file_exists {
        writeln!(file)?;
    }

    // Write the content to the file
    write!(file, "{}", content)?;

    Ok(())
}
