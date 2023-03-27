use skim::prelude::*;
use std::process;

mod misc;

fn main() {
    let (options, items) = misc::get_params();
    if let output @ Some(_) = Skim::run_with(&options, items) {
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let selected_items = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);

        for item in selected_items.iter() {
            match process::Command::new("make")
                .arg(item.output().to_string())
                .output()
            {
                Ok(output) => {
                    // TODO: extract as function?
                    println!("\n");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                }
                Err(_) => misc::print_error("fail to execute make command".to_string()),
            }
        }
    }
}
