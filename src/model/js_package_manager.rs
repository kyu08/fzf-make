use super::{command, pnpm, runner};
use crate::file::path_to_content;
use codespan::Files;
use json_spanned_value::{self as jsv, spanned};
use std::path::PathBuf;

pub fn get_js_package_manager(current_dir: PathBuf) -> runner::Runner {
    // TODO: lockfileの種類に応じて各JSパッケージマネージャの初期化コードを呼び出す
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
        Ok(c) => parse_package_json(c),
        Err(_) => vec![],
    }
}

fn parse_package_json(content: String) -> Vec<command::Command> {
    let mut files = Files::new();
    let file = files.add(JS_PACKAGE_METADATA_FILE_NAME, content.clone());
    let mut result = vec![];

    // TODO: これだと不正なJSONだったときにコケそうなのでresultをちゃんとハンドリングする
    let example: spanned::Object = jsv::from_str(content.as_str()).unwrap();
    for (k, v) in example {
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
                    runner_type: super::runner_type::RunnerType::Pnpm, // TODO: ここは動的に受け取る
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
