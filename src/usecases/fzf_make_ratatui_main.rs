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

    // TODO: ratatuiのUIが起動するようにする
    // まずはtutorialのコードが動くようにする https://github.com/ratatui-org/ratatui-book/tree/main/src/tutorial/json-editor/ratatui-json-editor-app
    // そこからUIを少しずつ作っていく
    fn run(&self) {
        let _ = ratatui::main();
    }
}
