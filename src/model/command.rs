use super::runner_type;
use std::{fmt, path::PathBuf};

#[derive(PartialEq, Clone, Debug)]
pub struct Command {
    pub runner_type: runner_type::RunnerType,
    pub args: String,
    pub file_path: PathBuf,
    pub line_number: u32,
}

impl Command {
    pub fn new(
        runner_type: runner_type::RunnerType,
        args: String,
        file_path: PathBuf,
        line_number: u32,
    ) -> Self {
        Self {
            runner_type,
            args,
            file_path,
            line_number,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.runner_type, self.args)
    }
}
