use skim::prelude::{Receiver, Skim, SkimItem, SkimItemReader, SkimOptions, SkimOptionsBuilder};
use std::{io::Cursor, process, sync::Arc};

use crate::parser::makefile::Makefile;

pub fn run(makefile: Makefile) {
    let preview_command = get_preview_command(makefile.to_include_path_string());
    let options = get_skimoptions(&preview_command);
    let items = get_skimitem(makefile.to_target_string());

    if let output @ Some(_) = Skim::run_with(&options, items) {
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let selected_items = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);

        for item in selected_items.iter() {
            println!("executing `make {}`...", item.output().to_string());
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

fn get_preview_command(file_paths: Vec<String>) -> String {
    // MEMO: result has format like `test.mk:2:echo-mk`
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

fn get_skimoptions(preview_command: &str) -> SkimOptions {
    SkimOptionsBuilder::default()
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap()
}

fn get_skimitem(targets: Vec<String>) -> Option<Receiver<Arc<dyn SkimItem>>> {
    let targets = targets.join("\n");
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(targets));

    Some(items)
}
