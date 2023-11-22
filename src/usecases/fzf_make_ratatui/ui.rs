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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.size());

    // TODO: current_painは色を変えるだけに変更する
    let (fzf_make_title, history_title) = {
        match model.current_pain {
            super::app::CurrentPain::Main => ("fzf-make-current", "history"),
            super::app::CurrentPain::History => ("fzf-make", "history-current"),
        }
    };

    let list = rounded_border_block(fzf_make_title);
    f.render_widget(list, chunks[0]);

    let title_block = rounded_border_block(history_title);

    f.render_widget(title_block, chunks[1]);
    let current_keys_hint = {
        Span::styled(
            "(q): to quit, <tab> move to next tab",
            Style::default().fg(Color::Red),
        )
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(rounded_border_block("hints"));
    f.render_widget(key_notes_footer, chunks[2]);
}

fn rounded_border_block(title: &str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default())
}
