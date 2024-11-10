use crate::usecase::usecase_main::Usecase;
use anyhow::{anyhow, Result};

use super::{
    execute_make_command::execute_make_target,
    tui::{
        app::{AppState, Model},
        config,
    },
};

pub struct Repeat;

impl Repeat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for Repeat {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--repeat", "-r", "repeat"]
    }

    fn run(&self) -> Result<()> {
        match Model::new(config::Config::default()) {
            Err(e) => Err(e),
            Ok(model) => match model.app_state {
                AppState::SelectTarget(model) => {
                    match model.histories.map(|h| {
                        // todo!("ここの仕様どうする？");
                        // 1. historyのうちcwdから始まるものを探してきて最新を実行（今の情報だと最新かどうかわからんわ）（履歴ファイルの中の並び順で時系列を表現する？e.g.一番上が最新）
                        // 2. 複数候補がありそうなときは選択肢を表示して選ばせる?
                        match &model.runners.first() {
                            Some(runner) => {
                                h.get_latest_target(&runner.path()).map(execute_make_target)
                            }
                            None => None,
                        }
                    }) {
                        Some(Some(_)) => Ok(()),
                        _ => Err(anyhow!("No target found")),
                    }
                }
                _ => Err(anyhow!("Invalid state")),
            },
        }
    }
}
