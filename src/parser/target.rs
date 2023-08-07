use regex::Regex;

pub type Targets = Vec<String>;

pub fn content_to_targets(content: String) -> Targets {
    let mut result: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(t) = line_to_target(line.to_string()) {
            result.push(t);
        }
    }

    result
}

fn line_to_target(line: String) -> Option<String> {
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
    use super::*;

    #[test]
    fn content_to_targets_test() {
        struct Case {
            title: &'static str,
            contents: &'static str,
            expect: Vec<String>, // NOTE: order of elements of `expect` order should be same as vec function returns
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
                expect: vec![
                    "run".to_string(),
                    "build".to_string(),
                    "check".to_string(),
                    "test".to_string(),
                    "echo".to_string(),
                ],
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
                expect: vec!["clone".to_string(), "build".to_string()],
            },
            Case {
                title: "invalid format",
                contents: "echo hello",
                expect: vec![],
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect,
                content_to_targets(case.contents.to_string()),
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
