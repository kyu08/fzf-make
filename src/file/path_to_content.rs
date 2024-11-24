use anyhow::{anyhow, Result};
use std::{fs::read_to_string, path::PathBuf};

pub fn path_to_content(path: PathBuf) -> Result<String> {
    read_to_string(path.as_path()).map_err(|e| anyhow!(e))
}
