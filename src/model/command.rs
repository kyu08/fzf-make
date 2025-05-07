use super::{histories, runner_type};
use std::{fmt, path::PathBuf};

#[derive(PartialEq, Clone, Debug)]
pub struct CommandWithPreview {
    pub runner_type: runner_type::RunnerType,
    pub args: String,
    pub file_path: PathBuf,
    pub line_number: u32,
}

impl CommandWithPreview {
    pub fn new(runner_type: runner_type::RunnerType, args: String, file_path: PathBuf, line_number: u32) -> Self {
        Self {
            runner_type,
            args,
            file_path,
            line_number,
        }
    }
}

impl fmt::Display for CommandWithPreview {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.runner_type, self.args)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct CommandForExec {
    pub runner_type: runner_type::RunnerType,
    pub args: String,
}

impl From<CommandWithPreview> for CommandForExec {
    fn from(c: CommandWithPreview) -> CommandForExec {
        CommandForExec {
            runner_type: c.runner_type.clone(),
            args: c.args.clone(),
        }
    }
}

impl From<histories::HistoryCommand> for CommandForExec {
    fn from(c: histories::HistoryCommand) -> CommandForExec {
        CommandForExec {
            runner_type: c.runner_type.clone(),
            args: c.args.clone(),
        }
    }
}

impl fmt::Display for CommandForExec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.runner_type, self.args)
    }
}
