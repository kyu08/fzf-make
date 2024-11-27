use super::command;
use anyhow::{anyhow, Result};
use std::{path::PathBuf, process};

#[derive(Clone, Debug, PartialEq)]
pub struct Pnpm {
    pub path: PathBuf,
    commands: Vec<command::Command>,
}

impl Pnpm {
    pub fn command_to_run(&self, command: &command::Command) -> Result<String> {
        // To ensure that the command exists, it is necessary to check the command name.
        // If implementation is wrong, developers can notice it here.
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        Ok(format!("pnpm run {}", command.name))
    }

    pub fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    pub fn new(path: PathBuf, commands: Vec<command::Command>) -> Pnpm {
        Pnpm { path, commands }
    }

    fn get_command(&self, command: command::Command) -> Option<&command::Command> {
        self.commands.iter().find(|c| **c == command)
    }

    pub fn execute(&self, command: &command::Command) -> Result<()> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        let child = process::Command::new("pnpm")
            .stdin(process::Stdio::inherit())
            .arg("run")
            .arg(&command.name)
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
