use crate::{
    file::toml,
    model::{
        command,
        histories::{self, history_file_path},
        make::Make,
        runner, runner_type,
    },
};

use super::{config, ui::ui};
use anyhow::{anyhow, bail, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::ListState,
    Terminal,
};
use std::{
    collections::HashMap,
    env,
    io::{self, Stderr},
    panic,
    path::PathBuf,
    process,
};
use tui_textarea::TextArea;

// #[cfg(test)]
// use crate::model::histories::Histories;

// AppState represents the state of the application.
// "Making impossible states impossible"
// The type of `AppState` is defined according to the concept of 'Making Impossible States Impossible'.
// See: https://www.youtube.com/watch?v=IcgmSRJHu_8
#[derive(PartialEq, Debug)]
pub enum AppState<'a> {
    SelectTarget(SelectTargetState<'a>),
    ExecuteTarget(ExecuteTargetState),
    ShouldQuit,
}

#[derive(PartialEq, Debug)]
pub struct Model<'a> {
    pub app_state: AppState<'a>,
}

impl Model<'_> {
    pub fn new(config: config::Config) -> Result<Self> {
        match SelectTargetState::new(config) {
            Ok(s) => Ok(Model {
                app_state: AppState::SelectTarget(s),
            }),
            Err(e) => Err(e),
        }
    }

    fn handle_key_input(&self, key: KeyEvent) -> Option<Message> {
        match &self.app_state {
            AppState::SelectTarget(s) => match key.code {
                KeyCode::Tab => Some(Message::MoveToNextPane),
                KeyCode::Esc => Some(Message::Quit),
                _ => match s.current_pane {
                    CurrentPane::Main => match key.code {
                        KeyCode::Down => Some(Message::NextTarget),
                        KeyCode::Up => Some(Message::PreviousTarget),
                        KeyCode::Enter => Some(Message::ExecuteTarget),
                        _ => Some(Message::SearchTextAreaKeyInput(key)),
                    },
                    CurrentPane::History => match key.code {
                        KeyCode::Char('q') => Some(Message::Quit),
                        KeyCode::Down => Some(Message::NextHistory),
                        KeyCode::Up => Some(Message::PreviousHistory),
                        KeyCode::Enter | KeyCode::Char(' ') => Some(Message::ExecuteTarget),
                        _ => None,
                    },
                },
            },
            _ => None,
        }
    }

    // returns available commands in cwd from history file
    fn get_histories(
        current_working_directory: PathBuf,
        runners: Vec<runner::Runner>,
    ) -> Vec<command::Command> {
        let histories = toml::Histories::into(toml::Histories::get_history());

        let mut result: Vec<command::Command> = Vec::new();
        for history in histories.histories {
            if history.path != current_working_directory {
                continue;
            }
            result = Self::get_commands_from_history(history.commands, &runners);
            break;
        }
        result
    }

    fn get_commands_from_history(
        history_commands: Vec<histories::HistoryCommand>,
        runners: &Vec<runner::Runner>,
    ) -> Vec<command::Command> {
        // TODO: Make this more readable and more performant.
        let mut commands: Vec<command::Command> = Vec::new();
        for history_command in history_commands {
            match history_command.runner_type {
                runner_type::RunnerType::Make => {
                    for runner in runners {
                        if let runner::Runner::MakeCommand(make) = runner {
                            // PERF: This method is called every time. Memoize should be considered.
                            for c in make.to_commands() {
                                if c.name == history_command.name {
                                    commands.push(c);
                                    break;
                                }
                            }
                        }
                    }
                }
                runner_type::RunnerType::Pnpm => todo!(),
            };
        }
        commands
    }

    fn transition_to_execute_target_state(
        &mut self,
        runner: runner::Runner,
        command: command::Command,
    ) {
        self.app_state = AppState::ExecuteTarget(ExecuteTargetState::new(runner, command));
    }

    fn transition_to_should_quit_state(&mut self) {
        // TODO: remove mut
        self.app_state = AppState::ShouldQuit;
    }

    pub fn should_quit(&self) -> bool {
        self.app_state == AppState::ShouldQuit
    }

    pub fn is_target_selected(&self) -> bool {
        matches!(self.app_state, AppState::ExecuteTarget(_))
    }

    pub fn command_to_execute(&self) -> Option<(runner::Runner, command::Command)> {
        match &self.app_state {
            AppState::ExecuteTarget(command) => {
                let command = command.clone();
                Some((command.executor, command.command))
            }
            _ => None,
        }
    }
}

pub fn main(config: config::Config) -> Result<()> {
    let result = panic::catch_unwind(|| {
        enable_raw_mode()?;
        let mut stderr = io::stderr();
        execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stderr);
        let mut terminal = Terminal::new(backend)?;

        let model = Model::new(config);
        if let Err(e) = model {
            shutdown_terminal(&mut terminal)?;
            return Err(e);
        }
        let mut model = model.unwrap();

        let target = match run(&mut terminal, &mut model) {
            Ok(t) => t,
            Err(e) => {
                shutdown_terminal(&mut terminal)?;
                return Err(e);
            }
        };

        shutdown_terminal(&mut terminal)?;

        match target {
            Some((runner, command)) => {
                runner.show_command(&command);
                let _ = runner.execute(&command); // TODO: handle error
                Ok(())
            }
            None => Ok(()),
        }
    });

    match result {
        Ok(usecase_result) => usecase_result,
        Err(e) => {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            println!("panic: {:?}", e);
            process::exit(1);
        }
    }
}

fn run<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    model: &'a mut Model<'a>,
) -> Result<Option<(runner::Runner, command::Command)>> {
    loop {
        if let Err(e) = terminal.draw(|f| ui(f, model)) {
            return Err(anyhow!(e));
        }
        match handle_event(model) {
            Ok(message) => {
                update(model, message);
                if model.should_quit() || model.is_target_selected() {
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

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    if let Err(e) = terminal.show_cursor() {
        return Err(anyhow!(e));
    }

    Ok(())
}

enum Message {
    SearchTextAreaKeyInput(KeyEvent),
    ExecuteTarget,
    NextTarget,
    PreviousTarget,
    MoveToNextPane,
    NextHistory,
    PreviousHistory,
    Quit,
}

// TODO: make this method Model's method
fn handle_event(model: &Model) -> io::Result<Option<Message>> {
    match (
        crossterm::event::poll(std::time::Duration::from_millis(2000))?,
        crossterm::event::read()?,
    ) {
        (true, crossterm::event::Event::Key(key)) => Ok(model.handle_key_input(key)),
        _ => Ok(None),
    }
}

// TODO: make this method Model's method
// TODO: Make this function returns `Result` or have a field like Model.error to hold errors
fn update(model: &mut Model, message: Option<Message>) {
    if let AppState::SelectTarget(ref mut s) = model.app_state {
        match message {
            Some(Message::SearchTextAreaKeyInput(key_event)) => s.handle_key_input(key_event),
            Some(Message::ExecuteTarget) => {
                if let Some(command) = s.get_selected_target() {
                    // TODO: make this a method of SelectTargetState
                    s.store_history(&command);
                    let executor: runner::Runner = s.runners[0].clone();

                    model.transition_to_execute_target_state(executor, command);
                };
            }
            Some(Message::NextTarget) => s.next_target(),
            Some(Message::PreviousTarget) => s.previous_target(),
            Some(Message::MoveToNextPane) => s.move_to_next_pane(),
            Some(Message::NextHistory) => s.next_history(),
            Some(Message::PreviousHistory) => s.previous_history(),
            Some(Message::Quit) => model.transition_to_should_quit_state(),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct SelectTargetState<'a> {
    pub current_dir: PathBuf,
    pub current_pane: CurrentPane,
    pub runners: Vec<runner::Runner>,
    pub search_text_area: TextArea_<'a>,
    pub targets_list_state: ListState,
    // このフィールドをVec<Histories>にするかこのままにするか
    // 前者
    // 実行時に変換が必要
    // 後者
    // 保存時に変換が必要
    // 存在しないcommandを非表示にできる
    // 将来的にpreviewできる
    // TODO: ↑をコメントとして記述する
    pub histories: Vec<command::Command>,
    pub histories_list_state: ListState,
}

impl PartialEq for SelectTargetState<'_> {
    fn eq(&self, other: &Self) -> bool {
        let without_runners = self.current_pane == other.current_pane
            && self.search_text_area == other.search_text_area
            && self.targets_list_state == other.targets_list_state
            && self.histories == other.histories
            && self.histories_list_state == other.histories_list_state;
        if !without_runners {
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

impl SelectTargetState<'_> {
    pub fn new(config: config::Config) -> Result<Self> {
        let current_dir = match env::current_dir() {
            Ok(d) => d,
            Err(e) => bail!("Failed to get current directory: {}", e),
        };
        let makefile = match Make::create_makefile() {
            Err(e) => return Err(e),
            Ok(f) => f,
        };

        let current_pane = if config.get_focus_history() {
            CurrentPane::History
        } else {
            CurrentPane::Main
        };
        let runner = { runner::Runner::MakeCommand(makefile) };
        let runners = vec![runner];

        Ok(SelectTargetState {
            current_dir: current_dir.clone(),
            current_pane,
            runners: runners.clone(),
            search_text_area: TextArea_(TextArea::default()),
            targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
            histories: Model::get_histories(current_dir, runners),
            histories_list_state: ListState::with_selected(ListState::default(), Some(0)),
        })
    }

    fn get_selected_target(&self) -> Option<command::Command> {
        match self.current_pane {
            CurrentPane::Main => self.selected_target(),
            CurrentPane::History => self.selected_history(),
        }
    }

    fn move_to_next_pane(&mut self) {
        match self.current_pane {
            CurrentPane::Main => self.current_pane = CurrentPane::History,
            CurrentPane::History => self.current_pane = CurrentPane::Main,
        }
    }

    fn selected_target(&self) -> Option<command::Command> {
        match self.targets_list_state.selected() {
            Some(i) => self.narrow_down_targets().get(i).cloned(),
            None => None,
        }
    }

    fn selected_history(&self) -> Option<command::Command> {
        let history = self.get_history();
        if history.is_empty() {
            return None;
        }

        match self.histories_list_state.selected() {
            Some(i) => history.get(i).cloned(),
            None => None,
        }
    }

    pub fn narrow_down_targets(&self) -> Vec<command::Command> {
        let commands = {
            let mut commands: Vec<command::Command> = Vec::new();
            for runner in &self.runners {
                commands = [commands, runner.list_commands()].concat();
            }
            commands
        };

        if self.search_text_area.0.is_empty() {
            return commands;
        }

        // Store the commands in a temporary map in the form of map[command.to_string()]Command
        let mut temporary_command_map: HashMap<String, command::Command> = HashMap::new();
        for command in &commands {
            temporary_command_map.insert(command.to_string(), command.clone());
        }

        // filter the commands using fuzzy finder based on the user input
        let filtered_list: Vec<String> = {
            let matcher = SkimMatcherV2::default();
            let mut list: Vec<(i64, String)> = commands
                .into_iter()
                .filter_map(|target| {
                    let mut key_input = self.search_text_area.0.lines().join("");
                    key_input.retain(|c| !c.is_whitespace());
                    matcher
                        .fuzzy_indices(&target.to_string(), key_input.as_str())
                        .map(|(score, _)| (score, target.to_string()))
                })
                .collect();

            list.sort_by(|(score1, _), (score2, _)| score1.cmp(score2));
            list.reverse();

            list.into_iter().map(|(_, target)| target).collect()
        };

        let mut result: Vec<command::Command> = Vec::new();
        // Get the filtered values from the temporary map
        for c in filtered_list {
            if let Some(command) = temporary_command_map.get(&c) {
                result.push(command.clone());
            }
        }

        result
    }

    pub fn get_history(&self) -> Vec<command::Command> {
        // MEMO: mainではhistoriesの中からmakefile_pathのhistoryを取得する関数。
        // cwdの履歴だけ取得するようにすればこの関数はいらなくなるかも。
        // TODO:
        // そもそもapplication側ではcommand::Commandだけをhistoryとして持つべき。そうすれば実行時も楽だしpreviewも楽に出すことができる

        // TODO(#321): この関数内で
        // historyにあるcommandをself.runnersから取得するよう(行数やファイル名を最新状態からとってこないとちゃんとプレビュー表示できないため)(e.g. ファイル行番号が変わってる場合プレビューがずれる)
        vec![]
        // TODO(#321): implement when history function is implemented
        // UIに表示するためのhistory一覧を取得する関数。
        // runnersを渡すと関連するhistory一覧を返すようにするのがよさそう。
        // let paths = self
        //     .runners
        //     .iter()
        //     .map(|r| r.path())
        //     .collect::<Vec<PathBuf>>();
        //
        // self.histories
        //     .clone()
        //     .map_or(Vec::new(), |h| h.get_histories(paths))
    }

    fn next_target(&mut self) {
        if self.narrow_down_targets().is_empty() {
            self.targets_list_state.select(None);
            return;
        }

        let i = match self.targets_list_state.selected() {
            Some(i) => {
                if self.narrow_down_targets().len() - 1 <= i {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.targets_list_state.select(Some(i));
    }

    fn previous_target(&mut self) {
        if self.narrow_down_targets().is_empty() {
            self.targets_list_state.select(None);
            return;
        }

        let i = match self.targets_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.narrow_down_targets().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.targets_list_state.select(Some(i));
    }

    fn next_history(&mut self) {
        let history_list = self.get_history();
        if history_list.is_empty() {
            self.histories_list_state.select(None);
            return;
        };

        let i = match self.histories_list_state.selected() {
            Some(i) => {
                if history_list.len() - 1 <= i {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.histories_list_state.select(Some(i));
    }

    fn previous_history(&mut self) {
        let history_list_len = self.get_history().len();
        match history_list_len {
            0 => {
                self.histories_list_state.select(None);
            }
            _ => {
                let i = match self.histories_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            history_list_len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.histories_list_state.select(Some(i));
            }
        };
    }

    fn handle_key_input(&mut self, key_event: KeyEvent) {
        if let KeyCode::Char(_) = key_event.code {
            self.reset_selection();
        };
        self.search_text_area.0.input(key_event);
    }

    fn store_history(&self, command: &command::Command) {
        // NOTE: self.get_selected_target should be called before self.append_history.
        // Because self.histories_list_state.selected keeps the selected index of the history list
        // before update.
        if let Some((dir, file_name)) = history_file_path() {
            let new_history_commands: Vec<histories::HistoryCommand> =
                [vec![command.clone()], self.histories.clone()]
                    .concat()
                    .iter()
                    .map(|c| histories::HistoryCommand::from(c.clone()))
                    .collect();
            let history = histories::History {
                path: self.current_dir.clone(),
                commands: new_history_commands,
            };

            // TODO: handle error
            let _ = toml::store_history(self.current_dir.clone(), dir, file_name, history);
        };
    }

    fn reset_selection(&mut self) {
        if self.narrow_down_targets().is_empty() {
            self.targets_list_state.select(None);
        }
        self.targets_list_state.select(Some(0));
    }

    pub fn get_search_area_text(&self) -> String {
        self.search_text_area.0.lines().join("")
    }

    pub fn get_latest_command(&self) -> Option<&command::Command> {
        self.histories.first()
    }

    pub fn get_runner(&self, runner_type: &runner_type::RunnerType) -> Option<runner::Runner> {
        for runner in &self.runners {
            match (runner_type, runner) {
                (runner_type::RunnerType::Make, runner::Runner::MakeCommand(_)) => {
                    return Some(runner.clone());
                }
                (runner_type::RunnerType::Pnpm, runner::Runner::PnpmCommand(_)) => {
                    return Some(runner.clone());
                }
                _ => continue,
            }
        }
        None
    }

    #[cfg(test)]
    // fn init_histories(history_commands: Vec<histories::HistoryCommand>) -> Histories {
    //     use std::path::Path;
    //
    //     let mut commands: Vec<histories::HistoryCommand> = Vec::new();
    //
    //     for h in history_commands {
    //         commands.push(histories::HistoryCommand {
    //             runner_type: runner_type::RunnerType::Make,
    //             name: h.name,
    //         });
    //     }
    //
    //     Histories {
    //         histories: vec![histories::History {
    //             path: env::current_dir().unwrap().join(Path::new("Test.mk")),
    //             commands: commands,
    //         }],
    //     }
    // }
    #[cfg(test)]
    fn new_for_test() -> Self {
        use crate::model::runner_type;

        SelectTargetState {
            current_dir: env::current_dir().unwrap(),
            current_pane: CurrentPane::Main,
            runners: vec![runner::Runner::MakeCommand(Make::new_for_test())],
            search_text_area: TextArea_(TextArea::default()),
            targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
            histories: vec![
                command::Command {
                    runner_type: runner_type::RunnerType::Make,
                    name: "history0".to_string(),
                    file_name: PathBuf::from("Makefile"),
                    line_number: 1,
                },
                command::Command {
                    runner_type: runner_type::RunnerType::Make,
                    name: "history1".to_string(),
                    file_name: PathBuf::from("Makefile"),
                    line_number: 4,
                },
                command::Command {
                    runner_type: runner_type::RunnerType::Make,
                    name: "history2".to_string(),
                    file_name: PathBuf::from("Makefile"),
                    line_number: 7,
                },
            ],
            histories_list_state: ListState::with_selected(ListState::default(), Some(0)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecuteTargetState {
    /// It is possible to have one concrete type like Command struct here.
    /// But from the perspective of simpleness of code base, this field has trait object.
    executor: runner::Runner,
    command: command::Command,
}

impl ExecuteTargetState {
    fn new(executor: runner::Runner, command: command::Command) -> Self {
        ExecuteTargetState { executor, command }
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

impl<'a> PartialEq for TextArea_<'a> {
    // for testing
    fn eq(&self, other: &Self) -> bool {
        self.0.lines().join("") == other.0.lines().join("")
    }
}

#[cfg(test)]
mod test {
    use crate::model::runner_type;

    use super::*;
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
                    app_state: AppState::SelectTarget(SelectTargetState {
                        current_pane: CurrentPane::Main,
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::MoveToNextPane),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        current_pane: CurrentPane::History,
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "MoveToNextPane(History -> Main)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        current_pane: CurrentPane::History,
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::MoveToNextPane),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        current_pane: CurrentPane::Main,
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "Quit",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::Quit),
                expect_model: Model {
                    app_state: AppState::ShouldQuit,
                },
            },
            Case {
                title: "SearchTextAreaKeyInput(a)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::SearchTextAreaKeyInput(KeyEvent::from(
                    KeyCode::Char('a'),
                ))),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('a')));
                            TextArea_(text_area)
                        },
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "Next(0 -> 1)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::NextTarget),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "Next(2 -> 0)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(2)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::NextTarget),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "Previous(1 -> 0)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::PreviousTarget),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "Previous(0 -> 2)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::PreviousTarget),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(2)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "ExecuteTarget(Main)",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::ExecuteTarget),
                expect_model: Model {
                    app_state: AppState::ExecuteTarget(ExecuteTargetState::new(
                        runner::Runner::MakeCommand(Make::new_for_test()),
                        command::Command::new(
                            runner_type::RunnerType::Make,
                            "target0".to_string(),
                            PathBuf::new(),
                            1,
                        ),
                    )),
                },
            },
            // TODO(#321): comment in this test
            // Case {
            //     title: "ExecuteTarget(History)",
            //     model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(1),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            //     message: Some(Message::ExecuteTarget),
            //     expect_model: Model {
            //         app_state: AppState::ExecuteTarget(ExecuteTargetState::new(
            //             runner::Runner::MakeCommand(Make::new_for_test()),
            //             command::Command::new(
            //                 runner_type::RunnerType::Make,
            //                 "history1".to_string(),
            //                 PathBuf::new(),
            //                 4,
            //             )
            //         )),
            //     },
            // },
            Case {
                title: "Selecting position should be reset if some kind of char
                    was inputted when the target located not in top of the targets",
                model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(1)),
                        ..SelectTargetState::new_for_test()
                    }),
                },
                message: Some(Message::SearchTextAreaKeyInput(KeyEvent::from(
                    KeyCode::Char('a'),
                ))),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('a')));
                            TextArea_(text_area)
                        },
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "NextTarget when there is no targets to select, panic should not occur",
                model: {
                    let mut m = Model {
                        app_state: AppState::SelectTarget(SelectTargetState {
                            targets_list_state: ListState::with_selected(
                                ListState::default(),
                                None,
                            ),
                            ..SelectTargetState::new_for_test()
                        }),
                    };
                    update(
                        // There should not be targets because init_model has ["target0", "target1", "target2"] as target.
                        &mut m,
                        Some(Message::SearchTextAreaKeyInput(KeyEvent::from(
                            KeyCode::Char('w'),
                        ))),
                    );
                    m
                },
                message: Some(Message::NextTarget),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), None),
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('w')));
                            TextArea_(text_area)
                        },
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            Case {
                title: "PreviousTarget when there is no targets to select, panic should not occur",
                model: {
                    let mut m = Model {
                        app_state: AppState::SelectTarget(SelectTargetState {
                            targets_list_state: ListState::with_selected(
                                ListState::default(),
                                None,
                            ),
                            ..SelectTargetState::new_for_test()
                        }),
                    };
                    update(
                        // There should not be targets because init_model has ["target0", "target1", "target2"] as target.
                        &mut m,
                        Some(Message::SearchTextAreaKeyInput(KeyEvent::from(
                            KeyCode::Char('w'),
                        ))),
                    );
                    m
                },
                message: Some(Message::PreviousTarget),
                expect_model: Model {
                    app_state: AppState::SelectTarget(SelectTargetState {
                        targets_list_state: ListState::with_selected(ListState::default(), None),
                        search_text_area: {
                            let mut text_area = TextArea::default();
                            text_area.input(KeyEvent::from(KeyCode::Char('w')));
                            TextArea_(text_area)
                        },
                        ..SelectTargetState::new_for_test()
                    }),
                },
            },
            // TODO(#321): comment in this test
            // Case {
            //     title: "NextHistory",
            //     model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(0),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            //     message: Some(Message::NextHistory),
            //     expect_model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(1),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            // },
            // TODO(#321): comment in this test
            // Case {
            //     title: "PreviousHistory",
            //     model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(0),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            //     message: Some(Message::NextHistory),
            //     expect_model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(1),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            // },
            // TODO(#321): comment in this test
            // Case {
            //     title: "When the last history is selected and NextHistory is received, it returns to the beginning.",
            //     model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(2),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            //     message: Some(Message::NextHistory),
            //     expect_model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(0),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            // },
            // TODO(#321): comment in this test
            // Case {
            //     title: "When the first history is selected and PreviousHistory is received, it moves to the last history.",
            //     model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(0),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            //     message: Some(Message::PreviousHistory),
            //     expect_model: Model {
            //         app_state: AppState::SelectTarget(SelectTargetState {
            //             current_pane: CurrentPane::History,
            //             histories_list_state: ListState::with_selected(
            //                 ListState::default(),
            //                 Some(2),
            //             ),
            //             ..SelectTargetState::new_for_test()
            //         }),
            //     },
            // },
        ];

        // NOTE: When running tests, you need to set FZF_MAKE_IS_TESTING=true. Otherwise, the developer's history file will be overwritten.
        env::set_var("FZF_MAKE_IS_TESTING", "true");

        for mut case in cases {
            update(&mut case.model, case.message);
            assert_eq!(
                case.expect_model, case.model,
                "\nFailed: 🚨{:?}🚨\n",
                case.title,
            );
        }
    }
}
