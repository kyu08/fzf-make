//! Scenario based snapshot tests for the TUI.
//!
//! Each test walks through one scenario of `docs/MANUAL_TEST_CASES.md`: it builds the model for a
//! fixture directory under `test_data`, feeds key events through the same
//! `handle_key_input` -> `update` path the real event loop uses, and snapshots the rendered screen
//! after every step. The snapshots are what a user would see, so a layout or wording regression
//! fails the suite instead of waiting to be noticed by hand.
//!
//! Only the characters are captured, not the colors. The two pieces of state that colors carry are
//! also visible in the characters: the focused pane is drawn with a thick border (`┏━┓`) and the
//! unfocused one with a plain border (`┌─┐`), and the selected row is prefixed with `> `.
//!
//! Run `cargo insta review` after an intentional UI change to accept the new snapshots.

use super::{
    app::{AppState, Model, update},
    config,
    ui::ui,
};
use crate::model::{histories, runner, runner_type};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::TestBackend};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

/// A terminal large enough to show the preview window.
const WIDTH: u16 = 120;
const HEIGHT: u16 = 30;

/// The preview is highlighted on a background thread, so the screen is only stable once the
/// highlighting is done. Waiting keeps the snapshots from depending on how fast the machine is.
const HIGHLIGHTING_TIMEOUT: Duration = Duration::from_secs(10);

struct Scenario<'a> {
    model: Model<'a>,
    terminal: Terminal<TestBackend>,
}

impl Scenario<'_> {
    fn new(fixture: &str) -> Self {
        Self::with_size(fixture, config::Config::default(), WIDTH, HEIGHT)
    }

    fn with_size(fixture: &str, config: config::Config, width: u16, height: u16) -> Self {
        let mut model = Model::new_in(fixture_dir(fixture), config).unwrap();
        let AppState::SelectingCommand(s) = &mut model.app_state else {
            panic!("the model is not selecting a command");
        };

        // The runners that are discovered depend on the machine: `Just` looks for a justfile in
        // every ancestor directory, and `Task` asks the `task` binary for its task list. These
        // scenarios are about the make runner, so the rest is dropped to keep the screen the same
        // everywhere.
        s.runners.retain(|r| matches!(r, runner::Runner::MakeCommand(_)));
        // The history is read from the user's history file, which is not part of the fixture.
        // Scenarios that need one set it through `with_history`.
        s.history = vec![];

        Scenario {
            model,
            terminal: Terminal::new(TestBackend::new(width, height)).unwrap(),
        }
    }

    /// The history file is written by the user, not by the fixture, so scenarios that need a
    /// history put it here instead of depending on whatever the machine happens to have stored.
    fn with_history(mut self, commands: Vec<(runner_type::RunnerType, &str)>) -> Self {
        let AppState::SelectingCommand(s) = &mut self.model.app_state else {
            panic!("the model is not selecting a command");
        };
        s.history = commands
            .into_iter()
            .map(|(runner_type, args)| histories::HistoryCommand {
                runner_type,
                args: args.to_string(),
            })
            .collect();
        self
    }

    fn key(&mut self, code: KeyCode) {
        self.send(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn ctrl(&mut self, c: char) {
        self.send(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
    }

    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.key(KeyCode::Char(c));
        }
    }

    fn send(&mut self, mut key: KeyEvent) {
        key.kind = KeyEventKind::Press;
        let message = self.model.handle_key_input(key);
        update(&mut self.model, message);
    }

    /// screen renders the model the way the event loop does and returns the visible characters.
    fn screen(&mut self) -> String {
        if let AppState::SelectingCommand(s) = &self.model.app_state {
            s.load_preview();
            let deadline = Instant::now() + HIGHLIGHTING_TIMEOUT;
            while s.preview_cache.is_highlighting() {
                assert!(Instant::now() < deadline, "the preview is still being highlighted");
                thread::sleep(Duration::from_millis(5));
            }
        }

        let model = &mut self.model;
        self.terminal.draw(|f| ui(f, model)).unwrap();
        self.terminal.backend().to_string()
    }

    fn selected_command(&self) -> Option<String> {
        self.model.command_to_execute().map(|(_, command)| command.to_string())
    }
}

fn fixture_dir(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data")
        .join(fixture)
}

/// assert_screen snapshots the screen. The version is replaced because it changes on every release
/// and would otherwise invalidate every snapshot.
macro_rules! assert_screen {
    ($name:expr, $scenario:expr) => {
        let screen = $scenario.screen();
        insta::with_settings!({filters => vec![(r"\s*v\d+\.\d+\.\d+", " vX.Y.Z")]}, {
            insta::assert_snapshot!($name, screen);
        });
    };
}

#[test]
fn make_initial_screen() {
    let mut s = Scenario::new("make");

    // Every target of the Makefile and of the file it includes is listed, the first one is
    // selected, the main pane has the focus and the search box shows its placeholder.
    assert_screen!("make_initial_screen", s);
}

#[test]
fn make_narrowing_down_matches() {
    let mut s = Scenario::new("make");

    s.type_text("with");
    assert_screen!("make_narrowing_down_matches", s);
}

#[test]
fn make_narrowing_down_matches_nothing() {
    let mut s = Scenario::new("make");

    s.type_text("nosuchcommand");
    assert_screen!("make_narrowing_down_matches_nothing", s);
}

#[test]
fn make_selection_resets_when_a_character_is_typed() {
    let mut s = Scenario::new("make");

    s.ctrl('n');
    s.ctrl('n');
    assert_screen!("make_selection_moved_down_twice", s);

    s.type_text("r");
    assert_screen!("make_selection_reset_by_typing", s);
}

#[test]
fn make_selection_resets_when_backspace_is_pressed() {
    let mut s = Scenario::new("make");

    s.type_text("run");
    s.ctrl('n');
    assert_screen!("make_selection_moved_down_after_narrowing_down", s);

    s.key(KeyCode::Backspace);
    assert_screen!("make_selection_reset_by_backspace", s);
}

#[test]
fn make_preview_follows_the_selected_command() {
    let mut s = Scenario::new("make");

    // `target-included` is defined in another file, so the preview has to switch files, not only
    // scroll.
    s.type_text("included");
    assert_screen!("make_preview_of_an_included_target", s);
}

#[test]
fn make_preview_is_hidden_when_the_terminal_is_short() {
    let mut s = Scenario::with_size("make", config::Config::default(), WIDTH, 19);

    assert_screen!("make_short_terminal", s);
}

#[test]
fn make_moving_to_the_history_pane() {
    let mut s = Scenario::new("make").with_history(vec![
        (runner_type::RunnerType::Make, "run"),
        (runner_type::RunnerType::Make, "run-with-arg ARG1=1"),
    ]);

    assert_screen!("make_history_listed_while_the_main_pane_has_focus", s);

    s.key(KeyCode::Tab);
    assert_screen!("make_history_pane_focused", s);
}

#[test]
fn make_launching_with_the_history_pane_focused() {
    let mut s = Scenario::with_size("make", config::Config::new(true), WIDTH, HEIGHT)
        .with_history(vec![(runner_type::RunnerType::Make, "run")]);

    assert_screen!("make_launched_with_history_focused", s);
}

#[test]
fn make_executing_the_selected_command() {
    let mut s = Scenario::new("make");

    s.type_text("with-args");
    s.key(KeyCode::Enter);

    assert_eq!(Some("make run-with-args".to_string()), s.selected_command());
}

#[test]
fn make_executing_a_command_from_the_history_pane() {
    let mut s = Scenario::new("make").with_history(vec![
        (runner_type::RunnerType::Make, "run"),
        (runner_type::RunnerType::Make, "run-with-arg ARG1=1"),
    ]);

    s.key(KeyCode::Tab);
    s.ctrl('n');
    s.key(KeyCode::Enter);

    assert_eq!(Some("make run-with-arg ARG1=1".to_string()), s.selected_command());
}

#[test]
fn make_passing_no_additional_argument() {
    let mut s = Scenario::new("make");

    s.ctrl('o');
    s.key(KeyCode::Enter);

    // The popup is opened and closed without typing anything, so the command is unchanged.
    assert_eq!(Some("make run".to_string()), s.selected_command());
}

#[test]
fn make_passing_one_additional_argument() {
    let mut s = Scenario::new("make");

    s.type_text("with-arg");
    s.ctrl('o');
    s.type_text("ARG1=1");
    s.key(KeyCode::Enter);

    // `run-with-arg` is a prefix of `run-with-args`, so no search text matches only the former.
    // Both get the same score, so the one defined first in the Makefile is selected.
    assert_eq!(Some("make run-with-args ARG1=1".to_string()), s.selected_command());
}

#[test]
fn make_passing_multiple_additional_arguments() {
    let mut s = Scenario::new("make");

    s.type_text("with-args");
    s.ctrl('o');
    assert_screen!("make_additional_arguments_popup_opened", s);

    s.type_text("ARG1=1 ARG2=2");
    assert_screen!("make_additional_arguments_typed", s);

    s.key(KeyCode::Enter);
    assert_eq!(Some("make run-with-args ARG1=1 ARG2=2".to_string()), s.selected_command());
}

#[test]
fn make_closing_the_additional_arguments_popup() {
    let mut s = Scenario::new("make");

    s.ctrl('o');
    s.type_text("ARG1=1");
    s.key(KeyCode::Esc);

    // The popup is gone and the command is not executed.
    assert_screen!("make_additional_arguments_popup_closed", s);
    assert_eq!(None, s.selected_command());
}

#[test]
fn make_quitting_with_esc() {
    let mut s = Scenario::new("make");

    s.key(KeyCode::Esc);

    assert!(matches!(s.model.app_state, AppState::Quitting(Ok(()))));
}

#[test]
fn make_pressing_enter_without_any_matching_command() {
    let mut s = Scenario::new("make");

    s.type_text("nosuchcommand");
    s.key(KeyCode::Enter);

    // Nothing matches, so fzf-make quits with an error instead of executing something.
    assert_eq!(None, s.selected_command());
    match &s.model.app_state {
        AppState::Quitting(Err(e)) => assert_eq!("No command selected", e.to_string()),
        other => panic!("unexpected state: {:?}", other),
    }
}
