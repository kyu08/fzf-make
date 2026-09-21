use anyhow::{Context, Result};
use ratatui::style::Style;
use rust_embed::RustEmbed;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Color as SColor, Theme, ThemeSet},
    parsing::SyntaxSet,
};
use syntect_tui::into_span;

/// Upper bound of the time spent highlighting a single file.
///
/// Highlighting runs in a background thread, so exceeding this budget never blocks the TUI.
/// The budget only exists to stop burning CPU on a file whose every line is pathological.
const HIGHLIGHT_TIME_BUDGET: Duration = Duration::from_millis(1000);

/// A source line split into styled fragments.
pub type StyledLine = Vec<(Style, String)>;

#[derive(Debug, Clone, PartialEq)]
enum Highlight {
    /// Highlighting is still running in the background.
    InProgress,
    /// `lines[i]` holds the styled fragments of source line `i`.
    Done(Vec<StyledLine>),
}

#[derive(Debug, Clone, PartialEq)]
struct PreviewFile {
    /// Source lines with tabs expanded.
    lines: Vec<String>,
    highlight: Highlight,
}

/// Holds the syntax highlighting result of every file shown in the preview pane.
///
/// A file is read and highlighted once, on the first draw that needs it, and the result is
/// reused for every later draw. Without this, the preview was re-read and re-highlighted on
/// every key press and it makes whole performance worse.
///
/// The in-memory cache is never purged till quitting fzf-make, so edits made to a file while fzf-make is running are not
/// reflected in the preview.
///
/// Deliberately not `Clone`: dropping the cache cancels the background work, so a second owner
/// would make the cancellation fire while a task is still running.
#[derive(Debug, Default)]
pub struct PreviewCache {
    files: Arc<Mutex<HashMap<PathBuf, PreviewFile>>>,
    cancelled: Arc<AtomicBool>,
}

impl Drop for PreviewCache {
    /// Stops the background highlighting.
    ///
    /// `#[tokio::main]` waits for every running blocking task before the process exits, so a
    /// highlight that is still in flight delays the exit even though the TUI has already shut
    /// down. Cancelling here bounds that wait to the line being highlighted right now.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl PreviewCache {
    /// Reads `path` and starts highlighting it in the background. Returns immediately.
    ///
    /// Does nothing if `path` is already cached, so this is safe to call on every draw.
    pub fn load(&self, path: &Path, extension: &'static str) {
        let lines = {
            let mut files = match self.files.lock() {
                Ok(files) => files,
                Err(_) => return,
            };
            if files.contains_key(path) {
                return;
            }
            // HACK: tabs are expanded here as a workaround for
            // https://github.com/ratatui/ratatui/issues/876
            let lines: Vec<String> = match fs::read_to_string(path) {
                Ok(content) => content.lines().map(|line| line.replace('\t', "    ")).collect(),
                Err(_) => return,
            };
            files.insert(
                path.to_path_buf(),
                PreviewFile {
                    lines: lines.clone(),
                    highlight: Highlight::InProgress,
                },
            );
            lines
        };

        let files = self.files.clone();
        let cancelled = self.cancelled.clone();
        let path = path.to_path_buf();
        let highlight = move || {
            let highlighted = highlight_lines(&lines, extension, &cancelled);
            if let Ok(mut files) = files.lock()
                && let Some(file) = files.get_mut(&path)
            {
                file.highlight = Highlight::Done(highlighted);
            }
        };

        // Highlighting a single line can take hundreds of milliseconds, so it must not run on
        // the thread that draws the TUI. Outside a tokio runtime (tests) it runs inline.
        match tokio::runtime::Handle::try_current() {
            Ok(_) => {
                tokio::task::spawn_blocking(highlight);
            }
            Err(_) => highlight(),
        }
    }

    /// Returns the styled fragments of lines `start_index..=end_index` of `path`.
    ///
    /// Lines whose highlighting has not finished yet are returned unstyled, so the preview shows
    /// the file content immediately and gains colour once the background work completes.
    pub fn styled_lines(&self, path: &Path, start_index: usize, end_index: usize) -> Vec<StyledLine> {
        let files = match self.files.lock() {
            Ok(files) => files,
            Err(_) => return vec![],
        };
        let file = match files.get(path) {
            Some(file) => file,
            None => return vec![],
        };

        let end_index = end_index.min(file.lines.len().saturating_sub(1));
        if file.lines.is_empty() || file.lines.len() <= start_index {
            return vec![];
        }

        match &file.highlight {
            Highlight::Done(highlighted) => highlighted[start_index..=end_index].to_vec(),
            Highlight::InProgress => file.lines[start_index..=end_index]
                .iter()
                .map(|line| vec![(Style::default(), line.clone())])
                .collect(),
        }
    }
}

/// Highlights every line of a file with a single stateful highlighter.
///
/// Reusing one `HighlightLines` across the whole file is what keeps this fast. syntect's Makefile
/// grammar matches a target definition with a deeply nested regex, and a line such as
/// `$(eval X := $(shell ...))` makes that regex backtrack catastrophically. The grammar only
/// attempts it while the parser sits in the root context, so carrying the parse state from the
/// previous lines skips it for every recipe line. Highlighting such a file line by line, with a
/// fresh highlighter per line, took ~680ms; carrying the state takes ~3ms.
/// See https://github.com/kyu08/fzf-make/issues/595.
fn highlight_lines(lines: &[String], extension: &str, cancelled: &AtomicBool) -> Vec<StyledLine> {
    let syntax_set = syntax_set();
    let syntax = syntax_set
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, theme());

    let started = Instant::now();
    let mut result: Vec<StyledLine> = Vec::with_capacity(lines.len());
    for line in lines {
        // A pathological line placed before any context is established still costs hundreds of
        // milliseconds, so give up on the rest of the file once the budget is spent, or as soon
        // as the cache is dropped because fzf-make is exiting.
        if HIGHLIGHT_TIME_BUDGET < started.elapsed() || cancelled.load(Ordering::Relaxed) {
            result.extend(lines[result.len()..].iter().map(|line| plain(line)));
            break;
        }

        result.push(match highlighter.highlight_line(line, syntax_set) {
            Ok(segments) => segments
                .into_iter()
                .filter_map(|segment| into_span(segment).ok())
                .map(|span| (span.style, span.content.into_owned()))
                .collect(),
            Err(_) => plain(line),
        });
    }
    result
}

fn plain(line: &str) -> StyledLine {
    vec![(Style::default(), line.to_string())]
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let mut theme_set = ThemeSet::load_defaults();
        if let Ok(path) = load_syntax_highlighting_theme() {
            let _ = theme_set.add_from_folder(path);
        }

        let mut theme = theme_set
            .themes
            .get("OneHalfDark")
            .unwrap_or(&theme_set.themes["base16-ocean.dark"])
            .clone();
        // Make the background transparent so the preview keeps ratatui's background.
        // The background of the row that defines the selected command is applied when rendering.
        theme.settings.background = Some(SColor {
            r: 94,
            g: 120,
            b: 200,
            a: 0,
        });
        theme
    })
}

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

fn load_syntax_highlighting_theme() -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("fzf-make-syntax-highlighting-assets");
    let version_file = temp_dir.join(".version");
    let current_version = env!("CARGO_PKG_VERSION");

    let should_extract = if temp_dir.exists() {
        match fs::read_to_string(&version_file) {
            // extract is done only once per version
            Ok(v) => v.trim() != current_version,
            Err(_) => true,
        }
    } else {
        true
    };

    if should_extract {
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).context("Failed to remove existing temp directory")?;
        }
        fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

        let theme_file_name = "OneHalfDark.tmTheme";
        let path = temp_dir.join(theme_file_name);
        let content = Asset::get(theme_file_name).context("Failed to get embedded asset")?;

        fs::write(path, content.data).context("Failed to write asset file")?;
        fs::write(version_file, current_version).context("Failed to write version file")?;
    }

    Ok(temp_dir)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    const PATHOLOGICAL_LINE: &str = "    $(eval RESOLVED_TARGETS := $(shell bash resolve.sh $(DEPENDENCY_SERVICES)))";

    /// Highlights without ever cancelling.
    fn highlight_all(lines: &[String], extension: &str) -> Vec<StyledLine> {
        highlight_lines(lines, extension, &AtomicBool::new(false))
    }

    /// Guards against a regression where the preview lost all of its colour, which is what
    /// happened while the pathological lines were worked around by skipping the highlighter.
    #[test]
    fn highlight_lines_actually_applies_more_than_one_style() {
        let lines = [
            ".PHONY: build",
            "build:",
            "    @cargo build --release",
            PATHOLOGICAL_LINE,
        ]
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>();

        let styles: Vec<Style> = highlight_all(&lines, "mk")
            .into_iter()
            .flatten()
            .map(|(style, _)| style)
            .collect();

        assert!(
            1 < styles.iter().collect::<std::collections::HashSet<_>>().len(),
            "Expected the highlighter to produce more than one style, but got {styles:?}",
        );
    }

    #[test]
    fn highlight_lines_returns_one_entry_per_source_line() {
        struct Case {
            title: &'static str,
            lines: Vec<String>,
        }
        let cases = vec![
            Case {
                title: "empty file",
                lines: vec![],
            },
            Case {
                title: "normal makefile",
                lines: [".PHONY: build", "build:", "    @cargo build --release"]
                    .iter()
                    .map(|l| l.to_string())
                    .collect(),
            },
            Case {
                title: "pathological line inside a recipe",
                lines: [
                    "deploy:",
                    PATHOLOGICAL_LINE,
                    "    docker compose rm -fsv $(RESOLVED_TARGETS)",
                ]
                .iter()
                .map(|l| l.to_string())
                .collect(),
            },
        ];

        for case in cases {
            assert_eq!(case.lines.len(), highlight_all(&case.lines, "mk").len(), "\nFailed: 🚨{:?}🚨\n", case.title,);
        }
    }

    #[test]
    fn highlight_lines_concatenates_back_to_the_source_line() {
        let lines = vec!["deploy:".to_string(), PATHOLOGICAL_LINE.to_string()];

        for (styled, source) in highlight_all(&lines, "mk").iter().zip(lines.iter()) {
            let concatenated: String = styled.iter().map(|(_, content)| content.as_str()).collect();
            assert_eq!(source.trim_end(), concatenated.trim_end());
        }
    }

    #[test]
    fn highlight_lines_stays_within_the_time_budget() {
        // Every line is pathological, so the budget is what stops the loop.
        let lines = vec![PATHOLOGICAL_LINE.to_string(); 30];

        let started = Instant::now();
        let highlighted = highlight_all(&lines, "mk");
        let elapsed = started.elapsed();

        assert_eq!(lines.len(), highlighted.len());
        assert!(
            elapsed < HIGHLIGHT_TIME_BUDGET * 3,
            "Highlighting took {elapsed:?}, which is far beyond the budget of {HIGHLIGHT_TIME_BUDGET:?}",
        );
    }

    #[test]
    fn highlight_lines_stops_as_soon_as_it_is_cancelled() {
        // Every line is pathological, so without cancellation this takes seconds.
        let lines = vec![PATHOLOGICAL_LINE.to_string(); 30];

        let started = Instant::now();
        let highlighted = highlight_lines(&lines, "mk", &AtomicBool::new(true));
        let elapsed = started.elapsed();

        assert_eq!(lines.len(), highlighted.len());
        assert!(
            elapsed < Duration::from_millis(100),
            "Cancelled highlighting took {elapsed:?}, so it did not stop at the first line",
        );
    }

    #[test]
    fn dropping_the_cache_cancels_the_background_work() {
        let cache = PreviewCache::default();
        let cancelled = cache.cancelled.clone();

        assert!(!cancelled.load(Ordering::Relaxed));
        drop(cache);
        assert!(cancelled.load(Ordering::Relaxed));
    }

    #[test]
    fn styled_lines_returns_empty_for_an_unknown_file() {
        let cache = PreviewCache::default();
        assert_eq!(Vec::<StyledLine>::new(), cache.styled_lines(Path::new("/no/such/file"), 0, 10));
    }

    #[test]
    fn styled_lines_clamps_the_end_index_to_the_file_length() {
        let cache = PreviewCache::default();
        let path = std::env::current_dir().unwrap().join("Makefile");
        cache.load(&path, "mk");

        let line_count = fs::read_to_string(&path).unwrap().lines().count();
        assert_eq!(line_count, cache.styled_lines(&path, 0, line_count + 100).len());
    }
}
