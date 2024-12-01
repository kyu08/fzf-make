use super::{command, js_package_manager, make};
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Runner {
    MakeCommand(make::Make),
    JsPackageManager(js_package_manager::JsPackageManager),
}

impl Runner {
    pub fn list_commands(&self) -> Vec<command::Command> {
        match self {
            Runner::MakeCommand(make) => make.to_commands(),
            Runner::JsPackageManager(js) => js.to_commands(),
        }
    }

    pub fn path(&self) -> PathBuf {
        match self {
            Runner::MakeCommand(make) => make.path.clone(),
            Runner::JsPackageManager(js) => js.path(),
        }
    }

    pub fn show_command(&self, command: &command::Command) {
        let command_or_error_message = match self {
            Runner::MakeCommand(make) => make.command_to_run(command),
            Runner::JsPackageManager(js) => js.command_to_run(command),
        };

        println!(
            "{}",
            command_or_error_message
                .unwrap_or_else(|e| e.to_string())
                .truecolor(161, 220, 156)
        );
    }

    pub fn execute(&self, command: &command::Command) -> Result<()> {
        match self {
            Runner::MakeCommand(make) => make.execute(command),
            Runner::JsPackageManager(js) => js.execute(command),
        }
    }
}
