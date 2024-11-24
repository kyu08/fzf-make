use super::toml::{Histories, History, HistoryCommand};
use crate::model;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct HistoriesOld {
    histories_old: Vec<HistoryOld>,
}

impl HistoriesOld {
    fn into_histories(self) -> Histories {
        let mut result: Vec<History> = vec![];
        for h in self.histories_old.clone() {
            let mut commands: Vec<HistoryCommand> = vec![];
            for c in h.executed_targets {
                commands.push(HistoryCommand::new(model::runner_type::RunnerType::Make, c));
            }
            result.push(History::new(PathBuf::from(h.path), commands));
        }

        Histories::new(result)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
struct HistoryOld {
    path: String,
    executed_targets: Vec<String>,
}

pub fn parse_history(content: String) -> Result<Histories> {
    let histories: HistoriesOld = toml::from_str(&content)?;
    Ok(histories.into_histories())
}
