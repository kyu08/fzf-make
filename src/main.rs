mod controller;
mod err;
mod file;
mod model;
mod usecase;
use std::panic;

#[tokio::main]
async fn main() {
    // let result = panic::catch_unwind(|| async {
    // controller::controller_main::run().await;
    // });
    // if let Err(e) = result.await {
    controller::controller_main::run().await;
    // if let Err(e) = .await.unwrap() {
    //     println!("{}", err::any_to_string::any_to_string(&*e));
    //     std::process::exit(1);
    // }
}
