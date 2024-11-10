use super::runner_type;

#[allow(dead_code)] // TODO: remove
#[derive(PartialEq, Debug, Clone)]
pub struct Command {
    pub runner_type: runner_type::RunnerType,
    pub command_name: String,
    pub file_name: String,
    pub line_number: u32,
}

impl Command {
    pub fn new(
        runner_type: runner_type::RunnerType,
        command_name: String,
        file_name: String,
        line_number: u32,
    ) -> Self {
        Self {
            runner_type,
            command_name,
            file_name,
            line_number,
        }
    }

    pub fn print(&self) -> String {
        format!("({}) {}", self.runner_type, self.command_name,)
    }
}
