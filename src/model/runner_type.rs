use super::{js_package_manager::js_package_manager_main as js, runner};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self},
};
use std::fmt;

#[derive(Hash, PartialEq, Debug, Clone, Eq)]
pub enum RunnerType {
    Make,
    JsPackageManager(JsPackageManager),
    Just,
    Task,
}

#[derive(Hash, PartialEq, Debug, Clone, Serialize, Deserialize, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JsPackageManager {
    Pnpm,
    Yarn,
}

impl RunnerType {
    pub fn to_runner(&self, runners: &Vec<runner::Runner>) -> Option<runner::Runner> {
        for r in runners {
            if self.clone() == RunnerType::from(r.clone()) {
                return Some(r.clone());
            }
        }
        None
    }

    pub fn get_extension_for_highlighting(&self) -> &str {
        match self {
            RunnerType::Make => "mk",
            // HACK: If `just` is passed to syntect, it will be highlighted as just a plain text.
            // So yaml which is similar to just is used intensionally.
            RunnerType::Just => "yaml",
            RunnerType::JsPackageManager(_) => "json",
            RunnerType::Task => "yaml",
        }
    }
}

impl From<runner::Runner> for RunnerType {
    fn from(runner: runner::Runner) -> RunnerType {
        match runner {
            runner::Runner::MakeCommand(_) => RunnerType::Make,
            runner::Runner::JsPackageManager(js) => match js {
                js::JsPackageManager::JsPnpm(_) => RunnerType::JsPackageManager(JsPackageManager::Pnpm),
                js::JsPackageManager::JsYarn(_) => RunnerType::JsPackageManager(JsPackageManager::Yarn),
            },
            runner::Runner::Just(_) => RunnerType::Just,
            runner::Runner::Task(_) => RunnerType::Task,
        }
    }
}

impl fmt::Display for RunnerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            RunnerType::Make => "make",
            RunnerType::JsPackageManager(js) => match js {
                JsPackageManager::Pnpm => "pnpm",
                JsPackageManager::Yarn => "yarn",
            },
            RunnerType::Just => "just",
            RunnerType::Task => "task",
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
            "yarn" => Ok(RunnerType::JsPackageManager(JsPackageManager::Yarn)),
            "just" => Ok(RunnerType::Just),
            "task" => Ok(RunnerType::Task),
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
            RunnerType::JsPackageManager(JsPackageManager::Pnpm) => serializer.serialize_str("pnpm"),
            RunnerType::JsPackageManager(JsPackageManager::Yarn) => serializer.serialize_str("yarn"),
            RunnerType::Just => serializer.serialize_str("just"),
            RunnerType::Task => serializer.serialize_str("task"),
        }
    }
}
