use std::process;

use colored::Colorize;

pub fn execute_make_target(t: &String) {
    println!("{}", ("make ".to_string() + t).blue());
    process::Command::new("make")
        .stdin(process::Stdio::inherit())
        .arg(t)
        .spawn()
        .expect("Failed to execute process")
        .wait()
        .expect("Failed to execute process");
}
