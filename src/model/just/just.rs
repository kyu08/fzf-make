use crate::model::command;
use anyhow::anyhow;
use std::{path::PathBuf, process};

#[derive(Debug, Clone, PartialEq)]
pub struct Just {
    path: PathBuf,
    commands: Vec<command::Command>,
}
impl Just {
    // TODO: add new
    pub(crate) fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub(crate) fn command_to_run(
        &self,
        command: &command::Command,
    ) -> Result<String, anyhow::Error> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        Ok(format!("just {}", command.args))
    }

    pub(crate) fn execute(&self, command: &command::Command) -> Result<(), anyhow::Error> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        let child = process::Command::new("just")
            .stdin(process::Stdio::inherit())
            .arg(&command.args)
            .spawn();

        match child {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("failed to run: {}", e)),
            },
            Err(e) => Err(anyhow!("failed to spawn: {}", e)),
        }
    }

    fn get_command(&self, command: command::Command) -> Option<command::Command> {
        self.to_commands()
            .iter()
            .find(|c| **c == command)
            .map(|_| command)
    }
}
