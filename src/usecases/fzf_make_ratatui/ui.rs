use super::app::Model;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn ui<B: Backend>(f: &mut Frame<B>, model: &mut Model) {
    // Create the layout sections.
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(f.size());
    let fzf_preview_and_history_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(3), Constraint::Max(40)])
        .split(main_chunks[0]);
    let fzf_make_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(fzf_preview_and_history_chunks[0]);
    let fzf_make_preview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(fzf_make_chunks[0]);

    f.render_widget(
        rounded_border_block("Preview", model.current_pain.is_main()),
        fzf_make_preview_chunks[0],
    );
    f.render_stateful_widget(
        targets_block(
            "Targets",
            model.narrow_down_targets(),
            model.current_pain.is_main(),
        ),
        fzf_make_preview_chunks[1],
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.state,
    );
    f.render_widget(
        // NOTE: To show cursor, use rhysd/tui-textarea
        input_block("Input", &model.key_input, model.current_pain.is_main()),
        fzf_make_chunks[1],
    );
    f.render_widget(
        rounded_border_block("History", model.current_pain.is_history()),
        fzf_preview_and_history_chunks[1],
    );

    let hint_text = match model.current_pain {
        super::app::CurrentPain::Main => {
            "(Any key except the following): Narrow down targets, <esc>: Quit, <tab> Move to next tab"
        }
        super::app::CurrentPain::History => "q / <esc>: Quit, <tab> Move to next tab",
    };
    let current_keys_hint = { Span::styled(hint_text, Style::default()) };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(rounded_border_block("hints", false));
    f.render_widget(key_notes_footer, main_chunks[1]);
}

fn fg_color() -> Color {
    Color::LightBlue
}

fn input_block<'a>(title: &'a str, target_input: &'a str, is_current: bool) -> Paragraph<'a> {
    let fg_color = if is_current {
        fg_color()
    } else {
        Color::default()
    };

    Paragraph::new(Line::from(target_input))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(fg_color))
                .style(Style::default())
                .padding(ratatui::widgets::Padding::new(2, 0, 0, 0)),
        )
        .style(Style::default())
}
fn targets_block(title: &str, narrowed_down_targets: Vec<String>, is_current: bool) -> List<'_> {
    let fg_color = if is_current {
        fg_color()
    } else {
        Color::default()
    };

    let list: Vec<ListItem> = narrowed_down_targets
        .into_iter()
        .map(|target| ListItem::new(target).style(Style::default()))
        .collect();

    List::new(list)
        .style(Style::default())
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(fg_color))
                .style(Style::default())
                .padding(ratatui::widgets::Padding::new(2, 0, 0, 0)),
        )
        .highlight_style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .highlight_symbol("> ")
}

fn rounded_border_block(title: &str, is_current: bool) -> Block {
    let fg_color = if is_current {
        fg_color()
    } else {
        Color::default()
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(fg_color))
        .style(Style::default())
}
