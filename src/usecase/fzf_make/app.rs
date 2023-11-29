use crate::models::makefile::Makefile;

use super::ui::ui;
use anyhow::{anyhow, Result};
use colored::*;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
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

#[derive(Clone)]
pub enum CurrentPain {
    Main,
    History,
}

impl CurrentPain {
    pub fn is_main(&self) -> bool {
        matches!(self, CurrentPain::Main)
    }

    pub fn is_history(&self) -> bool {
        matches!(self, CurrentPain::History)
    }
}

enum Message {
    MoveToNextPain,
    Quit,
    KeyInput(String),
    Backspace, // TODO: Delegate to rhysd/tui-textarea
    DeleteAll,
    Next,
    Previous,
    ExecuteTarget,
}

#[derive(Clone)]
pub struct Model {
    pub current_pain: CurrentPain,
    pub key_input: String,
    pub makefile: Makefile,
    // TODO: It is better make `should_quit` like following `quit || notQuuitYe || executeTarget (String)`.
    pub should_quit: bool,
    pub targets_list_state: ListState,
    pub selected_target: Option<String>,
}

impl Model {
    pub fn new() -> Result<Self> {
        let makefile = match Makefile::create_makefile() {
            Err(e) => return Err(e),
            Ok(f) => f,
        };

        Ok(Model {
            key_input: String::new(),
            current_pain: CurrentPain::Main,
            should_quit: false,
            makefile: makefile.clone(),
            targets_list_state: ListState::with_selected(ListState::default(), Some(0)),
            selected_target: None,
        })
    }

    pub fn update_key_input(&self, key_input: String) -> String {
        self.key_input.clone() + &key_input
    }

    pub fn pop(&self) -> String {
        let mut origin = self.key_input.clone();
        origin.pop();
        origin
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

    fn reset(&mut self) {
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
                print_error(&e);
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
        Ok(_) => Ok(()),
        Err(e) => {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            println!("panic: {:?}", e);
            process::exit(1);
        }
    }
}

// TODO: いずれはmainかcontrollerに移動するはず
fn print_error(e: &anyhow::Error) {
    println!("{}", e.to_string().red());
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
    let message = if crossterm::event::poll(std::time::Duration::from_millis(250))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Tab => Some(Message::MoveToNextPain),
                KeyCode::Esc => Some(Message::Quit),
                _ => match model.current_pain {
                    CurrentPain::Main => match key.code {
                        KeyCode::Backspace => Some(Message::Backspace),
                        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(Message::Backspace)
                        }
                        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            Some(Message::DeleteAll)
                        }
                        KeyCode::Down => Some(Message::Next),
                        KeyCode::Up => Some(Message::Previous),
                        KeyCode::Enter => Some(Message::ExecuteTarget),
                        KeyCode::Char(char) => Some(Message::KeyInput(char.to_string())),
                        _ => None,
                    },
                    CurrentPain::History => match key.code {
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

// TODO: Add UT
fn update(model: &mut Model, message: Option<Message>) {
    match message {
        Some(Message::MoveToNextPain) => match model.current_pain {
            CurrentPain::Main => model.current_pain = CurrentPain::History,
            CurrentPain::History => model.current_pain = CurrentPain::Main,
        },
        Some(Message::Quit) => model.should_quit = true,
        Some(Message::KeyInput(key_input)) => {
            model.key_input = model.update_key_input(key_input);
            model.reset();
        }
        Some(Message::Backspace) => model.key_input = model.pop(),
        Some(Message::DeleteAll) => model.key_input = String::new(),
        Some(Message::Next) => model.next(),
        Some(Message::Previous) => model.previous(),
        Some(Message::ExecuteTarget) => {
            model.selected_target = model
                .targets_list_state
                .selected()
                .map(|i| model.narrow_down_targets()[i].clone());
        }
        None => {}
    }
}

fn shutdown_terminal(terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

