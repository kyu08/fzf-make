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
    pub fn append(&self, current_dir: PathBuf, command: command::CommandForExec) -> Self {
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
        match new_histories.iter().position(|h| h.path == new_history.path) {
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
    /// The commands are sorted in descending order of execution time.
    /// This means that the first element is the most recently executed command.
    pub commands: Vec<HistoryCommand>,
}

impl History {
    fn append(&self, executed_command: command::CommandForExec) -> Self {
        let mut updated_commands = self.commands.clone();
        // removes the executed_command from the history
        updated_commands.retain(|t| *t != HistoryCommand::from(executed_command.clone()));
        updated_commands.insert(0, HistoryCommand::from(executed_command.clone()));

        const MAX_LENGTH: usize = 50;
        if MAX_LENGTH < updated_commands.len() {
            updated_commands.truncate(MAX_LENGTH);
        }

        Self {
            path: self.path.clone(),
            commands: updated_commands,
        }
    }
}

/// In the history file, the command has only the name of the command and the runner type though
/// command::Command has `file_path`, `line_number` as well.
/// Because its file name where it's defined and line number is variable.
/// So we search them every time fzf-make is launched instead of storing them in the history file.
#[derive(PartialEq, Clone, Debug)]
pub struct HistoryCommand {
    pub runner_type: runner_type::RunnerType,
    pub args: String,
}

impl From<command::CommandForExec> for HistoryCommand {
    fn from(command: command::CommandForExec) -> Self {
        Self {
            runner_type: command.runner_type,
            args: command.args,
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
            before: Histories,
            command_to_append: command::CommandForExec,
            after: Histories,
        }

        let path_to_append = PathBuf::from("/Users/user/code/fzf-make".to_string());
        let cases = vec![
            Case {
                // Use raw string literal as workaround for
                // https://github.com/rust-lang/rustfmt/issues/4800.
                title: r#"The command executed is appended to the existing history if there is history for cwd."#,
                before: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                args: "history0".to_string(),
                            }],
                        },
                        History {
                            path: path_to_append.clone(),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                args: "history0".to_string(),
                            }],
                        },
                    ],
                },
                command_to_append: command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "append".to_string(),
                },
                after: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                args: "history0".to_string(),
                            }],
                        },
                        History {
                            path: path_to_append.clone(),
                            commands: vec![
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    args: "append".to_string(),
                                },
                                HistoryCommand {
                                    runner_type: runner_type::RunnerType::Make,
                                    args: "history0".to_string(),
                                },
                            ],
                        },
                    ],
                },
            },
            Case {
                title: r#"A new history is appended if there is no history for cwd."#,
                before: Histories {
                    histories: vec![History {
                        path: PathBuf::from("/Users/user/code/rustc".to_string()),
                        commands: vec![HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        }],
                    }],
                },
                command_to_append: command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "append".to_string(),
                },
                after: Histories {
                    histories: vec![
                        History {
                            path: path_to_append.clone(),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                args: "append".to_string(),
                            }],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            commands: vec![HistoryCommand {
                                runner_type: runner_type::RunnerType::Make,
                                args: "history0".to_string(),
                            }],
                        },
                    ],
                },
            },
        ];

        for case in cases {
            assert_eq!(
                case.after,
                case.before.append(path_to_append.clone(), case.command_to_append),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }

    #[test]
    fn history_append_test() {
        struct Case {
            title: &'static str,
            before: History,
            command_to_append: command::CommandForExec,
            after: History,
        }
        let path = PathBuf::from("/Users/user/code/fzf-make".to_string());
        let cases = vec![
            Case {
                title: "Append to head",
                before: History {
                    path: path.clone(),
                    commands: vec![
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                    ],
                },
                command_to_append: command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history2".to_string(),
                },
                after: History {
                    path: path.clone(),
                    commands: vec![
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history2".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                    ],
                },
            },
            Case {
                title: "Append to head(Append to empty)",
                before: History {
                    path: path.clone(),
                    commands: vec![],
                },
                command_to_append: command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history0".to_string(),
                },
                after: History {
                    path: path.clone(),
                    commands: vec![HistoryCommand {
                        runner_type: runner_type::RunnerType::Make,
                        args: "history0".to_string(),
                    }],
                },
            },
            Case {
                title: "Append to head(Remove duplicated command)",
                before: History {
                    path: path.clone(),
                    commands: vec![
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history2".to_string(),
                        },
                    ],
                },
                command_to_append: command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history2".to_string(),
                },
                after: History {
                    path: path.clone(),
                    commands: vec![
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history2".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                    ],
                },
            },
            Case {
                title: "Truncate when length exceeds 50",
                before: History {
                    path: path.clone(),
                    commands: vec![
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history2".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history3".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history4".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history5".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history6".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history7".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history8".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history9".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history10".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history11".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history12".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history13".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history14".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history15".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history16".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history17".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history18".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history19".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history20".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history21".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history22".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history23".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history24".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history25".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history26".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history27".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history28".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history29".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history30".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history31".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history32".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history33".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history34".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history35".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history36".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history37".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history38".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history39".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history40".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history41".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history42".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history43".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history44".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history45".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history46".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history47".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history48".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history49".to_string(),
                        },
                    ],
                },
                command_to_append: command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history50".to_string(),
                },
                after: History {
                    path: path.clone(),
                    commands: vec![
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history50".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history0".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history2".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history3".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history4".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history5".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history6".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history7".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history8".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history9".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history10".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history11".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history12".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history13".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history14".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history15".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history16".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history17".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history18".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history19".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history20".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history21".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history22".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history23".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history24".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history25".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history26".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history27".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history28".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history29".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history30".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history31".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history32".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history33".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history34".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history35".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history36".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history37".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history38".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history39".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history40".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history41".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history42".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history43".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history44".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history45".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history46".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history47".to_string(),
                        },
                        HistoryCommand {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history48".to_string(),
                        },
                    ],
                },
            },
        ];

        for case in cases {
            assert_eq!(case.after, case.before.append(case.command_to_append), "\nFailed: 🚨{:?}🚨\n", case.title,)
        }
    }
}
