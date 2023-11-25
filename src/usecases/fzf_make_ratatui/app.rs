use crate::models::makefile::Makefile;

use super::ui::ui;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::ListState,
    Terminal,
};
use std::{error::Error, io, process};

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
    Next,
    Previous,
}

#[derive(Clone)]
pub struct Model {
    pub current_pain: CurrentPain,
    pub key_input: String,
    pub makefile: Makefile,
    // the current screen the user is looking at, and will later determine what is rendered.
    pub should_quit: bool,
    pub state: ListState,
}

impl Model {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let makefile = match Makefile::create_makefile() {
            Err(e) => {
                println!("[ERR] {}", e);
                process::exit(1)
            }
            Ok(f) => f,
        };

        Ok(Model {
            key_input: String::new(),
            current_pain: CurrentPain::Main,
            should_quit: false,
            makefile: makefile.clone(),
            state: ListState::with_selected(ListState::default(), Some(0)),
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

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.makefile.to_targets_string().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.makefile.to_targets_string().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn reset(&mut self) {
        if self.makefile.to_targets_string().is_empty() {
            self.state.select(None);
        }
        self.state.select(Some(0));
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
                                   // TODO: あとで検討する
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    if let Ok(model) = Model::new() {
        let _ = run(&mut terminal, model); // TODO: error handling
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut model: Model) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut model.clone()))?;
        match handle_event(&model) {
            Ok(message) => {
                update(&mut model, message);
                if model.should_quit {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Ok(())
}

fn handle_event(model: &Model) -> io::Result<Option<Message>> {
    let message = if crossterm::event::poll(std::time::Duration::from_millis(250))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            match key.code {
                // Commons key bindings
                KeyCode::Tab => Some(Message::MoveToNextPain),
                KeyCode::Esc => Some(Message::Quit),
                _ => match model.current_pain {
                    CurrentPain::Main => match key.code {
                        KeyCode::Backspace => Some(Message::Backspace),
                        KeyCode::Down => Some(Message::Next),
                        KeyCode::Up => Some(Message::Previous),
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
        Some(Message::Next) => model.next(),
        Some(Message::Previous) => model.previous(),
        None => {}
    }
}
