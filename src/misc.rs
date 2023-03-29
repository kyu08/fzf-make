use regex::Regex;
use skim::prelude::*;
use std::{
    fs::File,
    io::{Cursor, Read},
    process,
    sync::Arc,
};

// FIXME rename module

pub fn print_error(error_message: String) {
    println!("[ERR] {}", error_message);
}

// TODO: Maybe skim related could be combined into one module.
pub fn get_params<'a>() -> (SkimOptions<'a>, Option<Receiver<Arc<dyn SkimItem>>>) {
    let preview_command = r"line=$(bat Makefile | grep -nE '^{}\s*:' | sed -e 's/:.*//g'); bat --style=numbers --color=always --line-range $line: --highlight-line $line Makefile";
    let options = SkimOptionsBuilder::default()
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap();
    let commands = match extract_command_from_makefile() {
        // TODO: use some method
        Ok(s) => s,
        Err(e) => {
            print_error(e.to_string());
            process::exit(1)
        }
    };
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(commands));

    (options, Some(items))
}

pub fn extract_command_from_makefile() -> Result<String, &'static str> {
    let mut file = read_makefile()?;
    let contents = read_file_contents(&mut file)?;
    let commands = contents_to_commands(contents)?;
    Ok(commands.join("\n"))
}

fn read_makefile() -> Result<File, &'static str> {
    File::open("Makefile").map_err(|_| "Makefile not found")
}

fn read_file_contents(file: &mut File) -> Result<String, &'static str> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map(|_| contents)
        .map_err(|_| "fail to read Makefile contents")
}

fn contents_to_commands(contents: String) -> Result<Vec<String>, &'static str> {
    let mut result: Vec<String> = Vec::new();
    for line in contents.lines() {
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
    let regex = Regex::new(r"^[^.#\s].+:$").unwrap();
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
    fn contents_to_commands_test() {
        struct Case {
            contents: &'static str,
            expect: Result<Vec<&'static str>, &'static str>, // NOTE: order of elements of `expect` order should be same as vec function returns
        }
        let cases = vec![
            Case {
                contents: "\
    .PHONY: run build check

    run:
    		@cargo run

    build:
    		@cargo build

    check:
    		@cargo check

    echo:
    	@echo good",
                expect: Ok(vec!["run", "build", "check", "echo"]),
            },
            Case {
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
                contents: "echo hello",
                expect: Err("make command not found"),
            },
        ];

        for case in cases {
            let a = case.expect.map(|x| {
                x.iter()
                    .map(|y| String::from_str(y).unwrap())
                    .collect::<Vec<String>>()
            });
            assert_eq!(a, contents_to_commands(case.contents.to_string()));
        }
    }

    #[test]
    fn extract_command_test() {
        struct Case {
            contents: &'static str,
            expect: Option<&'static str>,
        }
        let cases = vec![
            Case {
                contents: "echo:",
                expect: Some("echo"),
            },
            Case {
                contents: "main.o:",
                expect: Some("main.o"),
            },
            Case {
                contents: "test::",
                expect: Some("test"),
            },
            Case {
                contents: "test ::",
                expect: Some("test"),
            },
            Case {
                contents: "echo",
                expect: None,
            },
            Case {
                contents: "		@git clone https://example.com",
                expect: None,
            },
            Case {
                contents: ".PHONY:",
                expect: None,
            },
            Case {
                contents: ".DEFAULT:",
                expect: None,
            },
            Case {
                contents: "https://example.com",
                expect: None,
            },
            Case {
                contents: "# run:",
                expect: None,
            },
            Case {
                contents: " # run:",
                expect: None,
            },
        ];

        for case in cases {
            assert_eq!(
                case.expect.map(|e| e.to_string()),
                line_to_command(case.contents.to_string())
            );
        }
    }
}
