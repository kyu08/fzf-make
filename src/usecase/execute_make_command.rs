use colored::Colorize;
use std::process;

pub fn execute_make_target(t: &String) {
    println!("{}", ("make ".to_string() + t).truecolor(161, 220, 156));
    process::Command::new("make")
        .stdin(process::Stdio::inherit())
        .arg(t)
        .spawn()
        .expect("Failed to execute process")
        .wait()
        .expect("Failed to execute process");
}
