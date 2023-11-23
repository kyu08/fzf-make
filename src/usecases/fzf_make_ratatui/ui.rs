use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::Model;

pub fn ui<B: Backend>(f: &mut Frame<B>, model: &Model) {
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

    let list = rounded_border_block("Preview", model.current_pain.is_main());
    f.render_widget(list, fzf_make_preview_chunks[0]);
    let list = rounded_border_block("Targets", model.current_pain.is_main());
    f.render_widget(list, fzf_make_preview_chunks[1]);
    let list = rounded_border_block("Input", model.current_pain.is_main());
    f.render_widget(list, fzf_make_chunks[1]);

    let title_block = rounded_border_block("History", model.current_pain.is_history());
    f.render_widget(title_block, fzf_preview_and_history_chunks[1]);

    let hint_text = match model.current_pain {
        super::app::CurrentPain::Main => "<esc>: to quit, <tab> move to next tab",
        super::app::CurrentPain::History => "q / <esc>: to quit, <tab> move to next tab",
    };
    let current_keys_hint = { Span::styled(hint_text, Style::default()) };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(rounded_border_block("hints", false));
    f.render_widget(key_notes_footer, main_chunks[1]);
}

fn rounded_border_block(title: &str, is_current: bool) -> Block {
    if is_current {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default())
    } else {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default())
    }
}
