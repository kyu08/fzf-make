use super::js_package_manager_main as js;
use crate::model::runner_type;
use anyhow::{Result, anyhow};
use std::{path::PathBuf, process, sync::OnceLock};

const LOCKFILE_NAME: &str = "pnpm-lock.yaml";

#[derive(Debug)]
pub(super) struct Pnpm;

impl js::PackageManager for Pnpm {
    fn runner_type(&self) -> runner_type::RunnerType {
        runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Pnpm)
    }

    fn program(&self) -> &'static str {
        "pnpm"
    }

    fn lockfile_names(&self) -> Vec<&'static str> {
        vec![LOCKFILE_NAME]
    }

    // pnpm executes a script following format: `pnpm {script_name}`
    fn root_script_args(&self, script_name: &str) -> String {
        script_name.to_string()
    }

    // pnpm executes a workspace script following format: `pnpm --filter {package_name} {script_name}`
    // e.g. `pnpm --filter app4 build`
    fn workspace_script_args(&self, package_name: &str, script_name: &str) -> String {
        format!("--filter {} {}", package_name, script_name)
    }

    fn should_skip_script(&self, script_name: &str, script_body: &str) -> bool {
        Self::use_filtering(script_body) || Self::is_hidden_script(script_name)
    }

    // workspace_package_json_paths parses the result of `pnpm -r exec pwd` and returns the path of
    // `package.json` of each package.
    fn workspace_package_json_paths(&self) -> Result<Vec<PathBuf>> {
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

        if !output.status.success() {
            return Err(anyhow!("pnpm -r exec pwd failed"));
        }

        let output = String::from_utf8(output.stdout)?;
        // split by newline to remove unnecessary lines.
        let lines = output.split('\n').filter(|l| !l.is_empty()).collect::<Vec<&str>>();

        Ok(lines
            .iter()
            .map(|line| PathBuf::from(line).join(js::METADATA_FILE_NAME))
            .collect())
    }
}

impl Pnpm {
    // ref: https://pnpm.io/filtering
    fn use_filtering(script_body: &str) -> bool {
        let args = script_body.split_whitespace().collect::<Vec<&str>>();

        let start_with_pnpm = args.first().map(|arg| *arg == "pnpm").unwrap_or(false);
        let has_filtering_or_dir_option = args
            .iter()
            .any(|arg| *arg == "-F" || *arg == "--filter" || *arg == "-C" || *arg == "--dir");
        let has_run = args.contains(&"run");

        start_with_pnpm && has_filtering_or_dir_option && !has_run
    }

    fn pnpm_version() -> &'static str {
        static VERSION: OnceLock<String> = OnceLock::new();
        VERSION.get_or_init(|| {
            process::Command::new("pnpm")
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
    }

    // ref: https://pnpm.io/scripts#hidden-scripts
    fn is_hidden_script(script_name: &str) -> bool {
        script_name.starts_with(".") && Self::is_pnpm_version_11_or_higher(Self::pnpm_version())
    }

    fn is_pnpm_version_11_or_higher(version: &str) -> bool {
        let major_version = version.split('.').next().unwrap_or("0");
        major_version.parse::<u32>().unwrap_or(0) >= 11
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use js::PackageManager;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_script_args() {
        assert_eq!("build", Pnpm.root_script_args("build"));
        assert_eq!("--filter app1 build", Pnpm.workspace_script_args("app1", "build"));
    }

    #[test]
    fn test_is_filtering() {
        assert_eq!(true, Pnpm::use_filtering("pnpm -F app1"));
        assert_eq!(true, Pnpm::use_filtering("pnpm -F \"app1\""));
        assert_eq!(true, Pnpm::use_filtering("pnpm --filter app2"));
        assert_eq!(true, Pnpm::use_filtering("pnpm -r --filter app3"));
        assert_eq!(true, Pnpm::use_filtering("pnpm -C packages/app3"));
        assert_eq!(true, Pnpm::use_filtering("pnpm --dir packages/app3"));
        assert_eq!(true, Pnpm::use_filtering("pnpm -F"));
        assert_eq!(true, Pnpm::use_filtering("pnpm --filter"));
        assert_eq!(false, Pnpm::use_filtering("pnpm -C packages/app1 run test"));
        assert_eq!(false, Pnpm::use_filtering("pnpm --filter app1 run test"));
        assert_eq!(false, Pnpm::use_filtering("yarn run"));
        assert_eq!(false, Pnpm::use_filtering("pnpm run"));
        assert_eq!(false, Pnpm::use_filtering("pnpm -r hoge"));
        assert_eq!(false, Pnpm::use_filtering("yarn -r --filter app3"));
    }

    #[test]
    fn test_is_pnpm_version_11_or_higher() {
        assert_eq!(false, Pnpm::is_pnpm_version_11_or_higher("7.0.0"));
        assert_eq!(false, Pnpm::is_pnpm_version_11_or_higher("10.9.9"));
        assert_eq!(true, Pnpm::is_pnpm_version_11_or_higher("11.0.0"));
        assert_eq!(true, Pnpm::is_pnpm_version_11_or_higher("11.0.1"));
        assert_eq!(true, Pnpm::is_pnpm_version_11_or_higher("12.0.0"));
    }
}
