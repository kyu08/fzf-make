mod controller;
mod file;
mod model;
mod usecase;
use std::panic;

fn main() {
    let result = panic::catch_unwind(|| {
        controller::controller_main::run();
    });
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
