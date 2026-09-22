use super::js_package_manager_main as js;
use crate::model::runner_type;
use anyhow::{Result, anyhow};
use std::{path::PathBuf, process};

const LOCKFILE_NAME: &str = "package-lock.json";

#[derive(Debug)]
pub(super) struct Npm;

impl js::PackageManager for Npm {
    fn runner_type(&self) -> runner_type::RunnerType {
        runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Npm)
    }

    fn program(&self) -> &'static str {
        "npm"
    }

    fn lockfile_names(&self) -> Vec<&'static str> {
        vec![LOCKFILE_NAME]
    }

    // npm executes a script following format: `npm run {script_name}`
    fn root_script_args(&self, script_name: &str) -> String {
        format!("run {}", script_name)
    }

    // npm executes a workspace script following format: `npm run {script_name} --workspace={package_name}`
    // e.g. `npm run build --workspace=app1`
    fn workspace_script_args(&self, package_name: &str, script_name: &str) -> String {
        format!("run {} --workspace={}", script_name, package_name)
    }

    // workspace_package_json_paths uses `npm query .workspace` to get workspace package.json paths.
    // This requires npm 8.16.0+ but provides a clean JSON output.
    fn workspace_package_json_paths(&self) -> Result<Vec<PathBuf>> {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use js::PackageManager;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_script_args() {
        assert_eq!("run build", Npm.root_script_args("build"));
        assert_eq!("run build --workspace=app1", Npm.workspace_script_args("app1", "build"));
    }
}
