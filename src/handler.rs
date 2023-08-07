use crate::{fuzzy_finder::fuzzy_finder, parser::makefile::Makefile};
use std::process;

pub fn run() {
    let makefile = match Makefile::create_makefile() {
        Err(e) => {
            println!("[ERR] {}", e.to_string());
            process::exit(1)
        }
        Ok(f) => f,
    };

    fuzzy_finder::run(makefile);
}
