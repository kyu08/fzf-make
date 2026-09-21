use crate::model::{
    command::{self},
    runner::Runner,
    runner_type::{self},
};
use anyhow::{Result, anyhow};
use std::{
    path::PathBuf,
    process::{self},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    path: PathBuf,
    commands: Vec<command::CommandWithPreview>,
}

impl Runner for Task {
    fn runner_type(&self) -> runner_type::RunnerType {
        runner_type::RunnerType::Task
    }

    fn program(&self) -> &'static str {
        "task"
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

impl Task {
    pub fn new(cwd: PathBuf) -> Result<Task> {
        let commands = Self::get_available_commands()?;
        Ok(Task { path: cwd, commands })
    }

    // get_available_commands executes `task --list-all --json` and parse the result from it.
    fn get_available_commands() -> Result<Vec<command::CommandWithPreview>, anyhow::Error> {
        if process::Command::new("task").arg("--version").output().is_err() {
            return Err(anyhow!("command not found: task"));
        }

        let output = process::Command::new("task").arg("--list-all").arg("--json").output()?;
        let output_json = String::from_utf8(output.stdout)?;
        let tasks = Self::parse_task_json(&output_json)?;

        let output = tasks
            .iter()
            .map(|task| {
                command::CommandWithPreview::new(
                    runner_type::RunnerType::Task,
                    task.task.clone(),
                    task.location.taskfile.clone(),
                    task.location.line,
                )
            })
            .collect();
        Ok(output)
    }

    fn parse_task_json(json_str: &str) -> Result<Vec<TaskListJson>, anyhow::Error> {
        /* Example json_str:
        {
          "tasks": [
            {
              "name": "deploy",
              "task": "deploy",
              "desc": "Deploy nested service",
              "summary": "",
              "aliases": [],
              "up_to_date": false,
              "location": {
                "line": 9,
                "column": 3,
                "taskfile": "/Users/username/fzf-make/test_data/task/nested/Taskfile.yml"
              }
            },
            {
              "name": "setup",
              "task": "setup",
              "desc": "Setup nested environment",
              "summary": "",
              "aliases": [],
              "up_to_date": false,
              "location": {
                "line": 4,
                "column": 3,
                "taskfile": "/Users/username/fzf-make/test_data/task/nested/Taskfile.yml"
              }
            }
          ],
          "location": "/Users/username/fzf-make/test_data/task/nested/Taskfile.yml"
        }
        */
        #[derive(serde::Deserialize, Debug)]
        struct Output {
            tasks: Vec<TaskListJson>,
        }

        if let Ok(serde_json::Value::Object(map)) = serde_json::from_slice::<serde_json::Value>(json_str.as_bytes()) {
            if let Ok(output) = serde_json::from_value::<Output>(serde_json::Value::Object(map)) {
                Ok(output.tasks)
            } else {
                Err(anyhow!("Failed to parse task JSON structure"))
            }
        } else {
            Err(anyhow!("Failed to parse JSON"))
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
struct TaskListJson {
    task: String,
    location: Location,
}
#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
struct Location {
    line: u32,
    taskfile: PathBuf,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_task_json_test() {
        struct Case {
            title: &'static str,
            input: &'static str,
            expected_result: Result<Vec<TaskListJson>, ()>,
        }

        let cases = vec![
            Case {
                title: "Should parse valid JSON with single task",
                input: r#"{
                    "tasks": [
                        {
                            "task": "deploy",
                            "location": {
                                "line": 9,
                                "taskfile": "/Users/test/Taskfile.yml"
                            }
                        }
                    ]
                }"#,
                expected_result: Ok(vec![TaskListJson {
                    task: "deploy".to_string(),
                    location: Location {
                        line: 9,
                        taskfile: PathBuf::from("/Users/test/Taskfile.yml"),
                    },
                }]),
            },
            Case {
                title: "Should parse valid JSON with multiple tasks",
                input: r#"{
                    "tasks": [
                        {
                            "task": "deploy",
                            "location": {
                                "line": 9,
                                "taskfile": "/Users/test/Taskfile.yml"
                            }
                        },
                        {
                            "task": "setup",
                            "location": {
                                "line": 4,
                                "taskfile": "/Users/test/nested/Taskfile.yml"
                            }
                        }
                    ]
                }"#,
                expected_result: Ok(vec![
                    TaskListJson {
                        task: "deploy".to_string(),
                        location: Location {
                            line: 9,
                            taskfile: PathBuf::from("/Users/test/Taskfile.yml"),
                        },
                    },
                    TaskListJson {
                        task: "setup".to_string(),
                        location: Location {
                            line: 4,
                            taskfile: PathBuf::from("/Users/test/nested/Taskfile.yml"),
                        },
                    },
                ]),
            },
            Case {
                title: "Should parse empty tasks array",
                input: r#"{"tasks": []}"#,
                expected_result: Ok(vec![]),
            },
            Case {
                title: "Should fail with invalid JSON",
                input: r#"{"tasks": ["#,
                expected_result: Err(()),
            },
            Case {
                title: "Should fail when tasks field is missing",
                input: r#"{"other_field": []}"#,
                expected_result: Err(()),
            },
        ];

        for case in cases {
            let result = Task::parse_task_json(case.input);
            match case.expected_result {
                Ok(expected) => {
                    assert!(result.is_ok(), "Case: {} - Should succeed", case.title);
                    if let Ok(actual) = result {
                        assert_eq!(actual, expected, "Case: {} - Result should match expected", case.title);
                    }
                }
                Err(_) => {
                    assert!(result.is_err(), "Case: {} - Should fail", case.title);
                }
            }
        }
    }
}
