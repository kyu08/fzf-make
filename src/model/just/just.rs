use crate::model::command;
use anyhow::{anyhow, bail, Result};
use std::{fs, path::PathBuf, process};
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Clone, PartialEq)]
pub struct Just {
    path: PathBuf,
    commands: Vec<command::Command>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // パーサーの設定
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_just::language())?;

    // Justfile の内容を読み込む
    let source_code = fs::read_to_string("Justfile")?;
    let tree = parser.parse(&source_code, None).unwrap();

    // レシピ定義を検索するクエリ
    let query = Query::new(
        &tree_sitter_just::language(),
        "(recipe_definition name: (identifier) @recipe_name)",
    )?;
    let mut cursor = QueryCursor::new();

    // クエリを実行してレシピ名と行番号を取得
    let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
    for match_ in matches {
        for capture in match_.captures {
            let node = capture.node;
            let start_line = node.start_position().row + 1;
            let recipe_name = &source_code[node.byte_range()];
            println!("Recipe '{}' at line {}", recipe_name, start_line);
        }
    }

    Ok(())
}

impl Just {
    pub fn new(current_dir: PathBuf) -> Result<Just> {
        let justfile_path = match Just::find_justfile(current_dir.clone()) {
            Some(path) => path,
            None => bail!("justfile not found"),
        };
        // tree-sitterを使ってパースしつつ行番号を取得
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

    fn find_justfile(current_dir: PathBuf) -> Option<PathBuf> {
        for path in current_dir.ancestors() {
            for entry in PathBuf::from(path).read_dir().unwrap() {
                let entry = entry.unwrap();
                let file_name = entry.file_name().to_string_lossy().to_lowercase();
                if file_name == "justfile" || file_name == ".justfile" {
                    return Some(entry.path());
                }
            }
        }
        None
    }
}

#[cfg(test)]
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
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let justfile_path = test_target_dir.join("justfile");
            std::fs::File::create(&justfile_path).unwrap();
            assert_eq!(Just::find_justfile(test_target_dir), Some(justfile_path));
        }

        // .justfile exists in temp_dir
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let justfile_path = test_target_dir.join(".justfile");
            std::fs::File::create(&justfile_path).unwrap();
            assert_eq!(Just::find_justfile(test_target_dir), Some(justfile_path));
        }

        // justfile exists in the one of ancestors of temp_dir
        {
            let parent = test_root_dir.join(Uuid::new_v4().to_string());
            let test_target_dir = parent.join("child_dir");
            std::fs::create_dir_all(&test_target_dir).unwrap();

            let justfile_path = parent.join("justfile");
            std::fs::File::create(&justfile_path).unwrap();
            assert_eq!(Just::find_justfile(test_target_dir), Some(justfile_path));
        }

        // no justfile exists
        {
            let parent = test_root_dir.join(Uuid::new_v4().to_string());
            let test_target_dir = parent.join("child_dir");
            std::fs::create_dir_all(&test_target_dir).unwrap();

            assert_eq!(Just::find_justfile(test_target_dir), None);
        }

        let _ = std::fs::remove_dir_all(&test_root_dir);
    }
}
