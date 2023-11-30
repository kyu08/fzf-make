use std::path::PathBuf;

use regex::Regex;

use super::util;

#[derive(Debug, Clone, PartialEq)]
pub struct Targets(pub Vec<String>);

impl Targets {
    pub fn new(content: String) -> Targets {
        let mut result: Vec<String> = Vec::new();
        for line in content.lines() {
            if let Some(t) = line_to_target(line.to_string()) {
                result.push(t);
            }
        }

        Targets(result)
    }
}

pub fn target_line_number(path: PathBuf, target_to_search: String) -> Option<u32> {
    let content = util::path_to_content(path);
    for (index, line) in content.lines().enumerate() {
        if let Some(t) = line_to_target(line.to_string()) {
            if t == target_to_search {
                return Some(index as u32 + 1); // Line number starts from 1
            }
        }
    }

    None
}

fn line_to_target(line: String) -> Option<String> {
    let regex = Regex::new(r"^ *[^.#\s　][^=]+:[^=]*$").unwrap();
    regex.find(line.as_str()).map(|m| {
        m.as_str()
            .to_string()
            .split_once(':')
            .unwrap()
            .0
            .trim()
            .to_string()
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn content_to_targets_test() {
        struct Case {
            title: &'static str,
            contents: &'static str,
            expect: Targets, // NOTE: order of elements of `expect` order should be same as vec function returns
        }
        let cases = vec![
            Case {
                title: "comment in same line",
                contents: "\
.PHONY: run build check test

run:
		@cargo run

build:
		@cargo build

check:
		@cargo check


test: # run test
        @cargo test

echo:
	@echo good",
                expect: Targets(vec![
                    "run".to_string(),
                    "build".to_string(),
                    "check".to_string(),
                    "test".to_string(),
                    "echo".to_string(),
                ]),
            },
            Case {
                title: "comment line",
                contents: "\
.PHONY: clone build

# https://example.com
clone:
		@git clone https://example.com

build:
		@cargo build",
                expect: Targets(vec!["clone".to_string(), "build".to_string()]),
            },
            Case {
                title: "invalid format",
                contents: "echo hello",
                expect: Targets(vec![]),
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                Targets::new(case.contents.to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }

    #[test]
    fn line_to_target_test() {
        struct Case {
            title: &'static str,
            contents: &'static str,
            expect: Option<&'static str>,
        }
        let cases = vec![
            Case {
                title: "echo:",
                contents: "echo:",
                expect: Some("echo"),
            },
            Case {
                title: "main.o:",
                contents: "main.o:",
                expect: Some("main.o"),
            },
            Case {
                title: "test::",
                contents: "test::",
                expect: Some("test"),
            },
            Case {
                title: "test ::",
                contents: "test ::",
                expect: Some("test"),
            },
            Case {
                title: "echo",
                contents: "echo",
                expect: None,
            },
            Case {
                title: "		@git clone https://example.com",
                contents: "		@git clone https://example.com",
                expect: None,
            },
            Case {
                title: ".PHONY:",
                contents: ".PHONY:",
                expect: None,
            },
            Case {
                title: ".DEFAULT:",
                contents: ".DEFAULT:",
                expect: None,
            },
            Case {
                title: "# run:",
                contents: "# run:",
                expect: None,
            },
            Case {
                title: " # run:",
                contents: " # run:",
                expect: None,
            },
            Case {
                title: "hoge := 1",
                contents: "hoge := 1",
                expect: None,
            },
            Case {
                title: "hoge?=fuga:1",
                contents: "hoge?=fuga:1",
                expect: None,
            },
            Case {
                title: "hoge=fuga:1",
                contents: "hoge=fuga:1",
                expect: None,
            },
            Case {
                title: " hoge:",
                contents: " hoge:",
                expect: Some("hoge"),
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect.map(|e| e.to_string()),
                line_to_target(case.contents.to_string()),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }
}
