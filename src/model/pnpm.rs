use crate::file::path_to_content;

use super::{command, js_package_manager, runner_type};
use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf, process};

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

    pub fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    fn get_command(&self, command: command::Command) -> Option<&command::Command> {
        self.commands.iter().find(|c| **c == command)
    }

    pub fn new(path: PathBuf, commands: Vec<command::Command>) -> Pnpm {
        Pnpm { path, commands }
    }

    pub fn use_pnpm(file_name: String) -> bool {
        file_name == PNPM_LOCKFILE_NAME
    }

    pub fn scripts_to_commands(
        current_dir: PathBuf,
        scripts_parsing_result: (String, Vec<(String, String, u32)>),
    ) -> Vec<command::Command> {
        let mut result = vec![];

        let (_, object) = scripts_parsing_result;
        for (key, value, line_number) in object {
            if Pnpm::use_filtering(value.clone()) {
                continue;
            }

            // MEMO: If needed, -C option may be considered.
            // ref: https://pnpm.io/pnpm-cli#-c-path---dir-path

            // normal command
            result.push(command::Command::new(
                runner_type::RunnerType::JsPackageManager(runner_type::JsPackageManager::Pnpm),
                key,
                current_dir.clone().join("package.json"),
                line_number,
            ));
        }

        // pnpm-workspace.yamlで指定されていた場合はpackages配下が対象になる。指定されていない場合は全てのpackage.jsonが対象になる。
        // ネストは考慮しなくてよい`./packages/app1/package.json`は見る必要があるが、`./packages/app1/app2/package.json`は見る必要がない。
        // filteringにかかわらずすべてのpackage.jsonからscriptsを収集する必要がある
        // `pnpm --filter app1 run`のような形式で実行された場合は`pnpm-workspace.yaml`の`packages`や
        // `package.json`の`scripts`に`"app1": "pnpm -F \"app1\"", `に指定する必要がないため。

        // TODO: pnpm-workspace.yamlを考慮しない実装
        // TODO: node_modules以外のディレクトリを再帰的に検索する(2つしたまででOK)
        // あとでけすNOTE: ネストは考慮しなくてよい`./packages/app1/package.json`は見る必要があるが、`./packages/app1/app2/package.json`は見る必要がない。
        let skip =
            |entry: &fs::DirEntry| entry.path().is_file() || entry.file_name() == "node_modules";

        let entries_cwd = fs::read_dir(current_dir.clone()).unwrap();
        entries_cwd.for_each(|entry_cwd| {
            if let Ok(entry_in_cwd) = entry_cwd {
                if skip(&entry_in_cwd) {
                    return;
                }
                // ↑ ./packages
                let entries_of_packages = fs::read_dir(entry_in_cwd.path()).unwrap();
                entries_of_packages.for_each(|entry_package| {
                    // app1
                    if let Ok(entry_package) = entry_package {
                        if skip(&entry_package) {
                            return;
                        }

                        let entries_of_each_package = fs::read_dir(entry_package.path()).unwrap();
                        // ↑ app1/の中身
                        entries_of_each_package.for_each(|entry_of_each_package| {
                            if let Ok(entry_of_each_package) = entry_of_each_package {
                                if entry_of_each_package.file_name() != "package.json" {
                                    return;
                                }
                                if let Ok(c) =
                                    path_to_content::path_to_content(entry_of_each_package.path())
                                {
                                    if let Some((name, parsing_result)) =
                                        js_package_manager::JsPackageManager::parse_package_json(&c)
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

        // TODO: pnpm-workspace.yamlを考慮した実装
        result
    }

    // is filtering used
    // ref: https://pnpm.io/filtering
    fn use_filtering(value: String) -> bool {
        let args = value.split_whitespace().collect::<Vec<&str>>();
        match (
            args.iter().enumerate().find(|(_, e)| **e == "-F"),
            args.iter().enumerate().find(|(_, e)| **e == "--filter"),
        ) {
            (Some((index, _)), _) | (_, Some((index, _))) => args.get(index + 1).is_some(),
            _ => false,
        }
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
        assert_eq!(false, Pnpm::use_filtering("pnpm --filter".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm -F".to_string()));
        assert_eq!(false, Pnpm::use_filtering("yarn run".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm run".to_string()));
        assert_eq!(false, Pnpm::use_filtering("pnpm -r hoge".to_string()));
    }
}
