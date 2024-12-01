use super::{command, pnpm, runner_type};
use crate::file::path_to_content;
use anyhow::Result;
use codespan::Files;
use json_spanned_value::{self as jsv, spanned};
use std::{fs, path::PathBuf};

const METADATA_FILE_NAME: &str = "package.json";
const METADATA_COMMAND_KEY: &str = "scripts";
const PNPM_LOCKFILE_NAME: &str = "pnpm-lock.yaml"; // TODO: pnpm.rsに移動したい

// こいつをrunner_typeのvariantに変更すべきでは？
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum JsPackageManager {
    JsPnpm(pnpm::Pnpm),
    JsYarn,
}

impl JsPackageManager {
    pub fn command_to_run(&self, command: &command::Command) -> Result<String> {
        match self {
            JsPackageManager::JsPnpm(pnpm) => pnpm.command_to_run(command),
            JsPackageManager::JsYarn => todo!(),
        }
    }

    pub fn to_commands(&self) -> Vec<command::Command> {
        match self {
            JsPackageManager::JsPnpm(pnpm) => pnpm.to_commands(),
            JsPackageManager::JsYarn => todo!(),
        }
    }

    pub fn execute(&self, command: &command::Command) -> Result<()> {
        match self {
            JsPackageManager::JsPnpm(pnpm) => pnpm.execute(command),
            JsPackageManager::JsYarn => todo!(),
        }
    }

    pub fn path(&self) -> PathBuf {
        match self {
            JsPackageManager::JsPnpm(pnpm) => pnpm.path.clone(),
            JsPackageManager::JsYarn => todo!(),
        }
    }

    fn new(current_dir: PathBuf, file_names: Vec<String>) -> Option<Self> {
        for file_name in file_names {
            // TODO: ここのロジックもpnpm側に持っていったほうがよさそう
            if file_name == PNPM_LOCKFILE_NAME {
                let commands =
                    match path_to_content::path_to_content(PathBuf::from(METADATA_FILE_NAME)) {
                        Ok(c) => parse_package_json(&c, runner_type::RunnerType::Pnpm),
                        Err(_) => return None,
                    };
                let pnpm = pnpm::Pnpm::new(current_dir.join(METADATA_FILE_NAME), commands);

                return Some(JsPackageManager::JsPnpm(pnpm));
            }
        }
        None
    }

    fn to_runner_type(&self) -> runner_type::RunnerType {
        match self {
            JsPackageManager::JsPnpm(_) => runner_type::RunnerType::Pnpm,
            JsPackageManager::JsYarn => todo!(),
        }
    }

    fn command_name_to_args(&self, command_name: &str) -> String {
        match self {
            // これここに実装すべきなのか...?
            // TODO: pnpm.rsに実装したい
            JsPackageManager::JsPnpm(_) => ("run".to_string() + command_name).to_string(),
            JsPackageManager::JsYarn => todo!(),
        }
    }
}

// TODO: runnerではなくPnpmを返すのがただしそう（そもそもPnpmではなくJsPackageManagerを返すべきという説も）
pub fn get_js_package_manager_runner(current_dir: PathBuf) -> Option<JsPackageManager> {
    let entries = fs::read_dir(current_dir.clone()).unwrap();
    let file_names = entries
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();

    JsPackageManager::new(current_dir, file_names)
}

// TODO: make this function method
fn parse_package_json(
    content: &str,
    runner_type: runner_type::RunnerType,
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
                    runner_type: runner_type.clone(),
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
                parse_package_json(case.file_content, runner_type::RunnerType::Pnpm),
                "\nfailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }
}
