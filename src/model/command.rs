use super::runner;

pub struct Command {
    pub runner_type: runner::RunnerType,
    pub command_name: String,
    pub file_name: String,
    pub line_number: u32,
}
