mod controller;
mod file;
mod model;
mod usecase;

use crate::controller::controller_main;
use std::panic;

fn main() {
    let result = panic::catch_unwind(|| {
        controller_main::run();
    });
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
