use super::js_package_manager_main as js;
use crate::{
    file::path_to_content,
    model::{command, runner_type},
};
use anyhow::{anyhow, Result};
use std::{path::PathBuf, process};

const YARN_LOCKFILE_NAME: &str = "yarn.lock";

#[derive(Clone, Debug, PartialEq)]
pub struct Yarn {
    pub path: PathBuf,
    commands: Vec<command::Command>,
}

enum YarnVersion {
    V1,
    V2OrLater,
}

impl Yarn {
    pub fn command_to_run(&self, command: &command::Command) -> Result<String> {
        // To ensure that the command exists, it is necessary to check the command name.
        // If implementation is wrong, developers can notice it here.
        match self.get_command(command.clone()) {
            Some(c) => Ok(format!("yarn {}", c.args)),
            None => Err(anyhow!("command not found")),
        }
    }

    pub fn execute(&self, command: &command::Command) -> Result<()> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        let child = process::Command::new("yarn")
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

    pub fn new(current_dir: PathBuf, cwd_file_names: Vec<String>) -> Option<Yarn> {
        if Iterator::find(&mut cwd_file_names.iter(), |&f| f == YARN_LOCKFILE_NAME).is_some() {
            match Yarn::collect_workspace_scripts(current_dir.clone()) {
                Some(commands) => {
                    return Some(Yarn {
                        path: current_dir,
                        commands,
                    })
                }
                None => return None,
            }
        }

        // executed in child packages of yarn workspaces || using an other package manager
        match Self::get_yarn_version() {
            Some(yarn_version) => {
                let workspace_output = match yarn_version {
                    YarnVersion::V1 => process::Command::new("yarn")
                        .arg("workspaces")
                        .arg("info")
                        .arg("--json")
                        .output(),
                    YarnVersion::V2OrLater => process::Command::new("yarn")
                        .arg("workspaces")
                        .arg("list")
                        .arg("--json")
                        .output(),
                };
                let workspace_output = match workspace_output {
                    Ok(output) => output,
                    Err(_) => return None,
                };

                if workspace_output.status.code().is_none()
                // If `yarn workspaces info --json` returns non-zero status code, it means that the current directory is not a yarn workspace.
                || workspace_output.status.code().unwrap() != 0
                {
                    return None;
                }

                Some(Yarn {
                    path: current_dir.clone(),
                    commands: Self::collect_scripts_in_package_json(current_dir),
                })
            }
            None => None, // yarn is not installed
        }
    }

    pub fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    fn get_command(&self, command: command::Command) -> Option<&command::Command> {
        self.commands.iter().find(|c| **c == command)
    }

    // scripts_to_commands collects all scripts by following steps:
    // 1. Collect scripts defined in package.json in the current directory(which fzf-make is launched)
    // 2. Collect the paths of all `package.json` in the workspace.
    // 3. Collect all scripts defined in given `package.json` paths.
    fn collect_workspace_scripts(current_dir: PathBuf) -> Option<Vec<command::Command>> {
        // Collect scripts defined in package.json in the current directory(which fzf-make is launched)
        let mut result = Self::collect_scripts_in_package_json(current_dir.clone());

        // Collect the paths of all `package.json` in the workspace.
        let package_json_in_workspace = match Self::get_yarn_version() {
            Some(YarnVersion::V1) => Self::get_workspace_packages_for_v1(),
            Some(YarnVersion::V2OrLater) => Self::get_workspace_packages_for_v2_or_later(),
            None => return None,
        };

        // Collect all scripts defined in given `package.json` paths.
        if let Ok(workspace_package_json_paths) = package_json_in_workspace {
            for path in workspace_package_json_paths {
                if let Ok(c) = path_to_content::path_to_content(&path) {
                    if let Some((name, parsing_result)) =
                        js::JsPackageManager::parse_package_json(&c)
                    {
                        for (key, _, line_number) in parsing_result {
                            result.push(command::Command::new(
                                runner_type::RunnerType::JsPackageManager(
                                    runner_type::JsPackageManager::Yarn,
                                ),
                                // yarn executes workspace script following format: `yarn workspace {package_name} {script_name}`
                                // e.g. `yarn workspace app4 build`
                                format!("workspace {} {}", name.clone(), key.as_str()),
                                path.clone(),
                                line_number,
                            ));
                        }
                    }
                };
            }
        };

        Some(result)
    }

    fn collect_scripts_in_package_json(current_dir: PathBuf) -> Vec<command::Command> {
        let parsed_scripts_part_of_package_json =
            match path_to_content::path_to_content(&current_dir.join(js::METADATA_FILE_NAME)) {
                Ok(c) => match js::JsPackageManager::parse_package_json(&c) {
                    Some(result) => result.1,
                    None => return vec![],
                },
                Err(_) => return vec![],
            };

        parsed_scripts_part_of_package_json
            .iter()
            .map(|(key, _value, line_number)| {
                command::Command::new(
                    runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Yarn),
                    key.to_string(),
                    current_dir.clone().join(js::METADATA_FILE_NAME),
                    *line_number,
                )
            })
            .collect()
    }

    /// Determines the installed Yarn version, if available.
    /// yarn v1 support `yarn workspaces info --json` instead of `yarn workspaces list --json`.
    ///  We need to handle them separately, because their output format is different.
    ///
    /// # Returns
    /// - `Some(YarnVersion::V1)` if Yarn v1 is detected.
    /// - `Some(YarnVersion::V2OrLater)` if Yarn v2 or later is detected.
    /// - `None` if Yarn is not installed or cannot be executed.
    fn get_yarn_version() -> Option<YarnVersion> {
        let output = process::Command::new("yarn").arg("--version").output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    return None;
                }

                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                /* Example output:
                "1.22.22\n"
                */

                if version.starts_with("1.") {
                    Some(YarnVersion::V1)
                } else {
                    Some(YarnVersion::V2OrLater)
                }
            }
            Err(_) => None,
        }
    }

    // get_workspaces_list parses the result of `yarn workspaces info --json` and return path of `package.json` of each package.
    fn get_workspace_packages_for_v1() -> Result<Vec<PathBuf>> {
        let output = process::Command::new("yarn")
            .arg("workspaces")
            .arg("info")
            .arg("--json")
            .output()?;
        /* output is like:
        yarn workspaces v1.22.22
        {
          "app1": {
            "location": "packages/app",
            "workspaceDependencies": [],
            "mismatchedWorkspaceDependencies": []
          }
        }
        ✨  Done in 0.02s.
         */

        #[derive(serde::Deserialize, Debug)]
        struct Workspace {
            // Relation path to the package
            location: String,
        }

        let workspaces_json = {
            let output = String::from_utf8(output.stdout)?;
            /* output is like:
            "yarn workspaces v1.22.22\n{\n  \"app1\": {\n    \"location\": \"packages/app\",\n    \"workspaceDependencies\": [],\n    \"mismatchedWorkspaceDependencies\": []\n  }\n}\nDone in 0.01s.\n"
            */

            // split by newline to remove unnecessary lines.
            let lines = output.split("\n").collect::<Vec<&str>>();

            // remove the first and last line and the second line from the end.
            match lines.get(1..(lines.len() - 2)) {
                Some(lines) => lines.join(""),
                None => return Err(anyhow!("unexpected output")),
            }
        };

        // parse json
        let mut workspaces: Vec<Workspace> = vec![];
        if let Ok(serde_json::Value::Object(map)) =
            serde_json::from_slice::<serde_json::Value>(workspaces_json.as_bytes())
        {
            for (_, value) in map {
                if let Ok(workspace) = serde_json::from_value(value) {
                    workspaces.push(workspace)
                }
            }
        }

        Ok(workspaces
            .iter()
            .map(|w| PathBuf::from(w.location.clone()).join(js::METADATA_FILE_NAME))
            .collect())
    }

    // get_workspaces_list parses the result of `yarn workspaces list --json` and return path of `package.json` of each package.
    fn get_workspace_packages_for_v2_or_later() -> Result<Vec<PathBuf>> {
        let output = process::Command::new("yarn")
            .arg("workspaces")
            .arg("list")
            .arg("--json")
            .output()?;

        // The format is the same as v1 by chance, so we do not unify intentionally.
        #[derive(serde::Deserialize, Debug)]
        struct Workspace {
            // Relation path to the package
            location: String,
        }
        let mut workspaces: Vec<Workspace> = vec![];
        /* output is like:
        "{\"location\":\".\",\"name\":\"project\"}\n{\"location\":\"packages/app1\",\"name\":\"app1\"}\n"
         */
        let output = String::from_utf8(output.stdout)?;

        for line in output.lines() {
            // To parse json like above, use `serde_json::from_slice(line.as_bytes())`.
            // see: https://stackoverflow.com/a/69001942.
            if let Ok(workspace) = serde_json::from_slice(line.as_bytes()) {
                workspaces.push(workspace)
            }
        }

        Ok(workspaces
            .iter()
            .filter(|workspace| workspace.location != ".") // Ignore package.json in the current directory.
            .map(|w| PathBuf::from(w.location.clone()).join(js::METADATA_FILE_NAME))
            .collect())
    }
}
