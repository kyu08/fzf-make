use super::app::Model;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::sync::{Arc, RwLock};
use tui_term::widget::PseudoTerminal;

pub fn ui(f: &mut Frame, model: &mut Model) {
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
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(fzf_make_chunks[0]);

    // MEMO: ここから
    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();

    let selecting_target = &model.narrow_down_targets()[model.state.selected().unwrap_or(0)];
    let (file_name, line_number) = model
        .makefile
        .target_to_file_and_line_number(selecting_target);

    // FIXME: 関数に切り出したら固定値を返しているのを修正する
    let file_name = match file_name {
        Some(file_name) => file_name,
        None => "Makefile".to_string(),
    };
    let line_number = match line_number {
        Some(line_number) => line_number,
        None => 1,
    };

    let mut cmd = CommandBuilder::new("bat");
    cmd.cwd(cwd);
    cmd.args([
        file_name.as_str(),
        "-p",
        "--style=numbers",
        "--color=always",
        "--line-range",
        (line_number.to_string() + ":").as_str(),
        "--highlight-line",
        line_number.to_string().as_str(),
    ]);

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

    let fg_color = if model.current_pain.is_main() {
        fg_color()
    } else {
        Color::default()
    };

    let binding = parser.read().unwrap();
    let screen = binding.screen();
    let title = Line::from("Preview");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(fg_color))
        .title(title)
        .style(Style::default());
    // MEMO: ここまで

    f.render_widget(
        PseudoTerminal::new(screen).block(block),
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
