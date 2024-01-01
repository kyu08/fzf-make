use super::app::Model;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::sync::{Arc, RwLock};
use tui_term::widget::PseudoTerminal;

pub fn ui(f: &mut Frame, model: &mut Model) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(5)])
        .split(f.size());
    render_key_bindings_block(model, f, main_chunks[1]);

    let fzf_preview_and_history_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(3), Constraint::Max(40)])
        .split(main_chunks[0]);
    render_history_block(model, f, fzf_preview_and_history_chunks[1]);

    let fzf_make_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(fzf_preview_and_history_chunks[0]);
    render_input_block(model, f, fzf_make_chunks[1]);

    let fzf_make_preview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(fzf_make_chunks[0]);
    render_preview_block(model, f, fzf_make_preview_chunks[0]);
    render_targets_block(model, f, fzf_make_preview_chunks[1]);
}

fn fg_color_selected() -> Color {
    Color::LightBlue
}
fn fg_color_not_selected() -> Color {
    Color::DarkGray
}

fn rounded_border_block(title: &str, is_current: bool) -> Block {
    let fg_color = if is_current {
        fg_color_selected()
    } else {
        fg_color_not_selected()
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(fg_color))
        .style(Style::default())
}

// Because the setup process of the terminal and render_widget function need to be done in the same scope, the call of the render_widget function is included.
fn render_preview_block(model: &Model, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let pty_system = NativePtySystem::default();

    let narrow_down_targets = model.narrow_down_targets();
    let selecting_target =
        &narrow_down_targets.get(model.targets_list_state.selected().unwrap_or(0));
    let (file_name, line_number) = model
        .makefile
        .target_to_file_and_line_number(selecting_target);

    let file_name = file_name.unwrap_or(model.makefile.path.to_string_lossy().to_string());
    let line_number = line_number.unwrap_or(1);
    let cmd = preview_command(file_name, line_number);

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

    let fg_color_ = if model.current_pane.is_main() {
        fg_color_selected()
    } else {
        fg_color_not_selected()
    };

    let binding = parser.read().unwrap();
    let screen = binding.screen();
    let title = Line::from(" Preview ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(fg_color_))
        .title(title)
        .style(Style::default());

    f.render_widget(
        PseudoTerminal::new(screen)
            .cursor(tui_term::widget::Cursor::default().symbol(""))
            .block(block),
        chunk,
    );
}

fn preview_command(file_name: String, line_number: u32) -> CommandBuilder {
    let cwd = std::env::current_dir().unwrap();
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
    cmd
}

fn render_targets_block(model: &mut Model, f: &mut Frame, chunk: ratatui::layout::Rect) {
    f.render_stateful_widget(
        targets_block(
            " Targets ",
            model.narrow_down_targets(),
            model.current_pane.is_main(),
        ),
        chunk,
        // NOTE: It is against TEA's way to update the model value on the UI side, but it is unavoidable so it is allowed.
        &mut model.targets_list_state,
    );
}

fn render_input_block(model: &mut Model, f: &mut Frame, chunk: ratatui::layout::Rect) {
    f.render_widget(
        // NOTE: To show cursor, use rhysd/tui-textarea
        // input_block(" Input ", &model.key_input, model.current_pane.is_main()),
        // TODO: text_areaのスタイルを良い感じにする
        model.text_area.widget(),
        chunk,
    );
}

fn render_history_block(model: &mut Model, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let history_block = Paragraph::new(Line::from("Coming soon...")).block(
        rounded_border_block(" History ", model.current_pane.is_history())
            .padding(ratatui::widgets::Padding::new(2, 0, 0, 0)),
    );
    f.render_widget(history_block, chunk);
}

fn render_key_bindings_block(model: &mut Model, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let hint_text = match model.current_pane {
        super::app::CurrentPane::Main => {
            "(Any key except the following): Narrow down targets, <UP>/<DOWN>/<c-n>/<c-p>: Move cursor, <Enter>: Execute target, <esc>: Quit, <tab> Move to next tab, <BACKSPACE>/<c-h>: Delete last character, <c-w>: Delete all key input"
        }
        super::app::CurrentPane::History => "q/<esc>: Quit, <tab> Move to next tab",
    };
    let current_keys_hint = Span::styled(hint_text, Style::default().fg(fg_color_selected()));

    let title = " Key bindings ";
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(Color::default()))
        .style(Style::default())
        .padding(ratatui::widgets::Padding::new(2, 2, 0, 0));
    let key_notes_footer = Paragraph::new(current_keys_hint)
        .wrap(Wrap { trim: true })
        .block(block);

    f.render_widget(key_notes_footer, chunk);
}
fn input_block<'a>(title: &'a str, target_input: &'a str, is_current: bool) -> Paragraph<'a> {
    let fg_color = if is_current {
        fg_color_selected()
    } else {
        fg_color_not_selected()
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
        fg_color_selected()
    } else {
        fg_color_not_selected()
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
