use std::path::PathBuf;

use anyhow::Result;

pub trait Runner
where
    Self: Selector + Executor,
{
}

pub trait Selector: std::fmt::Debug {
    fn list_commands(&self) -> Vec<String>;
    fn path(&self) -> PathBuf;
    fn command_to_file_and_line_number(
        &self,
        command: &Option<&String>,
    ) -> (Option<String>, Option<u32>);
}

pub trait Executor {
    fn execute(&self) -> Result<()>;
}
