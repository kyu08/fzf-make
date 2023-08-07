// module for file manipulation util
use std::{fs::File, io::Read, path::PathBuf};

pub fn path_to_content(path: PathBuf) -> String {
    let mut content = String::new();
    let mut f = File::open(&path).unwrap();
    f.read_to_string(&mut content).unwrap();

    content
}
