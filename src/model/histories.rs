use simple_home_dir::home_dir;
use std::{env, path::PathBuf};

#[derive(Clone, PartialEq, Debug)]
pub struct Histories {
    histories: Vec<History>,
}

impl Histories {
    pub fn new(makefile_path: PathBuf, histories: Vec<(PathBuf, Vec<String>)>) -> Self {
        match histories.len() {
            0 => Self::default(makefile_path),
            _ => Self::from(makefile_path, histories),
        }
    }

    pub fn get_history(&self, path: &PathBuf) -> Option<Vec<String>> {
        self.histories
            .iter()
            .find(|h| h.path == *path)
            .map(|h| h.executed_targets.clone())
    }

    pub fn append(&self, path: &PathBuf, executed_target: &str) -> Option<Self> {
        let mut new_histories = self.histories.clone();

        new_histories
            .iter()
            .position(|h| h.path == *path)
            .map(|index| {
                let new_history = new_histories[index].append(executed_target.to_string());
                new_histories[index] = new_history;

                Self {
                    histories: new_histories,
                }
            })
    }

    pub fn to_tuple(&self) -> Vec<(PathBuf, Vec<String>)> {
        let mut result = Vec::new();

        for history in &self.histories {
            result.push((history.path.clone(), history.executed_targets.clone()));
        }
        result
    }

    fn default(path: PathBuf) -> Self {
        let histories = vec![History::default(path)];
        Self { histories }
    }

    fn from(makefile_path: PathBuf, histories: Vec<(PathBuf, Vec<String>)>) -> Self {
        let mut result = Histories {
            histories: Vec::new(),
        };

        for history in histories.clone() {
            result.histories.push(History::from(history));
        }

        if !histories.iter().any(|h| h.0 == makefile_path) {
            result.histories.push(History::default(makefile_path));
        }

        result
    }
}

pub fn history_file_path() -> Option<PathBuf> {
    match env::var("FZF_MAKE_IS_TESTING") {
        Ok(_) => {
            // When testing
            let cwd = std::env::current_dir().unwrap();
            Some(cwd.join(PathBuf::from("test_dir/history.toml")))
        }
        _ => {
            home_dir().map(|home_dir| home_dir.join(PathBuf::from(".config/fzf-make/history.toml")))
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct History {
    path: PathBuf,
    executed_targets: Vec<String>,
}

impl History {
    fn default(path: PathBuf) -> Self {
        Self {
            path,
            executed_targets: Vec::new(),
        }
    }

    fn from(histories: (PathBuf, Vec<String>)) -> Self {
        Self {
            path: histories.0,
            executed_targets: histories.1,
        }
    }

    fn append(&self, executed_target: String) -> Self {
        let mut executed_targets = self.executed_targets.clone();
        executed_targets.retain(|t| *t != executed_target);
        executed_targets.insert(0, executed_target.clone());
        // const MAX_LENGTH: usize = 10;
        // if executed_targets.len() > MAX_LENGTH {
        //     executed_targets.truncate(MAX_LENGTH);
        // }

        Self {
            path: self.path.clone(),
            executed_targets,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn histories_new_test() {
        struct Case {
            title: &'static str,
            makefile_path: PathBuf,
            histories: Vec<(PathBuf, Vec<String>)>,
            expect: Histories,
        }
        let cases = vec![
            Case {
                title: "histories.len() == 0",
                makefile_path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                histories: vec![],
                expect: Histories {
                    histories: vec![History {
                        path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                        executed_targets: vec![],
                    }],
                },
            },
            Case {
                title: "histories.len() != 0(Including makefile_path)",
                makefile_path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                histories: vec![
                    (
                        PathBuf::from("/Users/user/code/fzf-make".to_string()),
                        vec!["target1".to_string(), "target2".to_string()],
                    ),
                    (
                        PathBuf::from("/Users/user/code/rustc".to_string()),
                        vec!["target-a".to_string(), "target-b".to_string()],
                    ),
                ],
                expect: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                            executed_targets: vec!["target1".to_string(), "target2".to_string()],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            executed_targets: vec!["target-a".to_string(), "target-b".to_string()],
                        },
                    ],
                },
            },
            Case {
                title: "histories.len() != 0(Not including makefile_path)",
                makefile_path: PathBuf::from("/Users/user/code/cargo".to_string()),
                histories: vec![
                    (
                        PathBuf::from("/Users/user/code/fzf-make".to_string()),
                        vec!["target1".to_string(), "target2".to_string()],
                    ),
                    (
                        PathBuf::from("/Users/user/code/rustc".to_string()),
                        vec!["target-a".to_string(), "target-b".to_string()],
                    ),
                ],
                expect: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                            executed_targets: vec!["target1".to_string(), "target2".to_string()],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            executed_targets: vec!["target-a".to_string(), "target-b".to_string()],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/cargo".to_string()),
                            executed_targets: vec![],
                        },
                    ],
                },
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                Histories::new(case.makefile_path, case.histories),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }

    #[test]
    fn histories_append_test() {
        struct Case {
            title: &'static str,
            path: PathBuf,
            appending_target: &'static str,
            histories: Histories,
            expect: Option<Histories>,
        }
        let cases = vec![
            Case {
                title: "Success",
                path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                appending_target: "history1",
                histories: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            executed_targets: vec!["history0".to_string(), "history1".to_string()],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                            executed_targets: vec![
                                "history0".to_string(),
                                "history1".to_string(),
                                "history2".to_string(),
                            ],
                        },
                    ],
                },
                expect: Some(Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            executed_targets: vec!["history0".to_string(), "history1".to_string()],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                            executed_targets: vec![
                                "history1".to_string(),
                                "history0".to_string(),
                                "history2".to_string(),
                            ],
                        },
                    ],
                }),
            },
            Case {
                title: "Returns None when path is not found",
                path: PathBuf::from("/Users/user/code/non-existent-dir".to_string()),
                appending_target: "history1",
                histories: Histories {
                    histories: vec![
                        History {
                            path: PathBuf::from("/Users/user/code/rustc".to_string()),
                            executed_targets: vec!["history0".to_string(), "history1".to_string()],
                        },
                        History {
                            path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                            executed_targets: vec![
                                "history0".to_string(),
                                "history1".to_string(),
                                "history2".to_string(),
                            ],
                        },
                    ],
                },
                expect: None,
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                case.histories.append(&case.path, case.appending_target),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }

    #[test]
    fn history_append_test() {
        struct Case {
            title: &'static str,
            appending_target: &'static str,
            history: History,
            expect: History,
        }
        let path = PathBuf::from("/Users/user/code/fzf-make".to_string());
        let cases = vec![
            Case {
                title: "Append to head",
                appending_target: "history2",
                history: History {
                    path: path.clone(),
                    executed_targets: vec!["history0".to_string(), "history1".to_string()],
                },
                expect: History {
                    path: path.clone(),
                    executed_targets: vec![
                        "history2".to_string(),
                        "history0".to_string(),
                        "history1".to_string(),
                    ],
                },
            },
            Case {
                title: "Append to head(Append to empty)",
                appending_target: "history0",
                history: History {
                    path: path.clone(),
                    executed_targets: vec![],
                },
                expect: History {
                    path: path.clone(),
                    executed_targets: vec!["history0".to_string()],
                },
            },
            Case {
                title: "Append to head(Remove duplicated)",
                appending_target: "history1",
                history: History {
                    path: path.clone(),
                    executed_targets: vec![
                        "history0".to_string(),
                        "history1".to_string(),
                        "history2".to_string(),
                    ],
                },
                expect: History {
                    path: path.clone(),
                    executed_targets: vec![
                        "history1".to_string(),
                        "history0".to_string(),
                        "history2".to_string(),
                    ],
                },
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                case.history.append(case.appending_target.to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }
}
