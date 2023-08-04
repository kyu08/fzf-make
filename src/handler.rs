use crate::file::file;
use crate::parser;
use skim::prelude::{Receiver, Skim, SkimItem, SkimItemReader, SkimOptions, SkimOptionsBuilder};
use std::{io::Cursor, process, sync::Arc};

pub fn run() {
    let file_paths = match file::get_makefile_file_names() {
        Err(e) => {
            print_error(e.to_string());
            process::exit(1)
        }
        Ok(f) => f,
    };

    let preview_command = get_preview_command(file_paths.clone());
    let options = get_options(&preview_command);
    let items = get_skimitem(file_paths.clone());

    run_fuzzy_finder(options, items);
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

fn get_options(preview_command: &str) -> SkimOptions {
    SkimOptionsBuilder::default()
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap()
}

fn get_skimitem(file_paths: Vec<String>) -> Option<Receiver<Arc<dyn SkimItem>>> {
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
    let contents = file::concat_file_contents(file_paths)?;
    let commands = parser::target::contents_to_commands(contents)?;
    Ok(commands.join("\n"))
}

fn print_error(error_message: String) {
    println!("[ERR] {}", error_message);
}
