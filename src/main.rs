use skim::prelude::Skim;
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
