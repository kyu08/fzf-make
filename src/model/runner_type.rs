use super::runner;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunnerType {
    Make,
    Pnpm,
}

impl std::cmp::Eq for RunnerType {}

impl RunnerType {
    pub fn to_runner(&self, runners: &Vec<runner::Runner>) -> Option<runner::Runner> {
        match self {
            RunnerType::Make => {
                for r in runners {
                    if matches!(r, runner::Runner::MakeCommand(_)) {
                        return Some(r.clone());
                    }
                }
                None
            }
            RunnerType::Pnpm => todo!("implement and write test"),
        }
    }

    pub fn from(runner: &runner::Runner) -> Self {
        match runner {
            runner::Runner::MakeCommand(_) => RunnerType::Make,
            runner::Runner::PnpmCommand(_) => RunnerType::Pnpm,
        }
    }
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
