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

    pub fn new(current_dir: PathBuf, result: Vec<(String, String, u32)>) -> Yarn {
        let commands = Yarn::scripts_to_commands(current_dir.clone(), result);

        Yarn {
            path: current_dir,
            commands,
        }
    }

    pub fn use_yarn(file_name: &str) -> bool {
        file_name == YARN_LOCKFILE_NAME
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
    fn scripts_to_commands(
        current_dir: PathBuf,
        parsed_scripts_part_of_package_json: Vec<(String, String, u32)>,
    ) -> Vec<command::Command> {
        let mut result = vec![];

        // Collect scripts defined in package.json in the current directory(which fzf-make is launched)
        for (key, _value, line_number) in parsed_scripts_part_of_package_json {
            result.push(command::Command::new(
                runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Yarn),
                key,
                current_dir.clone().join(js::METADATA_FILE_NAME),
                line_number,
            ));
        }

        // Collect the paths of all `package.json` in the workspace.
        let package_json_in_workspace = if Self::is_yarn_v1() {
            Self::get_workspace_packages_for_v1()
        } else {
            Self::get_workspace_packages_for_v2_or_later()
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

        result
    }

    // yarn v1 support `yarn workspaces info --json` instead of `yarn workspaces list --json`.
    //  We need to handle them separately, because their output format is different.
    fn is_yarn_v1() -> bool {
        let output = process::Command::new("yarn")
            .arg("--version")
            .output()
            .expect("failed to run yarn --version");
        let output = String::from_utf8(output.stdout).expect("failed to convert to string");
        /* output is like:
        "1.22.22\n"
        */

        output.trim().starts_with("1.")
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
