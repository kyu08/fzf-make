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
    // get_makefile_file_names returns filenames of Makefile and the files included by Makefile
    pub fn create_makefile() -> Result<Makefile, &'static str> {
        let Some(makefile_name) = Makefile::specify_makefile_name(".".to_string()) else { return Err("makefile not found\n") };

        Ok(Makefile::new(Path::new(&makefile_name).to_path_buf()))
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

    // I gave up writing tests using temp_dir because it was too difficult (it was necessary to change the implementation to some extent).
    // It is not difficult to ensure that it works with manual tests, so I will not support it for now.
    fn new(path: PathBuf) -> Makefile {
        // If the file path does not exist, the make command cannot be executed in the first place,
        // so it is not handled here.
        let file_content = file::path_to_content(path.clone());
        let include_files = include::content_to_include_file_paths(file_content.clone())
            .iter()
            .map(|included_file_path| Makefile::new(included_file_path.clone()))
            .collect();

        Makefile {
            path,
            include_files,
            targets: target::content_to_targets(file_content),
        }
    }

    fn specify_makefile_name(target_path: String) -> Option<String> {
        //  By default, when make looks for the makefile, it tries the following names, in order: GNUmakefile, makefile and Makefile.
        //  https://www.gnu.org/software/make/manual/make.html#Makefile-Names
        // enumerate `Makefile` too not only `makefile` to make it work on case insensitive file system
        let makefile_name_options = vec!["GNUmakefile", "makefile", "Makefile"];

        for file_name in makefile_name_options {
            let path = Path::new(&target_path).join(file_name);
            if path.is_file() {
                return Some(file_name.to_string());
            }
            continue;
        }

        None
    }
}

#[cfg(test)]
mod test {
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
                files: vec!["makefile", "GNUmakefile", "README.md", "Makefile"],
                expect: Some("GNUmakefile".to_string()),
            },
            Case {
                title: "makefile",
                files: vec!["makefile", "Makefile", "README.md"],
                expect: Some("makefile".to_string()),
            },
            // NOTE: not use this test case because there is a difference in handling case sensitivity between macOS and linux.
            // to use this test case, you need to determine whether the file system is
            // case-sensitive or not when test execute.
            // Case {
            // title: "Makefile",
            // files: vec!["Makefile", "README.md"],
            // expect: Some("makefile".to_string()),
            // },
        ];

        for case in cases {
            let random_dir_name = Uuid::new_v4().to_string();
            let tmp_dir = std::env::temp_dir().join(random_dir_name);
            match fs::create_dir(tmp_dir.as_path()) {
                Err(e) => panic!("fail to create dir: {:?}", e),
                Ok(_) => {}
            }

            for file in case.files {
                match File::create(tmp_dir.join(file)) {
                    Err(e) => panic!("fail to create file: {:?}", e),
                    Ok(_) => {}
                }
            }

            assert_eq!(
                case.expect,
                Makefile::specify_makefile_name(tmp_dir.to_string_lossy().to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
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
                "\nFailed: 🚨{:?}🚨\n",
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
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            )
        }
    }
}
