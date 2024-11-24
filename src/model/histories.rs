use super::{command, runner_type};
use std::path::PathBuf;

/// Histories is a all collection of History. This equals whole content of history.toml.
/// For now, we can define this as tuple like `pub struct Histories(Vec<History>);` but we don't.
/// We respect that we can add some fields in the future easily.
#[derive(Clone, PartialEq, Debug)]
pub struct Histories {
    pub histories: Vec<History>,
}

impl Histories {
    pub fn append(&self, current_dir: PathBuf, command: command::Command) -> Self {
        // Update the command history for the current directory.
        let new_history = {
            match self.histories.iter().find(|h| h.path == current_dir) {
                Some(history) => history.append(command.clone()),
                None => History {
                    path: current_dir,
                    commands: vec![HistoryCommand::from(command)],
                },
            }
        };

        // Update the whole histories.
        let mut new_histories = self.histories.clone();
        match new_histories
            .iter()
            .position(|h| h.path == new_history.path)
        {
            Some(index) => {
                new_histories[index] = new_history;
            }
            None => {
                new_histories.insert(0, new_history);
            }
        }

        Histories {
            histories: new_histories,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct History {
    pub path: PathBuf,
    pub commands: Vec<HistoryCommand>,
}

impl History {
    // TODO: ut
    fn append(&self, executed_command: command::Command) -> Self {
        let mut updated_commands = self.commands.clone();
        // removes the executed_command from the history
        updated_commands.retain(|t| *t != HistoryCommand::from(executed_command.clone()));
        updated_commands.insert(0, HistoryCommand::from(executed_command.clone()));

        const MAX_LENGTH: usize = 10;
        if MAX_LENGTH < updated_commands.len() {
            updated_commands.truncate(MAX_LENGTH);
        }

        Self {
            path: self.path.clone(),
            commands: updated_commands,
        }
    }
}

/// In the history file, the command has only the name of the command and the runner type.
/// Because its file name where it's defined and file number is variable.
/// So we search them every time fzf-make is launched.
#[derive(PartialEq, Clone, Debug)]
pub struct HistoryCommand {
    pub runner_type: runner_type::RunnerType,
    pub name: String,
}

impl HistoryCommand {
    pub fn from(command: command::Command) -> Self {
        Self {
            runner_type: command.runner_type,
            name: command.name,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn histories_append_test() {
        struct Case {
            title: &'static str,
            histories: Histories,
            command_to_append: command::Command,
            expect: Histories,
        }

        let path_to_append = PathBuf::from("/Users/user/code/fzf-make".to_string());
        let cases = vec![
            Case {
                // Use raw string literal as workaround for
                // https://github.com/rust-lang/rustfmt/issues/4800.
                title: r#"The command executed is appended to the existing history if there is history for cwd."#,
                histories: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                name: "history0".to_string(),
                            }],
                        },
                        History {
                            path: path_to_append.clone(),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                name: "history0".to_string(),
                            }],
                        },
                    ],
                },
                command_to_append: command::Command {
                    runner_type: runner_type::RunnerType::Make,
                    name: "append".to_string(),
                    file_name: PathBuf::from("Makefile"),
                    line_number: 1,
                },
                expect: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                name: "history0".to_string(),
                            }],
                        },
                        History {
                            path: path_to_append.clone(),
                            commands: vec![
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "append".to_string(),
                                },
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    name: "history0".to_string(),
                                },
                            ],
                        },
                    ],
                },
            },
            Case {
                title: r#"A new history is appended if there is no history for cwd."#,
                histories: Histories {
                    histories: vec![History {
                        path: PathBuf::from("/Users/user/code/rustc".to_string()),
                        commands: vec![HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            name: "history0".to_string(),
                        }],
                    }],
                },
                command_to_append: command::Command {
                    runner_type: runner_type::RunnerType::Make,
                    name: "append".to_string(),
                    file_name: PathBuf::from("Makefile"),
                    line_number: 1,
                },
                expect: Histories {
                    histories: vec![
                        History {
                            path: path_to_append.clone(),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                name: "append".to_string(),
                            }],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                name: "history0".to_string(),
                            }],
                        },
                    ],
                },
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                case.histories
                    .append(path_to_append.clone(), case.command_to_append),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }

    // TODO(#321): comment in this test
    // #[test]
    // fn history_append_test() {
    //     struct Case {
    //         title: &'static str,
    //         appending_target: command::Command,
    //         history: History,
    //         expect: History,
    //     }
    //     let path = PathBuf::from("/Users/user/code/fzf-make".to_string());
    //     let cases = vec![
    //         Case {
    //             title: "Append to head",
    //             appending_target: command::Command::new(
    //                 runner_type::RunnerType::Make,
    //                 "history2".to_string(),
    //                 PathBuf::from("Makefile"),
    //                 1,
    //             ),
    //             history: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history0".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history1".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                 ],
    //             },
    //             expect: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history2".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history0".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history1".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                 ],
    //             },
    //         },
    //         Case {
    //             title: "Append to head(Append to empty)",
    //             appending_target: command::Command::new(
    //                 runner_type::RunnerType::Make,
    //                 "history0".to_string(),
    //                 PathBuf::from("Makefile"),
    //                 4,
    //             ),
    //             history: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![],
    //             },
    //             expect: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![command::Command::new(
    //                     runner_type::RunnerType::Make,
    //                     "history0".to_string(),
    //                     PathBuf::from("Makefile"),
    //                     4,
    //                 )],
    //             },
    //         },
    //         Case {
    //             title: "Append to head(Remove duplicated)",
    //             appending_target: command::Command::new(
    //                 runner_type::RunnerType::Make,
    //                 "history1".to_string(),
    //                 PathBuf::from("Makefile"),
    //                 4,
    //             ),
    //             history: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history0".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history1".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history2".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                 ],
    //             },
    //             expect: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history1".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history0".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history2".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                 ],
    //             },
    //         },
    //         Case {
    //             title: "Truncate when length exceeds 10",
    //             appending_target: command::Command::new(
    //                 runner_type::RunnerType::Make,
    //                 "history11".to_string(),
    //                 PathBuf::from("Makefile"),
    //                 1,
    //             ),
    //             history: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history0".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history1".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history2".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history3".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history4".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history5".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history6".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history7".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history8".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history9".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                 ],
    //             },
    //             expect: History {
    //                 path: path.clone(),
    //                 executed_targets: vec![
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history11".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history0".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         1,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history1".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history2".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history3".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history4".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history5".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history6".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history7".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                     command::Command::new(
    //                         runner_type::RunnerType::Make,
    //                         "history8".to_string(),
    //                         PathBuf::from("Makefile"),
    //                         4,
    //                     ),
    //                 ],
    //             },
    //         },
    //     ];
    //
    //     for case in cases {
    //         assert_eq!(
    //             case.expect,
    //             case.history.append(case.appending_target),
    //             "\nFailed: 🚨{:?}🚨\n",
    //             case.title,
    //         )
    //     }
    // }
}
