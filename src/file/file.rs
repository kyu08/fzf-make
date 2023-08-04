// module for file manipulation
use std::{fs::OpenOptions, io::Read, path::Path};

use crate::parser::{self, makefile};

// get_makefile_file_names returns filenames of Makefile and the files included by Makefile
pub fn create_makefile() -> Result<makefile::Makefile, &'static str> {
    let Some(makefile_name) = specify_makefile_name(".".to_string()) else { return Err("makefile not found") };

    Ok(parser::makefile::Makefile::new(
        Path::new(&makefile_name).to_path_buf(),
    ))
}

pub fn concat_file_contents(file_paths: Vec<String>) -> Result<String, &'static str> {
    let mut contents = String::new();
    for path in file_paths {
        let mut content = String::new();
        // TODO: commonize convert file path to file content
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

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{self, File};
    use std::{io::Write, process};
    use uuid::Uuid;

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
}
