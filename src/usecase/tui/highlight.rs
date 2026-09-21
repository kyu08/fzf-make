use anyhow::{Context, Result};
use ratatui::style::Style;
use rust_embed::RustEmbed;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Color as SColor, Theme, ThemeSet},
    parsing::SyntaxSet,
};
use syntect_tui::into_span;

/// A source line split into styled fragments.
pub type StyledLine = Vec<(Style, String)>;

/// Holds the syntax highlighting result of every file shown in the preview pane.
///
/// A file is read and highlighted once, on the first draw that needs it, and the result is reused
/// for every later draw. Before this cache existed, the preview re-read the file, reloaded
/// syntect's syntax and theme definitions, and re-highlighted every visible line on every draw,
/// which meant on every key press.
///
/// The cache is never invalidated, so edits made to a file while fzf-make is running are not
/// reflected in the preview.
#[derive(Debug, Clone, Default)]
pub struct PreviewCache {
    files: Arc<Mutex<HashMap<PathBuf, Vec<StyledLine>>>>,
}

impl PreviewCache {
    /// Reads `path` and highlights it. Does nothing if `path` is already cached, so this is safe
    /// to call on every draw.
    pub fn load(&self, path: &Path, extension: &'static str) {
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
        files.insert(path.to_path_buf(), highlight_lines(&lines, extension));
    }

    /// Returns the styled fragments of lines `start_index..=end_index` of `path`.
    pub fn styled_lines(&self, path: &Path, start_index: usize, end_index: usize) -> Vec<StyledLine> {
        let files = match self.files.lock() {
            Ok(files) => files,
            Err(_) => return vec![],
        };
        let lines = match files.get(path) {
            Some(lines) => lines,
            None => return vec![],
        };

        if lines.len() <= start_index {
            return vec![];
        }
        lines[start_index..=end_index.min(lines.len() - 1)].to_vec()
    }
}

/// Highlights every line of a file with a single stateful highlighter.
///
/// Reusing one `HighlightLines` across the whole file is what keeps this fast, and it is the fix
/// for https://github.com/kyu08/fzf-make/issues/595. syntect's Makefile grammar recognises a
/// target definition with a deeply nested regex, and a line such as `$(eval X := $(shell ...))`
/// makes that regex backtrack catastrophically. The grammar only attempts it while the parser sits
/// in the root context, so carrying the parse state over from the preceding lines skips it
/// entirely for every line inside a recipe. Highlighting this repository's own Makefile with a
/// fresh highlighter per line took ~680ms; carrying the state takes ~3ms.
///
/// Carrying the state is also what syntect expects: a fresh highlighter cannot see that a line
/// belongs to a recipe, a `define` block or a continued line, so it used to colour those lines as
/// if each of them started a new file.
fn highlight_lines(lines: &[String], extension: &str) -> Vec<StyledLine> {
    let syntax_set = syntax_set();
    let syntax = syntax_set
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, theme());

    lines
        .iter()
        .map(|line| match highlighter.highlight_line(line, syntax_set) {
            Ok(segments) => segments
                .into_iter()
                .filter_map(|segment| into_span(segment).ok())
                .map(|span| (span.style, span.content.into_owned()))
                .collect(),
            Err(_) => plain(line),
        })
        .collect()
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
        // Make the background transparent so that the preview keeps ratatui's background.
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

    /// Guards against a regression where the preview lost all of its colour, which is what the
    /// preview looked like while these lines were worked around by skipping the highlighter.
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

        let styles: Vec<Style> = highlight_lines(&lines, "mk")
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
            assert_eq!(case.lines.len(), highlight_lines(&case.lines, "mk").len(), "\nFailed: 🚨{:?}🚨\n", case.title,);
        }
    }

    #[test]
    fn highlight_lines_concatenates_back_to_the_source_line() {
        let lines = vec!["deploy:".to_string(), PATHOLOGICAL_LINE.to_string()];

        for (styled, source) in highlight_lines(&lines, "mk").iter().zip(lines.iter()) {
            let concatenated: String = styled.iter().map(|(_, content)| content.as_str()).collect();
            assert_eq!(source.trim_end(), concatenated.trim_end());
        }
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
