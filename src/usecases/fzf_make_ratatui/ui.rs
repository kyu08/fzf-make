use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::makefile::Makefile;

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

    f.render_widget(
        // TODO: ハイライトしてtargetの内容を表示する
        rounded_border_block("Preview", model.current_pain.is_main()),
        fzf_make_preview_chunks[0],
    );
    f.render_widget(
        targets_block(
            "Targets",
            model.key_input.clone(),
            model.makefile.clone(),
            model.current_pain.is_main(),
        ),
        fzf_make_preview_chunks[1],
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

fn input_block<'a>(title: &'a str, target_input: &'a str, is_current: bool) -> Paragraph<'a> {
    let fg_color = if is_current {
        Color::Yellow
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
fn targets_block(title: &str, key_input: String, makefile: Makefile, is_current: bool) -> List<'_> {
    // TODO: 選択する
    let fg_color = if is_current {
        Color::Yellow
    } else {
        Color::default()
    };

    let matcher = SkimMatcherV2::default();
    let mut filtered_list: Vec<(Option<i64>, String)> = makefile
        .to_targets_string()
        .into_iter()
        .map(|target| match matcher.fuzzy_indices(&target, &key_input) {
            Some((score, _)) => (Some(score), target), // TODO: highligh matched part
            None => (None, target),
        })
        .filter(|(score, _)| score.is_some())
        .collect();

    filtered_list.sort_by(|(score1, _), (score2, _)| score1.cmp(score2));
    filtered_list.reverse();

    // Sort filtered_list by first element of tuple
    let list: Vec<ListItem> = filtered_list
        .into_iter()
        .map(|(_, target)| ListItem::new(target).style(Style::default().fg(Color::Yellow)))
        .collect();

    List::new(list)
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

fn rounded_border_block(title: &str, is_current: bool) -> Block {
    let fg_color = if is_current {
        Color::Yellow
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
