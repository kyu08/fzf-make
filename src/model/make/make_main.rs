use super::target::*;
use crate::model::{command, file_util};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

/// Make represents a Makefile.
#[derive(Clone, Debug, PartialEq)]
pub struct Make {
    pub path: PathBuf,
    include_files: Vec<Make>,
    targets: Targets,
}

impl Make {
    /// It is possible to implement this method as an associated function because it takes a
    /// command as an argument. However, if it is an associated function, it can be called
    /// from anywhere, so it is better to make it a method to limit the context.
    pub fn command_to_run(&self, command: &command::CommandForExec) -> Result<String> {
        Ok(format!("make {}", command.args))
    }

    pub fn new(current_dir: PathBuf) -> Result<Make> {
        let Some(makefile_name) = Make::specify_makefile_name(current_dir, ".".to_string()) else {
            return Err(anyhow!("makefile not found.\n"));
        };
        Make::new_internal(Path::new(&makefile_name).to_path_buf())
    }

    pub fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        let mut result: Vec<command::CommandWithPreview> = vec![];
        result.append(&mut self.targets.0.to_vec());
        for include_file in &self.include_files {
            Vec::append(&mut result, &mut include_file.to_commands());
        }

        result
    }

    pub fn execute(&self, command: &command::CommandForExec) -> Result<()> {
        let child = process::Command::new("make")
            .stdin(process::Stdio::inherit())
            .args(command.args.split_whitespace())
            .spawn();

        match child {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("failed to run: {}", e)),
            },
            Err(e) => Err(anyhow!("failed to spawn: {}", e)),
        }
    }

    // I gave up writing tests using temp_dir because it was too difficult (it was necessary to change the implementation to some extent).
    // It is not difficult to ensure that it works with manual tests, so I will not do it for now.
    fn new_internal(path: PathBuf) -> Result<Make> {
        // If the file path does not exist, the make command cannot be executed in the first place,
        // so it is not handled here.
        let file_content = file_util::path_to_content(path.clone())?;
        let include_files = content_to_include_file_paths(file_content.clone())
            .iter()
            .map(|included_file_path| Make::new_internal(included_file_path.clone()))
            .filter_map(Result::ok)
            .collect();

        Ok(Make {
            path: path.clone(),
            include_files,
            targets: Targets::new(file_content, path),
        })
    }

    fn specify_makefile_name(current_dir: PathBuf, target_path: String) -> Option<PathBuf> {
        //  By default, when make looks for the makefile, it tries the following names, in order: GNUmakefile, makefile and Makefile.
        //  https://www.gnu.org/software/make/manual/make.html#Makefile-Names
        // It needs to enumerate `Makefile` too not only `makefile` to make it work on case insensitive file system
        let makefile_name_options = ["GNUmakefile", "makefile", "Makefile"];

        let mut temp_result = Vec::<PathBuf>::new();
        let elements = fs::read_dir(target_path.clone()).unwrap();
        for e in elements {
            let file_name = e.unwrap().file_name();
            let file_name_string = file_name.to_str().unwrap();
            if makefile_name_options.contains(&file_name_string) {
                temp_result.push(current_dir.join(file_name));
            }
        }

        // It needs to return "GNUmakefile", "makefile", "Makefile" in order of priority
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
    pub fn new_for_test() -> Make {
        use crate::model::runner_type;
        use std::env;

        Make {
            path: env::current_dir().unwrap().join(Path::new("Test.mk")),
            include_files: vec![],
            targets: Targets(vec![
                command::CommandWithPreview::new(
                    runner_type::RunnerType::Make,
                    "target0".to_string(),
                    PathBuf::from(""),
                    1,
                ),
                command::CommandWithPreview::new(
                    runner_type::RunnerType::Make,
                    "target1".to_string(),
                    PathBuf::from(""),
                    4,
                ),
                command::CommandWithPreview::new(
                    runner_type::RunnerType::Make,
                    "target2".to_string(),
                    PathBuf::from(""),
                    7,
                ),
            ]),
        }
    }
}

/// The path should be relative path from current directory where make command is executed.
/// So the path can be treated as it is.
/// NOTE: path include `..` is not supported for now like `include ../c.mk`.
fn content_to_include_file_paths(file_content: String) -> Vec<PathBuf> {
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
    use super::*;
    use crate::model::runner_type;
    use pretty_assertions::assert_eq;
    use std::{
        env,
        fs::{self, File},
    };
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
                files: vec!["makefile", "GNUmakefile", "README.md", "Makefile"],
                expect: Some("GNUmakefile".to_string()),
            },
            Case {
                title: "makefile",
                files: vec!["makefile", "Makefile", "README.md"],
                expect: Some("makefile".to_string()),
            },
            Case {
                title: "Makefile",
                files: vec!["Makefile", "README.md"],
                expect: Some("Makefile".to_string()),
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
                Make::specify_makefile_name(env::current_dir().unwrap(), tmp_dir.to_string_lossy().to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }

    #[test]
    fn makefile_to_commands_test() {
        struct Case {
            title: &'static str,
            makefile: Make,
            expect: Vec<command::CommandWithPreview>,
        }

        let cases = vec![
            Case {
                title: "makefile with no target",
                makefile: Make {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: Targets(vec![]),
                },
                expect: vec![],
            },
            Case {
                title: "makefile with no include directive",
                makefile: Make {
                    path: Path::new("path").to_path_buf(),
                    include_files: vec![],
                    targets: Targets(vec![
                        command::CommandWithPreview::new(
                            runner_type::RunnerType::Make,
                            "test".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                        command::CommandWithPreview::new(
                            runner_type::RunnerType::Make,
                            "run".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                    ]),
                },
                expect: vec![
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "test".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "run".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                ],
            },
            Case {
                title: "makefile with nested include directive",
                makefile: Make {
                    path: Path::new("path1").to_path_buf(),
                    include_files: vec![
                        Make {
                            path: Path::new("path2").to_path_buf(),
                            include_files: vec![Make {
                                path: Path::new("path2-1").to_path_buf(),
                                include_files: vec![],
                                targets: Targets(vec![
                                    command::CommandWithPreview::new(
                                        runner_type::RunnerType::Make,
                                        "test2-1".to_string(),
                                        PathBuf::from(""),
                                        4,
                                    ),
                                    command::CommandWithPreview::new(
                                        runner_type::RunnerType::Make,
                                        "run2-1".to_string(),
                                        PathBuf::from(""),
                                        4,
                                    ),
                                ]),
                            }],
                            targets: Targets(vec![
                                command::CommandWithPreview::new(
                                    runner_type::RunnerType::Make,
                                    "test2".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                                command::CommandWithPreview::new(
                                    runner_type::RunnerType::Make,
                                    "run2".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                            ]),
                        },
                        Make {
                            path: Path::new("path3").to_path_buf(),
                            include_files: vec![],
                            targets: Targets(vec![
                                command::CommandWithPreview::new(
                                    runner_type::RunnerType::Make,
                                    "test3".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                                command::CommandWithPreview::new(
                                    runner_type::RunnerType::Make,
                                    "run3".to_string(),
                                    PathBuf::from(""),
                                    4,
                                ),
                            ]),
                        },
                    ],
                    targets: Targets(vec![
                        command::CommandWithPreview::new(
                            runner_type::RunnerType::Make,
                            "test1".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                        command::CommandWithPreview::new(
                            runner_type::RunnerType::Make,
                            "run1".to_string(),
                            PathBuf::from(""),
                            4,
                        ),
                    ]),
                },
                expect: vec![
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "test1".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "run1".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "test2".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "run2".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "test2-1".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "run2-1".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "test3".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "run3".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                ],
            },
        ];

        for case in cases {
            assert_eq!(case.expect, case.makefile.to_commands(), "\nFailed: 🚨{:?}🚨\n", case.title,)
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
                expect: Some(vec![Path::new("one.mk").to_path_buf(), Path::new("two.mk").to_path_buf()]),
            },
            Case {
                title: "-include",
                line: "-include one.mk two.mk",
                expect: Some(vec![Path::new("one.mk").to_path_buf(), Path::new("two.mk").to_path_buf()]),
            },
            Case {
                title: "sinclude",
                line: "sinclude hoge.mk fuga.mk",
                expect: Some(vec![Path::new("hoge.mk").to_path_buf(), Path::new("fuga.mk").to_path_buf()]),
            },
            Case {
                title: " include one.mk two.mk",
                line: " include one.mk two.mk",
                expect: Some(vec![Path::new("one.mk").to_path_buf(), Path::new("two.mk").to_path_buf()]),
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
                expect: Some(vec![Path::new("one.mk").to_path_buf(), Path::new("two.mk").to_path_buf()]),
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
