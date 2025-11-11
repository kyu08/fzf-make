use crate::model::{command, runner_type};
use regex::Regex;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Targets(pub Vec<command::CommandWithPreview>);

impl Targets {
    pub fn new(content: String, path: PathBuf, recipe_prefix: Option<char>) -> Targets {
        let mut result: Vec<command::CommandWithPreview> = Vec::new();
        let mut define_block_depth = 0;

        for (i, line) in content.lines().enumerate() {
            match get_line_type(line, recipe_prefix) {
                LineType::DefineStart => {
                    define_block_depth += 1;
                }
                LineType::DefineEnd => {
                    define_block_depth -= 1;
                }
                LineType::Normal => {
                    if define_block_depth == 0
                        && let Some(t) = line_to_target(line.to_string())
                    {
                        let command = command::CommandWithPreview::new(
                            runner_type::RunnerType::Make,
                            t,
                            path.clone(),
                            i as u32 + 1,
                        );
                        result.push(command);
                    }
                }
                LineType::Recipe => {
                    // Skip recipe lines - they're not targets
                }
            }
        }

        Targets(result)
    }
}

const DEFINE_BLOCK_START: &str = "define";
const DEFINE_BLOCK_END: &str = "endef";
const OVERRIDE: &str = "override";

#[derive(Debug, PartialEq)]
enum LineType {
    Normal,
    DefineStart,
    DefineEnd,
    Recipe,
}

fn get_line_type(line: &str, recipe_prefix: Option<char>) -> LineType {
    if let Some(prefix) = recipe_prefix {
        if line.starts_with(prefix) {
            return LineType::Recipe;
        }
    } else if line.starts_with('\t') {
        return LineType::Recipe;
    }

    let words: Vec<&str> = line.split_whitespace().collect();

    if words.is_empty() {
        return LineType::Normal;
    }

    if words.len() >= 2 && words[0] == OVERRIDE && words[1] == DEFINE_BLOCK_START {
        return LineType::DefineStart;
    }

    match words.first() {
        Some(&w) => match w {
            DEFINE_BLOCK_START => LineType::DefineStart,
            DEFINE_BLOCK_END => LineType::DefineEnd,
            _ => LineType::Normal,
        },
        None => LineType::Normal,
    }
}

fn line_to_target(line: String) -> Option<String> {
    let regex = Regex::new(r"^ *[^.#\s　][^=]*:[^=]*$").unwrap();
    regex
        .find(line.as_str())
        .map(|m| m.as_str().to_string().split_once(':').unwrap().0.trim().to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

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
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "run".to_string(),
                        PathBuf::from(""),
                        3,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "build".to_string(),
                        PathBuf::from(""),
                        6,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "check".to_string(),
                        PathBuf::from(""),
                        9,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "test".to_string(),
                        PathBuf::from(""),
                        13,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "echo".to_string(),
                        PathBuf::from(""),
                        16,
                    ),
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
                expect: Targets(vec![
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "clone".to_string(),
                        PathBuf::from(""),
                        4,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "build".to_string(),
                        PathBuf::from(""),
                        7,
                    ),
                ]),
            },
            Case {
                title: "invalid format",
                contents: "echo hello",
                expect: Targets(vec![]),
            },
            Case {
                title: "trap script as a define block",
                contents: "\
.PHONY: all

all: my_script
define script-block
#!/bin/bash

echo \"this is a trap: not good\"
endef
my_script:
	$(file >my_script,$(script-block))\n",
                expect: Targets(vec![
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "all".to_string(),
                        PathBuf::from(""),
                        3,
                    ),
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::Make,
                        "my_script".to_string(),
                        PathBuf::from(""),
                        9,
                    ),
                ]),
            },
            Case {
                title: "nested define",
                contents: "\
define lvl-1
a:
    define lvl2
a:

    endef
a:
endef
                ",
                expect: Targets(vec![]),
            },
            Case {
                title: "override define",
                contents: "\
override define foo
not-good:

endef
                ",
                expect: Targets(vec![]),
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                Targets::new(case.contents.to_string(), PathBuf::from(""), None),
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }

    #[test]
    fn get_line_type_test() {
        struct Case {
            title: &'static str,
            line: &'static str,
            expect: LineType,
        }

        let cases = vec![
            Case {
                title: "empty line",
                line: "",
                expect: LineType::Normal,
            },
            Case {
                title: "override define",
                line: "override define",
                expect: LineType::DefineStart,
            },
            Case {
                title: "define",
                line: "define",
                expect: LineType::DefineStart,
            },
            Case {
                title: "endef",
                line: "endef",
                expect: LineType::DefineEnd,
            },
            Case {
                title: "define whitespace",
                line: "  define   foo",
                expect: LineType::DefineStart,
            },
            Case {
                title: "endef whitespace",
                line: "  endef  ",
                expect: LineType::DefineEnd,
            },
        ];

        for case in cases {
            assert_eq!(case.expect, get_line_type(case.line, None), "\nFailed: 🚨{:?}🚨\n", case.title,);
        }
    }

    #[test]
    fn get_line_type_with_recipe_prefix_test() {
        struct Case {
            title: &'static str,
            line: &'static str,
            recipe_prefix: Option<char>,
            expect: LineType,
        }

        let cases = vec![
            Case {
                title: "recipe line with > prefix",
                line: ">echo test",
                recipe_prefix: Some('>'),
                expect: LineType::Recipe,
            },
            Case {
                title: "recipe line with tab (default)",
                line: "\techo test",
                recipe_prefix: None,
                expect: LineType::Recipe,
            },
            Case {
                title: "normal line with > prefix set but line doesn't start with >",
                line: "target:",
                recipe_prefix: Some('>'),
                expect: LineType::Normal,
            },
            Case {
                title: "target line should not be recipe even with tab",
                line: "target:",
                recipe_prefix: None,
                expect: LineType::Normal,
            },
        ];

        for case in cases {
            assert_eq!(case.expect, get_line_type(case.line, case.recipe_prefix), "\nFailed: 🚨{:?}🚨\n", case.title,);
        }
    }

    #[test]
    fn content_to_targets_with_recipe_prefix_test() {
        struct Case {
            title: &'static str,
            contents: &'static str,
            recipe_prefix: Option<char>,
            expect: Targets,
        }
        let cases = vec![Case {
            title: "makefile with .RECIPEPREFIX = >",
            contents: "\
.RECIPEPREFIX = >

.PHONY: test

test:
>echo \"This uses > instead of tab\"
>echo \"Another line\"

build:
>cargo build",
            recipe_prefix: Some('>'),
            expect: Targets(vec![
                command::CommandWithPreview::new(
                    runner_type::RunnerType::Make,
                    "test".to_string(),
                    PathBuf::from(""),
                    5,
                ),
                command::CommandWithPreview::new(
                    runner_type::RunnerType::Make,
                    "build".to_string(),
                    PathBuf::from(""),
                    9,
                ),
            ]),
        }];

        for case in cases {
            assert_eq!(
                case.expect,
                Targets::new(case.contents.to_string(), PathBuf::from(""), case.recipe_prefix),
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
            Case {
                title: "%:",
                contents: "%:",
                expect: Some("%"),
            },
            Case {
                title: "a:",
                contents: "a:",
                expect: Some("a"),
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
