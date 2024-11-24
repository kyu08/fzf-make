mod controller;
mod err;
mod file;
mod model;
mod usecase;
use std::panic;

fn main() {
    let result = panic::catch_unwind(|| {
        controller::controller_main::run();
    });
    if let Err(e) = result {
        println!("{}", err::any_to_string::any_to_string(&*e));
        std::process::exit(1);
    }
}
