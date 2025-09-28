use crate::model::{
    command::{self, CommandWithPreview},
    file_util,
    runner_type::RunnerType,
};
use anyhow::{Result, anyhow, bail};
use std::{
    fs::{self},
    path::PathBuf,
    process,
};
use tree_sitter::Parser;

#[derive(Debug, Clone, PartialEq)]
pub struct Just {
    // path represents the path to the justfile.
    path: PathBuf,
    // Just can declare modules to import by `mod` directive.
    // ref: https://github.com/casey/just#modules1190
    modules: Vec<Module>,
    commands: Vec<command::CommandWithPreview>,
}

#[derive(Debug, Clone, PartialEq)]
struct Module {
    mod_name: String,
    mod_path: Option<PathBuf>,
    content: Just,
}

impl Just {
    pub fn new(current_dir: PathBuf) -> Result<Just> {
        let justfile_path = match Just::find_justfile(current_dir.clone()) {
            Some(path) => path,
            None => bail!("justfile not found"),
        };
        let source_code = fs::read_to_string(&justfile_path)?;

        match Just::parse_justfile(justfile_path.clone(), source_code) {
            Some(c) => Ok(c),
            None => Err(anyhow!("failed to parse justfile")),
        }
    }

    pub fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        self.commands.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn command_to_run(&self, command: &command::CommandForExec) -> Result<String, anyhow::Error> {
        Ok(format!("just {}", command.args))
    }

    pub fn execute(&self, command: &command::CommandForExec) -> Result<(), anyhow::Error> {
        let child = process::Command::new("just")
            .stdin(process::Stdio::inherit())
            .args(command.args.split_whitespace())
            .spawn();

        match child {
            Ok(mut child) => match child.wait() {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("failed to run: {}", e)),
            },
            Err(e) => Err(anyhow!("failed to spawn: {}", e)),
        }
    }

    fn find_justfile(current_dir: PathBuf) -> Option<PathBuf> {
        file_util::find_file_in_ancestors(current_dir, vec!["justfile", ".justfile"])
    }

    fn parse_justfile(justfile_path: PathBuf, source_code: String) -> Option<Just> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_just::language()).unwrap();
        let tree = parser.parse(&source_code, None).unwrap();
        // source_file
        // ├── shebang
        // │   └── language
        // ├── module
        //     └── name: identifier
        //     └── string
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
        let mut modules = vec![];

        // At first, it seemed that it is more readable if we can use `Node#children_by_field_name` instead of `Node#children`.
        // But the elements wanted to be extracted here do not have names.
        // So we had no choice but to use `Node#children`.
        'recipe: for recipes_and_its_siblings in tree.root_node().named_children(&mut tree.walk()) {
            if recipes_and_its_siblings.kind() == "module" {
                // should parse this: `(module name: (identifier) (string))`
                let mut mod_name = String::new();
                let mut mod_path: Option<PathBuf> = None;

                // If field name doesn't work, iterate through children
                if mod_name.is_empty() {
                    for recipe_child in recipes_and_its_siblings.named_children(&mut tree.walk()) {
                        match recipe_child.kind() {
                            "identifier" => {
                                mod_name = source_code[recipe_child.byte_range()].to_string();
                            }
                            "string" => {
                                let raw_path = &source_code[recipe_child.byte_range()];
                                // Remove surrounding quotes
                                mod_path =
                                    Some(PathBuf::from(raw_path.trim_matches(|c| c == '\'' || c == '"').to_string()));
                            }
                            _ => {}
                        }
                    }
                }

                // TODO: ここで再帰的に module の中身をパースしてmodulesにpushする。
                // mod_nameとmod_pathから可能性のあるpathを優先度順に生成する
                // for loopで先頭から試していく
                for path in Just::calc_justfile_path_from_mod_info(mod_name.clone(), mod_path) {
                    // TODO: fileが見つからなかったらcontinueする
                    let file_content = String::from("TODO: retrieve actual file content from the path");
                    if let Some(j) = Just::parse_justfile(path.clone(), file_content) {
                        modules.push(Module {
                            mod_name: mod_name.clone(),
                            // TODO: pathは構造体のにフィールドから削除してもいいかも
                            mod_path: Some(path),
                            content: j,
                        })
                    };
                }
            }
            if recipes_and_its_siblings.kind() == "recipe" {
                let mut should_skip = false;
                recipes_and_its_siblings.children(&mut tree.walk()).for_each(|attr| {
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
                        let trimmed = recipe_name.split(':').collect::<Vec<&str>>();
                        if let Some(r) = trimmed.first() {
                            // If recipe includes an argument, it will be like `run arg:`.
                            // So we need to split it by space and take the first element.
                            let command_name = r.split_whitespace().next().unwrap_or("").to_string();

                            commands.push(CommandWithPreview::new(
                                RunnerType::Just,
                                command_name,
                                justfile_path.clone(),
                                recipe_child.start_position().row as u32 + 1,
                            ))
                        };
                        continue 'recipe;
                    };
                }
            }
        }

        if commands.is_empty() {
            None
        } else {
            Some(Just {
                path: justfile_path,
                modules,
                commands,
            })
        }
    }

    fn calc_justfile_path_from_mod_info(mod_name: String, mod_path: Option<PathBuf>) -> Vec<PathBuf> {
        match mod_path {
            Some(mod_path) => {
                // TODO: mod_pathありのパターン
                vec![]
            }
            None => {
                // TODO: mod_nameのみのパターン
                vec![]
            }
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
            expected: Option<Just>,
        }
        let cases = vec![
            Case {
                name: "empty justfile",
                source_code: "",
                expected: None,
            },
            Case {
                name: "justfile with multiple recipes",
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
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    modules: vec![],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "test".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 4,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 8,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 12,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "fmt".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 16,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "clippy".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 26,
                        },
                    ],
                }),
            },
            Case {
                name: "justfile with recipes including a recipe with argument",
                source_code: r#"#!/usr/bin/env -S just --justfile

[group: 'misc']
run arg:
  echo "run {{arg}}"

[group: 'misc']
build:
  echo build
        "#,
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    modules: vec![],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 4,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 8,
                        },
                    ],
                }),
            },
        ];

        for case in cases {
            assert_eq!(
                Just::parse_justfile(PathBuf::from("justfile"), case.source_code.to_string()),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn test_parse_justfile_mod() {
        struct Case {
            name: &'static str,
            source_code: &'static str,
            expected: Option<Just>,
        }
        let cases = vec![
            Case {
                name: "justfile with `mod mod_name`",
                source_code: r#"#!/usr/bin/env -S just --justfile

mod other1

[group: 'misc']
run arg:
  echo "run {{arg}}"

[group: 'misc']
build:
  echo build
        "#,
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    modules: vec![Module {
                        mod_name: String::from("other1"),
                        mod_path: None,
                    }],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 6,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 10,
                        },
                    ],
                }),
            },
            Case {
                name: "justfile with `mod? mod_name`",
                source_code: r#"#!/usr/bin/env -S just --justfile

mod? other1

[group: 'misc']
run arg:
  echo "run {{arg}}"

[group: 'misc']
build:
  echo build
        "#,
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    modules: vec![Module {
                        mod_name: String::from("other1"),
                        mod_path: None,
                    }],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 6,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 10,
                        },
                    ],
                }),
            },
            Case {
                name: "justfile with `mod mod_name mod_path`",
                source_code: r#"#!/usr/bin/env -S just --justfile

mod other1 './other1.just'

[group: 'misc']
run arg:
  echo "run {{arg}}"

[group: 'misc']
build:
  echo build
        "#,
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    modules: vec![Module {
                        mod_name: String::from("other1"),
                        mod_path: Some(String::from("./other1.just")),
                    }],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 6,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 10,
                        },
                    ],
                }),
            },
            Case {
                name: "justfile with `mod? mod_name mod_path`",
                source_code: r#"#!/usr/bin/env -S just --justfile

mod? other1 './other1.just'

[group: 'misc']
run arg:
  echo "run {{arg}}"

[group: 'misc']
build:
  echo build
        "#,
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    modules: vec![Module {
                        mod_name: String::from("other1"),
                        mod_path: Some(String::from("./other1.just")),
                    }],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 6,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 10,
                        },
                    ],
                }),
            },
            Case {
                name: "justfile with `mod? mod_name mod_path` surrounded by double quotes",
                source_code: r#"#!/usr/bin/env -S just --justfile

mod? other1 "./other1.just"

[group: 'misc']
run arg:
  echo "run {{arg}}"

[group: 'misc']
build:
  echo build
        "#,
                expected: Some(Just {
                    path: PathBuf::from("justfile"),
                    // modules: vec![Module {
                    //     mod_name: String::from("other1"),
                    //     mod_path: Some(String::from("./other1.just")),
                    //     content: foo,
                    // }],
                    // TODO: Comment in above code. At the time, we should use temp file to test
                    // it.
                    modules: vec![],
                    commands: vec![
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "run".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 6,
                        },
                        CommandWithPreview {
                            runner_type: RunnerType::Just,
                            args: "build".to_string(),
                            file_path: PathBuf::from("justfile"),
                            line_number: 10,
                        },
                    ],
                }),
            },
        ];

        for case in cases {
            assert_eq!(
                1, 1,
                // Just::parse_justfile(PathBuf::from("justfile"), case.source_code.to_string()),
                // case.expected,
                "{}",
                case.name
            );
        }
    }
}
