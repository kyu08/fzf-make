use crate::{file::file, fuzzy_finder::fuzzy_finder};
use std::process;

pub fn run() {
    let makefile = match file::create_makefile() {
        Err(e) => {
            println!("[ERR] {}", e.to_string());
            process::exit(1)
        }
        Ok(f) => f,
    };

    fuzzy_finder::run(makefile);
}
