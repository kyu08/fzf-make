use crate::usecases::fzf_make::app;
use crate::usecases::usecase::Usecase;

pub struct FzfMake;

impl FzfMake {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for FzfMake {
    fn command_str(&self) -> Vec<&'static str> {
        vec![]
    }

    fn run(&self) {
        let _ = app::main();
    }
}
