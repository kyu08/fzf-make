use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::model::command;

#[derive(Debug, Serialize, Deserialize)]
struct Histories {
    history: Vec<History>,
}

impl Histories {
    fn from(histories: Vec<(PathBuf, Vec<command::Command>)>) -> Self {
        let mut result: Vec<History> = vec![];
        for h in histories {
            result.push(History {
                path: h.0.to_str().unwrap().to_string(),
                executed_targets: h.1,
            });
        }
        Histories { history: result }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct History {
    path: String,
    executed_targets: Vec<Histories::>,
}

pub fn parse_history(content: String) -> Result<Vec<(PathBuf, Vec<command::Command>)>> {
    let histories: Histories = toml::from_str(&content)?;

    let mut result: Vec<(PathBuf, Vec<command::Command>)> = Vec::new();

    for history in histories.history {
        result.push((PathBuf::from(history.path), history.executed_targets));
    }
    Ok(result)
}

#[allow(dead_code)] // TODO(#321): remove
pub fn store_history(
    history_directory_path: PathBuf,
    history_file_name: String,
    histories_tuple: Vec<(PathBuf, Vec<command::Command>)>,
) -> Result<()> {
    let histories = Histories::from(histories_tuple);

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
            expect: Result<Vec<(PathBuf, Vec<command::Command>)>>,
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
                expect: Ok(vec![
                    (
                        PathBuf::from("/Users/user/code/fzf-make".to_string()),
                        vec![
                            command::Command::new(
                                runner_type::RunnerType::Make,
                                // WARN: ここにコマンドファイル名って持つ必要ないんだっけ？
                                // ないならいらないフィールドをoptionにするか構造体を分けるかしたいなー
                                "test".to_string(),
                                PathBuf::from("Makefile"),
                                1,
                            ),
                            command::Command::new(
                                runner_type::RunnerType::Make,
                                "check".to_string(),
                                PathBuf::from("Makefile"),
                                4,
                            ),
                            command::Command::new(
                                runner_type::RunnerType::Make,
                                "spell-check".to_string(),
                                PathBuf::from("Makefile"),
                                4,
                            ),
                        ],
                    ),
                    (
                        PathBuf::from("/Users/user/code/golang/go-playground".to_string()),
                        vec![
                            command::Command::new(
                                runner_type::RunnerType::Make,
                                "run".to_string(),
                                PathBuf::from("Makefile"),
                                1,
                            ),
                            command::Command::new(
                                runner_type::RunnerType::Make,
                                "echo1".to_string(),
                                PathBuf::from("Makefile"),
                                4,
                            ),
                        ],
                    ),
                ]),
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
