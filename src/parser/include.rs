use regex::Regex;
use std::path::{Path, PathBuf};

/// The path should be relative path from current directory where make command is executed.
/// So the path can be treated as it is.
/// NOTE: path include `..` is not supported for now like `include ../c.mk`.
pub fn content_to_include_file_paths(file_content: String) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    for line in file_content.lines() {
        let Some(include_files) = line_to_including_file_paths(line.to_string()) else { continue };

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
    regex.find(line.as_str()).and_then(|line| {
        let line_excluding_comment = match line.as_str().to_string().split_once("#") {
            Some((before, _)) => before.to_string(),
            None => line.as_str().to_string(),
        };

        let mut directive_and_file_names: Vec<PathBuf> = line_excluding_comment
            .split_whitespace()
            .map(|e| Path::new(e).to_path_buf())
            .collect();

        // remove directive itself. (include or -include or sinclude)
        directive_and_file_names.remove(0);

        Some(directive_and_file_names)
    })
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;
    use uuid::Uuid;

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
            match fs::create_dir(tmp_dir.as_path()) {
                Err(e) => panic!("fail to create dir: {:?}", e),
                Ok(_) => {}
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
            match fs::create_dir(tmp_dir.as_path()) {
                Err(e) => panic!("fail to create dir: {:?}", e),
                Ok(_) => {}
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
