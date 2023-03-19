use regex::Regex;
use skim::prelude::*;

use std::{fs::File, io::Cursor, io::Read, process};

fn main() {
    let (options, items) = get_params();

    if let output @ Some(_) = Skim::run_with(&options, items) {
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let selected_items = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);
        for item in selected_items.iter() {
            let output = process::Command::new("make")
                .arg(item.output().to_string())
                .output();
            match output {
                Ok(output) => {
                    println!("\n");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Err(_) => print_error("fail to execute make command".to_string()),
            }
        }
    }
}

fn get_params<'a>() -> (SkimOptions<'a>, Option<Receiver<Arc<dyn SkimItem>>>) {
    // TODO: use cat when bat is unavailable
    let preview_command = "bat --style=numbers --color=always --highlight-line $(bat Makefile | grep -n {}: | sed -e 's/:.*//g') Makefile";
    // TODO: hide fzf window when fzf-make terminated
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap();
    let commands = match extract_command_from_makefile() {
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

/// Makefileからcommandを抽出
fn extract_command_from_makefile() -> Result<String, &'static str> {
    let mut file = match File::open("Makefile") {
        Ok(f) => f,
        Err(_) => return Err("Makefile not found"),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err("fail to read Makefile contents");
    }

    let commands = extract_commands(contents);

    if !commands.is_empty() {
        Ok(commands.join("\n"))
    } else {
        Err("no make command found")
    }
}

fn print_error(error_message: String) {
    println!("[ERR] {}", error_message);
}

fn extract_commands(contents: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for line in contents.lines() {
        if let Some(c) = extract_command(line.to_string()) {
            result.push(c);
        }
    }

    result
}

fn extract_command(line: String) -> Option<String> {
    let regex = Regex::new(r"^[^\.PHONY]+:$").unwrap();
    match regex.find(line.as_str()) {
        // TODO: もう少しいい書き方がありそう...
        Some(m) => Some(
            m.as_str()
                .to_string()
                .split_once(':')
                .unwrap()
                .0
                .to_string(),
        ),
        None => None,
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn extract_commands_test() {
        struct Case {
            contents: String,
            expect: Vec<String>,
        }
        let contents1 = "\
.PHONY: run build check

run:
		@cargo run

build:
		@cargo build

check:
		@cargo check

echo:
	@echo good";
        let contents2 = "\
.PHONY: close build

# https://example.com
clone:
		@git clone https://example.com

build:
		@cargo build";

        let cases = vec![
            Case {
                contents: contents1.to_string(),
                expect: vec![
                    "run".to_string(),
                    "build".to_string(),
                    "check".to_string(),
                    "echo".to_string(),
                ],
            },
            Case {
                contents: String::from_str(contents2).unwrap(),
                expect: vec!["clone".to_string(), "build".to_string()],
            },
        ];

        for case in cases {
            assert_eq!(case.expect, extract_commands(case.contents));
        }
    }

    #[test]
    fn extract_command_test() {
        struct Case {
            contents: String,
            expect: Option<String>,
        }
        let cases = vec![
            Case {
                contents: "echo:".to_string(),
                expect: Some("echo".to_string()),
            },
            Case {
                contents: "echo".to_string(),
                expect: None,
            },
            Case {
                contents: ".PHONY:".to_string(),
                expect: None,
            },
            Case {
                contents: "https://example.com".to_string(),
                expect: None,
            },
        ];

        for case in cases {
            assert_eq!(case.expect, extract_command(case.contents));
        }
    }
}
