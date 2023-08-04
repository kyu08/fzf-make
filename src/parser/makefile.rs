use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use super::{include, target};

/// Makefile represents a Makefile.
pub struct Makefile {
    path: PathBuf,
    include_files: Vec<Makefile>,
    targets: target::Targets,
}

impl Makefile {
    pub fn new(path: PathBuf) -> Makefile {
        let file_content = Makefile::patn_to_content(path.clone());
        let including_file_paths = include::extract_including_file_paths(file_content.clone());
        let include_files = including_file_paths
            .iter()
            .map(|path| Makefile::new(Path::new(&path).to_path_buf()))
            .collect();
        let targets = target::content_to_commands(file_content).unwrap();

        Makefile {
            path,
            include_files,
            targets,
        }
    }

    // TODO: add UT
    pub fn to_include_path_string(&self) -> Vec<String> {
        self.include_files
            .iter()
            .map(|m| m.path.to_string_lossy().to_string())
            .collect()
    }

    // TODO: add UT
    pub fn to_target_string(&self) -> Vec<String> {
        self.targets.clone()
    }

    fn patn_to_content(path: PathBuf) -> String {
        let mut contents = String::new();
        // TODO: handling error of result
        let mut f = File::open(&path).unwrap();
        // TODO: handling error of result
        f.read_to_string(&mut contents).unwrap();
        contents
    }
}
