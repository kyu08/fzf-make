use super::tui::config;
use crate::usecase::{tui::app, usecase_main::Usecase};
use anyhow::Result;
use futures::{FutureExt, future::BoxFuture};

pub struct FzfMake;

impl FzfMake {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for FzfMake {
    fn command_str(&self) -> Vec<&'static str> {
        vec![]
    }

    fn run(&self) -> BoxFuture<'_, Result<()>> {
        async { app::main(config::Config::default()).await }.boxed()
    }
}
