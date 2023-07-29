use regex::Regex;
use skim::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::{io::Cursor, process, sync::Arc};

// FIXME rename module

pub fn print_error(error_message: String) {
    println!("[ERR] {}", error_message);
}

// TODO: Maybe skim related could be combined into one module.
pub fn get_params<'a>() -> (SkimOptions<'a>, Option<Receiver<Arc<dyn SkimItem>>>) {
    // result has format like `test.mk:2:echo-mk`
    // TODO: `files` が Makefileと *.mk を指すようにする(find . hogeとかやる必要ありそう)
    let preview_command = r#"
    files="Makefile test.mk" \
    result=$(grep -rnE '^{}\s*:' $(echo $files)); \
    IFS=':' read -r filename lineno _ <<< $result; \
    bat --style=numbers --color=always --line-range $lineno: \
    --highlight-line $lineno $filename;"#;
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

fn extract_command_from_makefile() -> Result<String, &'static str> {
    let files = get_makefile_file_names()?;
    let contents = concat_file_contents(files)?;
    let commands = contents_to_commands(contents)?;
    Ok(commands.join("\n"))
}

// get_makefile_file_names returns filenames of Makefile and the files named like *.mk
fn get_makefile_file_names() -> Result<Vec<String>, &'static str> {
    let mut file_names: Vec<String> = Vec::new();

    let makefile = "Makefile";
    match File::open(makefile).map_err(|_| "Makefile not found") {
        Err(err) => return Err(err),
        Ok(_) => file_names.push(makefile.to_string()),
    }

    // add *.mk to `makefiles` if exist
    let entries = match std::fs::read_dir(".") {
        Err(_) => return Err("fail to read directory"),
        Ok(entries) => entries,
    };

    for entry in entries {
        let entry = match entry {
            Err(_) => continue,
            Ok(e) => e,
        };

        let path = entry.path();
        let ext = match path.extension() {
            None => continue,
            Some(ext) => ext,
        };
        if ext != "mk" {
            continue;
        }

        let file_name = match entry.file_name().into_string() {
            // c.f. https://zenn.dev/suzuki_hoge/books/2023-03-rust-strings-8868f207b3ed18/viewer/4-os-string-and-os-str
            Err(entry) => panic!("file name is not utf-8: {:?}", entry),
            Ok(f) => f,
        };
        file_names.push(file_name);
    }

    Ok(file_names)
}

fn concat_file_contents(file_paths: Vec<String>) -> Result<String, &'static str> {
    let mut contents = String::new();
    for path in file_paths {
        let mut content = String::new();
        let mut file = match OpenOptions::new().read(true).open(path) {
            Err(_) => return Err("fail to open file"),
            Ok(f) => f,
        };

        match file.read_to_string(&mut content) {
            Err(e) => {
                print!("fail to read file: {:?}", e);
                return Err("fail to read file");
            }
            Ok(_) => {
                if !contents.is_empty() {
                    contents += "\n";
                }

                contents += &content;
            }
        }
    }
    Ok(contents)
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
    use std::{io::Write, str::FromStr};
    use uuid::Uuid;

    use super::*;

    #[test]
    fn concat_file_contents_test() {
        struct Case {
            file_contents: Vec<&'static str>,
            expect: Result<&'static str, &'static str>,
        }
        let cases = vec![Case {
            file_contents: vec![
                "\
.PHONY: test-1
test-1:
    @cargo run",
                "\
.PHONY: test-2
test-2:
    @cargo run",
            ],
            expect: Ok("\
.PHONY: test-1
test-1:
    @cargo run
.PHONY: test-2
test-2:
    @cargo run"),
        }];

        for case in cases {
            let in_file_names: Vec<String> = case
                .file_contents
                .iter()
                .map(|content| {
                    let random_file_name = Uuid::new_v4().to_string();
                    test_file_from_content(random_file_name, content)
                })
                .collect();

            assert_eq!(
                case.expect.map(|e| e.to_string()),
                concat_file_contents(in_file_names)
            );
        }
    }

    fn test_file_from_content(file_name: String, content: &'static str) -> String {
        let tmp_dir = std::env::temp_dir();
        let file_name = file_name + ".mk";
        let file_path = tmp_dir.join(&file_name);

        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open(&file_path)
        {
            Err(err) => panic!("fail to create file: {:?}", err),
            Ok(file) => file,
        };

        match file.write_all(content.as_bytes()) {
            Err(e) => {
                print!("fail to write file: {:?}", e);
                process::exit(1);
            }
            Ok(_) => {}
        }

        file_path.to_path_buf().to_str().unwrap().to_string()
    }

    #[test]
    fn contents_to_commands_test() {
        struct Case {
            contents: &'static str,
            expect: Result<Vec<&'static str>, &'static str>, // NOTE: order of elements of `expect` order should be same as vec function returns
        }
        let cases = vec![
            Case {
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
            let expect = case.expect.map(|x| {
                x.iter()
                    .map(|y| String::from_str(y).unwrap())
                    .collect::<Vec<String>>()
            });
            assert_eq!(expect, contents_to_commands(case.contents.to_string()));
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
