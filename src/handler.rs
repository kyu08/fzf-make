use regex::Regex;
use skim::prelude::Skim;
use skim::prelude::*;
use std::fs::{File, OpenOptions, ReadDir};
use std::io::Read;
use std::path::Path;
use std::{io::Cursor, process, sync::Arc};

pub fn run() {
    let file_paths = match get_makefile_file_names() {
        Err(e) => {
            print_error(e.to_string());
            process::exit(1)
        }
        Ok(f) => f,
    };

    let preview_command = get_preview_command(file_paths.clone());
    let options = get_options(&preview_command);
    let items = get_items(file_paths.clone());
    run_fuzzy_finder(options, items);
}

fn run_fuzzy_finder(options: SkimOptions, items: Option<Receiver<Arc<dyn SkimItem>>>) {
    if let output @ Some(_) = Skim::run_with(&options, items) {
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let selected_items = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);

        for item in selected_items.iter() {
            println!("make {}", item.output().to_string());
            process::Command::new("make")
                .stdin(process::Stdio::inherit())
                .arg(item.output().to_string())
                .spawn()
                .expect("Failed to execute process")
                .wait()
                .expect("Failed to execute process");
        }
    }
}

// get_makefile_file_names returns filenames of Makefile and the files named like *.mk
fn get_makefile_file_names() -> Result<Vec<String>, &'static str> {
    let mut file_names: Vec<String> = Vec::new();

    let makefile_name = match specify_makefile_name(".".to_string()) {
        None => return Err("makefile not found"),
        Some(f) => f,
    };
    match File::open(makefile_name.clone()).map_err(|_| "makefile not found") {
        Err(err) => return Err(err),
        Ok(_) => file_names.push(makefile_name.to_string()),
    }

    //   TODO: makefileでincludeしているファイルの名前を特定する
    let entries = match std::fs::read_dir(".") {
        Err(_) => return Err("fail to read directory"),
        Ok(entries) => entries,
    };
    for file_path in get_including_files(entries) {
        file_names.push(file_path);
    }

    Ok(file_names)
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

fn get_including_files(entries: ReadDir) -> Vec<String> {
    // TODO: いい感じにする
    let mut file_names: Vec<String> = Vec::new();
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

    file_names
}

fn get_preview_command(file_paths: Vec<String>) -> String {
    // result has format like `test.mk:2:echo-mk`
    // 1. param: Makefileのcontent, result: Vec<includeしているファイル名(.mk))>な関数をRust側で作る
    //  このときGNUmakefile, makefile and Makefile の順で探索する。はじめに見つかった1つのみを使う。(実際に試してみたところそのような挙動だった)
    // 1. include しているファイルのVecをつくる
    // 1. そのなかから *.mkを探す
    // それを以下の $files に格納する

    let preview_command = format!(
        r#"
    files="{}" \
    result=$(grep -rnE '^{}\s*:' $(echo $files)); \
    IFS=':' read -r filename lineno _ <<< $result; \
    bat --style=numbers --color=always --line-range $lineno: \
    --highlight-line $lineno $filename;"#,
        file_paths.join(" "),
        "{}",
    );

    preview_command
}

fn get_options(preview_command: &str) -> SkimOptions {
    SkimOptionsBuilder::default()
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap()
}

fn print_error(error_message: String) {
    println!("[ERR] {}", error_message);
}

fn get_items(file_paths: Vec<String>) -> Option<Receiver<Arc<dyn SkimItem>>> {
    let commands = match extract_command_from_makefiles(file_paths) {
        Err(e) => {
            print_error(e.to_string());
            process::exit(1)
        }
        Ok(s) => s,
    };
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(commands));

    Some(items)
}

fn extract_command_from_makefiles(file_paths: Vec<String>) -> Result<String, &'static str> {
    let contents = concat_file_contents(file_paths)?;
    let commands = contents_to_commands(contents)?;
    Ok(commands.join("\n"))
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
    use std::{fs, io::Write, str::FromStr};
    use uuid::Uuid;

    use super::*;

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
                specify_makefile_name(tmp_dir.to_string_lossy().to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }
    #[test]
    fn concat_file_contents_test() {
        struct Case {
            title: &'static str,
            file_contents: Vec<&'static str>,
            expect: Result<&'static str, &'static str>,
        }
        let cases = vec![
            Case {
                title: "two files",
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
            },
            Case {
                title: "single file",
                file_contents: vec![
                    "\
.PHONY: test-1
test-1:
    @cargo run",
                ],
                expect: Ok("\
.PHONY: test-1
test-1:
    @cargo run"),
            },
        ];

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
                concat_file_contents(in_file_names),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
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
                contents_to_commands(case.contents.to_string()),
                "\nFailed in the 🚨{:?}🚨",
                case.title,
            );
        }
    }

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
}
