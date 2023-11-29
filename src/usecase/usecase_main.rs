use anyhow::Result;

pub trait Usecase {
    fn command_str(&self) -> Vec<&'static str>;
    fn run(&self) -> Result<()>;
}
