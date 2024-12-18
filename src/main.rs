mod controller;
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
mod err;
mod file;
mod model;
mod usecase;

#[tokio::main]
async fn main() {
    let result = controller::controller_main::run();
    // ref: https://zenn.dev/techno_tanoc/articles/4c207397df3ab0#assertunwindsafe
    let res = AssertUnwindSafe(result).catch_unwind().await;

    if let Err(e) = res {
        println!("{}", err::any_to_string::any_to_string(&*e));
        std::process::exit(1);
    }
}
