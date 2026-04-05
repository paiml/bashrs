// REPL Tab Completion Module
//
// Task: REPL-006-001 - Tab completion for commands and modes
// Task: REPL-009-002 - File path completion for :load and :source
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 15+ scenarios
// - Integration tests: Tab completion behavior with assert_cmd
// - Mutation score: ≥90%
// - Complexity: <10 per function

use crate::repl::highlighting::highlight_bash;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::fs;
use std::path::Path;

/// Tab completion helper for bashrs REPL
///
/// Provides intelligent completion for:
/// - REPL commands (:mode, :parse, :purify, :lint, :load, :source, :functions, :reload, :history, :vars, :clear)
/// - Mode names (normal, purify, lint, debug, explain)
/// - File paths (for :load and :source commands)
/// - Common bash constructs in explain mode
///
/// # Examples
///
/// ```
/// use bashrs::repl::completion::ReplCompleter;
///
/// let completer = ReplCompleter::new();
/// // User types ":mo" + Tab → completes to ":mode"
/// // User types ":mode p" + Tab → completes to ":mode purify"
/// // User types ":load ex" + Tab → completes to ":load examples/"
/// ```
#[derive(Debug, Clone)]
pub struct ReplCompleter {
    /// REPL commands (without : prefix)
    commands: Vec<String>,
    /// Available mode names
    modes: Vec<String>,
    /// Common bash constructs for explain mode
    bash_constructs: Vec<String>,
}

impl ReplCompleter {
    /// Create a new ReplCompleter with default completions
    pub fn new() -> Self {
        Self {
            commands: vec![
                "mode".to_string(),
                "parse".to_string(),
                "purify".to_string(),
                "lint".to_string(),
                "load".to_string(),
                "source".to_string(),
                "functions".to_string(),
                "reload".to_string(),
                "history".to_string(),
                "vars".to_string(),
                "clear".to_string(),
            ],
            modes: vec![
                "normal".to_string(),
                "purify".to_string(),
                "lint".to_string(),
                "debug".to_string(),
                "explain".to_string(),
            ],
            bash_constructs: vec![
                "${var:-default}".to_string(),
                "${var:=default}".to_string(),
                "${var:?error}".to_string(),
                "${var:+alternate}".to_string(),
                "${#var}".to_string(),
                "for i in".to_string(),
                "if [".to_string(),
                "while".to_string(),
                "case".to_string(),
            ],
        }
    }

    /// Get command completions (for lines starting with :)
    fn complete_command(&self, word: &str) -> Vec<Pair> {
        let word_lower = word.to_lowercase();

        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(&word_lower))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: format!(":{}", cmd),
            })
            .collect()
    }

    /// Get mode name completions (for `:mode <tab>`)
    fn complete_mode(&self, word: &str) -> Vec<Pair> {
        let word_lower = word.to_lowercase();

        self.modes
            .iter()
            .filter(|mode| mode.starts_with(&word_lower))
            .map(|mode| Pair {
                display: mode.clone(),
                replacement: mode.clone(),
            })
            .collect()
    }

    /// Get bash construct completions (for explain mode)
    fn complete_bash_construct(&self, word: &str) -> Vec<Pair> {
        self.bash_constructs
            .iter()
            .filter(|construct| construct.starts_with(word))
            .map(|construct| Pair {
                display: construct.clone(),
                replacement: construct.clone(),
            })
            .collect()
    }

    /// Build the replacement string for a completion entry
    fn build_replacement(dir_path: &Path, file_name: &str, is_dir: bool) -> String {
        if dir_path.to_str() == Some(".") {
            if is_dir {
                format!("{}/", file_name)
            } else {
                file_name.to_string()
            }
        } else {
            let full_path = dir_path.join(file_name);
            let path_str = full_path.to_str().unwrap_or("");
            if is_dir {
                format!("{}/", path_str)
            } else {
                path_str.to_string()
            }
        }
    }

    /// Get file path completions (for :load and :source commands)
    fn complete_file_path(&self, partial_path: &str) -> Vec<Pair> {
        let path = Path::new(partial_path);
        let (dir_path, file_prefix) = if partial_path.ends_with('/') {
            (path, "")
        } else {
            match path.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => {
                    let prefix = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    (parent, prefix)
                }
                _ => (Path::new("."), path.to_str().unwrap_or("")),
            }
        };

        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        let mut completions = Vec::new();
        for entry in entries.flatten() {
            let file_name_os = entry.file_name();
            let file_name = match file_name_os.to_str() {
                Some(name) => name,
                None => continue,
            };

            if !file_name.starts_with(file_prefix) {
                continue;
            }
            if file_name.starts_with('.') && !file_prefix.starts_with('.') {
                continue;
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let display_name = if is_dir {
                format!("{}/", file_name)
            } else {
                file_name.to_string()
            };
            let replacement = Self::build_replacement(dir_path, file_name, is_dir);

            completions.push(Pair {
                display: display_name,
                replacement,
            });
        }

        completions.sort_by(|a, b| {
            let a_is_dir = a.display.ends_with('/');
            let b_is_dir = b.display.ends_with('/');
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.display.cmp(&b.display),
            }
        });

        completions
    }
}

impl Default for ReplCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for ReplCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let line_before_cursor = &line[..pos];

        // Complete REPL commands (lines starting with :)
        if let Some(after_colon) = line_before_cursor.strip_prefix(':') {
            // Check if we're completing a file path after `:load ` or `:source `
            if line_before_cursor.starts_with(":load ") {
                let path_start = 6; // Position after ":load "
                let partial_path = &line_before_cursor[path_start..];
                let completions = self.complete_file_path(partial_path);
                return Ok((path_start, completions));
            }

            if line_before_cursor.starts_with(":source ") {
                let path_start = 8; // Position after ":source "
                let partial_path = &line_before_cursor[path_start..];
                let completions = self.complete_file_path(partial_path);
                return Ok((path_start, completions));
            }

            // Check if we're completing a mode name after `:mode `
            if line_before_cursor.starts_with(":mode ") {
                let mode_start = 6; // Position after ":mode "
                let word = &line_before_cursor[mode_start..];
                let completions = self.complete_mode(word);
                return Ok((mode_start, completions));
            }

            // Complete the command itself
            let completions = self.complete_command(after_colon);
            return Ok((0, completions));
        }

        // Complete bash constructs (for any other input)
        // Find the last word boundary
        let word_start = line_before_cursor
            .rfind(char::is_whitespace)
            .map_or(0, |i| i + 1);

        let word = &line_before_cursor[word_start..];
        let completions = self.complete_bash_construct(word);

        Ok((word_start, completions))
    }
}

impl Hinter for ReplCompleter {
    type Hint = String;
}

impl Highlighter for ReplCompleter {
    /// Apply syntax highlighting to bash input
    ///
    /// Highlights keywords, strings, variables, commands, operators, and comments
    /// using ANSI color codes for terminal display.
    ///
    /// # Arguments
    ///
    /// * `line` - The input line to highlight
    /// * `_pos` - Cursor position (unused, required by trait)
    ///
    /// # Returns
    ///
    /// A `Cow::Owned` string with ANSI color codes applied
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Apply syntax highlighting from highlighting module
        Cow::Owned(highlight_bash(line))
    }

    /// Enable character-by-character highlighting
    ///
    /// Returns true to enable live syntax highlighting as the user types.
    ///
    /// # Arguments
    ///
    /// * `_line` - The current input line (unused)
    /// * `_pos` - Cursor position (unused)
    /// * `_kind` - Command kind (unused, but required by trait)
    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        // Enable live highlighting
        true
    }
}

impl Validator for ReplCompleter {}

impl Helper for ReplCompleter {}

#[cfg(test)]
#[path = "completion_tests_ext.rs"]
mod tests_ext;
