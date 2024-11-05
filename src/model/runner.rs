use anyhow::Result;

pub trait Runner
where
    Self: Selector + Executor,
{
}

pub trait Selector {
    fn list_commands(&self) -> Vec<String>;
}

pub trait Executor {
    fn execute(&self) -> Result<()>;
}
