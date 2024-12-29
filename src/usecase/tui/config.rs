pub struct Config {
    focus_history: bool,
}

impl Config {
    pub fn new(focus_history: bool) -> Self {
        Self { focus_history }
    }

    pub fn default() -> Self {
        Self { focus_history: false }
    }

    pub fn get_focus_history(&self) -> bool {
        self.focus_history
    }
}
