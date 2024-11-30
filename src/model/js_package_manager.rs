use super::{command, pnpm, runner, runner_type};
use crate::file::path_to_content;
use codespan::Files;
use json_spanned_value::{self as jsv, spanned};
use std::{fs, path::PathBuf};

const METADATA_FILE_NAME: &str = "package.json";
const METADATA_COMMAND_KEY: &str = "scripts";
const PNPM_LOCKFILE_NAME: &str = "pnpm-lock.yaml";

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
enum JsPackageManager {
    Pnpm,
    Yarn,
}

impl JsPackageManager {
    fn new(file_names: Vec<String>) -> Option<Self> {
        for file_name in file_names {
            if file_name == PNPM_LOCKFILE_NAME {
                return Some(JsPackageManager::Pnpm);
            }
        }
        None
    }

    fn to_runner_type(&self) -> runner_type::RunnerType {
        match self {
            JsPackageManager::Pnpm => runner_type::RunnerType::Pnpm,
            JsPackageManager::Yarn => todo!(),
        }
    }

    fn to_runner(
        &self,
        command_file_path: PathBuf,
        commands: Vec<command::Command>,
    ) -> runner::Runner {
        match self {
            JsPackageManager::Pnpm => {
                runner::Runner::PnpmCommand(pnpm::Pnpm::new(command_file_path, commands))
            }
            JsPackageManager::Yarn => todo!(),
        }
    }
}

pub fn get_js_package_manager_runner(current_dir: PathBuf) -> Option<runner::Runner> {
    let entries = fs::read_dir(current_dir.clone()).unwrap();
    let file_names = entries
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();

    match JsPackageManager::new(file_names) {
        Some(js_package_manager) => {
            let commands = match path_to_content::path_to_content(PathBuf::from(METADATA_FILE_NAME))
            {
                Ok(c) => parse_package_json(&c, &js_package_manager),
                Err(_) => return None,
            };
            Some(js_package_manager.to_runner(current_dir.join(METADATA_FILE_NAME), commands))
        }
        None => None,
    }
}

fn parse_package_json(
    content: &str,
    js_package_manager: &JsPackageManager,
) -> Vec<command::Command> {
    let mut files = Files::new();
    let file = files.add(METADATA_FILE_NAME, content);
    let json_object: spanned::Object = match jsv::from_str(content) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut result = vec![];
    for (k, v) in json_object {
        if k.as_str() != METADATA_COMMAND_KEY {
            continue;
        }

        // object is the content of "scripts" key
        if let Some(object) = v.as_object() {
            for (k, _) in object {
                let args = k.to_string();
                let line_number =
                    files.line_index(file, k.start() as u32).number().to_usize() as u32;

                result.push(command::Command {
                    runner_type: js_package_manager.to_runner_type(),
                    args,
                    file_name: PathBuf::from(METADATA_FILE_NAME),
                    line_number,
                });
            }
        };
        break;
    }

    result
}

#[cfg(test)]
mod test {
    use crate::model::runner_type;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_package_json() {
        struct Case {
            title: &'static str,
            file_content: &'static str,
            expected: Vec<command::Command>,
        }

        let cases = vec![
            Case {
                title: "valid json can be parsed successfully",
                file_content: r#"{
  "name": "project",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "build": "echo build",
    "start": "echo start",
    "test": "echo test"
  },
  "devDependencies": {
    "@babel/cli": "7.12.10"
  },
  "dependencies": {
    "firebase": "^8.6.8"
  }
}
                    "#,
                expected: vec![
                    command::Command {
                        runner_type: runner_type::RunnerType::Pnpm,
                        args: "build".to_string(),
                        file_name: "package.json".into(),
                        line_number: 6,
                    },
                    command::Command {
                        runner_type: runner_type::RunnerType::Pnpm,
                        args: "start".to_string(),
                        file_name: "package.json".into(),
                        line_number: 7,
                    },
                    command::Command {
                        runner_type: runner_type::RunnerType::Pnpm,
                        args: "test".to_string(),
                        file_name: "package.json".into(),
                        line_number: 8,
                    },
                ],
            },
            Case {
                title: "empty vec(empty string)",
                file_content: "",
                expected: vec![],
            },
            Case {
                title: "empty vec(invalid json)",
                file_content: "not a json format",
                expected: vec![],
            },
        ];

        for case in cases {
            assert_eq!(
                case.expected,
                parse_package_json(case.file_content, &JsPackageManager::Pnpm),
                "\nfailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }

    #[test]
    fn test_new() {
        struct Case {
            title: &'static str,
            file_names: Vec<String>,
            expected: Option<JsPackageManager>,
        }

        let cases = vec![
            Case {
                title: "pnpm",
                file_names: vec![
                    ".gitignore".to_string(),
                    "Makefile".to_string(),
                    "pnpm-lock.yaml".to_string(),
                ],
                expected: Some(JsPackageManager::Pnpm),
            },
            Case {
                title: "no js package manager found",
                file_names: vec![
                    ".gitignore".to_string(),
                    "Makefile".to_string(),
                    "cargo.toml".to_string(),
                ],
                expected: None,
            },
        ];

        for case in cases {
            let result = JsPackageManager::new(case.file_names);
            assert_eq!(case.expected, result, "\nfailed: 🚨{:?}🚨\n", case.title)
        }
    }
}
