use crate::usecases::fzf_make_ratatui::ratatui;
use crate::usecases::usecase::Usecase;

pub struct FzfMakeRatatui;

impl FzfMakeRatatui {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for FzfMakeRatatui {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--r", "-r", "r"]
    }

    fn run(&self) {
        let _ = ratatui::main();
    }
}
