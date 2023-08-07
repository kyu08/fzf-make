use std::path::{Path, PathBuf};

use crate::file::file;

use super::{include, target};

/// Makefile represents a Makefile.
pub struct Makefile {
    path: PathBuf,
    include_files: Vec<Makefile>,
    targets: target::Targets,
}

impl Makefile {
    // TODO: add UT
    pub fn new(path: PathBuf) -> Makefile {
        let file_content = file::path_to_content(path.clone());
        let including_file_paths = include::extract_including_file_paths(file_content.clone());
        let include_files: Vec<Makefile> = including_file_paths
            .iter()
            .map(|path| Makefile::new(Path::new(&path).to_path_buf()))
            .collect();

        Makefile {
            path,
            include_files,
            targets: target::content_to_commands(file_content),
        }
    }

    pub fn to_include_path_string(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        result.push(self.path.to_string_lossy().to_string());

        for include_file in &self.include_files {
            Vec::append(&mut result, &mut include_file.to_include_path_string());
        }

        result
    }

    pub fn to_target_string(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        (&mut result).append(&mut self.targets.clone());
        for include_file in &self.include_files {
            Vec::append(&mut result, &mut include_file.to_target_string());
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn makefile_new_test() {
        struct Case {
            title: &'static str,
            makefile: Makefile,
            expect: Vec<&'static str>,
        }

        let cases = vec![Case {
            title: "makefile with nested include directive",
            makefile: Makefile {
                path: Path::new("path1").to_path_buf(),
                include_files: vec![
                    Makefile {
                        path: Path::new("path2").to_path_buf(),
                        include_files: vec![Makefile {
                            path: Path::new("path2-1").to_path_buf(),
                            include_files: vec![],
                            targets: vec![],
                        }],
                        targets: vec![],
                    },
                    Makefile {
                        path: Path::new("path3").to_path_buf(),
                        include_files: vec![],
                        targets: vec![],
                    },
                ],
                targets: vec![],
            },
            expect: vec!["path1", "path2", "path2-1", "path3"],
        }];

        // tempdirにいい感じの環境をつくる
        // テストかく
        for case in cases {
            let mut expect_string: Vec<String> =
                case.expect.iter().map(|e| e.to_string()).collect();
            expect_string.sort();
            let sorted_result = case.makefile.to_include_path_string();

            assert_eq!(
                expect_string, sorted_result,
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            )
        }
    }

    #[test]
    fn makefile_to_include_path_string_test() {
        struct Case {
            title: &'static str,
            makefile: Makefile,
            expect: Vec<&'static str>,
        }

        let cases = vec![
            Case {
                title: "makefile with no include directive",
                makefile: Makefile {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: vec!["test".to_string(), "run".to_string()],
                },
                expect: vec!["path"],
            },
            Case {
                title: "makefile with nested include directive",
                makefile: Makefile {
                    path: Path::new("path1").to_path_buf(),
                    include_files: vec![
                        Makefile {
                            path: Path::new("path2").to_path_buf(),
                            include_files: vec![Makefile {
                                path: Path::new("path2-1").to_path_buf(),
                                include_files: vec![],
                                targets: vec![],
                            }],
                            targets: vec![],
                        },
                        Makefile {
                            path: Path::new("path3").to_path_buf(),
                            include_files: vec![],
                            targets: vec![],
                        },
                    ],
                    targets: vec![],
                },
                expect: vec!["path1", "path2", "path2-1", "path3"],
            },
        ];

        for case in cases {
            let mut expect_string: Vec<String> =
                case.expect.iter().map(|e| e.to_string()).collect();
            expect_string.sort();
            let sorted_result = case.makefile.to_include_path_string();

            assert_eq!(
                expect_string, sorted_result,
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            )
        }
    }

    #[test]
    fn makefile_to_target_string_test() {
        struct Case {
            title: &'static str,
            makefile: Makefile,
            expect: Vec<&'static str>,
        }

        let cases = vec![
            Case {
                title: "makefile with no target",
                makefile: Makefile {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: vec![],
                },
                expect: vec![],
            },
            Case {
                title: "makefile with no include directive",
                makefile: Makefile {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: vec!["test".to_string(), "run".to_string()],
                },
                expect: vec!["test", "run"],
            },
            Case {
                title: "makefile with nested include directive",
                makefile: Makefile {
                    path: Path::new("path1").to_path_buf(),
                    include_files: vec![
                        Makefile {
                            path: Path::new("path2").to_path_buf(),
                            include_files: vec![Makefile {
                                path: Path::new("path2-1").to_path_buf(),
                                include_files: vec![],
                                targets: vec!["test2-1".to_string(), "run2-1".to_string()],
                            }],
                            targets: vec!["test2".to_string(), "run2".to_string()],
                        },
                        Makefile {
                            path: Path::new("path3").to_path_buf(),
                            include_files: vec![],
                            targets: vec!["test3".to_string(), "run3".to_string()],
                        },
                    ],
                    targets: vec!["test1".to_string(), "run1".to_string()],
                },
                expect: vec![
                    "test1", "run1", "test2", "run2", "test2-1", "run2-1", "test3", "run3",
                ],
            },
        ];

        for case in cases {
            let expect_string: Vec<String> = case.expect.iter().map(|e| e.to_string()).collect();

            assert_eq!(
                expect_string,
                case.makefile.to_target_string(),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            )
        }
    }
}
