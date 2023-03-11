use skim::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::process::Command;

fn main() {
    let preview_command = "bat --style=numbers --color=always --highlight-line $(bat Makefile | grep -n {}: | sed -e 's/:.*//g') Makefile";
    // TODO: hide fzf window when fzf-make terminated
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap();
    let item_reader = SkimItemReader::default();
    let commands = match extract_command() {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };
    let items = item_reader.of_bufread(Cursor::new(commands));

    match Skim::run_with(&options, Some(items)) {
        output @ Some(_) => {
            if output.as_ref().unwrap().is_abort {
                return;
            }

            let selected_items = output
                .map(|out| out.selected_items)
                .unwrap_or_else(Vec::new);
            for item in selected_items.iter() {
                let output = Command::new("make")
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

/// Makefileからcommandを抽出
fn extract_command() -> Result<String, Box<dyn Error>> {
    let mut f = File::open("Makefile")?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let mut result = String::new();
    for line in contents.lines() {
        if let Some(t) = line.split_once(':') {
            if t.0.contains(".PHONY") {
                continue;
            }
            if !result.is_empty() {
                result += "\n";
            }
            result += t.0;
        }
    }

    Ok(result)
}
