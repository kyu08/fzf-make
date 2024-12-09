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

    fn scripts_to_commands(
        current_dir: PathBuf,
        parsed_scripts_part_of_package_json: Vec<(String, String, u32)>,
    ) -> Vec<command::Command> {
        let mut result = vec![];

        for (key, _value, line_number) in parsed_scripts_part_of_package_json {
            // scripts defined in package.json in the current directory(which fzf-make is launched)
            result.push(command::Command::new(
                runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Yarn),
                key,
                current_dir.clone().join(js::METADATA_FILE_NAME),
                line_number,
            ));
        }

        Self::collect_all_scripts_in_workspace();

        // Collect all scripts in the workspace.
        if let Ok(workspace_package_paths) = Self::get_workspace_packages_for_v2_or_later() {
            for entry_of_each_package in workspace_package_paths {
                let path = PathBuf::from(entry_of_each_package);
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

    fn collect_all_scripts_in_workspace() -> Vec<command::Command> {
        let mut result = vec![];

        // v1とv2で分岐する
        let hoge = Self::get_workspace_packages_for_v1();
        panic!("{:?}", hoge);

        result
    }

    // get_workspaces_list parses the result of `yarn workspaces list --json` and return path of `package.json` of each package.
    fn get_workspace_packages_for_v1() -> Result<Vec<String>> {
        let output = process::Command::new("yarn")
            .arg("workspaces")
            .arg("info")
            .arg("--json")
            .output()?;

        // raw output is like below.
        // yarn workspaces v1.22.22
        // {
        //   "app1": {
        //     "location": "packages/app",
        //     "workspaceDependencies": [],
        //     "mismatchedWorkspaceDependencies": []
        //   }
        // }
        // ✨  Done in 0.02s.

        #[derive(serde::Deserialize, Debug)]
        struct Workspace {
            // Relation path to the package
            location: String,
        }
        let mut workspaces: Vec<Workspace> = vec![];
        // output is like.
        // "yarn workspaces v1.22.22\n{\n  \"app1\": {\n    \"location\": \"packages/app\",\n    \"workspaceDependencies\": [],\n    \"mismatchedWorkspaceDependencies\": []\n  }\n}\nDone in 0.01s.\n"
        let output = String::from_utf8(output.stdout)?;
        let lines = output.split("\n").collect::<Vec<&str>>();
        let workspaces_json = {
            // remove the first and last line.
            match lines.get(1..(lines.len() - 2)) {
                // Vec<&str> into String
                Some(lines) => lines.join(""),
                None => return Err(anyhow!("unexpected output")),
            }
        };

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
            .map(|w| {
                PathBuf::from(w.location.clone())
                    .join(js::METADATA_FILE_NAME)
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<String>>())
    }

    // get_workspaces_list parses the result of `yarn workspaces list --json` and return path of `package.json` of each package.
    fn get_workspace_packages_for_v2_or_later() -> Result<Vec<String>> {
        let output = process::Command::new("yarn")
            .arg("workspaces")
            .arg("list")
            .arg("--json")
            .output()?;

        #[derive(serde::Deserialize, Debug)]
        struct Workspace {
            // Relation path to the package
            location: String,
        }
        let mut workspaces: Vec<Workspace> = vec![];
        // output is like.
        // "{\"location\":\".\",\"name\":\"project\"}\n{\"location\":\"packages/app1\",\"name\":\"app1\"}\n"
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
            .map(|w| {
                PathBuf::from(w.location.clone())
                    .join(js::METADATA_FILE_NAME)
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<String>>())
    }
}
