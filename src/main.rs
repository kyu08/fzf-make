mod controller;
mod models;
mod usecases;

use crate::controller::controller_main;

fn main() {
    // TODO: Catch panic
    controller_main::run();
}
