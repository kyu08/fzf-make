use crate::usecase::usecase::Usecase;
use crate::{fuzzy_finder, models::makefile::Makefile};
use std::process;

pub struct FzfMake;

impl Usecase for FzfMake {
    fn command_str(&self) -> Vec<&'static str> {
        vec![]
    }

    fn run(&self) {
        let makefile = match Makefile::create_makefile() {
            Err(e) => {
                println!("[ERR] {}", e.to_string());
                process::exit(1)
            }
            Ok(f) => f,
        };

        fuzzy_finder::run(makefile);
    }
}

impl FzfMake {
    pub fn new() -> Self {
        Self {}
    }
}
