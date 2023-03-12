use skim::prelude::*;

use std::{fs::File, io::Cursor, io::Read, process};

// TODO: add README
fn main() {
    let (options, items) = get_params();

    if let output @ Some(_) = Skim::run_with(&options, items) {
        // TODO: as_refちゃんと理解する
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let selected_items = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);
        for item in selected_items.iter() {
            let output = process::Command::new("make")
                .arg(item.output().to_string())
                .output();
            match output {
                Ok(output) => {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Err(_) => println!("[ERR] fail to execute make command"),
            }
        }
    }
}

fn get_params<'a>() -> (SkimOptions<'a>, Option<Receiver<Arc<dyn SkimItem>>>) {
    // TODO: batがなければcatを使う
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
            // TODO: ログの共通化
            println!("[ERR] {}", e);
            process::exit(1)
        }
    };
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(commands));

    (options, Some(items))
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

    let commands = extract_command(contents);

    if !commands.is_empty() {
        Ok(commands.join("\n"))
    } else {
        Err("no make command found")
    }
}

fn extract_command(contents: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for line in contents.lines() {
        if let Some(t) = line.split_once(':') {
            if t.0.contains(".PHONY") {
                continue;
            }
            result.push(t.0.to_string());
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn extract_command_test() {
        let contents = "\
.PHONY: run build check

run:
		@cargo run

build:
		@cargo build

check:
		@cargo check

echo:
	@echo good";

        assert_eq!(
            vec!["run", "build", "check", "echo"],
            extract_command(contents.to_string())
        );
    }
}
