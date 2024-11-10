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
                AppState::SelectTarget(model) => {
                    match model.histories.map(|_h| {
                        // TODO: Decide the specification of this.
                        // 1. Find the latest history that starts with cwd and execute it (need to save information about which one is the latest)
                        // 2. When there are multiple candidates, display the choices and let the user choose?
                        match &model.runners.first() {
                            Some(_runner) => {
                                None::<String> // TODO: Fix this when history function is implemented
                                               // h
                                               // .get_latest_target(&runner.path())
                                               // .map(execute_make_command),
                            }
                            None => None,
                        }
                    }) {
                        Some(Some(_)) => Ok(()),
                        _ => Err(anyhow!("No target found")),
                    }
                }
                _ => Err(anyhow!("Invalid state")),
            },
        }
    }
}
