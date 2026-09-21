use super::app::{AppState, CurrentPane, Model, SelectCommandState};
use crate::model::command;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

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

        let commands = if f.area().height < HEIGHT_THRESHOLD_TO_HIDE_PREVIEW_WINDOW {
            let preview_and_commands = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(main[0]);
            // When the window height is too small to show the preview window.

            preview_and_commands[0]
        } else {
            let preview_and_commands = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(main[0]);

            // Render the preview window only when the window height is enough.
            render_preview_block(model, f, preview_and_commands[0]);

            preview_and_commands[1]
        };

        let commands_and_history = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(commands);

        render_commands_block(model, f, commands_and_history[0]);
        render_history_block(model, f, commands_and_history[1]);
        // Render additional arguments popup if needed.
        render_additional_arguments_popup(model, f);
    }
}

const HEIGHT_THRESHOLD_TO_HIDE_PREVIEW_WINDOW: u16 = 20;
const FG_COLOR_SELECTED: ratatui::style::Color = Color::Rgb(161, 220, 156);
const FG_COLOR_NOT_SELECTED: ratatui::style::Color = Color::DarkGray;
const BORDER_STYLE_SELECTED: ratatui::widgets::block::BorderType = ratatui::widgets::BorderType::Thick;
const BORDER_STYLE_NOT_SELECTED: ratatui::widgets::block::BorderType = ratatui::widgets::BorderType::Plain;
const TITLE_STYLE: ratatui::style::Style = Style::new().add_modifier(Modifier::BOLD);
/// Background of the preview row that defines the selected command.
const COMMAND_ROW_BG_COLOR: ratatui::style::Color = Color::Rgb(94, 120, 200);

fn color_and_border_style_for_selectable(
    is_selected: bool,
    is_additional_arguments_popup_opened: bool,
) -> (Color, ratatui::widgets::block::BorderType) {
    if is_selected && !is_additional_arguments_popup_opened {
        (FG_COLOR_SELECTED, BORDER_STYLE_SELECTED)
    } else {
        (FG_COLOR_NOT_SELECTED, BORDER_STYLE_NOT_SELECTED)
    }
}

fn render_preview_block(model: &SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let narrow_down_commands = model.narrow_down_commands();
    let selecting_command = narrow_down_commands.get(model.commands_list_state.selected().unwrap_or(0));

    let lines = match selecting_command {
        Some(command) => {
            let command_row_index = command.line_number as usize - 1;
            let row_count = chunk.rows().count() - 2; // NOTE: chunk.rows().count() includes border lines
            let (start_index, end_index) = determine_rendering_position(row_count, command_row_index);

            model
                .preview_cache
                .styled_lines(&command.file_path, start_index, end_index)
                .into_iter()
                .enumerate()
                .map(|(index, styled_line)| {
                    let row_index = start_index + index;
                    // add row number
                    let mut spans = vec![Span::styled(format!("{:5} ", row_index + 1), Style::default())];
                    spans.extend(styled_line.into_iter().map(|(style, content)| {
                        // Every line is highlighted with a transparent background so that it keeps
                        // ratatui's background. Only the row that defines the command is filled.
                        let style = if row_index == command_row_index {
                            style.bg(COMMAND_ROW_BG_COLOR)
                        } else {
                            style
                        };
                        Span::styled(content, style)
                    }));

                    Line::from(spans)
                })
                .collect()
        }
        None => vec![],
    };

    let (fg_color_, border_style) = color_and_border_style_for_selectable(
        model.current_pane.is_main(),
        model.is_additional_arguments_popup_opened(),
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_style)
        .border_style(Style::default().fg(fg_color_))
        .title(" ✨ Preview ")
        .title_style(TITLE_STYLE);
    let preview_widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(block);
    f.render_widget(preview_widget, chunk);
}

fn determine_rendering_position(row_count: usize, command_row_index: usize) -> (usize, usize) {
    let middle_row_index = if row_count.is_multiple_of(2) {
        row_count / 2 - 1
    } else {
        row_count.div_ceil(2) - 1
    };

    if command_row_index < middle_row_index {
        (0, row_count - 1)
    } else {
        let start_index = command_row_index - middle_row_index;
        let end_index = start_index + row_count - 1;
        (start_index, end_index)
    }
}

fn render_commands_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    f.render_stateful_widget(
        commands_block(
            " 📢 Commands ",
            model.narrow_down_commands().into_iter().map(|c| c.into()).collect(),
            model.current_pane.is_main(),
            model.is_additional_arguments_popup_opened(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.commands_list_state,
    );
}

fn render_input_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let (fg_color, border_style) = color_and_border_style_for_selectable(
        model.current_pane.is_main(),
        model.is_additional_arguments_popup_opened(),
    );

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

fn render_notification_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let text = {
        if let Some(s) = &model.copy_command_state {
            match s {
                Ok(c) => format!("📋 Command copied to clipboard: {}", c),
                Err(e) => format!("⚠️ Failed to copy command to clipboard: {}", e),
            }
        } else {
            match &model.latest_version {
                Some(has_update) => {
                    if format!("v{}", env!("CARGO_PKG_VERSION")) != *has_update {
                        format!(
                            "📦️ A new release is available! v{} → {}.",
                            env!("CARGO_PKG_VERSION"),
                            has_update.as_str()
                        )
                    } else {
                        String::new()
                    }
                }
                None => String::new(),
            }
        }
    };

    let notification = Span::styled(text, Style::default());
    let block = Block::default()
        .padding(ratatui::widgets::Padding::new(1, 0, 1, 1))
        .style(Style::new().add_modifier(Modifier::BOLD).fg(Color::Yellow));
    let key_notes_footer = Paragraph::new(notification).wrap(Wrap { trim: true }).block(block);
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

fn render_history_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    f.render_stateful_widget(
        commands_block(
            " 📚 History ",
            model.get_history().into_iter().map(|c| c.into()).collect(),
            model.current_pane.is_history(),
            model.is_additional_arguments_popup_opened(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.history_list_state,
    );
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, x: u16, y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn render_additional_arguments_popup(model: &mut SelectCommandState, f: &mut Frame) {
    if model.additional_arguments_popup_state.is_none() {
        return;
    }
    // If this popup is going to be opened, model.get_selected_command() returns Some(command).
    // So we can call unwrap() safely.
    let command = model.get_selected_command().unwrap();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BORDER_STYLE_SELECTED)
        .border_style(Style::default().fg(FG_COLOR_SELECTED))
        .title(format!(" 👋 Pass additional arguments to `{}`", command));

    let area = popup_area(f.area(), 60, 3);
    // This clears out the background which is needed to allow
    // overdrawing
    f.render_widget(Clear, area);
    let mut additional_arguments_popup_state = model.additional_arguments_popup_state.clone().unwrap();
    additional_arguments_popup_state.arguments_text_area.0.set_block(block);
    f.render_widget(&additional_arguments_popup_state.arguments_text_area.0, area);
}

fn render_hint_block(model: &mut SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let hint_text = if model.is_additional_arguments_popup_opened() {
        "Execute the selected command: <enter> | Passing additional arguments: (type any character) | Close the popup window: <esc>"
    } else {
        match model.current_pane {
            CurrentPane::Main => {
                "Execute the selected command: <enter> | Select command: ↑(<c-p>)/↓(<c-n>) | Narrow down command: (type any character) | Quit: <c-c>/<esc> | Move to next tab: <tab> | Copy command to clipboard: <c-y> | Pass additional arguments: <c-o>"
            }
            CurrentPane::History => {
                "Execute the selected command: <enter> | Select command: ↑(<c-p>)/↓(<c-n>) | Quit: <c-c>/q/<esc> | Move to next tab: <tab> | Copy command to clipboard: <c-y> | Pass additional arguments: <c-o>"
            }
        }
    };
    let hint = Span::styled(hint_text, Style::default().fg(FG_COLOR_SELECTED));

    let block = Block::default().padding(ratatui::widgets::Padding::new(2, 2, 0, 0));
    let key_notes_footer = Paragraph::new(hint).wrap(Wrap { trim: true }).block(block);

    f.render_widget(key_notes_footer, chunk);
}

fn commands_block(
    title: &str,
    narrowed_down_commands: Vec<command::CommandForExec>,
    is_current: bool,
    is_additional_arguments_popup_opened: bool,
) -> List<'_> {
    let (fg_color, border_style) =
        color_and_border_style_for_selectable(is_current, is_additional_arguments_popup_opened);

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
