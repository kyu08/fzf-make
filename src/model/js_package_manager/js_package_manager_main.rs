use super::{npm, pnpm, yarn};
use crate::{
    file::path_to_content,
    model::{command, file_util, runner::Runner, runner_type},
};
use anyhow::Result;
use codespan::Files;
use json_spanned_value::{self as jsv, spanned};
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

pub(super) const METADATA_FILE_NAME: &str = "package.json";
const METADATA_PACKAGE_NAME_KEY: &str = "name";
const METADATA_COMMAND_KEY: &str = "scripts";

/// The order matters. The first package manager which is detected is the one used.
static PACKAGE_MANAGERS: &[&dyn PackageManager] = &[&pnpm::Pnpm, &npm::Npm, &yarn::Yarn];

/// PackageManager describes what differs between the JavaScript package managers which define
/// their commands as the `scripts` of `package.json`. Everything else is shared by `JsRunner`,
/// so adding a package manager means implementing this trait and listing it in
/// `PACKAGE_MANAGERS`.
pub(super) trait PackageManager: fmt::Debug + Send + Sync {
    fn runner_type(&self) -> runner_type::RunnerType;

    /// program is the name of the executable to spawn.
    fn program(&self) -> &'static str;

    /// lockfile_names are the names of the lockfile which indicates the root of the workspace.
    /// Some package managers accept more than one name for it.
    fn lockfile_names(&self) -> Vec<&'static str>;

    /// root_script_args builds the args to run a script defined in the `package.json` of the
    /// directory which fzf-make is launched in.
    fn root_script_args(&self, script_name: &str) -> String;

    /// workspace_script_args builds the args to run a script defined in the `package.json` of
    /// another package of the workspace.
    fn workspace_script_args(&self, package_name: &str, script_name: &str) -> String;

    /// workspace_package_json_paths returns the path to the `package.json` of every package of
    /// the workspace. Err means the packages could not be listed, which typically happens when
    /// the current directory is not the root of a workspace.
    fn workspace_package_json_paths(&self) -> Result<Vec<PathBuf>>;

    /// should_skip_script reports whether a script must be hidden from the command list.
    fn should_skip_script(&self, _script_name: &str, _script_body: &str) -> bool {
        false
    }

    /// is_workspace_member reports whether the current directory is a package of a workspace
    /// whose root is one of its ancestors.
    fn is_workspace_member(&self, current_dir: &Path) -> bool {
        file_util::find_file_in_ancestors(current_dir.to_path_buf(), self.lockfile_names()).is_some()
    }
}

/// JsRunner is the Runner shared by every JavaScript package manager. What differs between them
/// is held by `pm`.
#[derive(Debug, Clone)]
struct JsRunner {
    pm: &'static dyn PackageManager,
    path: PathBuf,
    commands: Vec<command::CommandWithPreview>,
}

impl Runner for JsRunner {
    fn runner_type(&self) -> runner_type::RunnerType {
        self.pm.runner_type()
    }

    fn program(&self) -> &'static str {
        self.pm.program()
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        self.commands.clone()
    }

    fn clone_box(&self) -> Box<dyn Runner> {
        Box::new(self.clone())
    }
}

impl PartialEq for JsRunner {
    fn eq(&self, other: &Self) -> bool {
        self.pm.runner_type() == other.pm.runner_type() && self.path == other.path && self.commands == other.commands
    }
}

/// get_js_package_manager_runner determines which JavaScript package manager is used in
/// `current_dir`. The detection order is defined by `PACKAGE_MANAGERS`.
pub fn get_js_package_manager_runner(current_dir: PathBuf) -> Option<Box<dyn Runner>> {
    let entries = fs::read_dir(current_dir.clone()).unwrap();
    let file_names: Vec<String> = entries.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();

    PACKAGE_MANAGERS
        .iter()
        .find_map(|pm| detect(*pm, current_dir.clone(), &file_names))
        .map(|runner| Box::new(runner) as Box<dyn Runner>)
}

fn detect(pm: &'static dyn PackageManager, current_dir: PathBuf, cwd_file_names: &[String]) -> Option<JsRunner> {
    if !cwd_file_names.iter().any(|f| f == METADATA_FILE_NAME) {
        return None;
    }

    let lockfile_exists_in_current_dir = pm
        .lockfile_names()
        .iter()
        .any(|lockfile| cwd_file_names.iter().any(|f| f == lockfile));

    let commands = if lockfile_exists_in_current_dir {
        collect_workspace_scripts(pm, &current_dir)?
    } else if pm.is_workspace_member(&current_dir) {
        collect_scripts_in_package_json(pm, &current_dir)?
    } else {
        // Neither the root of a workspace nor a package of it.
        // In this case, the package manager can not be determined.
        return None;
    };

    Some(JsRunner {
        pm,
        path: current_dir,
        commands,
    })
}

/// collect_workspace_scripts collects the commands of the whole workspace by following steps:
/// 1. Collect scripts defined in `package.json` in the current directory(which fzf-make is launched)
/// 2. Collect the paths of all `package.json` in the workspace.
/// 3. Collect all scripts defined in given `package.json` paths.
fn collect_workspace_scripts(pm: &dyn PackageManager, current_dir: &Path) -> Option<Vec<command::CommandWithPreview>> {
    let mut result = collect_scripts_in_package_json(pm, current_dir)?;

    // Listing the packages fails when the current directory is not the root of a workspace
    // (e.g. a project which is not a monorepo), so fall back to the scripts collected above.
    let workspace_package_json_paths = match pm.workspace_package_json_paths() {
        Ok(paths) => paths,
        Err(_) => return Some(result),
    };

    let current_package_json = current_dir.join(METADATA_FILE_NAME);
    for path in workspace_package_json_paths {
        // Skip the current directory's package.json to avoid duplication.
        if is_same_file(&path, &current_package_json) {
            continue;
        }

        let Ok(content) = path_to_content::path_to_content(&path) else {
            continue;
        };
        let Some((package_name, scripts)) = parse_package_json(&content) else {
            continue;
        };

        for (script_name, script_body, line_number) in scripts {
            if pm.should_skip_script(&script_name, &script_body) {
                continue;
            }
            result.push(command::CommandWithPreview::new(
                pm.runner_type(),
                pm.workspace_script_args(&package_name, &script_name),
                path.clone(),
                line_number,
            ));
        }
    }

    Some(result)
}

fn collect_scripts_in_package_json(
    pm: &dyn PackageManager,
    current_dir: &Path,
) -> Option<Vec<command::CommandWithPreview>> {
    let path = current_dir.join(METADATA_FILE_NAME);
    let content = path_to_content::path_to_content(&path).ok()?;
    let (_, scripts) = parse_package_json(&content)?;

    Some(
        scripts
            .iter()
            .filter(|(script_name, script_body, _)| !pm.should_skip_script(script_name, script_body))
            .map(|(script_name, _, line_number)| {
                command::CommandWithPreview::new(
                    pm.runner_type(),
                    pm.root_script_args(script_name),
                    path.clone(),
                    *line_number,
                )
            })
            .collect(),
    )
}

// Some package managers report the paths of the workspace packages as relative paths, so the
// canonicalized paths are compared. Fall back to a plain comparison when a path can not be
// canonicalized.
fn is_same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

// returns (package_name, [(script_name, script_content, line_number)]
#[allow(clippy::type_complexity)]
fn parse_package_json(content: &str) -> Option<(String, Vec<(String, String, u32)>)> {
    let mut files = Files::new();
    let file = files.add(METADATA_FILE_NAME, content);
    let json_object: spanned::Object = match jsv::from_str(content) {
        Ok(e) => e,
        Err(_) => return None,
    };

    let mut name = "".to_string();
    let mut result = vec![];
    for (k, v) in json_object {
        if k.as_str() == METADATA_PACKAGE_NAME_KEY && v.as_string().is_some() {
            name = v.as_string().unwrap().to_string();
        }
        if k.as_str() != METADATA_COMMAND_KEY {
            continue;
        }

        // object is the content of "scripts" key
        if let Some(object) = v.as_object() {
            for (k, v) in object {
                let args = k.to_string();
                let line_number = files.line_index(file, k.start() as u32).number().to_usize() as u32;
                if let Some(v) = v.as_string() {
                    result.push((args, v.to_string(), line_number));
                }
            }
        };
        break;
    }

    Some((name, result))
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_package_json() {
        struct Case {
            title: &'static str,
            file_content: &'static str,
            #[allow(clippy::type_complexity)]
            expected: Option<(String, Vec<(String, String, u32)>)>,
        }

        let cases = vec![
            Case {
                title: "valid json can be parsed successfully",
                file_content: r#"{
      "name": "project",
      "version": "1.0.0",
      "private": true,
      "scripts": {
        "build": "echo build",
        "start": "echo start",
        "test": "echo test"
      },
      "devDependencies": {
        "@babel/cli": "7.12.10"
      },
      "dependencies": {
        "firebase": "^8.6.8"
      }
    }
                        "#,
                expected: Some((
                    "project".to_string(),
                    vec![
                        ("build".to_string(), "echo build".to_string(), 6),
                        ("start".to_string(), "echo start".to_string(), 7),
                        ("test".to_string(), "echo test".to_string(), 8),
                    ],
                )),
            },
            Case {
                title: "empty vec(empty string)",
                file_content: "",
                expected: None,
            },
            Case {
                title: "empty vec(invalid json)",
                file_content: "not a json format",
                expected: None,
            },
        ];

        for case in cases {
            assert_eq!(case.expected, parse_package_json(case.file_content), "\nfailed: 🚨{:?}🚨\n", case.title,);
        }
    }

    #[test]
    fn test_detect() {
        struct Case {
            title: &'static str,
            pm: &'static dyn PackageManager,
            current_dir: PathBuf,
            cwd_file_names: Vec<String>,
            expected_is_some: bool,
        }

        let cases = vec![
            Case {
                title: "npm is detected in the root of a workspace",
                pm: &npm::Npm,
                current_dir: PathBuf::from("test_data/npm"),
                cwd_file_names: vec!["package.json".to_string(), "package-lock.json".to_string()],
                expected_is_some: true,
            },
            Case {
                title: "npm is not detected without package.json",
                pm: &npm::Npm,
                current_dir: std::env::temp_dir(),
                cwd_file_names: vec!["README.md".to_string()],
                expected_is_some: false,
            },
            Case {
                title: "npm is not detected when the lockfile exists neither in the current dir nor in the ancestors",
                pm: &npm::Npm,
                current_dir: std::env::temp_dir(),
                cwd_file_names: vec!["package.json".to_string()],
                expected_is_some: false,
            },
            Case {
                title: "pnpm is detected in the root of a workspace",
                pm: &pnpm::Pnpm,
                current_dir: PathBuf::from("test_data/pnpm"),
                cwd_file_names: vec!["package.json".to_string(), "pnpm-lock.yaml".to_string()],
                expected_is_some: true,
            },
            Case {
                title: "pnpm is not detected in a directory which npm manages",
                pm: &pnpm::Pnpm,
                current_dir: PathBuf::from("test_data/npm"),
                cwd_file_names: vec!["package.json".to_string(), "package-lock.json".to_string()],
                expected_is_some: false,
            },
        ];

        for case in cases {
            assert_eq!(
                case.expected_is_some,
                detect(case.pm, case.current_dir, &case.cwd_file_names).is_some(),
                "\nfailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }
}
