use crate::model::command::{self};
use anyhow::{Result, anyhow};
use std::{path::PathBuf, process};

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    path: PathBuf,
    commands: Vec<command::CommandWithPreview>,
}

impl Task {
    pub fn new(target_dir: PathBuf) -> Result<Task> {
        // TODO: The following code is simplified to parse the output of `task --list-all --json` directly.
        let commands = vec![];

        Ok(Task {
            path: target_dir,
            commands,
        })
    }

    pub fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        self.commands.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn command_to_run(&self, command: &command::CommandForExec) -> Result<String, anyhow::Error> {
        Ok(format!("task {}", command.args))
    }

    pub fn execute(&self, command: &command::CommandForExec) -> Result<(), anyhow::Error> {
        let child = process::Command::new("task")
            .stdin(process::Stdio::inherit())
            .args(command.args.split_whitespace())
            .spawn();

        match child {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("failed to run: {}", e)),
            },
            Err(e) => Err(anyhow!("failed to spawn: {}", e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn new_test() {
        struct Case {
            title: &'static str,
            target_dir: PathBuf,
            should_succeed: bool,
        }

        let cases = vec![
            Case {
                title: "Should find Taskfile in main directory",
                target_dir: PathBuf::from("test_data/task"),
                should_succeed: true,
            },
            Case {
                title: "Should find Taskfile in nested directory",
                target_dir: PathBuf::from("test_data/task/nested"),
                should_succeed: true,
            },
        ];

        for case in cases {
            let result = Task::new(case.target_dir.clone());
            if case.should_succeed {
                assert!(result.is_ok(), "Case: {} - Should succeed", case.title);
                if let Ok(task) = result {
                    assert!(!task.to_commands().is_empty(), "Case: {} - Should have commands", case.title);
                }
            } else {
                assert!(result.is_err(), "Case: {} - Should fail", case.title);
            }
        }
    }
}
