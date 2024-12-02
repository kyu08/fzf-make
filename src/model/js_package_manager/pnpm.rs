use super::js_package_manager_main;
use crate::{
    file::path_to_content,
    model::{command, runner_type},
};
use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf, process};

const PNPM_LOCKFILE_NAME: &str = "pnpm-lock.yaml";
const IGNORE_DIR_NAMES: [&str; 4] = ["node_modules", ".git", ".cache", ".next"];

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

    pub fn new(current_dir: PathBuf, result: Vec<(String, String, u32)>) -> Pnpm {
        let commands = Pnpm::scripts_to_commands(current_dir.clone(), result);

        Pnpm {
            path: current_dir,
            commands,
        }
    }

    pub fn use_pnpm(file_name: String) -> bool {
        file_name == PNPM_LOCKFILE_NAME
    }

    pub fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    fn get_command(&self, command: command::Command) -> Option<&command::Command> {
        self.commands.iter().find(|c| **c == command)
    }

    fn scripts_to_commands(
        current_dir: PathBuf,
        parsed_scripts_part_of_package_json: Vec<(String, String, u32)>,
    ) -> Vec<command::Command> {
        let mut result = vec![];

        for (key, value, line_number) in parsed_scripts_part_of_package_json {
            if Pnpm::use_filtering(value.clone()) {
                continue;
            }

            // scripts defined in package.json in the current directory(which fzf-make is launched)
            result.push(command::Command::new(
                runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Pnpm),
                key,
                current_dir
                    .clone()
                    .join(js_package_manager_main::METADATA_FILE_NAME),
                line_number,
            ));
        }

        /*
        ## Following is the implementation for collecting all scripts in the workspace.

        - If `packages` in pnpm-workspace.yaml is specified, the target to search is only under the directory defined at `packages`. If not specified, all package.json's are the target.
        - Nested packages do not need to be considered. `./packages/app1/package.json` needs to be considered, but `./packages/app1/app2/package.json` does not need to be considered.
        - If the directory structure is as follows, the examples will be shown in `entries_cwd.for_each(...)`.
            ${CWD}
            ├── package.json
            ├── node_modules/
            ├── packages
            │   ├── app1
            │   │   ├── package.json
            │   │   └── node_modules
            │   ├── app2
            │   │   ├── package.json
            │   │   └── node_modules
            │   └── app3
            │       ├── package.json
            │       └── node_modules
            ├── pnpm-lock.yaml
            └── pnpm-workspace.yaml
        */

        // TODO: consider `packages` in pnpm-workspace.yaml.
        // TODO: Add UT. (Use temp dir or fzf-make/test_data. If use temp dir, the test will be
        // robust, but troublesome for now...😇)
        let skip = |entry: &fs::DirEntry| {
            entry.path().is_file()
                || IGNORE_DIR_NAMES
                    .iter()
                    .any(|name| entry.file_name() == *name)
        };
        // In above example, entries_cwd: package.json, node_modules, packages/, pnpm-lock.yaml, pnpm-workspace.yaml
        let entries_cwd = fs::read_dir(current_dir.clone()).unwrap();
        entries_cwd.for_each(|entry_cwd| {
            if let Ok(entry_in_cwd) = entry_cwd {
                if skip(&entry_in_cwd) {
                    return;
                }
                // In above example, entries_of_packages: app1, app2, app3.
                let entries_of_packages = fs::read_dir(entry_in_cwd.path()).unwrap();
                entries_of_packages.for_each(|entry_package| {
                    if let Ok(entry_package) = entry_package {
                        if skip(&entry_package) {
                            return;
                        }

                        // In above example, entries_of_each_package: package.json, node_modules.
                        let entries_of_each_package = fs::read_dir(entry_package.path()).unwrap();
                        entries_of_each_package.for_each(|entry_of_each_package| {
                            if let Ok(entry_of_each_package) = entry_of_each_package {
                                if entry_of_each_package.file_name() != js_package_manager_main::METADATA_FILE_NAME {
                                    return;
                                }
                                if let Ok(c) =
                                    path_to_content::path_to_content(entry_of_each_package.path())
                                {
                                    if let Some((name, parsing_result)) =
                                        js_package_manager_main::JsPackageManager::parse_package_json(&c)
                                    {
                                        for (key, _, line_number) in parsing_result {
                                            result.push(command::Command::new(
                                                runner_type::RunnerType::JsPackageManager(
                                                    runner_type::JsPackageManager::Pnpm,
                                                ),
                                                format!(
                                                    "--filter {} {}",
                                                    name.clone(),
                                                    key.as_str()
                                                ),
                                                entry_of_each_package.path(),
                                                line_number,
                                            ));
                                        }
                                    }
                                };
                            }
                        });
                    }
                });
            }
        });

        result
    }

    // is filtering used
    // ref: https://pnpm.io/filtering
    fn use_filtering(value: String) -> bool {
        let args = value.split_whitespace().collect::<Vec<&str>>();

        let start_with_pnpm = args.first().map(|arg| *arg == "pnpm").unwrap_or(false);
        let has_filtering_or_dir_option = args
            .iter()
            .any(|arg| *arg == "-F" || *arg == "--filter" || *arg == "-C" || *arg == "--dir");
        let has_run = args.iter().any(|arg| *arg == "run");

        start_with_pnpm && has_filtering_or_dir_option && !has_run
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_is_filtering() {
        assert_eq!(true, Pnpm::use_filtering("pnpm -F app1".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm --filter app2".to_string()));
        assert_eq!(
            true,
            Pnpm::use_filtering("pnpm -r --filter app3".to_string())
        );
        assert_eq!(
            true,
            Pnpm::use_filtering("pnpm -C packages/app3".to_string())
        );
        assert_eq!(
            true,
            Pnpm::use_filtering("pnpm --dir packages/app3".to_string())
        );
        assert_eq!(true, Pnpm::use_filtering("pnpm -F".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm --filter".to_string()));
        assert_eq!(
            false,
            Pnpm::use_filtering("pnpm -C packages/app1 run test".to_string())
        );
        assert_eq!(
            false,
            Pnpm::use_filtering("pnpm --filter app1 run test".to_string())
        );
        assert_eq!(false, Pnpm::use_filtering("yarn run".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm run".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm -r hoge".to_string()));
        assert_eq!(
            false,
            Pnpm::use_filtering("yarn -r --filter app3".to_string())
        );
    }
}
