use super::command;
use anyhow::Result;
use std::{path::PathBuf, process};

#[derive(Clone, Debug, PartialEq)]
pub struct Pnpm {
    pub path: PathBuf,
    commands: Vec<command::Command>,
}

// JSの各パッケージマネージャ用に初期化コードをかくというよりは
// どのパッケージマネージャを使っているのか判定してそれをrunnerにappendするのがいいか？
impl Pnpm {
    pub fn command_to_run(command: &command::Command) -> String {
        format!("pnpm run {}", command.name)
    }

    pub fn new(current_dir: PathBuf) -> Result<Pnpm> {
        // TODO: ここ実装する
        // TODO: package.jsonのパース処理は共通化したほうがよさそう
        todo!("implement")
    }

    // // I gave up writing tests using temp_dir because it was too difficult (it was necessary to change the implementation to some extent).
    // // It is not difficult to ensure that it works with manual tests, so I will not do it for now.
    // fn new_internal(path: PathBuf) -> Result<Pnpm> {
    //     // If the file path does not exist, the make command cannot be executed in the first place,
    //     // so it is not handled here.
    //     let file_content = file_util::path_to_content(path.clone())?;
    //     let include_files = content_to_include_file_paths(file_content.clone())
    //         .iter()
    //         .map(|included_file_path| Pnpm::new_internal(included_file_path.clone()))
    //         .filter_map(Result::ok)
    //         .collect();
    //
    //     Ok(Pnpm {
    //         path: path.clone(),
    //         include_files,
    //         targets: Targets::new(file_content, path),
    //     })
    // }

    // pub fn to_commands(&self) -> Vec<command::Command> {
    //     let mut result: Vec<command::Command> = vec![];
    //     result.append(&mut self.targets.0.to_vec());
    //     for include_file in &self.include_files {
    //         Vec::append(&mut result, &mut include_file.to_commands());
    //     }
    //
    //     result
    // }

    // fn specify_makefile_name(current_dir: PathBuf, target_path: String) -> Option<PathBuf> {
    //     //  By default, when make looks for the makefile, it tries the following names, in order: GNUmakefile, makefile and Pnpmfile.
    //     //  https://www.gnu.org/software/make/manual/make.html#Pnpmfile-Names
    //     // It needs to enumerate `Pnpmfile` too not only `makefile` to make it work on case insensitive file system
    //     let makefile_name_options = ["GNUmakefile", "makefile", "Pnpmfile"];
    //
    //     let mut temp_result = Vec::<PathBuf>::new();
    //     let elements = fs::read_dir(target_path.clone()).unwrap();
    //     for e in elements {
    //         let file_name = e.unwrap().file_name();
    //         let file_name_string = file_name.to_str().unwrap();
    //         if makefile_name_options.contains(&file_name_string) {
    //             temp_result.push(current_dir.join(file_name));
    //         }
    //     }
    //
    //     // It needs to return "GNUmakefile", "makefile", "Pnpmfile" in order of priority
    //     for makefile_name_option in makefile_name_options {
    //         for result in &temp_result {
    //             if result.to_str().unwrap().contains(makefile_name_option) {
    //                 return Some(result.clone());
    //             }
    //         }
    //     }
    //
    //     None
    // }

    // pub fn execute(&self, command: &command::Command) -> Result<()> {
    //     process::Command::new("pnpm")
    //         .stdin(process::Stdio::inherit())
    //         .arg("run")
    //         .arg(&command.name)
    //         .spawn()
    //         .expect("Failed to execute process")
    //         .wait()
    //         .expect("Failed to execute process");
    //     Ok(())
    // }
}
