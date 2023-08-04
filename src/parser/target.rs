use regex::Regex;

pub type Targets = Vec<String>;

pub fn content_to_commands(content: String) -> Result<Targets, &'static str> {
    let mut result: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(c) = line_to_command(line.to_string()) {
            result.push(c);
        }
    }

    if !result.is_empty() {
        Ok(result)
    } else {
        Err("make command not found")
    }
}

fn line_to_command(line: String) -> Option<String> {
    let regex = Regex::new(r"^[^.#\s\t].+:.*$").unwrap();
    regex.find(line.as_str()).and_then(|m| {
        Some(
            m.as_str()
                .to_string()
                .split_once(':')
                .unwrap()
                .0
                .trim()
                .to_string(),
        )
    })
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn extract_command_test() {
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
        ];

        for case in cases {
            assert_eq!(
                case.expect.map(|e| e.to_string()),
                line_to_command(case.contents.to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }

    #[test]
    fn content_to_commands_test() {
        struct Case {
            title: &'static str,
            contents: &'static str,
            expect: Result<Vec<&'static str>, &'static str>, // NOTE: order of elements of `expect` order should be same as vec function returns
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
                expect: Ok(vec!["run", "build", "check", "test", "echo"]),
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
                expect: Ok(vec!["clone", "build"]),
            },
            Case {
                title: "invalid format",
                contents: "echo hello",
                expect: Err("make command not found"),
            },
        ];

        for case in cases {
            let expect = case.expect.map(|x| {
                x.iter()
                    .map(|y| String::from_str(y).unwrap())
                    .collect::<Vec<String>>()
            });
            assert_eq!(
                expect,
                content_to_commands(case.contents.to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }
}
