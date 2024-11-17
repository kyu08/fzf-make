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
                AppState::SelectTarget(state) => {
                    match (
                        state.runners.first(), // TODO: firstではなく最後に実行されたcommandのrunnerを使うべき
                        state.histories.get_latest_command(&state.current_dir),
                    ) {
                        (Some(r), Some(h)) => r.execute(h),
                        (_, _) => Err(anyhow!("fzf-make has not been executed in this path yet.")),
                    }
                }
                _ => Err(anyhow!("Invalid state")),
            },
        }
    }
}
