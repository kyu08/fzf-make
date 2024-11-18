use crate::usecase::usecase_main::Usecase;
use anyhow::{anyhow, Result};

use super::tui::{
    app::{AppState, Model},
    config,
};

pub struct Repeat;

impl Repeat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for Repeat {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--repeat", "-r", "repeat"]
    }

    fn run(&self) -> Result<()> {
        match Model::new(config::Config::default()) {
            Err(e) => Err(e),
            Ok(model) => match model.app_state {
                AppState::SelectTarget(state) => match state.get_latest_command() {
                    Some(c) => match state.get_runner(&c.runner_type) {
                        Some(runner) => runner.execute(c),
                        None => Err(anyhow!("runner not found.")),
                    },
                    None => Err(anyhow!("fzf-make has not been executed in this path yet.")),
                },
                _ => Err(anyhow!("Invalid state")),
            },
        }
    }
}
