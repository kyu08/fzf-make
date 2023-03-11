use skim::prelude::*;

use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::process;

// TODO: refactor
// TODO: 命名いい感じにする
fn main() {
    run()
}

fn run() {
    let (options, items) = get_params();
    match Skim::run_with(&options, Some(items)) {
        output @ Some(_) => {
            if output.as_ref().unwrap().is_abort {
                process::exit(0)
            }

            let selected_items = output
                .map(|out| out.selected_items)
                .unwrap_or_else(Vec::new);
            for item in selected_items.iter() {
                let output = process::Command::new("make")
                    .arg(item.output().to_string())
                    .output()
                    .expect("panic");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        None => {}
    }
}

fn get_params<'a>() -> (SkimOptions<'a>, Receiver<Arc<dyn SkimItem>>) {
    let preview_command = "bat --style=numbers --color=always --highlight-line $(bat Makefile | grep -n {}: | sed -e 's/:.*//g') Makefile";
    // TODO: hide fzf window when fzf-make terminated
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap();
    let commands = match extract_command_from_makefile() {
        Ok(s) => s,
        Err(e) => {
            println!("[ERR] {}", e);
            process::exit(1)
        }
    };
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(commands));

    (options, items)
}

/// Makefileからcommandを抽出
fn extract_command_from_makefile() -> Result<String, &'static str> {
    let mut file = match File::open("Makefile") {
        Ok(f) => f,
        Err(_) => return Err("Makefile not found"),
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return Err("fail to read Makefile contents");
    }

    Ok(extract_command(contents))
}

fn extract_command(contents: String) -> String {
    let mut result = Vec::new();
    for line in contents.lines() {
        if let Some(t) = line.split_once(':') {
            if t.0.contains(".PHONY") {
                continue;
            }
            result.push(t.0);
        }
    }

    result.join("\n")
}
