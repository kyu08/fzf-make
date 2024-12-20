use super::app::{AppState, CurrentPane, Model, SelectCommandState};
use crate::model::command;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::fs::File;
use std::io::{BufRead, BufReader};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SColor, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect_tui::into_span;

pub fn ui(f: &mut Frame, model: &mut Model) {
    if let AppState::SelectCommand(model) = &mut model.app_state {
        let main_and_key_bindings = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)])
            .split(f.area());
        render_hint_block(model, f, main_and_key_bindings[1]);

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(main_and_key_bindings[0]);

        let input_and_notification = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main[1]);
        render_input_block(model, f, input_and_notification[0]);

        let notification_and_current_version = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100), Constraint::Length(9)])
            .split(input_and_notification[1]);
        render_notification_block(model, f, notification_and_current_version[0]);
        render_current_version_block(f, notification_and_current_version[1]);

        let preview_and_commands = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main[0]);
        render_preview_block(model, f, preview_and_commands[0]);

        let commands = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(preview_and_commands[1]);
        render_commands_block(model, f, commands[0]);
        render_history_block(model, f, commands[1]);
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

fn render_preview_block(model: &SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    // NOTE: chunk.rows().count() includes border lines
    let row_count = chunk.rows().count() - 2;
    let narrow_down_commands = model.narrow_down_commands();
    let selecting_command =
        narrow_down_commands.get(model.commands_list_state.selected().unwrap_or(0));

    let command = selecting_command.unwrap(); // TODO: remove unwrap

    let file = File::open(command.file_name.clone()).unwrap(); // TODO: remove unwrap
    let reader = BufReader::new(file);

    let command_row_index = command.line_number as usize - 1;
    let (start_index, end_index) = determine_rendering_position(row_count, command_row_index);
    let source_lines: Vec<_> = reader
        .lines()
        .skip(start_index)
        .take(end_index - start_index + 1)
        // HACK: workaround for https://github.com/ratatui/ratatui/issues/876
        .map(|line| line.unwrap().replace("\t", "    "))
        .collect();

    let lines = {
        if let Some(_command) = selecting_command {
            let ps = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            // NOTE: extension is `rs` intentionally because it highlights `Makefile` and `json` files in a good way.(No unnecessary background color)
            let syntax = ps.find_syntax_by_extension("rs").unwrap();
            let theme = &mut ts.themes["base16-ocean.dark"].clone();

            let mut lines = vec![];
            for (index, line) in source_lines.iter().enumerate() {
                theme.settings.background = Some(SColor {
                    r: 94,
                    g: 120,
                    b: 200,
                    // To get bg same as ratatui's background, make the line other than includes command transparent.
                    a: if (start_index + index) == command_row_index {
                        50
                    } else {
                        0
                    },
                });
                let mut h = HighlightLines::new(syntax, theme);
                let mut spans: Vec<Span> = h
                    .highlight_line(line, &ps)
                    .unwrap()
                    .into_iter()
                    .filter_map(|segment| into_span(segment).ok())
                    .collect();

                // add row number
                spans.insert(
                    0,
                    Span::styled(format!("{:5} ", start_index + index + 1), Style::default()),
                );

                lines.push(Line::from(spans));
            }
            lines
        } else {
            vec![]
        }
    };

    let (fg_color_, border_style) =
        color_and_border_style_for_selectable(model.current_pane.is_main());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_style)
        .border_style(Style::default().fg(fg_color_))
        .title(" ✨ Preview ")
        .title_style(TITLE_STYLE);
    let preview_widget = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(block);
    f.render_widget(preview_widget, chunk);
}

fn determine_rendering_position(row_count: usize, command_row_index: usize) -> (usize, usize) {
    let middle_row_index = if row_count % 2 == 0 {
        row_count / 2 - 1
    } else {
        (row_count + 1) / 2 - 1
    };

    if command_row_index < middle_row_index {
        (0, row_count - 1)
    } else {
        let start_index = command_row_index - middle_row_index;
        let end_index = start_index + row_count - 1;
        (start_index, end_index)
    }
}

fn render_commands_block(
    model: &mut SelectCommandState,
    f: &mut Frame,
    chunk: ratatui::layout::Rect,
) {
    f.render_stateful_widget(
        commands_block(
            " 📢 Commands ",
            model.narrow_down_commands(),
            model.current_pane.is_main(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.commands_list_state,
    );
}

fn render_input_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
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
        .set_placeholder_text("Type text to search command");

    f.render_widget(&model.search_text_area.0, chunk);
}

fn render_notification_block(
    model: &mut SelectCommandState,
    f: &mut Frame,
    chunk: ratatui::layout::Rect,
) {
    let text = match &model.latest_version {
        Some(has_update) => {
            if format!("v{}", env!("CARGO_PKG_VERSION")) != *has_update {
                format!(
                    "📦️ A new release is available! v{} → {}.",
                    env!("CARGO_PKG_VERSION"),
                    has_update.as_str()
                )
            } else {
                "".to_string()
            }
        }
        None => "".to_string(),
    };
    let notification = Span::styled(text, Style::default());

    let block = Block::default()
        .padding(ratatui::widgets::Padding::new(1, 0, 1, 1))
        .style(Style::new().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let key_notes_footer = Paragraph::new(notification)
        .wrap(Wrap { trim: true })
        .block(block);
    f.render_widget(key_notes_footer, chunk);
}

fn render_current_version_block(f: &mut Frame, chunk: ratatui::layout::Rect) {
    let text = format!("v{}", env!("CARGO_PKG_VERSION"));
    let notification = Span::styled(text, Style::default());

    let block = Block::default().padding(ratatui::widgets::Padding::new(0, 1, 2, 0));
    let key_notes_footer = Paragraph::new(notification)
        .block(block)
        .right_aligned()
        .wrap(Wrap { trim: true });

    f.render_widget(key_notes_footer, chunk);
}

fn render_history_block(
    model: &mut SelectCommandState,
    f: &mut Frame,
    chunk: ratatui::layout::Rect,
) {
    f.render_stateful_widget(
        commands_block(
            " 📚 History ",
            model.get_history(),
            model.current_pane.is_history(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.history_list_state,
    );
}

fn render_hint_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let hint_text = match model.current_pane {
        CurrentPane::Main => {
            "Execute the selected command: <enter> | Select command: ↑/↓ | Narrow down command: (type any character) | Move to next tab: <tab> | Quit: <esc>"
        }
        CurrentPane::History => {
            "Execute the selected command: <enter> | Select command: ↑/↓ | Move to next tab: <tab> | Quit: q/<esc>"
        }
    };
    let hint = Span::styled(hint_text, Style::default().fg(FG_COLOR_SELECTED));

    let block = Block::default().padding(ratatui::widgets::Padding::new(2, 2, 0, 0));
    let key_notes_footer = Paragraph::new(hint).wrap(Wrap { trim: true }).block(block);

    f.render_widget(key_notes_footer, chunk);
}

fn commands_block(
    title: &str,
    narrowed_down_commands: Vec<command::Command>,
    is_current: bool,
) -> List<'_> {
    let (fg_color, border_style) = color_and_border_style_for_selectable(is_current);

    let list: Vec<ListItem> = narrowed_down_commands
        .into_iter()
        .map(|command| ListItem::new(command.to_string()).style(Style::default()))
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_determine_rendering_position() {
        // start is greater than 0(row_count is odd number)
        let (start, end) = determine_rendering_position(5, 4);
        assert_eq!(start, 2);
        assert_eq!(end, 6);

        // start is greater than 0(row_count is even number)
        let (start, end) = determine_rendering_position(6, 4);
        assert_eq!(start, 2);
        assert_eq!(end, 7);

        // start is 0
        let (start, end) = determine_rendering_position(10, 1);
        assert_eq!(start, 0);
        assert_eq!(end, 9);
    }
}
