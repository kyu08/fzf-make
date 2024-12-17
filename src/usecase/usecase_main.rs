use std::future::Future;

use anyhow::Result;

pub trait Usecase {
    fn command_str(&self) -> Vec<&'static str>;
    fn run(&self) -> impl Future<Output = Result<(), anyhow::Error>>;
    // impl Future<Output = Result<(), anyhow::Error>>
}
