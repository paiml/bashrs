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

    /// Get file path completions (for :load and :source commands)
    fn complete_file_path(&self, partial_path: &str) -> Vec<Pair> {
        // Split path into directory and filename prefix
        let path = Path::new(partial_path);
        let (dir_path, file_prefix) = if partial_path.ends_with('/') {
            // User typed a directory ending with /
            (path, "")
        } else {
            // Extract directory and filename
            match path.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => {
                    let prefix = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    (parent, prefix)
                }
                _ => {
                    // No parent directory, search current directory
                    let prefix = path.to_str().unwrap_or("");
                    (Path::new("."), prefix)
                }
            }
        };

        // Read directory contents
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(), // Directory doesn't exist or no permission
        };

        // Filter and map to completion pairs
        let mut completions = Vec::new();
        for entry in entries.flatten() {
            let file_name_os = entry.file_name();
            let file_name = match file_name_os.to_str() {
                Some(name) => name,
                None => continue, // Skip invalid UTF-8
            };

            // Filter by prefix
            if !file_name.starts_with(file_prefix) {
                continue;
            }

            // Skip hidden files (starting with .)
            if file_name.starts_with('.') && !file_prefix.starts_with('.') {
                continue;
            }

            // Build completion path
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let display_name = if is_dir {
                format!("{}/", file_name)
            } else {
                file_name.to_string()
            };

            // Build replacement path
            let replacement = if dir_path.to_str() == Some(".") {
                // Current directory - don't include "./"
                if is_dir {
                    format!("{}/", file_name)
                } else {
                    file_name.to_string()
                }
            } else {
                // Other directory - include full path
                let full_path = dir_path.join(file_name);
                if is_dir {
                    // Append / to directories
                    full_path
                        .to_str()
                        .map(|s| format!("{}/", s))
                        .unwrap_or_default()
                } else {
                    full_path.to_str().unwrap_or("").to_string()
                }
            };

            completions.push(Pair {
                display: display_name,
                replacement,
            });
        }

        // Sort: directories first, then files, alphabetically
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
            .map(|i| i + 1)
            .unwrap_or(0);

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
mod tests {
    use super::*;
    use rustyline::history::MemHistory;

    // ===== RED PHASE: Unit Tests (These should FAIL initially) =====

    #[test]
    fn test_REPL_006_001_completer_new() {
        let completer = ReplCompleter::new();

        assert_eq!(completer.commands.len(), 11);
        assert_eq!(completer.modes.len(), 5);
        assert!(completer.commands.contains(&"mode".to_string()));
        assert!(completer.commands.contains(&"load".to_string()));
        assert!(completer.commands.contains(&"source".to_string()));
        assert!(completer.modes.contains(&"normal".to_string()));
    }

    #[test]
    fn test_REPL_006_001_complete_command_prefix() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("mo");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, ":mode");
    }

    #[test]
    fn test_REPL_006_001_complete_command_multiple_matches() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("p");

        assert_eq!(completions.len(), 2); // "parse" and "purify"
        let replacements: Vec<_> = completions.iter().map(|p| p.replacement.as_str()).collect();
        assert!(replacements.contains(&":parse"));
        assert!(replacements.contains(&":purify"));
    }

    #[test]
    fn test_REPL_006_001_complete_command_no_match() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("xyz");

        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_REPL_006_001_complete_mode_prefix() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("pur");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "purify");
    }

    #[test]
    fn test_REPL_006_001_complete_mode_multiple_matches() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("l");

        assert_eq!(completions.len(), 1); // Only "lint"
        assert_eq!(completions[0].replacement, "lint");
    }

    #[test]
    fn test_REPL_006_001_complete_mode_empty_shows_all() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("");

        assert_eq!(completions.len(), 5); // All modes
    }

    #[test]
    fn test_REPL_006_001_complete_bash_construct_parameter_expansion() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_bash_construct("${var:");

        assert!(completions.len() >= 4); // At least :-,  :=, :?, :+
    }

    #[test]
    fn test_REPL_006_001_complete_bash_construct_for_loop() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_bash_construct("for");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "for i in");
    }

    #[test]
    fn test_REPL_006_001_complete_full_line_command() {
        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let (start, completions) = completer.complete(":mo", 3, &ctx).unwrap();

        assert_eq!(start, 0);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, ":mode");
    }

    #[test]
    fn test_REPL_006_001_complete_full_line_mode_name() {
        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let (start, completions) = completer.complete(":mode pur", 9, &ctx).unwrap();

        assert_eq!(start, 6); // Start after ":mode "
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "purify");
    }

    #[test]
    fn test_REPL_006_001_complete_bash_in_normal_line() {
        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let (start, completions) = completer.complete("for", 3, &ctx).unwrap();

        assert_eq!(start, 0);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "for i in");
    }

    #[test]
    fn test_REPL_006_001_complete_case_insensitive_command() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_command("MO");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, ":mode");
    }

    #[test]
    fn test_REPL_006_001_complete_case_insensitive_mode() {
        let completer = ReplCompleter::new();

        let completions = completer.complete_mode("PUR");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "purify");
    }

    #[test]
    fn test_REPL_006_001_default_trait() {
        let completer = ReplCompleter::default();

        assert_eq!(completer.commands.len(), 11);
        assert_eq!(completer.modes.len(), 5);
    }

    // ===== REPL-009-002: File Path Completion Tests =====

    #[test]
    fn test_REPL_009_002_complete_file_path_current_dir() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let completer = ReplCompleter::new();
        let completions = completer.complete_file_path("te");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(completions.len() > 0);
        assert!(completions.iter().any(|p| p.replacement == "test.sh"));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_with_directory() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("examples");
        std::fs::create_dir(&sub_dir).unwrap();
        let test_file = sub_dir.join("script.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        let completer = ReplCompleter::new();
        let path_str = format!("{}/scr", sub_dir.display());
        let completions = completer.complete_file_path(&path_str);

        assert!(completions.len() > 0);
        assert!(completions.iter().any(|p| p.display.contains("script.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_directories_first() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir_a");
        let dir2 = temp_dir.path().join("dir_b");
        let file1 = temp_dir.path().join("file_a.sh");
        std::fs::create_dir(&dir1).unwrap();
        std::fs::create_dir(&dir2).unwrap();
        std::fs::File::create(&file1).unwrap();

        let completer = ReplCompleter::new();
        let path_str = format!("{}/", temp_dir.path().display());
        let completions = completer.complete_file_path(&path_str);

        // Directories should come before files
        let dir_positions: Vec<usize> = completions
            .iter()
            .enumerate()
            .filter(|(_, p)| p.display.ends_with('/'))
            .map(|(i, _)| i)
            .collect();

        let file_positions: Vec<usize> = completions
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.display.ends_with('/'))
            .map(|(i, _)| i)
            .collect();

        if !dir_positions.is_empty() && !file_positions.is_empty() {
            assert!(dir_positions.iter().max().unwrap() < file_positions.iter().min().unwrap());
        }
    }

    #[test]
    fn test_REPL_009_002_complete_full_line_load_command() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("example.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let path_str = format!("{}/ex", temp_dir.path().display());
        let line = format!(":load {}", path_str);
        let (start, completions) = completer.complete(&line, line.len(), &ctx).unwrap();

        assert_eq!(start, 6); // Position after ":load "
        assert!(completions.len() > 0);
        assert!(completions.iter().any(|p| p.display.contains("example.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_full_line_source_command() {
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("script.sh");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "#!/bin/bash").unwrap();

        let completer = ReplCompleter::new();
        let history = MemHistory::new();
        let ctx = Context::new(&history);

        let path_str = format!("{}/scr", temp_dir.path().display());
        let line = format!(":source {}", path_str);
        let (start, completions) = completer.complete(&line, line.len(), &ctx).unwrap();

        assert_eq!(start, 8); // Position after ":source "
        assert!(completions.len() > 0);
        assert!(completions.iter().any(|p| p.display.contains("script.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_no_hidden_files() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let hidden_file = temp_dir.path().join(".hidden.sh");
        let visible_file = temp_dir.path().join("visible.sh");
        std::fs::File::create(&hidden_file).unwrap();
        std::fs::File::create(&visible_file).unwrap();

        let completer = ReplCompleter::new();
        let path_str = format!("{}/", temp_dir.path().display());
        let completions = completer.complete_file_path(&path_str);

        // Should not include hidden files unless user explicitly types "."
        assert!(completions.iter().any(|p| p.display.contains("visible.sh")));
        assert!(!completions.iter().any(|p| p.display.contains(".hidden.sh")));
    }

    #[test]
    fn test_REPL_009_002_complete_file_path_nonexistent_dir() {
        let completer = ReplCompleter::new();
        let completions = completer.complete_file_path("/nonexistent/path/file");

        // Should return empty vector for nonexistent directories
        assert_eq!(completions.len(), 0);
    }

    // ===== REPL-015-002-INT: Syntax Highlighting Integration Tests =====

    /// Test: REPL-015-002-INT-001 - Highlighter integration basic
    #[test]
    fn test_REPL_015_002_INT_001_highlighter_basic() {
        use crate::repl::highlighting::strip_ansi_codes;

        let completer = ReplCompleter::new();

        let input = "echo hello";
        let highlighted = completer.highlight(input, 0);

        // Should contain ANSI codes
        assert!(highlighted.contains("\x1b["));

        // Should preserve original text when stripped
        let stripped = strip_ansi_codes(&highlighted);
        assert_eq!(stripped, input);
    }

    /// Test: REPL-015-002-INT-002 - Highlight with variables
    #[test]
    fn test_REPL_015_002_INT_002_highlight_variables() {
        let completer = ReplCompleter::new();

        let input = "echo $HOME";
        let highlighted = completer.highlight(input, 0);

        // Should highlight 'echo' as command (cyan)
        assert!(highlighted.contains("\x1b[36mecho\x1b[0m"));

        // Should highlight '$HOME' as variable (yellow)
        assert!(highlighted.contains("\x1b[33m$HOME\x1b[0m"));
    }

    /// Test: REPL-015-002-INT-003 - Highlight with keywords
    #[test]
    fn test_REPL_015_002_INT_003_highlight_keywords() {
        let completer = ReplCompleter::new();

        let input = "if [ -f test ]; then echo found; fi";
        let highlighted = completer.highlight(input, 0);

        // Should highlight keywords (blue)
        assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));
    }

    /// Test: REPL-015-002-INT-004 - Highlight multiline input
    #[test]
    fn test_REPL_015_002_INT_004_highlight_multiline() {
        let completer = ReplCompleter::new();

        let input = "for i in 1 2 3\ndo echo $i\ndone";
        let highlighted = completer.highlight(input, 0);

        // Should highlight keywords across lines
        assert!(highlighted.contains("\x1b[1;34mfor\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mdo\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;34mdone\x1b[0m"));

        // Should highlight variable
        assert!(highlighted.contains("\x1b[33m$i\x1b[0m"));
    }

    /// Test: REPL-015-002-INT-005 - Empty input
    #[test]
    fn test_REPL_015_002_INT_005_empty_input() {
        let completer = ReplCompleter::new();

        let highlighted = completer.highlight("", 0);

        // Should handle empty input gracefully
        assert_eq!(highlighted.as_ref(), "");
    }

    /// Test: REPL-015-002-INT-006 - Special characters
    #[test]
    fn test_REPL_015_002_INT_006_special_characters() {
        let completer = ReplCompleter::new();

        let input = "echo \"test\" | grep 'pattern' && exit 0";
        let highlighted = completer.highlight(input, 0);

        // Should highlight strings (green)
        assert!(highlighted.contains("\x1b[32m\"test\"\x1b[0m"));
        assert!(highlighted.contains("\x1b[32m'pattern'\x1b[0m"));

        // Should highlight operators (magenta)
        assert!(highlighted.contains("\x1b[35m|\x1b[0m"));
        assert!(highlighted.contains("\x1b[35m&&\x1b[0m"));
    }
}
