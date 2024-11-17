use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::model::{histories, runner_type};

#[derive(Debug, Serialize, Deserialize)]
struct Histories {
    histories: Vec<History>,
}

impl Histories {
    fn from(histories: histories::Histories) -> Self {
        let mut result: Vec<History> = vec![];
        for h in histories.histories {
            result.push(History::from(h));
        }
        Self { histories: result }
    }

    fn into(self) -> histories::Histories {
        let mut result: Vec<histories::History> = vec![];
        for h in self.histories {
            result.push(History::into(h));
        }
        histories::Histories { histories: result }
    }
}

impl std::default::Default for Histories {
    fn default() -> Self {
        Self { histories: vec![] }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct History {
    path: PathBuf,
    commands: Vec<HistoryCommand>,
}

impl History {
    fn from(history: histories::History) -> Self {
        let mut commands: Vec<HistoryCommand> = vec![];
        for h in history.executed_commands {
            commands.push(HistoryCommand::from(h));
        }

        History {
            path: history.path,
            commands,
        }
    }

    fn into(self) -> histories::History {
        let mut commands: Vec<histories::HistoryCommand> = vec![];
        for h in self.commands {
            commands.push(HistoryCommand::into(h));
        }

        histories::History {
            path: self.path,
            executed_commands: commands,
        }
    }
}

/// toml representation of histories::HistoryCommand.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HistoryCommand {
    runner_type: runner_type::RunnerType,
    name: String,
}

impl HistoryCommand {
    fn from(command: histories::HistoryCommand) -> Self {
        Self {
            runner_type: command.runner_type,
            name: command.name.clone(),
        }
    }

    fn into(self) -> histories::HistoryCommand {
        histories::HistoryCommand {
            runner_type: self.runner_type,
            name: self.name,
        }
    }
}

pub fn parse_history(content: String) -> Result<histories::Histories> {
    let histories = toml::from_str(&content)?;
    Ok(Histories::into(histories))
}

#[allow(dead_code)] // TODO(#321): remove
pub fn store_history(
    history_directory_path: PathBuf,
    history_file_name: String,
    histories: histories::Histories,
) -> Result<()> {
    let histories = Histories::from(histories);

    if !history_directory_path.is_dir() {
        fs::create_dir_all(history_directory_path.clone())?;
    }

    let mut history_file = File::create(history_directory_path.join(history_file_name))?;
    history_file.write_all(toml::to_string(&histories).unwrap().as_bytes())?;
    history_file.flush()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::model::runner_type;
    use super::*;
    use anyhow::Result;

    #[test]
    fn parse_history_test() {
        struct Case {
            title: &'static str,
            content: String,
            expect: Result<histories::Histories>,
        }
        let cases = vec![
            Case {
                title: "Success",
                content: r#"
[[tasks]]
path = "/Users/user/code/fzf-make"

[[tasks.commands]]
runner = "make"
command = "test"

[[tasks.commands]]
runner = "make"
command = "check"

[[tasks.commands]]
runner = "make"
command = "spell-check"

[[tasks]]]
path = "/Users/user/code/golang/go-playground"

[[tasks.commands]]
runner = "make"
command = "run"

[[tasks.commands]]
runner = "make"
command = "echo1"
                "#
                .to_string(),
                expect: Ok(
                    histories::Histories{ histories: vec![
                        histories::History{ path: PathBuf::from("/Users/user/code/fzf-make".to_string()), executed_commands: vec![
                            histories::HistoryCommand{ 
                                runner_type: runner_type::RunnerType::Make,  
                                name: "test".to_string() },
                        ] },
                    ] },
                    // (
                    //     PathBuf::from("/Users/user/code/fzf-make".to_string()),
                    //     vec![
                    //         command::Command::new(
                    //             runner_type::RunnerType::Make,
                    //             "test".to_string(),
                    //             PathBuf::from("Makefile"),
                    //             1,
                    //         ),
                    //         command::Command::new(
                    //             runner_type::RunnerType::Make,
                    //             "check".to_string(),
                    //             PathBuf::from("Makefile"),
                    //             4,
                    //         ),
                    //         command::Command::new(
                    //             runner_type::RunnerType::Make,
                    //             "spell-check".to_string(),
                    //             PathBuf::from("Makefile"),
                    //             4,
                    //         ),
                    //     ],
                    // ),
                    // (
                    //     PathBuf::from("/Users/user/code/golang/go-playground".to_string()),
                    //     vec![
                    //         command::Command::new(
                    //             runner_type::RunnerType::Make,
                    //             "run".to_string(),
                    //             PathBuf::from("Makefile"),
                    //             1,
                    //         ),
                    //         command::Command::new(
                    //             runner_type::RunnerType::Make,
                    //             "echo1".to_string(),
                    //             PathBuf::from("Makefile"),
                    //             4,
                    //         ),
                    //     ],
                    // ),
                ),
            },
            Case {
                title: "Error",
                content: r#"
                "#
                .to_string(),
                expect: Err(anyhow::anyhow!("TOML parse error at line 1, column 1\n  |\n1 | \n  | ^\nmissing field `history`\n")),
            },
        ];

        for case in cases {
            match case.expect {
                Ok(v) => assert_eq!(
                    v,
                    parse_history(case.content).unwrap(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
                Err(e) => assert_eq!(
                    e.to_string(),
                    parse_history(case.content).unwrap_err().to_string(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
            }
        }
    }
}
