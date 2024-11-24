use super::{command, make, pnpm};
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Runner {
    MakeCommand(make::Make),
    PnpmCommand(pnpm::Pnpm),
}

impl Runner {
    pub fn list_commands(&self) -> Vec<command::Command> {
        match self {
            Runner::MakeCommand(make) => make.to_commands(),
            Runner::PnpmCommand(_) => todo!(),
        }
    }

    pub fn path(&self) -> PathBuf {
        match self {
            Runner::MakeCommand(make) => make.path.clone(),
            Runner::PnpmCommand(_) => todo!(),
        }
    }

    pub fn show_command(&self, command: &command::Command) {
        let command_to_run = match self {
            Runner::MakeCommand(_) => make::Make::command_to_run(command),
            Runner::PnpmCommand(_) => todo!(),
        };
        println!("{}", (command_to_run).truecolor(161, 220, 156));
    }

    pub fn execute(&self, command: &command::Command) -> Result<()> {
        match self {
            Runner::MakeCommand(make) => make.execute(command),
            Runner::PnpmCommand(_) => todo!(),
        }
    }
}
