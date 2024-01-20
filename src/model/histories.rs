use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug)]
pub struct Histories {
    histories: Vec<History>,
}

impl Histories {
    pub fn new(path: PathBuf, histories: Vec<(PathBuf, Vec<String>)>) -> Self {
        match histories.len() {
            0 => Self::default(path),
            _ => Self::from(histories),
        }
    }

    fn default(path: PathBuf) -> Self {
        let histories = vec![History::default(path)];
        Self { histories }
    }

    fn from(histories: Vec<(PathBuf, Vec<String>)>) -> Self {
        let mut result = Histories {
            histories: Vec::new(),
        };

        for history in histories {
            result.histories.push(History::from(history));
        }
        result
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn histories_new_test() {
        struct Case {
            title: &'static str,
            path: PathBuf,
            histories: Vec<(PathBuf, Vec<String>)>,
            expect: Histories,
        }
        let cases = vec![
            Case {
                title: "histories.len() == 0",
                path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                histories: vec![],
                expect: Histories {
                    histories: vec![History {
                        path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
                        executed_targets: vec![],
                    }],
                },
            },
            Case {
                title: "histories.len() != 0",
                path: PathBuf::from("/Users/user/code/fzf-make".to_string()),
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
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                Histories::new(case.path, case.histories),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }
}
