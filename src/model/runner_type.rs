use serde::{Deserialize, Serialize};
use std::fmt;

// TODO(#321): remove
#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunnerType {
    Make,
    Pnpm,
}

impl fmt::Display for RunnerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            RunnerType::Make => "make",
            RunnerType::Pnpm => "pnpm",
        };
        write!(f, "{}", name)
    }
}
