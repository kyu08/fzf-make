use super::{js_package_manager::js_package_manager_main, runner};
use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Hash, PartialEq, Debug, Clone, Eq)]
pub enum RunnerType {
    Make,
    JsPackageManager(JsPackageManager),
}

#[derive(Hash, PartialEq, Debug, Clone, Serialize, Deserialize, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JsPackageManager {
    Pnpm,
}

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
                js_package_manager_main::JsPackageManager::JsPnpm(_) => {
                    RunnerType::JsPackageManager(JsPackageManager::Pnpm)
                }
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

/*
 It is not better that the implementation related specific data format in `model` module.
 The implementation cost is also high, so I will implement it here.
*/

// Need to Deserialize as value of runner-type in toml.
impl<'de> Deserialize<'de> for RunnerType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;

        match s.as_str() {
            "make" => Ok(RunnerType::Make),
            "pnpm" => Ok(RunnerType::JsPackageManager(JsPackageManager::Pnpm)),
            _ => Err(de::Error::custom(format!("Unknown runner type: {}", s))),
        }
    }
}

// Need to Serialize as value of runner-type in toml.
impl Serialize for RunnerType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RunnerType::Make => serializer.serialize_str("make"),
            RunnerType::JsPackageManager(JsPackageManager::Pnpm) => {
                serializer.serialize_str("pnpm")
            }
        }
    }
}
