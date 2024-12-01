use super::{js_package_manager, runner};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunnerType {
    Make,
    JsPackageManager(JsPackageManager), // tomlの構造は変えたくないな...
}

#[derive(Hash, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsPackageManager {
    Pnpm,
}

impl std::cmp::Eq for RunnerType {}

impl RunnerType {
    pub fn to_runner(&self, runners: &Vec<runner::Runner>) -> Option<runner::Runner> {
        for r in runners {
            if self.clone() == RunnerType::from(r) {
                return Some(r.clone());
            }
        }
        None
    }

    pub fn from(runner: &runner::Runner) -> Self {
        match runner {
            runner::Runner::MakeCommand(_) => RunnerType::Make,
            runner::Runner::JsPackageManager(js) => match js {
                js_package_manager::JsPackageManager::JsPnpm(_) => {
                    RunnerType::JsPackageManager(JsPackageManager::Pnpm)
                }
                js_package_manager::JsPackageManager::JsYarn => todo!(),
            },
        }
    }
}

impl fmt::Display for RunnerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            RunnerType::Make => "make",
            RunnerType::JsPackageManager(js) => match js {
                JsPackageManager::Pnpm => "pnpm",
            },
        };
        write!(f, "{}", name)
    }
}
