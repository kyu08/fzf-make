use super::js_package_manager_main as js;
use crate::{
    file::path_to_content,
    model::{command, file_util, runner_type},
};
use anyhow::{Result, anyhow};
use std::{path::PathBuf, process};

const PNPM_LOCKFILE_NAME: &str = "pnpm-lock.yaml";

#[derive(Clone, Debug, PartialEq)]
pub struct Pnpm {
    pub path: PathBuf,
    commands: Vec<command::CommandWithPreview>,
}

impl Pnpm {
    pub fn command_to_run(&self, command: &command::CommandForExec) -> Result<String> {
        Ok(format!("pnpm {}", command.args))
    }

    pub fn execute(&self, command: &command::CommandForExec) -> Result<()> {
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
        let package_json_exist = Iterator::find(&mut cwd_file_names.iter(), |&f| f == js::METADATA_FILE_NAME);
        let lockfile_exist_in_current_dir = Iterator::find(&mut cwd_file_names.iter(), |&f| f == PNPM_LOCKFILE_NAME);
        let lockfile_exist_in_ancestors =
            file_util::find_file_in_ancestors(current_dir.clone(), vec![PNPM_LOCKFILE_NAME]);

        match (package_json_exist, lockfile_exist_in_current_dir, lockfile_exist_in_ancestors) {
            (None, _, _) => None,
            (Some(_), Some(_), _) => Pnpm::collect_workspace_scripts(current_dir.clone()).map(|commands| Pnpm {
                path: current_dir,
                commands,
            }),
            (Some(_), None, Some(_)) => {
                Self::collect_scripts_in_package_json(current_dir.clone()).map(|commands| Pnpm {
                    path: current_dir,
                    commands,
                })
            }
            // Not a workspace children && not a workspace root
            // In this case, package manager can not be determined.
            (Some(_), None, None) => None,
        }
    }

    // scripts_to_commands collects all scripts by following steps:
    // 1. Collect scripts defined in package.json in the current directory(which fzf-make is launched)
    // 2. Collect the paths of all `package.json` in the workspace.
    // 3. Collect all scripts defined in given `package.json` paths.
    fn collect_workspace_scripts(current_dir: PathBuf) -> Option<Vec<command::CommandWithPreview>> {
        // Collect scripts defined in package.json in the current directory(which fzf-make is launched)
        let mut result = Self::collect_scripts_in_package_json(current_dir.clone())?;

        // Collect the paths of all `package.json` in the workspace.
        let workspace_package_json_paths = match Self::get_workspace_packages() {
            Ok(result) => result,
            Err(_) => return None,
        };

        // Collect all scripts defined in given `package.json` paths.
        for path in workspace_package_json_paths {
            if let Ok(c) = path_to_content::path_to_content(&path) {
                if let Some((name, parsing_result)) = js::JsPackageManager::parse_package_json(&c) {
                    for (key, value, line_number) in parsing_result {
                        if Self::use_filtering(value) {
                            continue;
                        }
                        result.push(command::CommandWithPreview::new(
                            runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Pnpm),
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

    fn collect_scripts_in_package_json(current_dir: PathBuf) -> Option<Vec<command::CommandWithPreview>> {
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
                .filter(|(_, value, _)| !Self::use_filtering(value.to_string()))
                .map(|(key, _value, line_number)| {
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Pnpm),
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
        let lines = output.split('\n').filter(|l| !l.is_empty()).collect::<Vec<&str>>();

        Ok(lines
            .iter()
            .map(|line| PathBuf::from(line).join(js::METADATA_FILE_NAME))
            .collect())
    }

    pub fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        self.commands.clone()
    }

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
        assert_eq!(true, Pnpm::use_filtering("pnpm -F \"app1\"".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm --filter app2".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm -r --filter app3".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm -C packages/app3".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm --dir packages/app3".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm -F".to_string()));
        assert_eq!(true, Pnpm::use_filtering("pnpm --filter".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm -C packages/app1 run test".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm --filter app1 run test".to_string()));
        assert_eq!(false, Pnpm::use_filtering("yarn run".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm run".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm -r hoge".to_string()));
        assert_eq!(false, Pnpm::use_filtering("yarn -r --filter app3".to_string()));
    }
}
