mod controller;
mod fuzzy_finder;
mod models;
mod usecase;

use crate::controller::controller_main;

fn main() {
    // TODO: Catch panic
    controller_main::run();
}
