use skim::prelude::{Receiver, Skim, SkimItem, SkimItemReader, SkimOptions, SkimOptionsBuilder};
use std::{io::Cursor, process, sync::Arc};

use crate::models::makefile::Makefile;

pub fn run(makefile: Makefile) -> String {
    let preview_command = get_preview_command(makefile.to_include_files_string());
    let options = get_skim_options(&preview_command);
    let item = get_skim_item(makefile.to_targets_string());

    if let output @ Some(_) = Skim::run_with(&options, item) {
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let selected_items = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);

        selected_items.first().unwrap().output().to_string()
    } else {
        println!("[ERR] Fail to get selected target.");
        process::exit(1)
    }
}

pub fn get_preview_command(mut file_paths: Vec<String>) -> String {
    // workaround for https://stackoverflow.com/questions/15432156/display-filename-before-matching-line
    // For more information, see https://github.com/kyu08/fzf-make/issues/53#issuecomment-1684872018
    if file_paths.len() == 1 {
        file_paths.push(String::from("/dev/null"));
    }
    // Make preview command selectable via config file(bat/cat) https://github.com/kyu08/fzf-make/issues/67
    // MEMO: result has format like `test.mk:2:echo-mk`
    let preview_command = format!(
        r#"
    files="{}" \
    result=$(grep -rnE '^\s*{}\s*:' $(echo $files)); \
    IFS=':' read -r filename lineno _ <<< $result; \
    bat --style=numbers --color=always --line-range $lineno: \
    --highlight-line $lineno $filename;"#,
        file_paths.join(" "),
        "{}",
    );

    // MEMO: Keep it for using cat
    // let preview_command = format!(
    //     r#"
    // files="{}" \
    // result=$(grep -rnE '^{}\s*:' $(echo $files)); \
    // IFS=':' read -r filename lineno _ <<< $result; \
    // total_lines=$(wc -l < $filename); \
    // cat -n $filename | tail -n $((total_lines - lineno + 1)) $filename"#,
    //     // cat -n $filename | tail -n$lineno;"#,
    //     // cat -n $filename | tail -n$lineno;"#,
    //     file_paths.join(" "),
    //     "{}",
    // );

    preview_command
}

fn get_skim_options(preview_command: &str) -> SkimOptions {
    SkimOptionsBuilder::default()
        .preview(Some(preview_command))
        // Make configurable via config file https://github.com/kyu08/fzf-make/issues/67
        .preview_window(Some("up:50%"))
        // Make configurable via config file https://github.com/kyu08/fzf-make/issues/67
        // .reverse(true)
        .build()
        .unwrap()
}

fn get_skim_item(targets: Vec<String>) -> Option<Receiver<Arc<dyn SkimItem>>> {
    let targets = targets.join("\n");
    let items = SkimItemReader::default().of_bufread(Cursor::new(targets));

    Some(items)
}
