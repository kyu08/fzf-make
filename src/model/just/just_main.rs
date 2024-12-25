use crate::model::{
    command::{self, Command},
    runner_type::RunnerType,
};
use anyhow::{anyhow, bail, Result};
use std::{
    fs::{self},
    path::PathBuf,
    process,
};
use tree_sitter::Parser;

#[derive(Debug, Clone, PartialEq)]
pub struct Just {
    path: PathBuf,
    commands: Vec<command::Command>,
}

impl Just {
    pub fn new(current_dir: PathBuf) -> Result<Just> {
        let justfile_path = match Just::find_justfile(current_dir.clone()) {
            Some(path) => path,
            None => bail!("justfile not found"),
        };
        let source_code = fs::read_to_string(&justfile_path)?;

        Ok(Just {
            path: justfile_path.clone(),
            commands: Just::parse_justfile(justfile_path, source_code).unwrap_or_default(),
        })
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

    fn parse_justfile(justfile_path: PathBuf, source_code: String) -> Option<Vec<Command>> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_just::language()).unwrap();
        let tree = parser.parse(&source_code, None).unwrap();
        // source_file
        // ├── shebang
        // │   └── language
        // └── recipe (multiple))
        //     ├── recipe_header
        //     │   └── name: identifier
        //     ├── recipe_body
        //     │   └── recipe_line
        //     │       └── text
        //     └── attribute (multiple, optional)
        //         ├── identifier
        //         └── argument: string
        let mut commands = vec![];

        // At first, it seemed that it is more readable if we can use `Node#children_by_field_name` instead of `Node#children`.
        // But the elements wanted to be extracted here do not have names.
        // So we had no choice but to use `Node#children`.
        for recipes_and_its_siblings in tree.root_node().named_children(&mut tree.walk()) {
            if recipes_and_its_siblings.kind() == "recipe" {
                let mut should_skip = false;
                recipes_and_its_siblings
                    .children(&mut tree.walk())
                    .for_each(|attr| {
                        let attr_name = &source_code[attr.byte_range()];
                        if attr_name.contains("private") {
                            should_skip = true;
                        }
                    });
                if should_skip {
                    continue;
                }

                for recipe_child in recipes_and_its_siblings.named_children(&mut tree.walk()) {
                    if recipe_child.kind() == "recipe_header" {
                        // `recipe_name` has format like: `fmt:`
                        let recipe_name = &source_code[recipe_child.byte_range()];
                        let trimmed = recipe_name.split(":").collect::<Vec<&str>>();
                        match trimmed.first() {
                            Some(r) => commands.push(Command::new(
                                RunnerType::Just,
                                r.trim().to_string(),
                                justfile_path.clone(),
                                recipe_child.start_position().row as u32 + 1,
                            )),
                            None => continue,
                        };
                    };
                }
            }
        }

        if commands.is_empty() {
            None
        } else {
            Some(commands)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
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

    #[test]
    fn test_parse_justfile() {
        struct Case {
            name: &'static str,
            source_code: &'static str,
            expected: Option<Vec<Command>>,
        }
        let cases = vec![
            Case {
                name: "empty justfile",
                source_code: "",
                expected: None,
            },
            Case {
                name: "justfile with one recipe",
                source_code: r#"
#!/usr/bin/env -S just --justfile

test:
  cargo test --all

[group: 'misc']
run:
  echo run

[group: 'misc']
build:
  echo build

[group: 'misc']
fmt : # https://example.com
  echo fmt

[group: 'misc']
[private ]
fmt-private:
  echo fmt

# everyone's favorite animate paper clip
[group: 'check']
clippy:
  echo clippy
        "#,
                expected: Some(vec![
                    Command {
                        runner_type: RunnerType::Just,
                        args: "test".to_string(),
                        file_name: PathBuf::from("justfile"),
                        line_number: 4,
                    },
                    Command {
                        runner_type: RunnerType::Just,
                        args: "run".to_string(),
                        file_name: PathBuf::from("justfile"),
                        line_number: 8,
                    },
                    Command {
                        runner_type: RunnerType::Just,
                        args: "build".to_string(),
                        file_name: PathBuf::from("justfile"),
                        line_number: 12,
                    },
                    Command {
                        runner_type: RunnerType::Just,
                        args: "fmt".to_string(),
                        file_name: PathBuf::from("justfile"),
                        line_number: 16,
                    },
                    Command {
                        runner_type: RunnerType::Just,
                        args: "clippy".to_string(),
                        file_name: PathBuf::from("justfile"),
                        line_number: 26,
                    },
                ]),
            },
        ];

        for case in cases {
            let commands =
                Just::parse_justfile(PathBuf::from("justfile"), case.source_code.to_string());
            assert_eq!(commands, case.expected, "{}", case.name);
        }
    }
}
