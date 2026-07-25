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
        let read_dir = match PathBuf::from(path).read_dir() {
            Ok(r) => r,
            Err(_) => continue,
        };

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
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

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::{self, File};
    use uuid::Uuid;

    #[test]
    fn find_file_in_ancestors_finds_file_in_ancestor() {
        // Layout:
        //   <tmp_root>/justfile
        //   <tmp_root>/child/grandchild   <- start searching from here
        let tmp_root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let start_dir = tmp_root.join("child").join("grandchild");
        fs::create_dir_all(&start_dir).unwrap();
        File::create(tmp_root.join("justfile")).unwrap();

        let result = find_file_in_ancestors(start_dir, vec!["justfile", ".justfile"]);

        assert_eq!(result, Some(tmp_root.join("justfile")));

        fs::remove_dir_all(&tmp_root).unwrap();
    }

    // Regression test for the Homebrew Linux C crash where `read_dir` returned
    // PermissionDenied on an ancestor directory and the function panicked.
    // https://github.com/Homebrew/homebrew-core/pull/295008#issuecomment-5071286646
    //
    // A directory with mode 0o111 (execute-only) can be traversed but not read,
    // so `read_dir` fails with PermissionDenied for it. The function must skip
    // such directories and keep searching readable ancestors instead of panicking.
    #[cfg(unix)]
    #[test]
    fn find_file_in_ancestors_skips_unreadable_ancestor() {
        use std::os::unix::fs::PermissionsExt;

        // Layout:
        //   <tmp_root>/justfile
        //   <tmp_root>/unreadable          <- mode 0o111: traversable but not readable
        //   <tmp_root>/unreadable/leaf     <- start searching from here
        let tmp_root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let unreadable = tmp_root.join("unreadable");
        let start_dir = unreadable.join("leaf");
        fs::create_dir_all(&start_dir).unwrap();
        File::create(tmp_root.join("justfile")).unwrap();

        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o111)).unwrap();

        // Must not panic on the unreadable ancestor, and must still find the file
        // in the readable ancestor above it.
        let result = find_file_in_ancestors(start_dir, vec!["justfile", ".justfile"]);

        assert_eq!(result, Some(tmp_root.join("justfile")));

        // Restore read permission so the cleanup can recurse into the directory.
        fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&tmp_root).unwrap();
    }
}
