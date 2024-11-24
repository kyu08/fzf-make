use super::{command, pnpm, runner};
use crate::file::path_to_content;
use std::path::PathBuf;

pub fn get_js_package_manager(current_dir: PathBuf) -> runner::Runner {
    // TODO: lockfileの種類に応じて各JSパッケージマネージャの初期化コードを呼び出す
    get_js_commands(current_dir.clone());
    runner::Runner::PnpmCommand(pnpm::Pnpm::new(current_dir).unwrap())
}

const JS_PACKAGE_METADATA_FILE_NAME: &str = "package.json";

fn get_js_commands(_current_dir: PathBuf) -> Vec<command::Command> {
    // TODO: lockfileの種類に応じて各JSパッケージマネージャの初期化コードを呼び出す
    match path_to_content::path_to_content(PathBuf::from(JS_PACKAGE_METADATA_FILE_NAME)) {
        Ok(c) => parse_package_json(c),
        Err(_) => vec![],
    }
}

fn parse_package_json(content: String) -> Vec<command::Command> {
    panic!("{}", content);
    // serdeでparseしてもいいが、行番号をとれるのかどうかがわからん。
    // 無理なら自力でparseするしかなさそう。
    // これが使えるかもしれない？https://github.com/MaulingMonkey/json-spanned-value/blob/db682d75b30438866b6e9d02f1187918eb329e1b/examples/demo.rs
    vec![]
}
