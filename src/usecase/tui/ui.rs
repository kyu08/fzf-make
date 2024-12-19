use super::app::{AppState, CurrentPane, Model, SelectCommandState};
use crate::model::command;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SColor, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect_tui::into_span;
use tui_term::widget::PseudoTerminal;

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
        render_preview_block2(model, f, preview_and_commands[0]);

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

// Because the setup process of the terminal and render_widget function need to be done in the same scope, the call of the render_widget function is included.
fn render_preview_block(model: &SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    let narrow_down_commands = model.narrow_down_commands();

    let selecting_command =
        narrow_down_commands.get(model.commands_list_state.selected().unwrap_or(0));

    let (fg_color_, border_style) =
        color_and_border_style_for_selectable(model.current_pane.is_main());

    let title = Line::from(" ✨ Preview ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_style)
        .border_style(Style::default().fg(fg_color_))
        .title(title)
        .title_style(TITLE_STYLE);

    if !model.get_search_area_text().is_empty() && narrow_down_commands.is_empty() {
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

fn render_preview_block2(model: &SelectCommandState, f: &mut Frame, chunk: ratatui::layout::Rect) {
    f.render_widget(Clear, chunk);
    // 枠線も含めた行数なので実際に表示できる行数はここから2行引いた数値
    let row_count = chunk.rows().count();
    // TODO: row numberを指定する必要がある
    // commandの情報からファイルの一部を抜いてくる
    let narrow_down_commands = model.narrow_down_commands();
    let selecting_command =
        narrow_down_commands.get(model.commands_list_state.selected().unwrap_or(0));

    let command = selecting_command.unwrap();
    let start = command.line_number as usize;
    let end = start + row_count - 1;
    let path = command.file_name.clone();

    let file = File::open(path).unwrap(); // TODO: remove unwrap
    let reader = BufReader::new(file);

    let source_lines: Vec<_> = reader
        .lines()
        // TODO: commandのファイルのfile_number行目から(file_number + row_count - 1)行数を取得する
        .skip(start - 1)
        .take(end - start + 1)
        .map(|line| line.unwrap())
        .collect();
    // let source_lines = source_lines.join("\n)");
    let lines = {
        if let Some(command) = selecting_command {
            let ps = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            // NOTE: いったんrsで指定してもMakefileやjsonのハイライトはできているのでこれでいく
            let syntax = ps.find_syntax_by_extension("rs").unwrap();
            let theme = &mut ts.themes["base16-ocean.dark"].clone();

            theme.settings.background = Some(SColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0, // To get bg same as ratatui's background, make this transparent.
            });
            // let fuga = LinesWithEndings::from(&source_lines);
            let mut lines = vec![];
            for (index, line) in source_lines.iter().enumerate() {
                // for (index, line) in fuga.enumerate() {
                theme.settings.background = Some(SColor {
                    r: 49,
                    g: 49,
                    b: 49,
                    // To get bg same as ratatui's background, make this transparent.
                    a: if index == 0_usize { 70 } else { 0 },
                });
                // データではなく表示が悪いことまでわかったのでそっちの方面でデバッグする
                // if command.args == "test" {
                //     panic!("{:?}", source_lines);
                // }
                let mut h = HighlightLines::new(syntax, theme);
                // LinesWithEndings enables use of newlines mode
                let spans: Vec<Span> = h
                    .highlight_line(line, &ps)
                    .unwrap()
                    .into_iter()
                    .filter_map(|segment| into_span(segment).ok())
                    .collect();

                lines.push(Line::from(spans));
            }
            lines
        } else {
            vec![]
        }
    };

    // TODO: nvim外のterminalでも問題なく表示できてそうかチェックする
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
    f.render_widget(Clear, chunk);
    f.render_widget(preview_widget, chunk);
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
