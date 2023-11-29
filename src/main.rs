mod controller;
mod models;
mod usecase;

use crate::controller::controller_main;
use std::panic;

fn main() {
    match panic::catch_unwind(|| {
        controller_main::run();
    }) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
