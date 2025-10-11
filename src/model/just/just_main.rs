use crate::model::{
    command::{self, CommandWithPreview},
    file_util,
    runner_type::RunnerType,
};
use anyhow::{Result, anyhow, bail};
use std::{
    fs::{self, File},
    path::PathBuf,
    process,
};
use tree_sitter::Parser;

const JUSTFILE_EXTENSION: &str = "just";
const JUSTFILE_NAME_MOD_JUST: &str = "mod.just";
const JUSTFILE_NAME_JUSTFILE: &str = "justfile";
const JUSTFILE_NAME_DOT_JUSTFILE: &str = ".justfile";

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
    content: Just,
}

#[derive(PartialEq, Debug)]
struct PossiblePaths {
    current: PathBuf,
    child: Vec<PathBuf>,
}

impl Just {
    pub fn new(current_dir: PathBuf) -> Result<Just> {
        let justfile_path = match Just::find_justfile(current_dir.clone()) {
            Some(path) => path,
            None => bail!("justfile not found"),
        };
        let source_code = fs::read_to_string(&justfile_path)?;

        match Just::parse_justfile(current_dir, justfile_path.clone(), source_code) {
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

    fn parse_justfile(current_dir: PathBuf, justfile_path: PathBuf, source_code: String) -> Option<Just> {
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
        let modules = vec![];

        let _ = File::create("debug_info.txt");
        // At first, it seemed that it is more readable if we can use `Node#children_by_field_name` instead of `Node#children`.
        // But the elements wanted to be extracted here do not have names.
        // So we had no choice but to use `Node#children`.
        'recipe: for recipes_and_its_siblings in tree.root_node().named_children(&mut tree.walk()) {
            if recipes_and_its_siblings.kind() == "module" {
                // Parse `mod` directive.
                // Its format is like `(module name: (identifier) (string))`.
                let mut mod_name = String::new();
                let mut mod_path: Option<PathBuf> = None;
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

                // Retrieve the justfiles for the modules recursively.
                if let Some(_path) = Just::get_mod_file_path(current_dir.clone(), mod_name.clone(), mod_path) {
                    // TODO: ファイル内容を取得する
                }
            }

            // Retrieve recipe names.
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

    fn calc_possible_justfile_path_from_mod_info(
        current_dir: PathBuf,
        mod_name: String,
        mod_path: &Option<PathBuf>,
    ) -> PossiblePaths {
        // Helper function to convert a path to absolute path
        let to_absolute = |path: PathBuf| -> PathBuf {
            if path.is_absolute() {
                path
            } else {
                let joined = current_dir.join(&path);
                fs::canonicalize(&joined).unwrap_or(joined)
            }
        };

        match mod_path {
            Some(mod_path) => {
                // Both mod_name and mod_path are specified. e.g. mod backend "./api"
                let base_path = to_absolute(PathBuf::from(&mod_path));
                PossiblePaths {
                    current: base_path.clone(),
                    child: vec![
                        base_path.join(JUSTFILE_NAME_MOD_JUST),
                        base_path.join(JUSTFILE_NAME_JUSTFILE),
                        base_path.join(JUSTFILE_NAME_DOT_JUSTFILE),
                    ],
                }
            }
            None => {
                // Only mod_name is specified. e.g. `mod backend`
                let base_path = to_absolute(PathBuf::from(&mod_name));
                let current_with_ext = base_path.with_extension(JUSTFILE_EXTENSION);
                PossiblePaths {
                    current: fs::canonicalize(&current_with_ext).unwrap_or(current_with_ext),
                    child: vec![
                        base_path.join(JUSTFILE_NAME_MOD_JUST),
                        base_path.join(JUSTFILE_NAME_JUSTFILE),
                        base_path.join(JUSTFILE_NAME_DOT_JUSTFILE),
                    ],
                }
            }
        }
    }

    // get_mod_file_path retrieves the justfile path for the specified module.
    //
    // If PATH is specified(e.g. mod foo ./bar), justfile path will be like below:
    // 1. {PATH}(if it's a file path)
    // 2. {PATH}/mod.just
    // 3. {PATH}/justfile
    // 4. {PATH}/.justfile
    //
    // If PATH is not specified(e.g. mod foo), justfile path will be like below:
    // 1. ./foo.just
    // 2. ./foo/mod.just
    // 3. ./foo/justfile
    // 4. ./foo/.justfile
    fn get_mod_file_path(current_dir: PathBuf, mod_name: String, mod_path: Option<PathBuf>) -> Option<PathBuf> {
        let possible_paths =
            Just::calc_possible_justfile_path_from_mod_info(current_dir.clone(), mod_name.clone(), &mod_path);

        // Check if possible_paths.current is an existing file.
        // This is for the 1. mentioned above in both patterns.
        if let Ok(metadata) = fs::metadata(&possible_paths.current) {
            if metadata.is_file() {
                return Some(possible_paths.current);
            }
        }

        // This is for the 2. - 4. mentioned above in both patterns.
        let target_path = match mod_path.clone() {
            // When PATH is specified.
            Some(mod_path) => {
                let joined = current_dir.join(mod_path);
                fs::canonicalize(&joined).unwrap_or(joined)
            }
            // When PATH is not specified.
            None => {
                let joined = current_dir.join(&mod_name);
                fs::canonicalize(&joined).unwrap_or(joined)
            }
        };
        if let Ok(metadata) = fs::metadata(&target_path) {
            // metadata should be a directory.
            if metadata.is_dir() {
                // If it's a directory, search for justfile inside
                if let Ok(elements) = fs::read_dir(target_path.clone()) {
                    for e in elements {
                        // justfile and .justfile are case-insensitive.
                        // Though mod.just is case-sensitive, it is not problem because it is
                        // already lowercase.
                        let file_name_string = e.unwrap().file_name().to_str().unwrap().to_lowercase();
                        for possible_path in &possible_paths.child {
                            if file_name_string == possible_path.file_name().unwrap().to_str().unwrap() {
                                let found_path = target_path.join(file_name_string);
                                return Some(fs::canonicalize(&found_path).unwrap_or(found_path));
                            }
                        }
                    }
                }
            }
        }
        None
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
                Just::parse_justfile(
                    // TODO: fix
                    PathBuf::from("FIXME_PATH"),
                    PathBuf::from("justfile"),
                    case.source_code.to_string()
                ),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn test_parse_justfile_retrieve_mod_recursively() {
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
                Just::parse_justfile(
                    // TODO: fix
                    PathBuf::from("FIXME_PATH"),
                    PathBuf::from("justfile"),
                    case.source_code.to_string()
                ),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn test_parse_justfile_parse_mod_directive() {
        // struct Case {
        //     name: &'static str,
        //     source_code: &'static str,
        //     expected: Option<Just>,
        // }
        // let cases: Vec<_> = vec![];

        // for case in cases {
        //     // assert_eq!(
        //     //     Just::parse_justfile(PathBuf::from("justfile"), case.source_code.to_string()),
        //     //     case.expected,
        //     //     "{}",
        //     //     case.name
        //     // );
        // }
    }

    #[test]
    fn test_calc_justfile_path_from_mod_info() {
        struct Case {
            name: &'static str,
            current_dir: &'static str,
            mod_name: &'static str,
            mod_path: Option<PathBuf>,
            expected: PossiblePaths,
        }

        let cases = vec![
            Case {
                name: "mod_path == None",
                current_dir: "/tmp/test_dir",
                mod_name: "backend",
                mod_path: None,
                expected: PossiblePaths {
                    current: PathBuf::from("/tmp/test_dir/backend.just"),
                    child: vec![
                        PathBuf::from("/tmp/test_dir/backend/mod.just"),
                        PathBuf::from("/tmp/test_dir/backend/justfile"),
                        PathBuf::from("/tmp/test_dir/backend/.justfile"),
                    ],
                },
            },
            Case {
                name: "mod_path == Some(rel_path)",
                current_dir: "/tmp/test_dir",
                mod_name: "backend",
                mod_path: Some(PathBuf::from("./backend")),
                expected: PossiblePaths {
                    current: PathBuf::from("/tmp/test_dir/backend"),
                    child: vec![
                        PathBuf::from("/tmp/test_dir/backend/mod.just"),
                        PathBuf::from("/tmp/test_dir/backend/justfile"),
                        PathBuf::from("/tmp/test_dir/backend/.justfile"),
                    ],
                },
            },
            Case {
                name: "mod_path == Some(abs_path)",
                current_dir: "/tmp/test_dir",
                mod_name: "backend",
                mod_path: Some(PathBuf::from("/Users/user/backend")),
                expected: PossiblePaths {
                    current: PathBuf::from("/Users/user/backend"),
                    child: vec![
                        PathBuf::from("/Users/user/backend/mod.just"),
                        PathBuf::from("/Users/user/backend/justfile"),
                        PathBuf::from("/Users/user/backend/.justfile"),
                    ],
                },
            },
        ];

        for case in cases {
            assert_eq!(
                Just::calc_possible_justfile_path_from_mod_info(
                    PathBuf::from(case.current_dir),
                    case.mod_name.to_string(),
                    &case.mod_path,
                ),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn test_get_mod_file_path() {
        // Setup test directory
        let test_root_dir = std::env::temp_dir().join("fzf_make_test_get_mod_file_path");
        let _ = std::fs::remove_dir_all(&test_root_dir);
        std::fs::create_dir_all(&test_root_dir).unwrap();

        // Test case 1: mod_path is None, and backend.just exists
        {
            let backend_just = test_root_dir.join("backend.just");
            std::fs::File::create(&backend_just).unwrap();

            let result = Just::get_mod_file_path(test_root_dir.clone(), "backend".to_string(), None);
            let expected = fs::canonicalize(&backend_just).unwrap_or(backend_just);
            assert_eq!(result, Some(expected));
        }

        // Test case 2: mod_path is None, backend.just doesn't exist, but backend/mod.just exists
        {
            let test_dir = test_root_dir.join("test_case_2");
            std::fs::create_dir_all(&test_dir).unwrap();

            let backend_dir = test_dir.join("backend");
            std::fs::create_dir_all(&backend_dir).unwrap();
            let mod_just = backend_dir.join("mod.just");
            std::fs::File::create(&mod_just).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), None);
            let expected = fs::canonicalize(&mod_just).unwrap_or(mod_just);
            assert_eq!(result, Some(expected));
        }

        // Test case 3: mod_path is None, backend.just doesn't exist, but backend/justfile exists
        {
            let test_dir = test_root_dir.join("test_case_3");
            std::fs::create_dir_all(&test_dir).unwrap();

            let backend_dir = test_dir.join("backend");
            std::fs::create_dir_all(&backend_dir).unwrap();
            let justfile = backend_dir.join("justfile");
            std::fs::File::create(&justfile).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), None);
            let expected = fs::canonicalize(&justfile).unwrap_or(justfile);
            assert_eq!(result, Some(expected));
        }

        // Test case 4: mod_path is None, backend.just doesn't exist, but backend/.justfile exists
        {
            let test_dir = test_root_dir.join("test_case_4");
            std::fs::create_dir_all(&test_dir).unwrap();

            let backend_dir = test_dir.join("backend");
            std::fs::create_dir_all(&backend_dir).unwrap();
            let dot_justfile = backend_dir.join(".justfile");
            std::fs::File::create(&dot_justfile).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), None);
            let expected = fs::canonicalize(&dot_justfile).unwrap_or(dot_justfile);
            assert_eq!(result, Some(expected));
        }
        // TODO:  case-insensitiveであることを確認？

        // Test case 5: mod_path is Some(relative path), and the path is a file
        {
            let test_dir = test_root_dir.join("test_case_5");
            std::fs::create_dir_all(&test_dir).unwrap();

            let api_just = test_dir.join("api.just");
            std::fs::File::create(&api_just).unwrap();

            let result =
                Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), Some(PathBuf::from("./api.just")));
            assert_eq!(result, Some(fs::canonicalize(&api_just).unwrap()));
        }

        // Test case 6: mod_path is Some(relative path), and the path is a directory with mod.just
        {
            let test_dir = test_root_dir.join("test_case_6");
            std::fs::create_dir_all(&test_dir).unwrap();

            let api_dir = test_dir.join("api");
            std::fs::create_dir_all(&api_dir).unwrap();
            let mod_just = api_dir.join("mod.just");
            std::fs::File::create(&mod_just).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), Some(PathBuf::from("./api")));
            let expected = fs::canonicalize(&mod_just).unwrap_or(mod_just);
            assert_eq!(result, Some(expected));
        }

        // Test case 7: mod_path is Some(relative path), and the path is a directory with justfile
        {
            let test_dir = test_root_dir.join("test_case_7");
            std::fs::create_dir_all(&test_dir).unwrap();

            let api_dir = test_dir.join("api");
            std::fs::create_dir_all(&api_dir).unwrap();
            let justfile = api_dir.join("justfile");
            std::fs::File::create(&justfile).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), Some(PathBuf::from("./api")));
            let expected = fs::canonicalize(&justfile).unwrap_or(justfile);
            assert_eq!(result, Some(expected));
        }

        // Test case 8: mod_path is Some(relative path), and the path is a directory with .justfile
        {
            let test_dir = test_root_dir.join("test_case_8");
            std::fs::create_dir_all(&test_dir).unwrap();

            let api_dir = test_dir.join("api");
            std::fs::create_dir_all(&api_dir).unwrap();
            let dot_justfile = api_dir.join(".justfile");
            std::fs::File::create(&dot_justfile).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), Some(PathBuf::from("./api")));
            let expected = fs::canonicalize(&dot_justfile).unwrap_or(dot_justfile);
            assert_eq!(result, Some(expected));
        }

        // Test case 9: case-insensitive filename - Justfile (capital J)
        {
            let test_dir = test_root_dir.join("test_case_9");
            std::fs::create_dir_all(&test_dir).unwrap();

            let backend_dir = test_dir.join("backend");
            std::fs::create_dir_all(&backend_dir).unwrap();
            let justfile_capital = backend_dir.join("Justfile");
            std::fs::File::create(&justfile_capital).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), None);
            let expected = fs::canonicalize(&justfile_capital).unwrap_or(justfile_capital);
            assert_eq!(result, Some(expected));
        }

        // Test case 10: case-insensitive filename - .Justfile (capital J)
        {
            let test_dir = test_root_dir.join("test_case_10");
            std::fs::create_dir_all(&test_dir).unwrap();

            let backend_dir = test_dir.join("backend");
            std::fs::create_dir_all(&backend_dir).unwrap();
            let dot_justfile_capital = backend_dir.join(".Justfile");
            std::fs::File::create(&dot_justfile_capital).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), None);
            let expected = fs::canonicalize(&dot_justfile_capital).unwrap_or(dot_justfile_capital);
            assert_eq!(result, Some(expected));
        }

        // Test case 11: mod_path is Some(absolute path), and the path is a file
        {
            let test_dir = test_root_dir.join("test_case_11");
            std::fs::create_dir_all(&test_dir).unwrap();

            let api_just = test_dir.join("api.just");
            std::fs::File::create(&api_just).unwrap();
            let absolute_path = fs::canonicalize(&api_just).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), Some(absolute_path.clone()));
            assert_eq!(result, Some(absolute_path));
        }

        // Test case 12: mod_path is Some(absolute path), and the path is a directory with mod.just
        {
            let test_dir = test_root_dir.join("test_case_12");
            std::fs::create_dir_all(&test_dir).unwrap();

            let api_dir = test_dir.join("api");
            std::fs::create_dir_all(&api_dir).unwrap();
            let mod_just = api_dir.join("mod.just");
            std::fs::File::create(&mod_just).unwrap();
            let absolute_path = fs::canonicalize(&api_dir).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), Some(absolute_path));
            let expected = fs::canonicalize(&mod_just).unwrap_or(mod_just);
            assert_eq!(result, Some(expected));
        }

        // Test case 13: no matching file exists
        {
            let test_dir = test_root_dir.join("test_case_13");
            std::fs::create_dir_all(&test_dir).unwrap();

            let result = Just::get_mod_file_path(test_dir.clone(), "backend".to_string(), None);
            assert_eq!(result, None);
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&test_root_dir);
    }
}
