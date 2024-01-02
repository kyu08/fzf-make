use crate::models::makefile::Makefile;

use super::ui::ui;
use anyhow::{anyhow, Result};
use colored::Colorize;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::ListState,
    Terminal,
};
use std::{
    io::{self, Stderr},
    panic, process,
};
use tui_textarea::TextArea;

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

enum Message {
    MoveToNextPane,
    Quit,
    SearchTextAreaKeyInput(KeyEvent),
    Next,
    Previous,
    ExecuteTarget,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Model<'a> {
    pub current_pane: CurrentPane,
    pub key_input: String,
    pub makefile: Makefile,
    // TODO: It is better make `should_quit` like following `quit || notQuuitYe || executeTarget (String)`.
    pub should_quit: bool,
    pub targets_list_state: ListState,
    pub selected_target: Option<String>,
    // TODO: key_inputと二重管理になっているの見直す
    pub search_text_area: TextArea_<'a>,
}

#[derive(Clone, Debug)]
pub struct TextArea_<'a>(pub TextArea<'a>);

impl<'a> PartialEq for TextArea_<'a> {
    // for testing
    fn eq(&self, other: &Self) -> bool {
        self.0.lines().join("") == other.0.lines().join("")
    }
}

impl Model<'_> {
    pub fn new() -> Result<Self> {
        let makefile = match Makefile::create_makefile() {
            Err(e) => return Err(e),
            Ok(f) => f,
        };
        let text_area = TextArea::default();
        Ok(Model {
            key_input: String::new(),
            current_pane: CurrentPane::Main,
            should_quit: false,
            makefile: makefile.clone(),
            targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
            selected_target: None,
            search_text_area: TextArea_(text_area),
        })
    }

    pub fn update_key_input(&self, key_input: String) -> String {
        self.key_input.clone() + &key_input
    }

    pub fn narrow_down_targets(&self) -> Vec<String> {
        if self.key_input.is_empty() {
            return self.makefile.to_targets_string();
        }

        let matcher = SkimMatcherV2::default();
        let mut filtered_list: Vec<(Option<i64>, String)> = self
            .makefile
            .to_targets_string()
            .into_iter()
            .map(|target| {
                let mut key_input = self.key_input.clone();
                key_input.retain(|c| !c.is_whitespace());
                match matcher.fuzzy_indices(&target, key_input.as_str()) {
                    Some((score, _)) => (Some(score), target),
                    None => (None, String::new()),
                }
            })
            .filter(|(score, _)| score.is_some())
            .collect();

        filtered_list.sort_by(|(score1, _), (score2, _)| score1.cmp(score2));
        filtered_list.reverse();

        filtered_list
            .into_iter()
            .map(|(_, target)| target)
            .collect()
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn reset_select(&mut self) {
        if self.makefile.to_targets_string().is_empty() {
            self.targets_list_state.select(None);
        }
        self.targets_list_state.select(Some(0));
    }
}

pub fn main() -> Result<()> {
    let result = panic::catch_unwind(|| {
        enable_raw_mode()?;
        let mut stderr = io::stderr();
        execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stderr);
        let mut terminal = Terminal::new(backend)?;

        let target: Result<Option<String>> = match Model::new() {
            Err(e) => Err(e),
            Ok(model) => run(&mut terminal, model),
        };

        let target = match target {
            Ok(t) => t,
            Err(e) => {
                shutdown_terminal(&mut terminal)?;
                return Err(e);
            }
        };

        shutdown_terminal(&mut terminal)?;

        match target {
            Some(t) => {
                // Make output color configurable via config file https://github.com/kyu08/fzf-make/issues/67
                println!("{}", ("make ".to_string() + &t).blue());
                process::Command::new("make")
                    .stdin(process::Stdio::inherit())
                    .arg(t)
                    .spawn()
                    .expect("Failed to execute process")
                    .wait()
                    .expect("Failed to execute process");

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

fn run<B: Backend>(terminal: &mut Terminal<B>, mut model: Model) -> Result<Option<String>> {
    loop {
        if let Err(e) = terminal.draw(|f| ui(f, &mut model.clone())) {
            return Err(anyhow!(e));
        }
        match handle_event(&model) {
            Ok(message) => {
                update(&mut model, message);
                if model.should_quit || model.selected_target.is_some() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Ok(model.selected_target)
}

fn handle_event(model: &Model) -> io::Result<Option<Message>> {
    let message = if crossterm::event::poll(std::time::Duration::from_millis(2000))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Tab => Some(Message::MoveToNextPane),
                KeyCode::Esc => Some(Message::Quit),
                _ => match model.current_pane {
                    CurrentPane::Main => match key.code {
                        KeyCode::Down => Some(Message::Next),
                        KeyCode::Up => Some(Message::Previous),
                        KeyCode::Enter => Some(Message::ExecuteTarget),
                        _ => Some(Message::SearchTextAreaKeyInput(key)),
                    },
                    CurrentPane::History => match key.code {
                        KeyCode::Char('q') => Some(Message::Quit),
                        _ => None,
                    },
                },
            }
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };
    Ok(message)
}

fn update(model: &mut Model, message: Option<Message>) {
    match message {
        Some(Message::MoveToNextPane) => match model.current_pane {
            CurrentPane::Main => model.current_pane = CurrentPane::History,
            CurrentPane::History => model.current_pane = CurrentPane::Main,
        },
        Some(Message::Quit) => model.should_quit = true,
        Some(Message::Next) => model.next(),
        Some(Message::Previous) => model.previous(),
        Some(Message::ExecuteTarget) => {
            model.selected_target = model
                .targets_list_state
                .selected()
                .map(|i| model.narrow_down_targets()[i].clone());
        }
        Some(Message::SearchTextAreaKeyInput(key_event)) => {
            // TODO: これを参考にして改行するイベントを無視する https://github.com/rhysd/tui-textarea?tab=readme-ov-file#single-line-input-like-input-in-html
            if let KeyCode::Char(key_input) = key_event.code {
                model.key_input = model.update_key_input(key_input.to_string());
                model.reset_select();
            };
            model.search_text_area.0.input(key_event);
            model.key_input = model
                .search_text_area
                .0
                .lines()
                .first()
                .unwrap()
                .to_string();
        }
        None => {}
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn update_test() {
        struct Case<'a> {
            title: &'static str,
            model: Model<'a>,
            message: Option<Message>,
            expect_model: Model<'a>,
        }
        let cases: Vec<Case> = vec![];
        // let cases: Vec<Case> = vec![Case {
        //     title: "MoveToNextPane",
        //     model: Model::default(),
        //     message: Some(Message::MoveToNextPane),
        //     expect_model: Model {
        //         current_pane: CurrentPane::History,
        //         ..Model::default()
        //     },
        // }];

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
