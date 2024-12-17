use super::tui::{app, config};
use crate::usecase::usecase_main::Usecase;
use anyhow::Result;

pub struct History;

impl History {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for History {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--history", "-h", "history"]
    }

    fn run(&self) -> Result<()> {
        Ok(())
        // app::main(config::Config::new(true)).await
    }
}
