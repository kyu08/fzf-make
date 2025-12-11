use super::{config, ui::ui};
use crate::{
    error::any_to_string,
    file::toml,
    model::{
        command::{self},
        histories::{self},
        js_package_manager::js_package_manager_main as js,
        just::just_main::Just,
        make::make_main::Make,
        runner::{self, Runner},
        runner_type,
        task::task_main::Task,
    },
};
use anyhow::{Result, anyhow, bail};
use arboard::Clipboard;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::FutureExt;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    widgets::ListState,
};
use std::{
    collections::HashMap,
    env,
    io::{self, Stderr},
    panic::AssertUnwindSafe,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::task;
use tui_textarea::TextArea;
use update_informer::{Check, registry};

// AppState represents the state of the application.
// "Making impossible states impossible"
// The type of `AppState` is defined according to the concept of 'Making Impossible States Impossible'.
// See: https://www.youtube.com/watch?v=IcgmSRJHu_8
#[derive(PartialEq, Debug)]
pub enum AppState<'a> {
    // Box the largest variant to reduce enum size (1176 bytes → ~8 bytes)
    // When there's a large size difference between variants, the entire enum
    // always allocates memory for the largest variant, which leads to:
    // - Poor memory efficiency
    // - Increased risk of stack overflow
    // See: https://rust-lang.github.io/rust-clippy/master/index.html#large_enum_variant
    SelectCommand(Box<SelectCommandState<'a>>),
    ExecuteCommand(ExecuteCommandState),
    ShouldQuit,
}

#[derive(PartialEq, Debug)]
pub struct Model<'a> {
    pub app_state: AppState<'a>,
}

impl Model<'_> {
    pub fn new(config: config::Config) -> Result<Self> {
        match SelectCommandState::new(config) {
            Ok(s) => Ok(Model {
                app_state: AppState::SelectCommand(Box::new(s)),
            }),
            Err(e) => Err(e),
        }
    }

    fn handle_key_input(&self, key: KeyEvent) -> Option<Message> {
        if let AppState::SelectCommand(s) = &self.app_state {
            let is_ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);

            // When additional arguments popup is opened
            if let Some(additional_arguments_popup_state) = &s.additional_arguments_popup_state {
                match key.code {
                    KeyCode::Esc => Some(Message::CloseAdditionalArgumentsWindow),
                    KeyCode::Tab => None, // no-op: same as Main pane.
                    KeyCode::Enter => {
                        Some(Message::ExecuteCommand(additional_arguments_popup_state.append_arguments()))
                    }
                    _ => Some(Message::AdditionalArgumentsKeyInput(key)),
                }
            } else {
                // When additional arguments popup is not opened
                match s.current_pane {
                    CurrentPane::Main => match (key.code, is_ctrl_pressed) {
                        (KeyCode::Tab, _) => Some(Message::MoveToNextPane),
                        (KeyCode::Esc, _) | (KeyCode::Char('c'), true) => Some(Message::Quit),
                        (KeyCode::Down, _) | (KeyCode::Char('n'), true) => Some(Message::NextCommand),
                        (KeyCode::Up, _) | (KeyCode::Char('p'), true) => Some(Message::PreviousCommand),
                        (KeyCode::Char('o'), true) => Some(Message::OpenAdditionalArgumentsWindow),
                        (KeyCode::Char('y'), true) => Some(Message::CopyCommandToClipboard),
                        (KeyCode::Enter, _) => Some(Message::ExecuteCommand(s.get_selected_command().unwrap())),
                        _ => Some(Message::SearchTextAreaKeyInput(key)),
                    },
                    CurrentPane::History => match (key.code, is_ctrl_pressed) {
                        (KeyCode::Tab, _) => Some(Message::MoveToNextPane),
                        (KeyCode::Esc, _) | (KeyCode::Char('c'), true) | (KeyCode::Char('q'), _) => Some(Message::Quit),
                        (KeyCode::Down, _) | (KeyCode::Char('n'), true) => Some(Message::NextHistory),
                        (KeyCode::Up, _) | (KeyCode::Char('p'), true) => Some(Message::PreviousHistory),
                        (KeyCode::Char('o'), true) => Some(Message::OpenAdditionalArgumentsWindow),
                        (KeyCode::Char('y'), true) => Some(Message::CopyCommandToClipboard),
                        (KeyCode::Enter, _) | (KeyCode::Char(' '), _) => {
                            Some(Message::ExecuteCommand(s.get_selected_command().unwrap()))
                        }
                        _ => None,
                    },
                }
            }
        } else {
            None
        }
    }

    // returns available commands in cwd from history file
    fn get_histories(current_working_directory: PathBuf) -> Vec<histories::HistoryCommand> {
        let histories = toml::Histories::into(toml::Histories::get_history());

        for history in histories.histories {
            if history.path != current_working_directory {
                continue;
            }

            // Originally, it was filtering out commands in history that no longer exist.
            // But due to the development of the feature to pass arguments to commands,
            // checking the existence of commands was removed.
            return history.commands;
        }

        vec![]
    }

    fn transition_to_execute_command_state(&mut self, runner: runner::Runner, command: command::CommandForExec) {
        self.app_state = AppState::ExecuteCommand(ExecuteCommandState::new(runner, command));
    }

    fn transition_to_should_quit_state(&mut self) {
        self.app_state = AppState::ShouldQuit;
    }

    fn should_quit(&self) -> bool {
        self.app_state == AppState::ShouldQuit
    }

    fn is_command_selected(&self) -> bool {
        matches!(self.app_state, AppState::ExecuteCommand(_))
    }

    fn command_to_execute(&self) -> Option<(runner::Runner, command::CommandForExec)> {
        match &self.app_state {
            AppState::ExecuteCommand(command) => {
                let command = command.clone();
                Some((command.executor, command.command))
            }
            _ => None,
        }
    }
}

pub async fn main(config: config::Config) -> Result<()> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let result = AssertUnwindSafe(async {
        match Model::new(config) {
            Ok(mut m) => {
                match run(&mut terminal, &mut m).await {
                    // If async closure will be stabilized, use map instead of match
                    Ok(command) => match command {
                        Some((runner, command)) => Ok(Some((runner, command))),
                        None => Ok(None), // If no command selected, show nothing.
                    },
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    })
    .catch_unwind()
    .await;

    shutdown_terminal(&mut terminal)?;

    match result {
        // some kind of command was selected
        Ok(Ok(Some((runner, command)))) => {
            runner.show_command(&command);
            runner.execute(&command)
        }
        Ok(Ok(None)) => Ok(()), // no command was selected
        Ok(Err(e)) => Err(e),   // Model::new or run returned Err
        Err(e) => {
            // Since panic content is printed by the panic hook defined in main.rs once,
            // it occurs before terminal is shutdown, so we should print it here again.
            use colored::Colorize;

            // Get panic info that was saved in panic hook
            if let Some((location, message)) = crate::panic_info::get_panic_info() {
                eprintln!("{}", format!("thread 'main' panicked at {}", location).red());
                eprintln!("{}", message.red());

                #[cfg(debug_assertions)]
                {
                    eprintln!("\n{}", "Panic details have been appended to `debug_info.txt`".red());
                }
            }

            Err(anyhow!(any_to_string::any_to_string(&*e))) // panic occurred
        }
    }
}

const VERSION_KEY: &str = "version";
async fn run<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    model: &'a mut Model<'a>,
) -> Result<Option<(runner::Runner, command::CommandForExec)>> {
    let shared_version_hash_map = Arc::new(Mutex::new(HashMap::new()));

    let cloned_hash_map = shared_version_hash_map.clone();
    tokio::spawn(get_latest_version(cloned_hash_map));

    loop {
        if let AppState::SelectCommand(s) = &mut model.app_state
            && s.latest_version.is_none()
            && let Some(new_version) = shared_version_hash_map.lock().unwrap().get(VERSION_KEY)
        {
            s.latest_version = Some(new_version.to_string());
        }

        if let Err(e) = terminal.draw(|f| ui(f, model)) {
            return Err(anyhow!(e));
        }
        match handle_event(model) {
            Ok(message) => {
                update(model, message);
                if model.should_quit() || model.is_command_selected() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Ok(model.command_to_execute())
}

fn shutdown_terminal(terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> Result<()> {
    if let Err(e) = disable_raw_mode() {
        return Err(anyhow!(e));
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    if let Err(e) = terminal.show_cursor() {
        return Err(anyhow!(e));
    }

    Ok(())
}

const PKG_NAME: &str = "kyu08/fzf-make";
async fn get_latest_version(share_clone: Arc<Mutex<HashMap<String, String>>>) {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    // check version once a day
    let informer =
        update_informer::new(registry::GitHub, PKG_NAME, current_version).interval(Duration::from_secs(60 * 60 * 24));

    let version_result = task::spawn_blocking(|| informer.check_version().map_err(|e| e.to_string()))
        .await
        .unwrap();
    if let Ok(Some(new_version)) = version_result {
        let mut data = share_clone.lock().unwrap();
        data.insert(VERSION_KEY.to_string(), new_version.to_string().clone());
    };
}

enum Message {
    SearchTextAreaKeyInput(KeyEvent),
    ExecuteCommand(command::CommandForExec),
    NextCommand,
    PreviousCommand,
    MoveToNextPane,
    NextHistory,
    PreviousHistory,
    Quit,
    // Additional arguments
    OpenAdditionalArgumentsWindow,
    CloseAdditionalArgumentsWindow,
    AdditionalArgumentsKeyInput(KeyEvent),
    // Copy command to clipboard
    CopyCommandToClipboard,
}

// TODO: make this method Model's method
fn handle_event(model: &Model) -> io::Result<Option<Message>> {
    if crossterm::event::poll(std::time::Duration::from_millis(500))? {
        match crossterm::event::read()? {
            crossterm::event::Event::Key(key) => Ok(model.handle_key_input(key)),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

// TODO: make this method Model's method
// TODO: Make this function returns `Result` or have a field like Model.error to hold errors
fn update(model: &mut Model, message: Option<Message>) {
    if let AppState::SelectCommand(ref mut s) = model.app_state {
        match message {
            Some(Message::SearchTextAreaKeyInput(key_event)) => s.handle_key_input(key_event),
            Some(Message::ExecuteCommand(command)) => {
                s.store_history(command.clone());
                if let Some(r) = command.runner_type.to_runner(&s.runners) {
                    model.transition_to_execute_command_state(r, command);
                }
            }
            Some(Message::NextCommand) => s.next_command(),
            Some(Message::PreviousCommand) => s.previous_command(),
            Some(Message::MoveToNextPane) => s.move_to_next_pane(),
            Some(Message::NextHistory) => s.next_history(),
            Some(Message::PreviousHistory) => s.previous_history(),
            Some(Message::Quit) => model.transition_to_should_quit_state(),
            Some(Message::OpenAdditionalArgumentsWindow) => s.open_additional_arguments_popup(),
            Some(Message::CloseAdditionalArgumentsWindow) => s.close_additional_arguments_popup(),
            Some(Message::AdditionalArgumentsKeyInput(key_event)) => s.handle_additional_arguments_key_input(key_event),
            Some(Message::CopyCommandToClipboard) => s.copy_command_to_clipboard(),
            None => {}
        }
    }
}

#[derive(Debug)]
pub struct SelectCommandState<'a> {
    pub current_dir: PathBuf,
    pub current_pane: CurrentPane,
    pub runners: Vec<runner::Runner>,
    pub search_text_area: TextArea_<'a>,
    pub commands_list_state: ListState,
    pub history: Vec<histories::HistoryCommand>,
    pub history_list_state: ListState,
    pub additional_arguments_popup_state: Option<AdditionalWindowState<'a>>,
    pub latest_version: Option<String>,
    // The first type variable of Result is intentionally defined as String because we need to get command from
    // the preview pane or the history pane.
    // In the history pane, we don't have file_path and line_number info.
    pub copy_command_state: Option<Result<String, String>>,
}

impl PartialEq for SelectCommandState<'_> {
    fn eq(&self, other: &Self) -> bool {
        let other_than_runners = self.current_pane == other.current_pane
            && self.search_text_area == other.search_text_area
            && self.commands_list_state == other.commands_list_state
            && self.history == other.history
            && self.history_list_state == other.history_list_state;
        if !other_than_runners {
            return false; // Early return for performance
        }

        let mut runner = false;
        for (i, r) in self.runners.iter().enumerate() {
            let other = other.runners.get(i).unwrap();
            if r.path() == other.path() && r.list_commands() == other.list_commands() {
                runner = true;
            } else {
                runner = false;
                break;
            }
        }
        runner
    }
}

impl SelectCommandState<'_> {
    pub fn new(config: config::Config) -> Result<Self> {
        let current_dir = match env::current_dir() {
            Ok(d) => d,
            Err(e) => bail!("Failed to get current directory: {}", e),
        };

        let current_pane = if config.get_focus_history() {
            CurrentPane::History
        } else {
            CurrentPane::Main
        };

        let runners = {
            let mut runners = vec![];

            if let Ok(f) = Make::new(current_dir.clone()) {
                runners.push(Runner::MakeCommand(f));
            };
            if let Some(js_package_manager) = js::get_js_package_manager_runner(current_dir.clone()) {
                runners.push(Runner::JsPackageManager(js_package_manager));
            };
            if let Ok(just) = Just::new(current_dir.clone()) {
                runners.push(Runner::Just(just));
            };
            if let Ok(task) = Task::new(current_dir.clone()) {
                runners.push(Runner::Task(task));
            };
            runners
        };

        if runners.is_empty() {
            Err(anyhow!(
                "No task runner found.\nRun following command to see usage.\nopen \"https://github.com/kyu08/fzf-make?tab=readme-ov-file#-usage\""
            ))
        } else {
            Ok(SelectCommandState {
                current_dir: current_dir.clone(),
                current_pane,
                runners: runners.clone(),
                search_text_area: TextArea_(TextArea::default()),
                commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
                history: Model::get_histories(current_dir),
                history_list_state: ListState::with_selected(ListState::default(), Some(0)),
                additional_arguments_popup_state: None,
                latest_version: None,
                copy_command_state: None,
            })
        }
    }

    pub fn get_selected_command(&self) -> Option<command::CommandForExec> {
        match self.current_pane {
            CurrentPane::Main => self.selected_command().map(|c| c.into()),
            CurrentPane::History => self.selected_history().map(|c| c.into()),
        }
    }

    fn move_to_next_pane(&mut self) {
        match self.current_pane {
            CurrentPane::Main => self.current_pane = CurrentPane::History,
            CurrentPane::History => self.current_pane = CurrentPane::Main,
        }
    }

    fn selected_command(&self) -> Option<command::CommandWithPreview> {
        match self.commands_list_state.selected() {
            Some(i) => self.narrow_down_commands().get(i).cloned(),
            None => None,
        }
    }

    fn selected_history(&self) -> Option<histories::HistoryCommand> {
        let history = self.get_history();
        if history.is_empty() {
            return None;
        }

        match self.history_list_state.selected() {
            Some(i) => history.get(i).cloned(),
            None => None,
        }
    }

    pub fn narrow_down_commands(&self) -> Vec<command::CommandWithPreview> {
        let commands = {
            let mut commands: Vec<command::CommandWithPreview> = Vec::new();
            for runner in &self.runners {
                commands = [commands, runner.list_commands()].concat();
            }
            commands
        };

        if self.search_text_area.0.is_empty() {
            return commands;
        }

        // Store the commands in a temporary map in the form of map[command.to_string()]Command
        let mut temporary_command_map: HashMap<String, command::CommandWithPreview> = HashMap::new();
        for command in &commands {
            temporary_command_map.insert(command.to_string(), command.clone());
        }

        // filter the commands using fuzzy finder based on the user input
        let filtered_list: Vec<String> = {
            let matcher = SkimMatcherV2::default();
            let mut list: Vec<(i64, String)> = commands
                .into_iter()
                .filter_map(|command| {
                    let mut key_input = self.search_text_area.0.lines().join("");
                    key_input.retain(|c| !c.is_whitespace());
                    matcher
                        .fuzzy_indices(&command.to_string(), key_input.as_str())
                        .map(|(score, _)| (score, command.to_string()))
                })
                .collect();

            list.sort_by(|(score1, _), (score2, _)| score1.cmp(score2));
            list.reverse();

            list.into_iter().map(|(_, command)| command).collect()
        };

        let mut result: Vec<command::CommandWithPreview> = Vec::new();
        // Get the filtered values from the temporary map
        for c in filtered_list {
            if let Some(command) = temporary_command_map.get(&c) {
                result.push(command.clone());
            }
        }

        result
    }

    pub fn get_history(&self) -> Vec<histories::HistoryCommand> {
        self.history.clone()
    }

    fn next_command(&mut self) {
        if self.narrow_down_commands().is_empty() {
            self.commands_list_state.select(None);
            return;
        }

        let i = match self.commands_list_state.selected() {
            Some(i) => {
                if self.narrow_down_commands().len() - 1 <= i {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.commands_list_state.select(Some(i));
    }

    fn previous_command(&mut self) {
        if self.narrow_down_commands().is_empty() {
            self.commands_list_state.select(None);
            return;
        }

        let i = match self.commands_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.narrow_down_commands().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.commands_list_state.select(Some(i));
    }

    fn next_history(&mut self) {
        let history_list = self.get_history();
        if history_list.is_empty() {
            self.history_list_state.select(None);
            return;
        };

        let i = match self.history_list_state.selected() {
            Some(i) => {
                if history_list.len() - 1 <= i {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.history_list_state.select(Some(i));
    }

    fn previous_history(&mut self) {
        let history_list_len = self.get_history().len();
        match history_list_len {
            0 => {
                self.history_list_state.select(None);
            }
            _ => {
                let i = match self.history_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            history_list_len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.history_list_state.select(Some(i));
            }
        };
    }

    fn handle_key_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(_) | KeyCode::Backspace => {
                self.reset_selection();
            }
            _ => {}
        }
        self.search_text_area.0.input(key_event);
    }

    fn open_additional_arguments_popup(&mut self) {
        if let Some(command) = self.get_selected_command()
            && self.additional_arguments_popup_state.is_none()
        {
            self.additional_arguments_popup_state = Some(AdditionalWindowState::new(command));
        }
    }

    fn close_additional_arguments_popup(&mut self) {
        if self.additional_arguments_popup_state.is_some() {
            self.additional_arguments_popup_state = None;
        }
    }

    fn handle_additional_arguments_key_input(&mut self, key_event: KeyEvent) {
        if let Some(ref mut additional_window) = self.additional_arguments_popup_state {
            additional_window.arguments_text_area.0.input(key_event);
        }
    }

    fn copy_command_to_clipboard(&mut self) {
        let command = match self.get_selected_command() {
            Some(c) => c.to_string(),
            None => {
                return;
            }
        };

        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&command) {
                Ok(_) => self.copy_command_state = Some(Ok(command.to_string())),
                Err(e) => {
                    self.copy_command_state = Some(Err(e.to_string()));
                }
            },
            Err(e) => {
                self.copy_command_state = Some(Err(e.to_string()));
            }
        }
    }

    fn store_history(&self, command: command::CommandForExec) {
        // NOTE: self.get_selected_command should be called before self.append_history.
        // Because self.histories_list_state.selected keeps the selected index of the history list
        // before update.
        if let Some((dir, file_name)) = toml::history_file_path() {
            let all_histories = toml::Histories::get_history()
                .into()
                .append(self.current_dir.clone(), command);

            // TODO: handle error
            let _ = toml::create_or_update_history_file(dir, file_name, all_histories);
        };
    }

    fn reset_selection(&mut self) {
        if self.narrow_down_commands().is_empty() {
            self.commands_list_state.select(None);
        }
        self.commands_list_state.select(Some(0));
    }

    pub fn get_latest_command(&self) -> Option<&histories::HistoryCommand> {
        self.history.first()
    }

    pub fn get_runner(&self, runner_type: &runner_type::RunnerType) -> Option<runner::Runner> {
        for runner in &self.runners {
            match (runner_type, runner) {
                (runner_type::RunnerType::Make, runner::Runner::MakeCommand(_)) => {
                    return Some(runner.clone());
                }
                (
                    runner_type::RunnerType::JsPackageManager(runner_type_js),
                    runner::Runner::JsPackageManager(runner_js),
                ) => match (runner_type_js, runner_js) {
                    (runner_type::JsPackageManager::Pnpm, js::JsPackageManager::JsPnpm(_)) => {
                        return Some(runner.clone());
                    }

                    (runner_type::JsPackageManager::Yarn, js::JsPackageManager::JsYarn(_)) => {
                        return Some(runner.clone());
                    }

                    // _ patterns. To prevent omission of corrections, _ is not used.
                    (runner_type::JsPackageManager::Pnpm, js::JsPackageManager::JsYarn(_))
                    | (runner_type::JsPackageManager::Yarn, js::JsPackageManager::JsPnpm(_)) => return None,
                },
                _ => continue,
            }
        }
        None
    }

    pub fn is_additional_arguments_popup_opened(&self) -> bool {
        self.additional_arguments_popup_state.is_some()
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        use crate::model::runner_type;

        SelectCommandState {
            current_dir: env::current_dir().unwrap(),
            current_pane: CurrentPane::Main,
            runners: vec![runner::Runner::MakeCommand(Make::new_for_test())],
            search_text_area: TextArea_(TextArea::default()),
            commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
            history: vec![
                histories::HistoryCommand {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history0".to_string(),
                },
                histories::HistoryCommand {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history1".to_string(),
                },
                histories::HistoryCommand {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history2".to_string(),
                },
            ],
            history_list_state: ListState::with_selected(ListState::default(), Some(0)),
            additional_arguments_popup_state: None,
            latest_version: None,
            copy_command_state: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdditionalWindowState<'a> {
    pub arguments_text_area: TextArea_<'a>,
    command: command::CommandForExec,
}

impl AdditionalWindowState<'_> {
    pub fn new(command: command::CommandForExec) -> Self {
        Self {
            arguments_text_area: TextArea_(TextArea::default()),
            command,
        }
    }

    pub fn append_arguments(&self) -> command::CommandForExec {
        let mut new_command = self.command.clone();
        let arguments = self.arguments_text_area.0.lines().join(" ").trim().to_string();
        if !arguments.is_empty() {
            new_command.args.push_str(&format!(" {}", arguments));
        }
        new_command
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecuteCommandState {
    /// It is possible to have one concrete type like Command struct here.
    /// But from the perspective of simpleness of code base, this field has trait object.
    executor: runner::Runner,
    command: command::CommandForExec,
}

impl ExecuteCommandState {
    fn new(executor: runner::Runner, command: command::CommandForExec) -> Self {
        ExecuteCommandState { executor, command }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum CurrentPane {
    Main,
    History,
}

impl CurrentPane {
    pub fn is_main(&self) -> bool {
        matches!(self, CurrentPane::Main)
    }

    pub fn is_history(&self) -> bool {
        matches!(self, CurrentPane::History)
    }
}

#[derive(Clone, Debug)]
pub struct TextArea_<'a>(pub TextArea<'a>);

impl PartialEq for TextArea_<'_> {
    // for testing
    fn eq(&self, other: &Self) -> bool {
        self.0.lines().join("") == other.0.lines().join("")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::runner_type;
    use pretty_assertions::assert_eq;
    use std::env;

    #[test]
    fn update_test() {
        struct Case<'a> {
            title: &'static str,
            model: Model<'a>,
            message: Option<Message>,
            expect_model: Model<'a>,
        }
        let cases: Vec<Case> = vec![
            Case {
                title: "MoveToNextPane(Main -> History)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::Main,
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::MoveToNextPane),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "MoveToNextPane(History -> Main)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::MoveToNextPane),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::Main,
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "Quit",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::Quit),
                expect_model: Model {
                    app_state: AppState::ShouldQuit,
                },
            },
            Case {
                title: "SearchTextAreaKeyInput(a)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::SearchTextAreaKeyInput(KeyEvent::from(KeyCode::Char('a')))),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('a')));
                            TextArea_(text_area)
                        },
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "when BackSpace is inputted, the selection should be reset",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::SearchTextAreaKeyInput(KeyEvent::from(KeyCode::Backspace))),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "Next(0 -> 1)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::NextCommand),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "Next(2 -> 0)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(2)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::NextCommand),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "Previous(1 -> 0)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::PreviousCommand),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "Previous(0 -> 2)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::PreviousCommand),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(2)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "ExecuteCommand(Main)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::ExecuteCommand(command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "target0".to_string(),
                })),
                expect_model: Model {
                    app_state: AppState::ExecuteCommand(ExecuteCommandState::new(
                        runner::Runner::MakeCommand(Make::new_for_test()),
                        command::CommandForExec {
                            runner_type: runner_type::RunnerType::Make,
                            args: "target0".to_string(),
                        },
                    )),
                },
            },
            Case {
                title: "ExecuteCommand(History)",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::ExecuteCommand(command::CommandForExec {
                    runner_type: runner_type::RunnerType::Make,
                    args: "history1".to_string(),
                })),
                expect_model: Model {
                    app_state: AppState::ExecuteCommand(ExecuteCommandState::new(
                        runner::Runner::MakeCommand(Make::new_for_test()),
                        command::CommandForExec {
                            runner_type: runner_type::RunnerType::Make,
                            args: "history1".to_string(),
                        },
                    )),
                },
            },
            Case {
                title: "Selecting position should be reset if some kind of char
                    was inputted when the command located not in top of the commands",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::SearchTextAreaKeyInput(KeyEvent::from(KeyCode::Char('a')))),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('a')));
                            TextArea_(text_area)
                        },
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "NextCommand when there is no commands to select, panic should not occur",
                model: {
                    let mut m = Model {
                        app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                            commands_list_state: ListState::with_selected(ListState::default(), None),
                            ..SelectCommandState::new_for_test()
                        })),
                    };
                    update(
                        // There should not be commands because init_model has ["target0", "target1", "target2"] as command.
                        &mut m,
                        Some(Message::SearchTextAreaKeyInput(KeyEvent::from(KeyCode::Char('w')))),
                    );
                    m
                },
                message: Some(Message::NextCommand),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), None),
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('w')));
                            TextArea_(text_area)
                        },
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "PreviousCommand when there is no commands to select,
                    panic should not occur",
                model: {
                    let mut m = Model {
                        app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                            commands_list_state: ListState::with_selected(ListState::default(), None),
                            ..SelectCommandState::new_for_test()
                        })),
                    };
                    update(
                        // There should not be commands because init_model has ["target0", "target1", "target2"] as command.
                        &mut m,
                        Some(Message::SearchTextAreaKeyInput(KeyEvent::from(KeyCode::Char('w')))),
                    );
                    m
                },
                message: Some(Message::PreviousCommand),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        commands_list_state: ListState::with_selected(ListState::default(), None),
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('w')));
                            TextArea_(text_area)
                        },
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "NextHistory",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::NextHistory),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "PreviousHistory",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::NextHistory),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "When the last history is selected and NextHistory is received,
                    it returns to the beginning.",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(2)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::NextHistory),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
            Case {
                title: "When the first history is selected and PreviousHistory is received,
                    it moves to the last history.",
                model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
                message: Some(Message::PreviousHistory),
                expect_model: Model {
                    app_state: AppState::SelectCommand(Box::new(SelectCommandState {
                        current_pane: CurrentPane::History,
                        history_list_state: ListState::with_selected(ListState::default(), Some(2)),
                        ..SelectCommandState::new_for_test()
                    })),
                },
            },
        ];

        // NOTE: When running tests, you need to set FZF_MAKE_IS_TESTING=true. Otherwise, the developer's history file will be overwritten.
        unsafe { env::set_var("FZF_MAKE_IS_TESTING", "true") };

        for mut case in cases {
            update(&mut case.model, case.message);
            assert_eq!(case.expect_model, case.model, "\nFailed: 🚨{:?}🚨\n", case.title,);
        }
    }
}
