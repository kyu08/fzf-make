use super::{file_util, target::*};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Pnpm {
    pub path: PathBuf,
    include_files: Vec<Pnpm>,
    targets: Targets,
}
// TODO(#321): delete unnecessary implementation
impl Pnpm {
    pub fn create_makefile() -> Result<Pnpm> {
        let Some(makefile_name) = Pnpm::specify_makefile_name(".".to_string()) else {
            return Err(anyhow!("makefile not found.\n"));
        };
        Pnpm::new(Path::new(&makefile_name).to_path_buf())
    }

    pub fn to_targets_string(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        result.append(
            &mut self
                .targets
                .0
                .iter()
                .map(|c| c.clone().to_string())
                .collect(),
        );
        for include_file in &self.include_files {
            Vec::append(&mut result, &mut include_file.to_targets_string());
        }

        result
    }

    // I gave up writing tests using temp_dir because it was too difficult (it was necessary to change the implementation to some extent).
    // It is not difficult to ensure that it works with manual tests, so I will not do it for now.
    fn new(path: PathBuf) -> Result<Pnpm> {
        // If the file path does not exist, the make command cannot be executed in the first place,
        // so it is not handled here.
        let file_content = file_util::path_to_content(path.clone())?;
        let include_files = content_to_include_file_paths(file_content.clone())
            .iter()
            .map(|included_file_path| Pnpm::new(included_file_path.clone()))
            .filter_map(Result::ok)
            .collect();

        Ok(Pnpm {
            path: path.clone(),
            include_files,
            targets: Targets::new(file_content, path),
        })
    }

    fn specify_makefile_name(target_path: String) -> Option<PathBuf> {
        let makefile_name_options = ["package.json"];

        let mut temp_result = Vec::<PathBuf>::new();
        let elements = fs::read_dir(target_path.clone()).unwrap();
        for e in elements {
            let file_name = e.unwrap().file_name();
            let file_name_string = file_name.to_str().unwrap();
            if makefile_name_options.contains(&file_name_string) {
                let current_dir = match env::current_dir() {
                    Err(_) => return None,
                    Ok(d) => d,
                };

                temp_result.push(current_dir.join(file_name));
            }
        }

        for makefile_name_option in makefile_name_options {
            for result in &temp_result {
                if result.to_str().unwrap().contains(makefile_name_option) {
                    return Some(result.clone());
                }
            }
        }

        None
    }

    #[cfg(test)]
    pub fn new_for_test() -> Pnpm {
        use super::runner_type;
        use crate::model::command;

        Pnpm {
            path: env::current_dir().unwrap().join(Path::new("Test.mk")),
            include_files: vec![],
            targets: Targets(vec![
                command::Command::new(
                    runner_type::RunnerType::Make,
                    "target0".to_string(),
                    PathBuf::from(""),
                    4,
                ),
                command::Command::new(
                    runner_type::RunnerType::Make,
                    "target1".to_string(),
                    PathBuf::from(""),
                    4,
                ),
                command::Command::new(
                    runner_type::RunnerType::Make,
                    "target2".to_string(),
                    PathBuf::from(""),
                    4,
                ),
            ]),
        }
    }
}

/// The path should be relative path from current directory where make command is executed.
/// So the path can be treated as it is.
/// NOTE: path include `..` is not supported for now like `include ../c.mk`.
pub fn content_to_include_file_paths(file_content: String) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    for line in file_content.lines() {
        let Some(include_files) = line_to_including_file_paths(line.to_string()) else {
            continue;
        };

        result = [result, include_files].concat();
    }

    result
}

/// The line that is only include directive is ignored.
/// Pattern like `include foo *.mk $(bar)` is not handled for now.
/// Additional search is not executed if file is not found based on current directory.
fn line_to_including_file_paths(line: String) -> Option<Vec<PathBuf>> {
    // not to allow tab character, ` ` is used instead of `\s`
    let regex = Regex::new(r"^ *(include|-include|sinclude).*$").unwrap();
    regex.find(line.as_str()).map(|line| {
        let line_excluding_comment = match line.as_str().to_string().split_once('#') {
            Some((before, _)) => before.to_string(),
            None => line.as_str().to_string(),
        };

        let mut directive_and_file_names: Vec<PathBuf> = line_excluding_comment
            .split_whitespace()
            .map(|e| Path::new(e).to_path_buf())
            .collect();

        // remove directive itself. (include or -include or sinclude)
        directive_and_file_names.remove(0);

        directive_and_file_names
    })
}

#[cfg(test)]
mod test {
    use crate::model::{command, runner_type};

    use super::*;

    use std::fs::{self, File};
    use uuid::Uuid;

    #[test]
    fn specify_makefile_name_test() {
        struct Case {
            title: &'static str,
            files: Vec<&'static str>,
            expect: Option<String>,
        }
        let cases = vec![
            Case {
                title: "no makefile",
                files: vec!["README.md"],
                expect: None,
            },
            Case {
                title: "GNUmakefile",
                files: vec!["makefile", "GNUmakefile", "README.md", "Pnpmfile"],
                expect: Some("GNUmakefile".to_string()),
            },
            Case {
                title: "makefile",
                files: vec!["makefile", "Pnpmfile", "README.md"],
                expect: Some("makefile".to_string()),
            },
            Case {
                title: "Pnpmfile",
                files: vec!["Pnpmfile", "README.md"],
                expect: Some("Pnpmfile".to_string()),
            },
        ];

        for case in cases {
            let random_dir_name = Uuid::new_v4().to_string();
            let tmp_dir = std::env::temp_dir().join(random_dir_name);
            if let Err(e) = fs::create_dir(tmp_dir.as_path()) {
                panic!("fail to create dir: {:?}", e)
            }

            for file in case.files {
                if let Err(e) = File::create(tmp_dir.join(file)) {
                    panic!("fail to create file: {:?}", e)
                }
            }

            let expect = match (env::current_dir(), case.expect) {
                (Ok(c), Some(e)) => Some(c.join(e)),
                _ => None,
            };

            assert_eq!(
                expect,
                Pnpm::specify_makefile_name(tmp_dir.to_string_lossy().to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }

    #[test]
    fn makefile_to_targets_string_test() {
        struct Case {
            title: &'static str,
            makefile: Pnpm,
            expect: Vec<&'static str>,
        }

        let cases = vec![
            Case {
                title: "makefile with no target",
                makefile: Pnpm {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: Targets(vec![]),
                },
                expect: vec![],
            },
            Case {
                title: "makefile with no include directive",
                makefile: Pnpm {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: Targets(vec![
                        command::Command::new(
                            runner_type::RunnerType::Make,
                            "test".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                        command::Command::new(
                            runner_type::RunnerType::Make,
                            "run".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                    ]),
                },
                expect: vec!["test", "run"],
            },
            Case {
                title: "makefile with nested include directive",
                makefile: Pnpm {
                    path: Path::new("path1").to_path_buf(),
                    include_files: vec![
                        Pnpm {
                            path: Path::new("path2").to_path_buf(),
                            include_files: vec![Pnpm {
                                path: Path::new("path2-1").to_path_buf(),
                                include_files: vec![],
                                targets: Targets(vec![
                                    command::Command::new(
                                        runner_type::RunnerType::Make,
                                        "test2-1".to_string(),
                                        PathBuf::from(""),
                                        4,
                                    ),
                                    command::Command::new(
                                        runner_type::RunnerType::Make,
                                        "run2-1".to_string(),
                                        PathBuf::from(""),
                                        4,
                                    ),
                                ]),
                            }],
                            targets: Targets(vec![
                                command::Command::new(
                                    runner_type::RunnerType::Make,
                                    "test2".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                                command::Command::new(
                                    runner_type::RunnerType::Make,
                                    "run2".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                            ]),
                        },
                        Pnpm {
                            path: Path::new("path3").to_path_buf(),
                            include_files: vec![],
                            targets: Targets(vec![
                                command::Command::new(
                                    runner_type::RunnerType::Make,
                                    "test3".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                                command::Command::new(
                                    runner_type::RunnerType::Make,
                                    "run3".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                            ]),
                        },
                    ],
                    targets: Targets(vec![
                        command::Command::new(
                            runner_type::RunnerType::Make,
                            "test1".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                        command::Command::new(
                            runner_type::RunnerType::Make,
                            "run1".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                    ]),
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
                case.makefile.to_targets_string(),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }

    #[test]
    fn extract_including_file_paths_test() {
        struct Case {
            title: &'static str,
            file_content: &'static str,
            expect: Vec<PathBuf>,
        }
        let cases = vec![
            Case {
                title: "has two lines of line includes include directive",
                file_content: "\
    include one.mk two.mk
    .PHONY: echo-test
    echo-test:
    	@echo good

    include three.mk four.mk

    .PHONY: test
    test:
    	cargo nextest run",
                expect: vec![
                    Path::new("one.mk").to_path_buf(),
                    Path::new("two.mk").to_path_buf(),
                    Path::new("three.mk").to_path_buf(),
                    Path::new("four.mk").to_path_buf(),
                ],
            },
            Case {
                title: "has no lines includes include directive",
                file_content: "\
    .PHONY: echo-test test
    echo-test:
    	@echo good

    test:
    	cargo nextest run",
                expect: vec![],
            },
        ];

        for mut case in cases {
            let random_dir_name = Uuid::new_v4().to_string();
            let tmp_dir = std::env::temp_dir().join(random_dir_name);
            if let Err(e) = fs::create_dir(tmp_dir.as_path()) {
                panic!("fail to create dir: {:?}", e)
            }

            case.expect.sort();
            let mut got = content_to_include_file_paths(case.file_content.to_string());
            got.sort();

            assert_eq!(case.expect, got, "\nFailed: 🚨{:?}🚨\n", case.title,);
        }
    }

    #[test]
    fn line_to_including_file_paths_test() {
        struct Case {
            title: &'static str,
            line: &'static str,
            expect: Option<Vec<PathBuf>>,
        }
        let cases = vec![
            Case {
                title: "include one.mk two.mk",
                line: "include one.mk two.mk",
                expect: Some(vec![
                    Path::new("one.mk").to_path_buf(),
                    Path::new("two.mk").to_path_buf(),
                ]),
            },
            Case {
                title: "-include",
                line: "-include one.mk two.mk",
                expect: Some(vec![
                    Path::new("one.mk").to_path_buf(),
                    Path::new("two.mk").to_path_buf(),
                ]),
            },
            Case {
                title: "sinclude",
                line: "sinclude hoge.mk fuga.mk",
                expect: Some(vec![
                    Path::new("hoge.mk").to_path_buf(),
                    Path::new("fuga.mk").to_path_buf(),
                ]),
            },
            Case {
                title: " include one.mk two.mk",
                line: " include one.mk two.mk",
                expect: Some(vec![
                    Path::new("one.mk").to_path_buf(),
                    Path::new("two.mk").to_path_buf(),
                ]),
            },
            Case {
                title: "include one.mk two.mk(tab is not allowed)",
                line: "	include one.mk two.mk",
                expect: None,
            },
            Case {
                title: "not include directive",
                line: ".PHONY: test",
                expect: None,
            },
            Case {
                title: "include comment",
                line: "include one.mk two.mk # three.mk",
                expect: Some(vec![
                    Path::new("one.mk").to_path_buf(),
                    Path::new("two.mk").to_path_buf(),
                ]),
            },
            Case {
                title: "# include one.mk two.mk # three.mk",
                line: "# include one.mk two.mk # three.mk",
                expect: None,
            },
            Case {
                title: "included",
                line: "included",
                expect: Some(vec![]),
            },
        ];

        for case in cases {
            let random_dir_name = Uuid::new_v4().to_string();
            let tmp_dir = std::env::temp_dir().join(random_dir_name);
            if let Err(e) = fs::create_dir(tmp_dir.as_path()) {
                panic!("fail to create dir: {:?}", e)
            }

            assert_eq!(
                case.expect,
                line_to_including_file_paths(case.line.to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }
}
