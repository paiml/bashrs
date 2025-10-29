// REPL Tab Completion Module
//
// Task: REPL-006-001 - Tab completion for commands and modes
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 15+ scenarios
// - Integration tests: Tab completion behavior with assert_cmd
// - Mutation score: ≥90%
// - Complexity: <10 per function

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

/// Tab completion helper for bashrs REPL
///
/// Provides intelligent completion for:
/// - REPL commands (:mode, :parse, :purify, :lint, :history, :vars, :clear)
/// - Mode names (normal, purify, lint, debug, explain)
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
        if line_before_cursor.starts_with(':') {
            // Check if we're completing a mode name after `:mode `
            if line_before_cursor.starts_with(":mode ") {
                let mode_start = 6; // Position after ":mode "
                let word = &line_before_cursor[mode_start..];
                let completions = self.complete_mode(word);
                return Ok((mode_start, completions));
            }

            // Complete the command itself
            let word = &line_before_cursor[1..]; // Skip the ':'
            let completions = self.complete_command(word);
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

impl Highlighter for ReplCompleter {}

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

        assert_eq!(completer.commands.len(), 7);
        assert_eq!(completer.modes.len(), 5);
        assert!(completer.commands.contains(&"mode".to_string()));
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
        let replacements: Vec<_> = completions.iter()
            .map(|p| p.replacement.as_str())
            .collect();
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

        assert_eq!(completer.commands.len(), 7);
        assert_eq!(completer.modes.len(), 5);
    }
}
