use simple_home_dir::home_dir;
use std::{env, path::PathBuf};

use super::{command, runner_type};

/// Histories is a all collection of History. This equals whole content of history.toml.
/// For now, we can define this as tuple like `pub struct Histories(Vec<History>);` but we don't.
/// We respect that we can add some fields in the future easily.
#[derive(Clone, PartialEq, Debug)]
pub struct Histories {
    pub histories: Vec<History>,
}

impl Histories {
    // TODO(#321): Make this fn returns Vec<runner::Command>
    // pub fn get_histories(&self, paths: Vec<PathBuf>) -> Vec<String> {
    //     let mut histories: Vec<String> = Vec::new();
    //
    //     for path in paths {
    //         let executed_targets = self
    //             .histories
    //             .iter()
    //             .find(|h| h.path == path)
    //             .map(|h| h.executed_targets.clone())
    //             .unwrap_or(Vec::new());
    //         histories = [histories, executed_targets].concat();
    //     }
    //
    //     histories
    // }

    // pub fn append(&self, path: &PathBuf, executed_target: &str) -> Option<Self> {
    //     let mut new_histories = self.histories.clone();
    //
    //     new_histories
    //         .iter()
    //         .position(|h| h.path == *path)
    //         .map(|index| {
    //             let new_history = new_histories[index].append(executed_target.to_string());
    //             new_histories[index] = new_history;
    //
    //             Self {
    //                 histories: new_histories,
    //             }
    //         })
    // }

    // pub fn to_tuple(&self) -> Vec<(PathBuf, Vec<String>)> {
    //     let mut result = Vec::new();
    //
    //     for history in &self.histories {
    //         result.push((history.path.clone(), history.executed_targets.clone()));
    //     }
    //     result
    // }

    pub fn get_latest_command(&self, path: &PathBuf) -> Option<&HistoryCommand> {
        self.histories
            .iter()
            .find(|h| h.path == *path)
            .map(|h| h.executed_commands.first())?
    }
}

// TODO(#321): should return Result not Option(returns when it fails to get the home dir)
pub fn history_file_path() -> Option<(PathBuf, String)> {
    const HISTORY_FILE_NAME: &str = "history.toml";

    match env::var("FZF_MAKE_IS_TESTING") {
        Ok(_) => {
            // When testing
            let cwd = std::env::current_dir().unwrap();
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

#[derive(Clone, PartialEq, Debug)]
pub struct History {
    pub path: PathBuf,
    pub executed_commands: Vec<HistoryCommand>, // TODO: rename to executed_commands
}

impl History {
    fn default(path: PathBuf) -> Self {
        Self {
            path,
            executed_commands: Vec::new(),
        }
    }

    fn from(histories: (PathBuf, Vec<command::Command>)) -> Self {
        Self {
            path: histories.0,
            executed_commands: histories.1,
        }
    }

    // TODO(#321): remove
    #[allow(dead_code)]
    fn append(&self, executed_target: command::Command) -> Self {
        let mut executed_targets = self.executed_commands.clone();
        executed_targets.retain(|t| *t != executed_target);
        executed_targets.insert(0, executed_target.clone());

        const MAX_LENGTH: usize = 10;
        if MAX_LENGTH < executed_targets.len() {
            executed_targets.truncate(MAX_LENGTH);
        }

        Self {
            path: self.path.clone(),
            executed_commands: executed_targets,
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

#[cfg(test)]
mod test {
    // TODO(#321): comment in this test
    // #[test]
    // fn histories_append_test() {
    //     struct Case {
    //         title: &'static str,
    //         path: PathBuf,
    //         appending_target: &'static str,
    //         histories: Histories,
    //         expect: Option<Histories>,
    //     }
    //     let cases = vec![
    //         Case {
    //             title: "Success",
    //             path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
    //             appending_target: "history1",
    //             histories: Histories {
    //                 histories: vec![
    //                     History {
    //                         path: PathBuf::from("/Users/user/code/rustc".to_string()),
    //                         executed_targets: vec!["history0".to_string(), "history1".to_string()],
    //                     },
    //                     History {
    //                         path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
    //                         executed_targets: vec![
    //                             "history0".to_string(),
    //                             "history1".to_string(),
    //                             "history2".to_string(),
    //                         ],
    //                     },
    //                 ],
    //             },
    //             expect: Some(Histories {
    //                 histories: vec![
    //                     History {
    //                         path: PathBuf::from("/Users/user/code/rustc".to_string()),
    //                         executed_targets: vec!["history0".to_string(), "history1".to_string()],
    //                     },
    //                     History {
    //                         path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
    //                         executed_targets: vec![
    //                             "history1".to_string(),
    //                             "history0".to_string(),
    //                             "history2".to_string(),
    //                         ],
    //                     },
    //                 ],
    //             }),
    //         },
    //         Case {
    //             title: "Returns None when path is not found",
    //             path: PathBuf::from("/Users/user/code/non-existent-dir".to_string()),
    //             appending_target: "history1",
    //             histories: Histories {
    //                 histories: vec![
    //                     History {
    //                         path: PathBuf::from("/Users/user/code/rustc".to_string()),
    //                         executed_targets: vec!["history0".to_string(), "history1".to_string()],
    //                     },
    //                     History {
    //                         path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
    //                         executed_targets: vec![
    //                             "history0".to_string(),
    //                             "history1".to_string(),
    //                             "history2".to_string(),
    //                         ],
    //                     },
    //                 ],
    //             },
    //             expect: None,
    //         },
    //     ];
    //
    //     for case in cases {
    //         assert_eq!(
    //             case.expect,
    //             case.histories.append(&case.path, case.appending_target),
    //             "\nFailed: 🚨{:?}🚨\n",
    //             case.title,
    //         )
    //     }
    // }
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
