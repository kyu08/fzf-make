use super::toml as fzf_make_toml;
use crate::model;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct HistoriesOld {
    histories_old: Vec<HistoryOld>,
}

impl HistoriesOld {
    fn into_histories(self) -> fzf_make_toml::Histories {
        let mut result: Vec<fzf_make_toml::History> = vec![];
        for h in self.histories_old.clone() {
            let mut commands: Vec<fzf_make_toml::HistoryCommand> = vec![];
            for c in h.executed_targets {
                commands.push(fzf_make_toml::HistoryCommand::new(
                    model::runner_type::RunnerType::Make,
                    c,
                ));
            }
            result.push(fzf_make_toml::History::new(PathBuf::from(h.path), commands));
        }

        fzf_make_toml::Histories::new(result)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(rename(deserialize = "history"))]
struct HistoryOld {
    path: String,
    executed_targets: Vec<String>,
}

pub fn parse_history(content: String) -> Result<fzf_make_toml::Histories> {
    let histories: HistoriesOld = toml::from_str(&content)?;
    Ok(histories.into_histories())
}
