pub struct Config {
    forcus_history: bool,
}

impl Config {
    pub fn new(forcus_history: bool) -> Self {
        Self { forcus_history }
    }

    pub fn default() -> Self {
        Self {
            forcus_history: false,
        }
    }

    pub fn get_forcus_history(&self) -> bool {
        self.forcus_history
    }
}
