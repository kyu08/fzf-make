use crate::models::makefile::Makefile;
use crate::usecases::{fzf_make::fuzzy_finder, usecase::Usecase};
use colored::*;
use std::process;

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

    fn run(&self) {
        let makefile = match Makefile::create_makefile() {
            Err(e) => {
                println!("[ERR] {}", e);
                process::exit(1)
            }
            Ok(f) => f,
        };

        let target = fuzzy_finder::run(makefile);

        println!("{}", ("make ".to_string() + &target).blue()); // TODO: Make output color configurable via config file
        process::Command::new("make")
            .stdin(process::Stdio::inherit())
            .arg(target)
            .spawn()
            .expect("Failed to execute process")
            .wait()
            .expect("Failed to execute process");
    }
}
