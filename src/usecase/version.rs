use crate::usecase::usecase_main::Usecase;
use anyhow::Result;
use std::env;

pub struct Version;

impl Version {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for Version {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--version", "-v", "version"]
    }

    fn run(&self) -> Result<()> {
        println!("v{}", get_version());
        Ok(())
    }
}

fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
