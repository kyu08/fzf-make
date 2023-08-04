use regex::Regex;

pub fn extract_including_file_names(file_content: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for line in file_content.lines() {
        let include_files = line_to_including_files(line.to_string());
        result = [result, include_files].concat();
    }

    result
}

// trunslate above to english
// ignore if line is only include
// give up to handle pattern like `include foo *.mk $(bar)`
// do not search if file is not found based on current directory
fn line_to_including_files(line: String) -> Vec<String> {
    // not to allow tab character, ` ` is used instead of `\s`
    let regex = Regex::new(r"^ *(include|-include|sinclude).*$").unwrap();
    let include_line = regex.find(line.as_str());
    match include_line {
        None => return Vec::new(),
        Some(line) => {
            let excluding_comment = match line.as_str().to_string().split_once("#") {
                Some((before, _)) => before.to_string(),
                None => line.as_str().to_string(),
            };

            let mut file_names: Vec<String> = excluding_comment
                .split_whitespace()
                .map(|e| e.to_string())
                .collect();

            // remove directive(include or -include or sinclude)
            file_names.remove(0);

            file_names
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use super::*;
    use uuid::Uuid;

    #[test]
    fn extract_including_file_names_test() {
        struct Case {
            title: &'static str,
            file_content: &'static str,
            expect: Vec<&'static str>,
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
                expect: vec!["one.mk", "two.mk", "three.mk", "four.mk"],
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

        for case in cases {
            let random_dir_name = Uuid::new_v4().to_string();
            let tmp_dir = std::env::temp_dir().join(random_dir_name);
            match fs::create_dir(tmp_dir.as_path()) {
                Err(e) => panic!("fail to create dir: {:?}", e),
                Ok(_) => {}
            }

            assert_eq!(
                case.expect,
                extract_including_file_names(case.file_content.to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }

    #[test]
    fn line_to_including_files_test() {
        struct Case {
            title: &'static str,
            line: &'static str,
            expect: Vec<&'static str>,
        }
        let cases = vec![
            Case {
                title: "include one.mk two.mk",
                line: "include one.mk two.mk",
                expect: vec!["one.mk", "two.mk"],
            },
            Case {
                title: "-include",
                line: "-include one.mk two.mk",
                expect: vec!["one.mk", "two.mk"],
            },
            Case {
                title: "sinclude",
                line: "sinclude hoge.mk fuga.mk",
                expect: vec!["hoge.mk", "fuga.mk"],
            },
            Case {
                title: " include one.mk two.mk",
                line: " include one.mk two.mk",
                expect: vec!["one.mk", "two.mk"],
            },
            Case {
                title: "include one.mk two.mk(tab is not allowed)",
                line: "	include one.mk two.mk",
                expect: vec![],
            },
            Case {
                title: "not include directive",
                line: ".PHONY: test",
                expect: vec![],
            },
            Case {
                title: "include comment",
                line: "include one.mk two.mk # three.mk",
                expect: vec!["one.mk", "two.mk"],
            },
            Case {
                title: "# include one.mk two.mk # three.mk",
                line: "# include one.mk two.mk # three.mk",
                expect: vec![],
            },
            Case {
                title: "included",
                line: "included",
                expect: vec![],
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
                line_to_including_files(case.line.to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }
}
