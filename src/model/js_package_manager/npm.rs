use super::js_package_manager_main as js;
use crate::{
    file::path_to_content,
    model::{command, file_util, runner_type},
};
use anyhow::{Result, anyhow};
use std::{path::PathBuf, process};

const NPM_LOCKFILE_NAME: &str = "package-lock.json";

#[derive(Clone, Debug, PartialEq)]
pub struct Npm {
    pub path: PathBuf,
    commands: Vec<command::CommandWithPreview>,
}

impl Npm {
    pub fn command_to_run(&self, command: &command::CommandForExec) -> Result<String> {
        Ok(format!("npm {}", command.args))
    }

    pub fn execute(&self, command: &command::CommandForExec) -> Result<()> {
        let child = process::Command::new("npm")
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

    pub fn new(current_dir: PathBuf, cwd_file_names: Vec<String>) -> Option<Npm> {
        let package_json_exist = Iterator::find(&mut cwd_file_names.iter(), |&f| f == js::METADATA_FILE_NAME);
        let lockfile_exist_in_current_dir = Iterator::find(&mut cwd_file_names.iter(), |&f| f == NPM_LOCKFILE_NAME);
        let lockfile_exist_in_ancestors =
            file_util::find_file_in_ancestors(current_dir.clone(), vec![NPM_LOCKFILE_NAME]);

        match (package_json_exist, lockfile_exist_in_current_dir, lockfile_exist_in_ancestors) {
            (None, _, _) => None,
            (Some(_), Some(_), _) => Npm::collect_workspace_scripts(current_dir.clone()).map(|commands| Npm {
                path: current_dir,
                commands,
            }),
            (Some(_), None, Some(_)) => {
                Self::collect_scripts_in_package_json(current_dir.clone()).map(|commands| Npm {
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
        // If npm query fails (e.g., for non-monorepo projects), just return scripts from current package.json
        let workspace_package_json_paths = match Self::get_workspace_packages() {
            Ok(result) => result,
            Err(_) => return Some(result),
        };

        // Collect all scripts defined in given `package.json` paths.
        for path in workspace_package_json_paths {
            // Skip the current directory's package.json to avoid duplication
            if path == current_dir.join(js::METADATA_FILE_NAME) {
                continue;
            }

            if let Ok(c) = path_to_content::path_to_content(&path)
                && let Some((name, parsing_result)) = js::JsPackageManager::parse_package_json(&c)
            {
                for (key, _, line_number) in parsing_result {
                    result.push(command::CommandWithPreview::new(
                        runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Npm),
                        // npm executes workspace script following format: `npm run {script_name} --workspace={package_name}`
                        // e.g. `npm run build --workspace=app1`
                        format!("run {} --workspace={}", key.as_str(), name.clone()),
                        path.clone(),
                        line_number,
                    ));
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
                .map(|(key, _value, line_number)| {
                    command::CommandWithPreview::new(
                        runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Npm),
                        format!("run {}", key),
                        current_dir.clone().join(js::METADATA_FILE_NAME),
                        *line_number,
                    )
                })
                .collect(),
        )
    }

    // get_workspace_packages uses `npm query .workspace` to get workspace package.json paths.
    // This requires npm 8.16.0+ but provides a clean JSON output.
    fn get_workspace_packages() -> Result<Vec<PathBuf>> {
        let output = process::Command::new("npm")
            .arg("query")
            .arg(".workspace")
            .arg("--json")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("npm query failed"));
        }

        #[derive(serde::Deserialize, Debug)]
        struct WorkspacePackage {
            path: String,
        }

        let output_str = String::from_utf8(output.stdout)?;
        let packages: Vec<WorkspacePackage> = serde_json::from_str(&output_str)?;

        Ok(packages
            .iter()
            .map(|pkg| PathBuf::from(&pkg.path).join(js::METADATA_FILE_NAME))
            .collect())
    }

    pub fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        self.commands.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_npm_detection() {
        // This test validates the detection logic for NPM projects
        // It requires actual test data directories to work properly

        // Test case 1: Workspace root (has package.json and package-lock.json)
        let npm_root = PathBuf::from("test_data/npm");
        let file_names = vec!["package.json".to_string(), "package-lock.json".to_string()];
        let result = Npm::new(npm_root.clone(), file_names);
        assert!(result.is_some(), "Should detect npm workspace root");

        // Test case 2: No package.json - should return None
        let empty_dir = std::env::temp_dir();
        let file_names = vec!["README.md".to_string()];
        let result = Npm::new(empty_dir, file_names);
        assert!(result.is_none(), "Should not detect npm without package.json");

        // Test case 3: package.json only, no lockfile anywhere - should return None
        let no_lockfile = std::env::temp_dir();
        let file_names = vec!["package.json".to_string()];
        let result = Npm::new(no_lockfile, file_names);
        assert!(result.is_none(), "Should not detect npm without lockfile");
    }

    #[test]
    fn test_command_format() {
        // Test that commands are formatted correctly
        let npm = Npm {
            path: PathBuf::from("test_data/npm"),
            commands: vec![],
        };

        let command = command::CommandForExec {
            runner_type: runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Npm),
            args: "run build".to_string(),
        };

        let result = npm.command_to_run(&command);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "npm run build");
    }

    #[test]
    fn test_workspace_command_format() {
        // Test workspace command format
        let npm = Npm {
            path: PathBuf::from("test_data/npm_monorepo"),
            commands: vec![],
        };

        let command = command::CommandForExec {
            runner_type: runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Npm),
            args: "run build --workspace=app1".to_string(),
        };

        let result = npm.command_to_run(&command);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "npm run build --workspace=app1");
    }
}
