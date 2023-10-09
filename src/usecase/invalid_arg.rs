use super::help;
use crate::usecase::usecase::Usecase;

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

    fn run(&self) {
        println!("{}", get_message());
        println!("{}", help::get_help());
    }
}

fn get_message() -> String {
    "Invalid argment.".to_string()
}
