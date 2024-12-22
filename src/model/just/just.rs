use crate::model::command;
use anyhow::{anyhow, bail, Result};
use std::{path::PathBuf, process};

#[derive(Debug, Clone, PartialEq)]
pub struct Just {
    path: PathBuf,
    commands: Vec<command::Command>,
}
impl Just {
    // TODO: add new
    pub fn new(current_dir: PathBuf) -> Result<Just> {
        // TODO: justではjustfileの子ディレクトリでもjust testのように実行できる。
        // 子ディレクトリでもfzf-makeを実行できるためにはjustfileのパスを取得する必要がある。
        // 現状justコマンドでこれをする方法が見つからなかったため、親方向にgit rootまで調べるくらいしか方法がないかもしれない。(git管理されてなかったらエラーにする)
        //
        // あとはひとまずjustfileが存在するディレクトリでの実行だけをサポートして、それから子ディレクトリでの実行をサポートするという手があるかも知れない。
        //
        // just --dumpでjustfileの内容を取得
        // tree-sitterを使ってパースしつつ行番号を取得
        // tmp_fileに保存してそのpathをcommandに格納する
        bail!("not implemented")
    }

    pub fn to_commands(&self) -> Vec<command::Command> {
        self.commands.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn command_to_run(&self, command: &command::Command) -> Result<String, anyhow::Error> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        Ok(format!("just {}", command.args))
    }

    pub fn execute(&self, command: &command::Command) -> Result<(), anyhow::Error> {
        let command = match self.get_command(command.clone()) {
            Some(c) => c,
            None => return Err(anyhow!("command not found")),
        };

        let child = process::Command::new("just")
            .stdin(process::Stdio::inherit())
            .arg(&command.args)
            .spawn();

        match child {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("failed to run: {}", e)),
            },
            Err(e) => Err(anyhow!("failed to spawn: {}", e)),
        }
    }

    fn get_command(&self, command: command::Command) -> Option<command::Command> {
        self.to_commands()
            .iter()
            .find(|c| **c == command)
            .map(|_| command)
    }

    fn find_justfile(current_dir: &PathBuf) -> Option<PathBuf> {
        // TODO: current_dirの親ディレクトリを取得
        // NOTE: current_dirは絶対パス
        let parent_dir = current_dir.ancestors()?;
        // TODO: justfileが見つかるまで子 -> 親方向に走査
        None
    }
}

mod test {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_find_justfile() {
        // cleanup before test
        let test_root_dir = std::env::temp_dir().join("fzf_make_test");
        // error will be returned if the directory does not exist.
        let _ = std::fs::remove_dir_all(&test_root_dir);
        std::fs::create_dir(&test_root_dir).unwrap();

        // justfile exists in temp_dir
        {
            let random_dir_name = Uuid::new_v4().to_string();
            let test_target_dir = test_root_dir.join(random_dir_name);
            std::fs::create_dir(&test_target_dir).unwrap();

            let justfile_path = test_target_dir.join("justfile");
            std::fs::File::create(&justfile_path).unwrap();
            assert_eq!(Just::find_justfile(&test_target_dir), Some(justfile_path));
        }
        let _ = std::fs::remove_dir_all(&test_root_dir);
    }
}
