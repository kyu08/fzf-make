use super::tui::config;
use crate::usecase::tui::app;
use crate::usecase::usecase_main::Usecase;
use anyhow::Result;

pub struct FzfMake;

impl FzfMake {
    pub fn new() -> Self {
        Self {}
    }
    // pub async fn run(&self) -> Result<()> {
    //     app::main(config::Config::default()).await
    // }
}

impl Usecase for FzfMake {
    fn command_str(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn run(&self) -> Result<()> {
        app::main(config::Config::default()).await
    }
}
