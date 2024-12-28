use super::js_package_manager_main as js;
use crate::{
    file::path_to_content,
    model::{command, runner_type},
};
use anyhow::{anyhow, Result};
use std::{path::PathBuf, process};

const PNPM_LOCKFILE_NAME: &str = "pnpm-lock.yaml";

#[derive(Clone, Debug, PartialEq)]
pub struct Pnpm {
    pub path: PathBuf,
    commands: Vec<command::Command>,
}

impl Pnpm {
    pub fn command_to_run(&self, command: &command::Command) -> Result<String> {
        // To ensure that the command exists, it is necessary to check the command name.
        // If implementation is wrong, developers can notice it here.
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        Ok(format!("pnpm {}", command.args))
    }

    pub fn execute(&self, command: &command::Command) -> Result<()> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        let child = process::Command::new("pnpm")
            .stdin(process::Stdio::inherit())
            .args(command.args.split_whitespace().collect::<Vec<&str>>())
            .spawn();

        match child {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("failed to run: {}", e)),
            },
            Err(e) => Err(anyhow!("failed to spawn: {}", e)),
        }
    }

    pub fn new(current_dir: PathBuf, cwd_file_names: Vec<String>) -> Option<Pnpm> {
        Iterator::find(&mut cwd_file_names.iter(), |&f| f == js::METADATA_FILE_NAME)?;
        if Iterator::find(&mut cwd_file_names.iter(), |&f| f == PNPM_LOCKFILE_NAME).is_some() {
            // package.json and lock file exist. It means that the current directory is a root of the workspace, or single package.
            Pnpm::collect_workspace_scripts(current_dir.clone()).map(|commands| Pnpm {
                path: current_dir,
                commands,
            })
        } else {
            // package.json exists, but lock file does not exist
            Self::collect_scripts_in_package_json(current_dir.clone()).map(|commands| Pnpm {
                path: current_dir,
                commands,
            })
        }
    }

    // scripts_to_commands collects all scripts by following steps:
    // 1. Collect scripts defined in package.json in the current directory(which fzf-make is launched)
    // 2. Collect the paths of all `package.json` in the workspace.
    // 3. Collect all scripts defined in given `package.json` paths.
    fn collect_workspace_scripts(current_dir: PathBuf) -> Option<Vec<command::Command>> {
        // Collect scripts defined in package.json in the current directory(which fzf-make is launched)
        let mut result = match Self::collect_scripts_in_package_json(current_dir.clone()) {
            Some(result) => result,
            None => return None,
        };

        // Collect the paths of all `package.json` in the workspace.
        let workspace_package_json_paths = match Self::get_workspace_packages() {
            Ok(result) => result,
            Err(_) => return None,
        };

        // Collect all scripts defined in given `package.json` paths.
        for path in workspace_package_json_paths {
            if let Ok(c) = path_to_content::path_to_content(&path) {
                if let Some((name, parsing_result)) = js::JsPackageManager::parse_package_json(&c) {
                    for (key, _, line_number) in parsing_result {
                        result.push(command::Command::new(
                            runner_type::RunnerType::JsPackageManager(
                                runner_type::JsPackageManager::Pnpm,
                            ),
                            // pnpm executes workspace script following format: `pnpm --filter {package_name} {script_name}`
                            // e.g. `pnpm --filter app4 build`
                            format!("--filter {} {}", name.clone(), key.as_str()),
                            path.clone(),
                            line_number,
                        ));
                    }
                }
            };
        }

        Some(result)
    }

    fn collect_scripts_in_package_json(current_dir: PathBuf) -> Option<Vec<command::Command>> {
        let parsed_scripts_part_of_package_json =
            match path_to_content::path_to_content(&current_dir.join(js::METADATA_FILE_NAME)) {
                Ok(c) => match js::JsPackageManager::parse_package_json(&c) {
                    Some(result) => result.1,
                    None => return None,
                },
                Err(_) => return None,
            };

        Some(
            parsed_scripts_part_of_package_json
                .iter()
                .map(|(key, _value, line_number)| {
                    command::Command::new(
                        runner_type::RunnerType::JsPackageManager(
                            runner_type::JsPackageManager::Pnpm,
                        ),
                        key.to_string(),
                        current_dir.clone().join(js::METADATA_FILE_NAME),
                        *line_number,
                    )
                })
                .collect(),
        )
    }

    // get_workspaces_list parses the result of `pnpm -r exec pwd` and return path of `package.json` of each package.
    fn get_workspace_packages() -> Result<Vec<PathBuf>> {
        let output = process::Command::new("pnpm")
            .arg("-r")
            .arg("exec")
            .arg("pwd")
            .output()?;
        /* Example output:
            /Users/kyu08/code/fzf-make/test_data/pnpm_monorepo/packages/app1
            /Users/kyu08/code/fzf-make/test_data/pnpm_monorepo/packages/app2
            /Users/kyu08/code/fzf-make/test_data/pnpm_monorepo/packages/app3
            /Users/kyu08/code/fzf-make/test_data/pnpm_monorepo/packages/sub_packages/sub_app
        */

        let output = String::from_utf8(output.stdout)?;
        // split by newline to remove unnecessary lines.
        let lines = output
            .split("\n")
            .filter(|l| !l.is_empty())
            .collect::<Vec<&str>>();

        Ok(lines
            .iter()
            .map(|line| PathBuf::from(line).join(js::METADATA_FILE_NAME))
            .collect())
    }

    pub fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    fn get_command(&self, command: command::Command) -> Option<&command::Command> {
        self.commands.iter().find(|c| **c == command)
    }
}
