use skim::prelude::*;
use std::{io::Cursor, process};

mod misc;

fn main() {
    let (options, items) = get_params();

    if let output @ Some(_) = Skim::run_with(&options, items) {
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
                    // TODO: extract as function
                    println!("\n");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Err(_) => print_error("fail to execute make command".to_string()),
            }
        }
    }
}

fn get_params<'a>() -> (SkimOptions<'a>, Option<Receiver<Arc<dyn SkimItem>>>) {
    // TODO: use cat when bat is unavailable
    let preview_command = "bat --style=numbers --color=always --highlight-line $(bat Makefile | grep -n {}: | sed -e 's/:.*//g') Makefile";
    // TODO: hide fzf window when fzf-make terminated
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .preview(Some(preview_command))
        .reverse(true)
        .build()
        .unwrap();
    let commands = match misc::extract_command_from_makefile() {
        // TODO: use some method
        Ok(s) => s,
        Err(e) => {
            print_error(e.to_string());
            process::exit(1)
        }
    };
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(commands));

    (options, Some(items))
}

fn print_error(error_message: String) {
    println!("[ERR] {}", error_message);
}
