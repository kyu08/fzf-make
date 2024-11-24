use super::toml as fzf_make_toml;
use crate::model;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Histories {
    history: Vec<History>,
}

impl Histories {
    fn into_histories(self) -> fzf_make_toml::Histories {
        let mut result: Vec<fzf_make_toml::History> = vec![];
        for h in self.history.clone() {
            let mut commands: Vec<fzf_make_toml::HistoryCommand> = vec![];
            for c in h.executed_targets {
                commands.push(fzf_make_toml::HistoryCommand::new(
                    model::runner_type::RunnerType::Make,
                    c,
                ));
            }
            // NOTE: In old format, the path includes the file name but new format does not.
            let mut makefile_path = PathBuf::from(h.path);
            makefile_path.pop();
            result.push(fzf_make_toml::History::new(makefile_path, commands));
        }

        fzf_make_toml::Histories::new(result)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename(deserialize = "history"))]
#[serde(rename_all = "kebab-case")]
struct History {
    path: String,
    executed_targets: Vec<String>,
}

pub fn parse_history(content: String) -> Result<fzf_make_toml::Histories> {
    toml::from_str(&content)
        .map(|h: Histories| h.into_histories())
        .map_err(|e| e.into())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file::toml as fzf_make_toml;
    use crate::model::runner_type;
    use pretty_assertions::assert_eq;

    #[test]
    fn into_histories_test() {
        struct Case {
            title: &'static str,
            before: Histories,
            after: fzf_make_toml::Histories,
        }
        let cases = vec![Case {
            title: "Success",
            before: Histories {
                history: vec![History {
                    path: "path/Makefile".to_string(),
                    executed_targets: vec!["command1".to_string(), "command2".to_string()],
                }],
            },
            after: fzf_make_toml::Histories::new(vec![fzf_make_toml::History::new(
                PathBuf::from("path"),
                vec![
                    fzf_make_toml::HistoryCommand::new(
                        runner_type::RunnerType::Make,
                        "command1".to_string(),
                    ),
                    fzf_make_toml::HistoryCommand::new(
                        runner_type::RunnerType::Make,
                        "command2".to_string(),
                    ),
                ],
            )]),
        }];

        for case in cases {
            assert_eq!(
                case.after,
                case.before.into_histories(),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }

    #[test]
    fn parse_history_test() {
        struct Case {
            title: &'static str,
            content: String,
            expect: Result<fzf_make_toml::Histories>,
        }
        let cases = vec![
            Case {
                title: "Success",
                content: r#"
[[history]]
path = "/Users/user/code/fzf-make/Makefile"
executed-targets = ["test", "check", "spell-check"]

[[history]]
path = "/Users/user/code/golang/go-playground/Makefile"
executed-targets = ["run", "echo1"]
                "#
                .to_string(),
                expect: Ok(fzf_make_toml::Histories::new(vec![
                    fzf_make_toml::History::new(
                        PathBuf::from("/Users/user/code/fzf-make"),
                        vec![
                            fzf_make_toml::HistoryCommand::new(
                                runner_type::RunnerType::Make,
                                "test".to_string(),
                            ),
                            fzf_make_toml::HistoryCommand::new(
                                runner_type::RunnerType::Make,
                                "check".to_string(),
                            ),
                            fzf_make_toml::HistoryCommand::new(
                                runner_type::RunnerType::Make,
                                "spell-check".to_string(),
                            ),
                        ],
                    ),
                    fzf_make_toml::History::new(
                        PathBuf::from("/Users/user/code/golang/go-playground"),
                        vec![
                            fzf_make_toml::HistoryCommand::new(
                                runner_type::RunnerType::Make,
                                "run".to_string(),
                            ),
                            fzf_make_toml::HistoryCommand::new(
                                runner_type::RunnerType::Make,
                                "echo1".to_string(),
                            ),
                        ],
                    ),
                ])),
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
                Ok(e) => assert_eq!(
                    e,
                    parse_history(case.content).unwrap(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
                Err(err) => assert_eq!(
                    err.to_string(),
                    parse_history(case.content).unwrap_err().to_string(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
            }
        }
    }
}
