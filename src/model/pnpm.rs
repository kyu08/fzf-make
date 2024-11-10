use super::command;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Pnpm {
    pub path: PathBuf,
    commands: Vec<command::Command>,
}
