use super::help;
use crate::usecase::usecase_main::Usecase;
use anyhow::Result;

pub struct InvalidArg;

impl InvalidArg {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for InvalidArg {
    fn command_str(&self) -> Vec<&'static str> {
        vec![]
    }

    fn run(&self) -> Result<()> {
        println!("{}", get_message());
        println!("{}", help::get_help());
        Ok(())
    }
}

fn get_message() -> String {
    "Invalid argument.".to_string()
}
