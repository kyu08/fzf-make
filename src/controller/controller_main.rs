use crate::usecase::{fzf_make, fzf_make::FzfMake, help, history, invalid_arg, repeat, usecase_main, version};
use colored::Colorize;
use std::{collections::HashMap, env, sync::Arc};

pub async fn run() {
    let command_line_args = env::args().collect();
    let usecase = args_to_usecase(command_line_args);

    if let Err(e) = usecase.run().await {
        print_error(&e);
        std::process::exit(1);
    }
}

fn args_to_usecase(args: Vec<String>) -> Arc<dyn usecase_main::Usecase> {
    // Currently, only fzf-make or fzf-make ${command} is accepted.
    if 2 < args.len() {
        return Arc::new(invalid_arg::InvalidArg);
    }

    let command = match args.get(1) {
        Some(s) => s,
        None => return Arc::new(FzfMake),
    };

    match usecases().get(command.as_str()) {
        Some(uc) => uc.clone(),
        None => Arc::new(invalid_arg::InvalidArg::new()),
    }
}

fn usecases() -> HashMap<&'static str, Arc<dyn usecase_main::Usecase>> {
    let usecases: Vec<Arc<dyn usecase_main::Usecase>> = vec![
        Arc::new(help::Help::new()),
        Arc::new(invalid_arg::InvalidArg::new()),
        Arc::new(version::Version::new()),
        Arc::new(fzf_make::FzfMake::new()),
        Arc::new(repeat::Repeat::new()),
        Arc::new(history::History::new()),
    ];

    let mut usecases_hash_map = HashMap::new();
    usecases.into_iter().for_each(|uc| {
        for cmd in uc.command_str() {
            usecases_hash_map.insert(cmd, uc.clone());
        }
    });

    usecases_hash_map
}

fn print_error(e: &anyhow::Error) {
    println!("{}", e.to_string().red());
}
