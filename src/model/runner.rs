use super::{command, make, pnpm};
use anyhow::Result;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Runner {
    // TODO(#321): Use associated constants if possible.
    // ref: https://doc.rust-lang.org/reference/items/associated-items.html#associated-constants
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

    pub fn show_command(&self, command: command::Command) -> String {
        let runner_name = match self {
            Runner::MakeCommand(_) => make::Make::runner_name(),
            Runner::PnpmCommand(_) => todo!(),
        };
        format!("({}) {}", runner_name, command.name)
    }

    pub fn execute(&self, command: command::Command) -> Result<()> {
        match self {
            Runner::MakeCommand(make) => make.execute(command),
            Runner::PnpmCommand(_) => todo!(),
        }
    }
}
