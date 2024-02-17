use crate::usecase::usecase_main::Usecase;
use anyhow::{anyhow, Result};

use super::{execute_make_command::execute_make_target, fzf_make::app::Model};

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
        match Model::new() {
            Err(e) => Err(e),
            Ok(model) => {
                match model.histories.map(|h| {
                    h.get_latest_target(&model.makefile.path)
                        .map(execute_make_target)
                }) {
                    Some(Some(_)) => Ok(()),
                    _ => Err(anyhow!("No target found")),
                }
            }
        }
    }
}
