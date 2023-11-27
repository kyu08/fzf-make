use std::{fs::File, io::Read, path::PathBuf};

pub fn path_to_content(path: PathBuf) -> String {
    let mut content = String::new();

    // Not handle cases where files are not found because make command cannot be
    // executed in the first place if Makefile or included files are not found.
    let mut f = File::open(path).unwrap();
    f.read_to_string(&mut content).unwrap();

    content
}
