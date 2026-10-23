use super::js_package_manager_main as js;
use crate::model::runner_type;
use anyhow::{Result, anyhow};
use std::{
    path::{Path, PathBuf},
    process,
};

const LOCKFILE_NAME: &str = "yarn.lock";

#[derive(Debug)]
pub(super) struct Yarn;

enum YarnVersion {
    V1,
    V2OrLater,
}

impl js::PackageManager for Yarn {
    fn runner_type(&self) -> runner_type::RunnerType {
        runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Yarn)
    }

    fn program(&self) -> &'static str {
        "yarn"
    }

    fn lockfile_names(&self) -> Vec<&'static str> {
        vec![LOCKFILE_NAME]
    }

    // yarn executes a script following format: `yarn {script_name}`
    fn root_script_args(&self, script_name: &str) -> String {
        script_name.to_string()
    }

    // yarn executes a workspace script following format: `yarn workspace {package_name} {script_name}`
    // e.g. `yarn workspace app4 build`
    fn workspace_script_args(&self, package_name: &str, script_name: &str) -> String {
        format!("workspace {} {}", package_name, script_name)
    }

    /// `yarn workspaces (info|list)` succeeds only when it is run inside a workspace, so ask yarn
    /// itself instead of looking for the lockfile in the ancestors.
    fn is_workspace_member(&self, _current_dir: &Path) -> bool {
        let output = match Self::get_yarn_version() {
            Some(version) => Self::run_workspaces_command(version),
            None => return false, // yarn is not installed
        };

        // If `yarn workspaces (info|list) --json` returns non-zero status code, it means that the
        // current directory is not a yarn workspace.
        output.map(|output| output.status.success()).unwrap_or(false)
    }

    fn workspace_package_json_paths(&self) -> Result<Vec<PathBuf>> {
        match Self::get_yarn_version() {
            Some(YarnVersion::V1) => Self::workspace_package_json_paths_for_v1(),
            Some(YarnVersion::V2OrLater) => Self::workspace_package_json_paths_for_v2_or_later(),
            None => Err(anyhow!("yarn is not installed")),
        }
    }
}

impl Yarn {
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

    fn run_workspaces_command(version: YarnVersion) -> std::io::Result<process::Output> {
        let subcommand = match version {
            YarnVersion::V1 => "info",
            YarnVersion::V2OrLater => "list",
        };

        process::Command::new("yarn")
            .arg("workspaces")
            .arg(subcommand)
            .arg("--json")
            .output()
    }

    // workspace_package_json_paths_for_v1 parses the result of `yarn workspaces info --json` and
    // returns the path of `package.json` of each package.
    fn workspace_package_json_paths_for_v1() -> Result<Vec<PathBuf>> {
        let output = Self::run_workspaces_command(YarnVersion::V1)?;
        /* Example output:
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

        if !output.status.success() {
            return Err(anyhow!("yarn workspaces info failed"));
        }

        #[derive(serde::Deserialize, Debug)]
        struct Workspace {
            // Relation path to the package
            location: String,
        }

        let workspaces_json = {
            let output = String::from_utf8(output.stdout)?;
            /* Example output:
            "yarn workspaces v1.22.22\n{\n  \"app1\": {\n    \"location\": \"packages/app\",\n    \"workspaceDependencies\": [],\n    \"mismatchedWorkspaceDependencies\": []\n  }\n}\nDone in 0.01s.\n"
            */

            // split by newline to remove unnecessary lines.
            let lines = output.split('\n').collect::<Vec<&str>>();

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

    // workspace_package_json_paths_for_v2_or_later parses the result of `yarn workspaces list --json`
    // and returns the path of `package.json` of each package.
    fn workspace_package_json_paths_for_v2_or_later() -> Result<Vec<PathBuf>> {
        let output = Self::run_workspaces_command(YarnVersion::V2OrLater)?;

        if !output.status.success() {
            return Err(anyhow!("yarn workspaces list failed"));
        }

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
            .map(|w| PathBuf::from(w.location.clone()).join(js::METADATA_FILE_NAME))
            .collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use js::PackageManager;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_script_args() {
        assert_eq!("build", Yarn.root_script_args("build"));
        assert_eq!("workspace app1 build", Yarn.workspace_script_args("app1", "build"));
    }
}
