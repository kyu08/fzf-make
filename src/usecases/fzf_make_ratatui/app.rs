use super::ui::ui;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};

pub enum CurrentPain {
    Main,
    History,
}

enum Message {
    MoveToNextPain,
    Quit,
}

pub struct Model {
    pub key_input: String,
    pub current_pain: CurrentPain, // the current screen the user is looking at, and will later determine what is rendered.
    pub should_quit: bool,
}

impl Model {
    pub fn new() -> Model {
        Model {
            key_input: String::new(),
            current_pain: CurrentPain::Main,
            should_quit: false,
        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
                                   // TODO: あとで検討する
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut model = Model::new();
    let _ = run(&mut terminal, &mut model); // TODO: error handling

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, model: &mut Model) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, model))?;
        match handle_event(model) {
            Ok(message) => {
                let model = update(model, message);
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
                        KeyCode::Char('e') => Some(Message::MoveToNextPain),
                        KeyCode::Tab => Some(Message::MoveToNextPain),
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

fn update(model: &mut Model, message: Option<Message>) -> &mut Model {
    match message {
        Some(Message::MoveToNextPain) => match model.current_pain {
            CurrentPain::Main => model.current_pain = CurrentPain::History,
            CurrentPain::History => model.current_pain = CurrentPain::Main,
        },
        Some(Message::Quit) => {
            model.should_quit = true;
        }
        None => {}
    }
    model
}
