use super::tui::{
    app::{AppState, Model},
    config,
};
use crate::{model::command, usecase::usecase_main::Usecase};
use anyhow::{Result, anyhow};
use futures::{FutureExt, future::BoxFuture};

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

    fn run(&self) -> BoxFuture<'_, Result<()>> {
        async move {
            match Model::new(config::Config::default()) {
                Err(e) => Err(e),
                Ok(model) => match model.app_state {
                    AppState::SelectCommand(state) => match state.get_latest_command() {
                        Some(c) => match state.get_runner(&c.runner_type) {
                            Some(runner) => {
                                runner.show_command(&command::CommandForExec::from(c.clone()));
                                runner.execute(&command::CommandForExec::from(c.clone()))
                            }
                            None => Err(anyhow!("runner not found.")),
                        },
                        None => Err(anyhow!("fzf-make has not been executed in this path yet.")),
                    },
                    _ => Err(anyhow!("Invalid state")),
                },
            }
        }
        .boxed()
    }
}
