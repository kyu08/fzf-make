use super::{command, pnpm, runner, runner_type};
use crate::file::path_to_content;
use codespan::Files;
use json_spanned_value::{self as jsv, spanned};
use std::path::PathBuf;

pub fn get_js_package_manager(current_dir: PathBuf) -> runner::Runner {
    todo!("lockfileの種類に応じて各JSパッケージマネージャの初期化コードを呼び出す");
    let commands = get_js_commands(current_dir.clone());
    runner::Runner::PnpmCommand(pnpm::Pnpm::new(
        current_dir.join(JS_PACKAGE_METADATA_FILE_NAME),
        commands,
    ))
}

const JS_PACKAGE_METADATA_FILE_NAME: &str = "package.json";
const JS_PACKAGE_METADATA_COMMAND_KEY: &str = "scripts";

fn get_js_commands(_current_dir: PathBuf) -> Vec<command::Command> {
    match path_to_content::path_to_content(PathBuf::from(JS_PACKAGE_METADATA_FILE_NAME)) {
        Ok(c) => parse_package_json(&c),
        Err(_) => vec![],
    }
}

fn parse_package_json(
    content: &str,
    runner_type: runner_type::RunnerType,
) -> Vec<command::Command> {
    // TODO: check runner_type is one of JS package manager
    let mut files = Files::new();
    let file = files.add(JS_PACKAGE_METADATA_FILE_NAME, content);
    let json_object: spanned::Object = match jsv::from_str(content) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut result = vec![];
    for (k, v) in json_object {
        if k.as_str() != JS_PACKAGE_METADATA_COMMAND_KEY {
            continue;
        }

        // object is the content of "scripts" key
        if let Some(object) = v.as_object() {
            for (k, _) in object {
                let name = k.to_string();
                let line_number =
                    files.line_index(file, k.start() as u32).number().to_usize() as u32;

                result.push(command::Command {
                    runner_type: runner_type.clone(),
                    name,
                    file_name: PathBuf::from(JS_PACKAGE_METADATA_FILE_NAME),
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
                        name: "build".to_string(),
                        file_name: "package.json".into(),
                        line_number: 6,
                    },
                    command::Command {
                        runner_type: runner_type::RunnerType::Pnpm,
                        name: "start".to_string(),
                        file_name: "package.json".into(),
                        line_number: 7,
                    },
                    command::Command {
                        runner_type: runner_type::RunnerType::Pnpm,
                        name: "test".to_string(),
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
