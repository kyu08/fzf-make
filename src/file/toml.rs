use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Histories {
    history: Vec<History>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct History {
    executed_targets: Vec<String>,
    path: String,
}

pub fn parse_history(content: String) -> Result<Vec<(PathBuf, Vec<String>)>> {
    let histories: Histories = toml::from_str(&content)?;
    let mut result: Vec<(PathBuf, Vec<String>)> = Vec::new();

    for history in histories.history {
        result.push((PathBuf::from(history.path), history.executed_targets));
    }
    Ok(result)
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
        let cases = vec![Case {
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
        }];

        for case in cases {
            match case.expect {
                Ok(v) => assert_eq!(
                    v,
                    parse_history(case.content).unwrap(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
                Err(e) => assert_eq!(
                    e.to_string(),
                    parse_history(case.content).unwrap_err().to_string(),
                    "\nFailed: 🚨{:?}🚨\n",
                    case.title,
                ),
            }
        }
    }
}
