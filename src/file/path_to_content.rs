use anyhow::{Result, anyhow};
use std::{fs::read_to_string, path::Path};

pub fn path_to_content(path: &Path) -> Result<String> {
    read_to_string(path).map_err(|e| anyhow!(e))
}
