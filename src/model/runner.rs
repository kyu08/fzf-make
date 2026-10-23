use super::{command, runner_type::RunnerType};
use anyhow::{Result, anyhow};
use colored::Colorize;
use std::{fmt, path::PathBuf, process};

/// Runner represents a task runner such as make, just, task or a JavaScript package manager.
///
/// Every runner is invoked as `{program} {args}`, so printing and executing a command are
/// provided as default methods. Implementors only have to expose their own metadata.
pub trait Runner: fmt::Debug + Send {
    fn runner_type(&self) -> RunnerType;

    /// program is the name of the executable to spawn.
    fn program(&self) -> &'static str;

    /// path is the path to the file which defines the commands.
    fn path(&self) -> PathBuf;

    fn to_commands(&self) -> Vec<command::CommandWithPreview>;

    /// clone_box exists to make `Box<dyn Runner>` cloneable.
    fn clone_box(&self) -> Box<dyn Runner>;

    /// It is possible to implement this method as an associated function because it takes a
    /// command as an argument. However, if it is an associated function, it can be called
    /// from anywhere, so it is better to make it a method to limit the context.
    fn command_to_run(&self, command: &command::CommandForExec) -> Result<String> {
        Ok(format!("{} {}", self.program(), command.args))
    }

    fn show_command(&self, command: &command::CommandForExec) {
        println!(
            "{}",
            self.command_to_run(command)
                .unwrap_or_else(|e| e.to_string())
                .truecolor(161, 220, 156)
        );
    }

    fn execute(&self, command: &command::CommandForExec) -> Result<()> {
        let child = process::Command::new(self.program())
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

impl Clone for Box<dyn Runner> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for dyn Runner {
    fn eq(&self, other: &Self) -> bool {
        self.runner_type() == other.runner_type()
            && self.path() == other.path()
            && self.to_commands() == other.to_commands()
    }
}
