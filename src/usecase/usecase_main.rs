use anyhow::Result;
use futures::future::BoxFuture;

pub trait Usecase: Send + Sync {
    fn command_str(&self) -> Vec<&'static str>;
    fn run(&self) -> BoxFuture<'_, Result<()>>;
}
