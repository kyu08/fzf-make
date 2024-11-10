use crate::model::runner;

pub fn execute_make_command(e: &Box<dyn runner::Runner>) {
    e.show_command();
    e.execute();

    // println!("{}", ("make ".to_string() + t).truecolor(161, 220, 156));
    // process::Command::new("make")
    //     .stdin(process::Stdio::inherit())
    //     .arg(t)
    //     .spawn()
    //     .expect("Failed to execute process")
    //     .wait()
    //     .expect("Failed to execute process");
}
