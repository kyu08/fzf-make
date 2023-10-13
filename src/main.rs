mod controller;
mod models;
mod usecase;

use crate::controller::controller_main;

// test
fn main() {
    // TODO: Catch panic
    controller_main::run();
}
