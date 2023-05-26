use skim::prelude::Skim;
use std::process;

mod misc;

fn main() {
    let (options, items) = misc::get_params();
    if let output @ Some(_) = Skim::run_with(&options, items) {
        if output.as_ref().unwrap().is_abort {
            process::exit(0)
        }

        let target = output
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new)
            .first()
            .unwrap()
            .output()
            .to_string();

        let fail_to_execute_message = "Failed to execute process";
        println!("make {}", target);
        process::Command::new("make")
            .stdin(process::Stdio::inherit())
            .arg(target)
            .spawn()
            .expect(fail_to_execute_message)
            .wait()
            .expect(fail_to_execute_message);
    }
}
