use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize)]
struct Histories {
    history: Vec<History>,
}

impl Histories {
    fn from(histories: Vec<(PathBuf, Vec<String>)>) -> Self {
        let mut result: Vec<History> = vec![];
        for h in histories {
            result.push(History {
                path: h.0.to_str().unwrap().to_string(),
                executed_targets: h.1,
            });
        }
        Histories { history: result }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct History {
    path: String,
    executed_targets: Vec<String>,
}

pub fn read_history(content: String) -> Result<Vec<(PathBuf, Vec<String>)>> {
    let histories: Histories = toml::from_str(&content)?;

    let mut result: Vec<(PathBuf, Vec<String>)> = Vec::new();

    for history in histories.history {
        result.push((PathBuf::from(history.path), history.executed_targets));
    }
    Ok(result)
}

pub fn write_history(
    history_directory_path: PathBuf,
    history_file_name: String,
    histories_tuple: Vec<(PathBuf, Vec<String>)>,
) -> Result<()> {
    let histories = Histories::from(histories_tuple);

    if !history_directory_path.is_dir() {
        fs::create_dir_all(history_directory_path.clone())?;
    }

    let mut history_file = File::create(history_directory_path.join(history_file_name))?;
    history_file.write_all(toml::to_string(&histories).unwrap().as_bytes())?;
    history_file.flush()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn parse_history_test() {
        struct Case {
            title: &'static str,
            content: String,
            expect: Result<Vec<(PathBuf, Vec<String>)>>,
        }
        let cases = vec![
            Case {
                title: "Success",
                content: r#"
[[history]]
path = "/Users/user/code/fzf-make"
executed-targets = ["test", "check", "spell-check"]

[[history]]
path = "/Users/user/code/golang/go-playground"
executed-targets = ["run", "echo1"]
                "#
                .to_string(),
                expect: Ok(vec![
                    (
                        PathBuf::from("/Users/user/code/fzf-make".to_string()),
                        vec![
                            "test".to_string(),
                            "check".to_string(),
                            "spell-check".to_string(),
                        ],
                    ),
                    (
                        PathBuf::from("/Users/user/code/golang/go-playground".to_string()),
                        vec!["run".to_string(), "echo1".to_string()],
                    ),
                ]),
            },
            Case {
                title: "Error",
                content: r#"
                "#
                .to_string(),
                expect: Err(anyhow::anyhow!("TOML parse error at line 1, column 1\n  |\n1 | \n  | ^\nmissing field `history`\n")),
            },
        ];

        for case in cases {
            match case.expect {
                Ok(v) => assert_eq!(
                    v,
                    read_history(case.content).unwrap(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
                Err(e) => assert_eq!(
                    e.to_string(),
                    read_history(case.content).unwrap_err().to_string(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
            }
        }
    }
}
