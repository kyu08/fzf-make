use crate::usecase::usecase_main::Usecase;
use anyhow::Result;
use futures::{FutureExt, future::BoxFuture};

pub struct Help;

impl Help {
    pub fn new() -> Self {
        Self {}
    }
}

impl Usecase for Help {
    fn command_str(&self) -> Vec<&'static str> {
        vec!["--help", "help"]
    }

    fn run(&self) -> BoxFuture<'_, Result<()>> {
        async {
            println!("{}", get_help());
            Ok(())
        }
        .boxed()
    }
}

// TODO: Make each command have the following information as a struct, and just display it here.
// Define the vector of usecases in only one place and refer to it.
pub fn get_help() -> String {
    r#"A command line tool that executes commands using fuzzy finder with preview window. Currently supporting make, pnpm, yarn, just, and task.

USAGE:
    Run `fzf-make` in the directory where you want to execute make, pnpm, yarn, just, or task command exists or `fzf-make [SUBCOMMAND]`.

SUBCOMMANDS:
    repeat, --repeat, -r
        Execute the last executed command.
    history, --history, -h
        Launch fzf-make with the history pane focused.
    help, --help
        Prints help message.
    version, --version, -v
        Prints version information.
    "#
    .to_string()
}
