use crate::usecase::usecase::Usecase;
use std::env;

pub struct Version;

impl Usecase for Version {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--version", "-v", "version"]
    }

    fn run(&self) {
        println!("v{}", get_version());
    }
}

impl Version {
    pub fn new() -> Self {
        Self {}
    }
}

fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
