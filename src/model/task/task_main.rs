use crate::model::{
    command::{self, CommandWithPreview},
    file_util,
    runner_type::RunnerType,
};
use anyhow::{Result, anyhow};
use serde_json::Value;
use std::{
    fs::{self},
    path::PathBuf,
    process,
};

// Taskfile supported extensions for auto-detection
const TASKFILE_EXTENSIONS: &[&str] = &["", ".yml", ".yaml"];

// Official supported file names from https://taskfile.dev/docs/guide#supported-file-names
// Listed in priority order (first match wins)
const SUPPORTED_TASKFILE_NAMES: &[&str] = &[
    "Taskfile.yml",      // Highest priority
    "taskfile.yml", 
    "Taskfile.yaml",
    "taskfile.yaml",
    "Taskfile.dist.yml",
    "taskfile.dist.yml",
    "Taskfile.dist.yaml", 
    "taskfile.dist.yaml", // Lowest priority
];

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    path: PathBuf,
    commands: Vec<command::CommandWithPreview>,
}

impl Task {
    #[allow(dead_code)]
    pub fn new(current_dir: PathBuf) -> Result<Task> {
        Task::new_with_dir(current_dir)
    }

    pub fn new_with_dir(target_dir: PathBuf) -> Result<Task> {
        let taskfile_path = Task::find_taskfile(target_dir.clone())
            .ok_or_else(|| anyhow!("Taskfile not found in directory: {}", target_dir.display()))?;

        let commands = Task::parse_taskfile(taskfile_path.clone())
            .map_err(|e| anyhow!("Failed to parse Taskfile: {}", e))?;

        Ok(Task {
            path: taskfile_path,
            commands,
        })
    }

    pub fn to_commands(&self) -> Vec<command::CommandWithPreview> {
        self.commands.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn command_to_run(&self, command: &command::CommandForExec) -> Result<String, anyhow::Error> {
        Ok(format!("task {}", command.args))
    }

    pub fn execute(&self, command: &command::CommandForExec) -> Result<(), anyhow::Error> {
        let child = process::Command::new("task")
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

    /// Resolves a taskfile path that can be either a file or directory.
    /// If it's a directory, searches for a Taskfile within that directory.
    /// If it's a file path without extension, tries common Taskfile extensions.
    fn resolve_taskfile_path(base_dir: &std::path::Path, taskfile_str: &str) -> Option<PathBuf> {
        let target_path = base_dir.join(taskfile_str);
        
        // If the path points to a file and it exists, use it directly
        if target_path.is_file() {
            return Some(target_path);
        }
        
        // If the path points to a directory, search for Taskfile in that directory
        if target_path.is_dir() {
            return Task::find_taskfile(target_path);
        }
        
        // If the path doesn't exist, try to find it with common extensions
        if !target_path.exists() {
            // Try common Taskfile extensions
            for &ext in TASKFILE_EXTENSIONS {
                let file_path = if ext.is_empty() {
                    target_path.clone()
                } else {
                    target_path.with_extension(&ext[1..]) // Remove the leading dot
                };
                
                if file_path.is_file() {
                    return Some(file_path);
                }
            }
            
            // Fallback: search in parent directory
            if let Some(parent) = target_path.parent() {
                if parent.is_dir() {
                    return Task::find_taskfile(parent.to_path_buf());
                }
            }
        }
        
        None
    }

    fn find_taskfile(current_dir: PathBuf) -> Option<PathBuf> {
        file_util::find_file_in_ancestors_with_priority(current_dir, SUPPORTED_TASKFILE_NAMES.to_vec())
    }

    fn parse_taskfile(taskfile_path: PathBuf) -> Result<Vec<CommandWithPreview>> {
        let content = fs::read_to_string(&taskfile_path)?;
        let yaml_value: Value = serde_yaml::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse YAML in {}: {}", taskfile_path.display(), e))?;

        let mut commands = Vec::new();
        let taskfile_dir = taskfile_path.parent().unwrap_or(&taskfile_path);

        // Parse main tasks
        Task::parse_main_tasks(&yaml_value, &content, &taskfile_path, &mut commands);

        // Parse included taskfiles
        Task::parse_includes(&yaml_value, taskfile_dir, &mut commands);

        // Parse root-level taskfile property
        Task::parse_root_taskfile(&yaml_value, taskfile_dir, &mut commands);

        if commands.is_empty() {
            return Err(anyhow!("No tasks found in Taskfile: {}", taskfile_path.display()));
        }

        Ok(commands)
    }

    fn parse_main_tasks(
        yaml_value: &Value,
        content: &str,
        taskfile_path: &PathBuf,
        commands: &mut Vec<CommandWithPreview>,
    ) {
        if let Some(tasks) = yaml_value.get("tasks").and_then(|t| t.as_object()) {
            for (task_name, _task_def) in tasks {
                let line_number = Task::find_task_line_number(content, task_name).unwrap_or(1);
                let command = CommandWithPreview::new(
                    RunnerType::Task,
                    task_name.clone(),
                    taskfile_path.clone(),
                    line_number,
                );
                commands.push(command);
            }
        }
    }

    fn parse_includes(
        yaml_value: &Value,
        taskfile_dir: &std::path::Path,
        commands: &mut Vec<CommandWithPreview>,
    ) {
        if let Some(includes) = yaml_value.get("includes").and_then(|i| i.as_object()) {
            for (namespace, include_def) in includes {
                let included_taskfile_path = Task::resolve_include_path(taskfile_dir, include_def);

                if let Some(path) = included_taskfile_path {
                    if let Ok(included_commands) = Task::parse_taskfile(path) {
                        for mut cmd in included_commands {
                            cmd.args = format!("{}:{}", namespace, cmd.args);
                            commands.push(cmd);
                        }
                    }
                }
            }
        }
    }

    fn parse_root_taskfile(
        yaml_value: &Value,
        taskfile_dir: &std::path::Path,
        commands: &mut Vec<CommandWithPreview>,
    ) {
        if let Some(taskfile_path_value) = yaml_value.get("taskfile") {
            if let Some(taskfile_str) = taskfile_path_value.as_str() {
                if let Some(included_taskfile_path) = Task::resolve_taskfile_path(taskfile_dir, taskfile_str) {
                    if let Ok(included_commands) = Task::parse_taskfile(included_taskfile_path) {
                        commands.extend(included_commands);
                    }
                }
            }
        }
    }

    fn resolve_include_path(taskfile_dir: &std::path::Path, include_def: &Value) -> Option<PathBuf> {
        if let Some(taskfile_str) = include_def.as_str() {
            // Simple string format: "namespace: ./path/to/Taskfile.yml" or "namespace: ./path/to/dir"
            Task::resolve_taskfile_path(taskfile_dir, taskfile_str)
        } else if let Some(include_obj) = include_def.as_object() {
            // Object format: "namespace: { taskfile: ./path/to/Taskfile.yml, dir: ./path }"
            if let Some(taskfile_str) = include_obj.get("taskfile").and_then(|t| t.as_str()) {
                Task::resolve_taskfile_path(taskfile_dir, taskfile_str)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn find_task_line_number(content: &str, task_name: &str) -> Option<u32> {
        let lines: Vec<&str> = content.lines().collect();
        
        // Look for the task definition pattern: "  task_name:" or "task_name:"
        let task_pattern = format!("{}:", task_name);
        
        for (line_index, line) in lines.iter().enumerate() {
            let trimmed_line = line.trim();
            
            // Check if this line defines the task
            if trimmed_line == task_pattern || trimmed_line.starts_with(&format!("{}: ", task_name)) {
                // Check if we're inside the tasks section
                if Task::is_inside_tasks_section(&lines, line_index) {
                    return Some((line_index + 1) as u32); // Convert to 1-based line number
                }
            }
        }
        
        None
    }

    fn is_inside_tasks_section(lines: &[&str], task_line_index: usize) -> bool {
        // Look backwards from the task line to find the "tasks:" section
        for i in (0..task_line_index).rev() {
            let line = lines[i].trim();
            
            // If we find "tasks:" at the beginning of a line, we're in the tasks section
            if line == "tasks:" {
                return true;
            }
            
            // If we find another top-level section (no indentation), we're not in tasks
            // Skip empty lines and comments
            if !line.is_empty() && !line.starts_with('#') {
                // Check if this is a top-level key (no leading whitespace and ends with colon)
                let original_line = lines[i];
                if !original_line.starts_with(' ') && !original_line.starts_with('\t') && line.ends_with(':') {
                    // This is a top-level section that's not "tasks:"
                    return false;
                }
            }
        }
        
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn find_taskfile_test() {
        struct Case {
            title: &'static str,
            current_dir: PathBuf,
            should_find: bool,
        }

        let cases = vec![
            Case {
                title: "Should find Taskfile.yml in main directory",
                current_dir: PathBuf::from("test_data/task"),
                should_find: true,
            },
            Case {
                title: "Should find Taskfile.dist.yml in dist directory",
                current_dir: PathBuf::from("test_data/task/dist"),
                should_find: true,
            },
            Case {
                title: "Should not find Taskfile in non-existent directory",
                current_dir: PathBuf::from("test_data/nonexistent"),
                should_find: false,
            },
        ];

        for case in cases {
            let result = Task::find_taskfile(case.current_dir);
            if case.should_find {
                assert!(result.is_some(), "Case: {} - Should find a taskfile", case.title);
                if let Some(path) = result {
                    assert!(path.exists(), "Case: {} - Found file should exist", case.title);
                }
            } else {
                assert!(result.is_none(), "Case: {} - Should not find taskfile", case.title);
            }
        }
    }

    #[test]
    fn new_with_dir_test() {
        struct Case {
            title: &'static str,
            target_dir: PathBuf,
            should_succeed: bool,
        }

        let cases = vec![
            Case {
                title: "Should find Taskfile in main directory",
                target_dir: PathBuf::from("test_data/task"),
                should_succeed: true,
            },
            Case {
                title: "Should find Taskfile in nested directory",
                target_dir: PathBuf::from("test_data/task/nested"),
                should_succeed: true,
            },
            Case {
                title: "Should fail when no Taskfile in directory",
                target_dir: PathBuf::from("test_data/make"),
                should_succeed: false,
            },
        ];

        for case in cases {
            let result = Task::new_with_dir(case.target_dir.clone());
            if case.should_succeed {
                assert!(result.is_ok(), "Case: {} - Should succeed", case.title);
                if let Ok(task) = result {
                    assert!(!task.to_commands().is_empty(), "Case: {} - Should have commands", case.title);
                }
            } else {
                assert!(result.is_err(), "Case: {} - Should fail", case.title);
            }
        }
    }

    #[test]
    fn resolve_taskfile_path_test() {
        struct Case {
            title: &'static str,
            base_dir: &'static str,
            taskfile_str: &'static str,
            should_find: bool,
        }

        let cases = vec![
            Case {
                title: "Should resolve file path directly",
                base_dir: "test_data/task",
                taskfile_str: "Taskfile.yml",
                should_find: true,
            },
            Case {
                title: "Should resolve directory to Taskfile",
                base_dir: "test_data/task",
                taskfile_str: "nested",
                should_find: true,
            },
            Case {
                title: "Should not find non-existent path",
                base_dir: "test_data/nonexistent",
                taskfile_str: "nonexistent",
                should_find: false,
            },
        ];

        for case in cases {
            let base_path = PathBuf::from(case.base_dir);
            let result = Task::resolve_taskfile_path(&base_path, case.taskfile_str);
            
            if case.should_find {
                assert!(result.is_some(), "Case: {} - Should find taskfile", case.title);
                if let Some(path) = result {
                    assert!(path.exists(), "Case: {} - Found file should exist: {}", case.title, path.display());
                }
            } else {
                assert!(result.is_none(), "Case: {} - Should not find taskfile", case.title);
            }
        }
    }

    #[test]
    fn parse_taskfile_with_directory_include_test() {
        // Test parsing the main Taskfile which includes the nested directory
        let taskfile_path = PathBuf::from("test_data/task/Taskfile.yml");
        let result = Task::parse_taskfile(taskfile_path);
        
        assert!(result.is_ok(), "Should parse Taskfile with directory include successfully");
        
        let commands = result.unwrap();
        let task_names: Vec<String> = commands.iter().map(|c| c.args.clone()).collect();
        
        // Should contain main tasks
        assert!(task_names.contains(&"build".to_string()), "Should contain build task");
        assert!(task_names.contains(&"test".to_string()), "Should contain test task");
        assert!(task_names.contains(&"clean".to_string()), "Should contain clean task");
        
        // Should contain nested tasks with namespace prefix
        assert!(task_names.contains(&"nested:setup".to_string()), "Should contain nested:setup task");
        assert!(task_names.contains(&"nested:deploy".to_string()), "Should contain nested:deploy task");
        
        // Should have 5 tasks total (3 main + 2 nested)
        assert_eq!(commands.len(), 5, "Should have exactly 5 tasks, found: {}", commands.len());
    }
}