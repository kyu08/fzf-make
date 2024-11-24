use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::model::command;

use super::app::{AppState, CurrentPane, Model, SelectTargetState};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use tui_term::widget::PseudoTerminal;

pub fn ui(f: &mut Frame, model: &mut Model) {
    if let AppState::SelectTarget(model) = &mut model.app_state {
        let main_and_key_bindings = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)])
            .split(f.area());
        render_hint_block(model, f, main_and_key_bindings[1]);

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(main_and_key_bindings[0]);
        render_input_block(model, f, main[1]);

        let preview_and_targets = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main[0]);
        render_preview_block(model, f, preview_and_targets[0]);

        let targets = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(preview_and_targets[1]);
        render_targets_block(model, f, targets[0]);
        render_history_block(model, f, targets[1]);
    }
}

const FG_COLOR_SELECTED: ratatui::style::Color = Color::Rgb(161, 220, 156);
const FG_COLOR_NOT_SELECTED: ratatui::style::Color = Color::DarkGray;
const BORDER_STYLE_SELECTED: ratatui::widgets::block::BorderType =
    ratatui::widgets::BorderType::Thick;
const BORDER_STYLE_NOT_SELECTED: ratatui::widgets::block::BorderType =
    ratatui::widgets::BorderType::Plain;
const TITLE_STYLE: ratatui::style::Style = Style::new().add_modifier(Modifier::BOLD);

fn color_and_border_style_for_selectable(
    is_selected: bool,
) -> (Color, ratatui::widgets::block::BorderType) {
    if is_selected {
        (FG_COLOR_SELECTED, BORDER_STYLE_SELECTED)
    } else {
        (FG_COLOR_NOT_SELECTED, BORDER_STYLE_NOT_SELECTED)
    }
}

// Because the setup process of the terminal and render_widget function need to be done in the same scope, the call of the render_widget function is included.
fn render_preview_block(model: &SelectTargetState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let narrow_down_targets = model.narrow_down_targets();

    let selecting_command =
        narrow_down_targets.get(model.commands_list_state.selected().unwrap_or(0));

    let (fg_color_, border_style) =
        color_and_border_style_for_selectable(model.current_pane.is_main());

    let title = Line::from(" ✨ Preview ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_style)
        .border_style(Style::default().fg(fg_color_))
        .title(title)
        .title_style(TITLE_STYLE);

    if !model.get_search_area_text().is_empty() && narrow_down_targets.is_empty() {
        f.render_widget(block, chunk);
        return;
    }

    let pty_system = NativePtySystem::default();
    let cmd = match selecting_command {
        Some(command) => preview_command(command.file_name.clone(), command.line_number),
        None => {
            return;
        }
    };
    let pair = pty_system
        .openpty(PtySize {
            rows: 1000,
            cols: 1000,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut child = pair.slave.spawn_command(cmd).unwrap();

    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let parser = Arc::new(RwLock::new(vt100::Parser::new(1000, 1000, 0)));

    {
        let parser = parser.clone();
        std::thread::spawn(move || {
            // Consume the output from the child
            let mut s = String::new();
            reader.read_to_string(&mut s).unwrap();
            if !s.is_empty() {
                let mut parser = parser.write().unwrap();
                parser.process(s.as_bytes());
            }
        });
    }

    {
        // Drop writer on purpose
        let _writer = pair.master.take_writer().unwrap();
    }

    // Wait for the child to complete
    let _child_exit_status = child.wait().unwrap();

    drop(pair.master);

    let binding = parser.read().unwrap();
    let screen = binding.screen();

    f.render_widget(
        PseudoTerminal::new(screen)
            .cursor(tui_term::widget::Cursor::default().symbol(""))
            .block(block),
        chunk,
    );
}

fn preview_command(file_path: PathBuf, line_number: u32) -> CommandBuilder {
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new("bat");
    cmd.cwd(cwd);
    cmd.args([
        file_path.to_string_lossy().to_string().as_str(),
        "-p",
        "--style=numbers",
        "--color=always",
        "--line-range",
        (line_number.to_string() + ":").as_str(),
        "--highlight-line",
        line_number.to_string().as_str(),
    ]);
    cmd
}

fn render_targets_block(
    model: &mut SelectTargetState,
    f: &mut Frame,
    chunk: ratatui::layout::Rect,
) {
    f.render_stateful_widget(
        targets_block(
            " 📢 Targets ",
            model.narrow_down_targets(),
            model.current_pane.is_main(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.commands_list_state,
    );
}

fn render_input_block(model: &mut SelectTargetState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let (fg_color, border_style) =
        color_and_border_style_for_selectable(model.current_pane.is_main());

    let block = Block::default()
        .title(" 🔍 Search ")
        .title_style(TITLE_STYLE)
        .borders(Borders::ALL)
        .border_type(border_style)
        .border_style(Style::default().fg(fg_color))
        .style(Style::default())
        .padding(ratatui::widgets::Padding::new(2, 2, 0, 0));

    model.search_text_area.0.set_block(block);
    model
        .search_text_area
        .0
        .set_placeholder_text("Type text to search target");

    f.render_widget(&model.search_text_area.0, chunk);
}

fn render_history_block(
    model: &mut SelectTargetState,
    f: &mut Frame,
    chunk: ratatui::layout::Rect,
) {
    f.render_stateful_widget(
        targets_block(
            " 📚 History ",
            model.get_history(),
            model.current_pane.is_history(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.history_list_state,
    );
}

fn render_hint_block(model: &mut SelectTargetState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let hint_text = match model.current_pane {
        CurrentPane::Main => {
            "Execute the selected target: <enter> | Select target: ↑/↓ | Narrow down target: (type any character) | Move to next tab: <tab> | Quit: <esc>"
        }
        CurrentPane::History => {
            "Execute the selected target: <enter> | Select target: ↑/↓ | Move to next tab: <tab> | Quit: q/<esc>"
        }
    };
    let hint = Span::styled(hint_text, Style::default().fg(FG_COLOR_SELECTED));

    let block = Block::default().padding(ratatui::widgets::Padding::new(2, 2, 0, 0));
    let key_notes_footer = Paragraph::new(hint).wrap(Wrap { trim: true }).block(block);

    f.render_widget(key_notes_footer, chunk);
}

fn targets_block(
    title: &str,
    narrowed_down_targets: Vec<command::Command>,
    is_current: bool,
) -> List<'_> {
    let (fg_color, border_style) = color_and_border_style_for_selectable(is_current);

    let list: Vec<ListItem> = narrowed_down_targets
        .into_iter()
        .map(|target| ListItem::new(target.to_string()).style(Style::default()))
        .collect();

    List::new(list)
        .style(Style::default())
        .block(
            Block::default()
                .title(title)
                .title_style(TITLE_STYLE)
                .borders(Borders::ALL)
                .border_type(border_style)
                .border_style(Style::default().fg(fg_color))
                .style(Style::default())
                .padding(ratatui::widgets::Padding::new(2, 0, 0, 0)),
        )
        .highlight_style(Style::default().fg(FG_COLOR_SELECTED))
        .highlight_symbol("> ")
}
