use super::runner_type;
use std::{fmt, path::PathBuf};

#[derive(PartialEq, Clone, Debug)]
pub struct Command {
    pub runner_type: runner_type::RunnerType,
    pub name: String,
    pub file_name: PathBuf,
    pub line_number: u32,
}

impl Command {
    pub fn new(
        runner_type: runner_type::RunnerType,
        name: String,
        file_name: PathBuf,
        line_number: u32,
    ) -> Self {
        Self {
            runner_type,
            name,
            file_name,
            line_number,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.runner_type, self.name)
    }
}
