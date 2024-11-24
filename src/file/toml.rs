use super::{path_to_content, toml_old};
use crate::model::{
    histories::{self},
    runner_type,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use simple_home_dir::home_dir;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Histories {
    histories: Vec<History>,
}

impl Histories {
    pub fn get_history() -> Histories {
        match history_file_path() {
            Some((history_file_dir, history_file_name)) => {
                match path_to_content::path_to_content(history_file_dir.join(history_file_name)) {
                    Ok(c) => Histories::parse_history_in_considering_history_file_format_version(c),
                    Err(_) => Histories::default(),
                }
            }
            None => Histories::default(),
        }
    }

    fn parse_history_in_considering_history_file_format_version(content: String) -> Histories {
        // NOTE: The history file format has changed after https://github.com/kyu08/fzf-make/pull/324.
        // So at first we try to parse it as the new format, and then try to parse it as the old format.
        match parse_history(content.to_string()) {
            Ok(h) => h,
            Err(_) => toml_old::parse_history(content.to_string()).unwrap_or_default(),
        }
    }

    pub fn new(histories: Vec<History>) -> Self {
        Self { histories }
    }

    fn from(histories: histories::Histories) -> Self {
        let mut result: Vec<History> = vec![];
        for h in histories.histories {
            result.push(History::from(h));
        }
        Self { histories: result }
    }

    pub fn into(self) -> histories::Histories {
        let mut result: Vec<histories::History> = vec![];
        for h in self.histories {
            result.push(History::into(h));
        }
        histories::Histories { histories: result }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct History {
    path: PathBuf,
    commands: Vec<HistoryCommand>,
}

impl History {
    fn from(history: histories::History) -> Self {
        let mut commands: Vec<HistoryCommand> = vec![];
        for h in history.commands {
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
            commands,
        }
    }

    pub fn new(path: PathBuf, commands: Vec<HistoryCommand>) -> Self {
        Self { path, commands }
    }
}

/// toml representation of histories::HistoryCommand.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct HistoryCommand {
    runner_type: runner_type::RunnerType,
    name: String,
}

impl HistoryCommand {
    pub fn new(runner_type: runner_type::RunnerType, name: String) -> Self {
        Self { runner_type, name }
    }

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

// TODO: should return Result not Option(returns when it fails to get the home dir)
pub fn history_file_path() -> Option<(PathBuf, String)> {
    const HISTORY_FILE_NAME: &str = "history.toml";

    match env::var("FZF_MAKE_IS_TESTING") {
        Ok(_) => {
            // When testing
            let cwd = env::current_dir().unwrap();
            Some((
                cwd.join(PathBuf::from("test_dir")),
                HISTORY_FILE_NAME.to_string(),
            ))
        }
        _ => home_dir().map(|home_dir| {
            (
                home_dir.join(PathBuf::from(".config/fzf-make")),
                HISTORY_FILE_NAME.to_string(),
            )
        }),
    }
}

pub fn parse_history(content: String) -> Result<Histories> {
    let histories = toml::from_str(&content)?;
    Ok(histories)
}

pub fn create_or_update_history_file(
    history_directory_path: PathBuf,
    history_file_name: String,
    new_history: histories::Histories,
) -> Result<()> {
    if !history_directory_path.is_dir() {
        fs::create_dir_all(history_directory_path.clone())?;
    }
    let mut history_file = File::create(history_directory_path.join(history_file_name))?;
    history_file.write_all(
        toml::to_string(&Histories::from(new_history))
            .unwrap()
            .as_bytes(),
    )?;
    history_file.flush()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::runner_type;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_history_test() {
        struct Case {
            title: &'static str,
            content: String,
            expect: Result<Histories>,
        }
        let cases = vec![
            Case {
                title: "Success",
                content: r#"
[[histories]]
path = "/Users/user/code/fzf-make"

[[histories.commands]]
runner-type = "make"
name = "test"

[[histories.commands]]
runner-type = "make"
name = "check"

[[histories.commands]]
runner-type = "make"
name = "spell-check"

[[histories]]
path = "/Users/user/code/golang/go-playground"

[[histories.commands]]
runner-type = "make"
name = "run"

[[histories.commands]]
runner-type = "make"
name = "echo1"
                "#
                .to_string(),
                expect: Ok(Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/fzf-make"),
                            commands: vec![
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "test".to_string(),
                                },
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "check".to_string(),
                                },
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "spell-check".to_string(),
                                },
                            ],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/golang/go-playground"),
                            commands: vec![
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "run".to_string(),
                                },
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "echo1".to_string(),
                                },
                            ],
                        },
                    ],
                }),
            },
            Case {
                title: "Error",
                content: r#"
                "#
                .to_string(),
                expect: Err(anyhow::anyhow!("TOML parse error at line 1, column 1\n  |\n1 | \n  | ^\nmissing field `histories`\n")),
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
