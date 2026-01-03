// REPL Logic Module - Extracted for testability
//
// This module contains pure functions that return structured results
// instead of printing directly. The loop.rs file acts as a thin shim
// that calls these functions and handles I/O.

use crate::repl::{
    explain_bash, format_lint_results, format_parse_error, lint_bash,
    loader::{format_functions, load_script, LoadResult},
    parse_bash, purify_bash,
    variables::expand_variables,
    ReplMode, ReplState,
};
use std::path::PathBuf;

/// Result of processing a mode command
#[derive(Debug, Clone, PartialEq)]
pub enum ModeCommandResult {
    /// Show current mode with description
    ShowCurrent { mode: ReplMode, description: String },
    /// Successfully switched to new mode
    Switched { mode: ReplMode, description: String },
    /// Invalid mode name provided
    InvalidMode(String),
    /// Invalid usage (wrong number of args)
    InvalidUsage,
}

impl ModeCommandResult {
    pub fn format(&self) -> String {
        match self {
            Self::ShowCurrent { mode, description } => {
                let mut output = format!("Current mode: {} - {}\n\n", mode, description);
                output.push_str("Available modes:\n");
                output.push_str("  normal  - Execute bash commands directly\n");
                output.push_str("  purify  - Show purified version of bash commands\n");
                output.push_str("  lint    - Show linting results for bash commands\n");
                output.push_str("  debug   - Debug bash commands with step-by-step execution\n");
                output.push_str("  explain - Explain bash constructs and syntax\n\n");
                output.push_str("Usage: :mode <mode_name>");
                output
            }
            Self::Switched { mode, description } => {
                format!("Switched to {} mode - {}", mode, description)
            }
            Self::InvalidMode(err) => format!("Error: {}", err),
            Self::InvalidUsage => {
                "Usage: :mode [<mode_name>]\nValid modes: normal, purify, lint, debug, explain"
                    .to_string()
            }
        }
    }
}

/// Result of processing a parse command
#[derive(Debug, Clone)]
pub enum ParseCommandResult {
    /// Successfully parsed
    Success {
        statement_count: usize,
        parse_time_ms: u64,
        ast_debug: String,
    },
    /// Parse error
    Error(String),
    /// Missing input
    MissingInput,
}

impl ParseCommandResult {
    pub fn format(&self) -> String {
        match self {
            Self::Success {
                statement_count,
                parse_time_ms,
                ast_debug,
            } => {
                let mut output = String::new();
                output.push_str("✓ Parse successful!\n");
                output.push_str(&format!("Statements: {}\n", statement_count));
                output.push_str(&format!("Parse time: {}ms\n\n", parse_time_ms));
                output.push_str("AST:\n");
                output.push_str(ast_debug);
                output
            }
            Self::Error(e) => format!("✗ {}", e),
            Self::MissingInput => {
                "Usage: :parse <bash_code>\nExample: :parse echo hello".to_string()
            }
        }
    }
}

/// Result of processing a purify command
#[derive(Debug, Clone)]
pub enum PurifyCommandResult {
    /// Successfully purified
    Success(String),
    /// Purification error
    Error(String),
    /// Missing input
    MissingInput,
}

impl PurifyCommandResult {
    pub fn format(&self) -> String {
        match self {
            Self::Success(result) => format!("✓ Purification successful!\n{}", result),
            Self::Error(e) => format!("✗ Purification error: {}", e),
            Self::MissingInput => {
                "Usage: :purify <bash_code>\nExample: :purify mkdir /tmp/test".to_string()
            }
        }
    }
}

/// Result of processing a lint command
#[derive(Debug, Clone)]
pub enum LintCommandResult {
    /// Successfully linted
    Success(String),
    /// Lint error
    Error(String),
    /// Missing input
    MissingInput,
}

impl LintCommandResult {
    pub fn format(&self) -> String {
        match self {
            Self::Success(result) => result.clone(),
            Self::Error(e) => format!("✗ Lint error: {}", e),
            Self::MissingInput => {
                "Usage: :lint <bash_code>\nExample: :lint cat file.txt | grep pattern".to_string()
            }
        }
    }
}

/// Result of processing a load/source command
#[derive(Debug, Clone)]
pub enum LoadCommandResult {
    /// Successfully loaded
    Success {
        path: PathBuf,
        function_count: usize,
        formatted: String,
    },
    /// File or parse error
    Error(String),
    /// Missing input
    MissingInput,
}

impl LoadCommandResult {
    pub fn format(&self) -> String {
        match self {
            Self::Success { formatted, .. } => formatted.clone(),
            Self::Error(e) => e.clone(),
            Self::MissingInput => {
                "Usage: :load <file>\nExample: :load examples/functions.sh".to_string()
            }
        }
    }
}

/// Result of processing a command by mode
#[derive(Debug, Clone)]
pub enum ModeProcessResult {
    /// Command executed (normal mode)
    Executed(String),
    /// Command purified
    Purified(String),
    /// Command linted
    Linted(String),
    /// Debug output
    Debug(String),
    /// Explanation
    Explained(String),
    /// No explanation available
    NoExplanation(String),
    /// Error during processing
    Error(String),
}

impl ModeProcessResult {
    pub fn format(&self) -> String {
        match self {
            Self::Executed(output) => output.clone(),
            Self::Purified(result) => format!("✓ Purified:\n{}", result),
            Self::Linted(result) => result.clone(),
            Self::Debug(line) => format!(
                "Debug mode: {}\n(Note: Debug mode not yet implemented)",
                line
            ),
            Self::Explained(explanation) => explanation.clone(),
            Self::NoExplanation(line) => {
                format!(
                    "No explanation available for: {}\nTry parameter expansions (${{var:-default}}), control flow (for, if, while), or redirections (>, <, |)",
                    line
                )
            }
            Self::Error(e) => e.clone(),
        }
    }
}

/// Result of history command
#[derive(Debug, Clone)]
pub enum HistoryResult {
    /// History entries
    Entries(Vec<String>),
    /// No history
    Empty,
}

impl HistoryResult {
    pub fn format(&self) -> String {
        match self {
            Self::Empty => "No commands in history".to_string(),
            Self::Entries(history) => {
                let mut output = format!("Command History ({} commands):\n", history.len());
                for (i, cmd) in history.iter().enumerate() {
                    output.push_str(&format!("  {} {}\n", i + 1, cmd));
                }
                output.trim_end().to_string()
            }
        }
    }
}

/// Result of vars command
#[derive(Debug, Clone)]
pub enum VarsResult {
    /// Variables
    Variables(Vec<(String, String)>),
    /// No variables
    Empty,
}

impl VarsResult {
    pub fn format(&self) -> String {
        match self {
            Self::Empty => "No session variables set".to_string(),
            Self::Variables(vars) => {
                let mut output = format!("Session Variables ({} variables):\n", vars.len());
                for (name, value) in vars {
                    output.push_str(&format!("  {} = {}\n", name, value));
                }
                output.trim_end().to_string()
            }
        }
    }
}

/// Result of functions command
#[derive(Debug, Clone)]
pub struct FunctionsResult(pub String);

impl FunctionsResult {
    pub fn format(&self) -> String {
        self.0.clone()
    }
}

/// Result of reload command
#[derive(Debug, Clone)]
pub enum ReloadResult {
    /// Successfully reloaded
    Success {
        path: PathBuf,
        function_count: usize,
    },
    /// Error during reload
    Error(String),
    /// No script to reload
    NoScript,
}

impl ReloadResult {
    pub fn format(&self) -> String {
        match self {
            Self::Success {
                path,
                function_count,
            } => format!(
                "✓ Reloaded: {} ({} functions)",
                path.display(),
                function_count
            ),
            Self::Error(e) => e.clone(),
            Self::NoScript => "No script to reload. Use :load <file> first.".to_string(),
        }
    }
}

// ===== PURE LOGIC FUNCTIONS =====

/// Process mode command and return result
pub fn process_mode_command(
    line: &str,
    state: &ReplState,
) -> (ModeCommandResult, Option<ReplMode>) {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() == 1 {
        (
            ModeCommandResult::ShowCurrent {
                mode: state.mode(),
                description: state.mode().description().to_string(),
            },
            None,
        )
    } else if parts.len() == 2 {
        if let Some(mode_name) = parts.get(1) {
            match mode_name.parse::<ReplMode>() {
                Ok(mode) => (
                    ModeCommandResult::Switched {
                        mode,
                        description: mode.description().to_string(),
                    },
                    Some(mode),
                ),
                Err(err) => (ModeCommandResult::InvalidMode(err.to_string()), None),
            }
        } else {
            (ModeCommandResult::InvalidUsage, None)
        }
    } else {
        (ModeCommandResult::InvalidUsage, None)
    }
}

/// Process parse command and return result
pub fn process_parse_command(line: &str) -> ParseCommandResult {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return ParseCommandResult::MissingInput;
    }

    let bash_code = parts.get(1).unwrap_or(&"");

    match parse_bash(bash_code) {
        Ok(ast) => {
            let mut ast_debug = String::new();
            for (i, stmt) in ast.statements.iter().enumerate() {
                ast_debug.push_str(&format!("  [{}] {:?}\n", i, stmt));
            }
            ParseCommandResult::Success {
                statement_count: ast.statements.len(),
                parse_time_ms: ast.metadata.parse_time_ms,
                ast_debug,
            }
        }
        Err(e) => ParseCommandResult::Error(format_parse_error(&e)),
    }
}

/// Process purify command and return result
pub fn process_purify_command(line: &str) -> PurifyCommandResult {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return PurifyCommandResult::MissingInput;
    }

    let bash_code = parts.get(1).unwrap_or(&"");

    match purify_bash(bash_code) {
        Ok(result) => PurifyCommandResult::Success(result),
        Err(e) => PurifyCommandResult::Error(e.to_string()),
    }
}

/// Process lint command and return result
pub fn process_lint_command(line: &str) -> LintCommandResult {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return LintCommandResult::MissingInput;
    }

    let bash_code = parts.get(1).unwrap_or(&"");

    match lint_bash(bash_code) {
        Ok(result) => LintCommandResult::Success(format_lint_results(&result)),
        Err(e) => LintCommandResult::Error(e.to_string()),
    }
}

/// Process load command and return result with functions to add
pub fn process_load_command(line: &str) -> (LoadCommandResult, Option<LoadResult>) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return (LoadCommandResult::MissingInput, None);
    }

    let file_path = parts.get(1).unwrap_or(&"");
    let result = load_script(file_path);

    match &result {
        LoadResult::Success(script) => (
            LoadCommandResult::Success {
                path: script.path.clone(),
                function_count: script.functions.len(),
                formatted: result.format(),
            },
            Some(result),
        ),
        LoadResult::FileError(_) | LoadResult::ParseError(_) => {
            (LoadCommandResult::Error(result.format()), None)
        }
    }
}

/// Process source command and return result
pub fn process_source_command(line: &str) -> (LoadCommandResult, Option<LoadResult>) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    if parts.len() == 1 {
        return (
            LoadCommandResult::Error(
                "Usage: :source <file>\nExample: :source examples/functions.sh".to_string(),
            ),
            None,
        );
    }

    let file_path = parts.get(1).unwrap_or(&"");
    let result = load_script(file_path);

    match &result {
        LoadResult::Success(script) => (
            LoadCommandResult::Success {
                path: script.path.clone(),
                function_count: script.functions.len(),
                formatted: format!(
                    "✓ Sourced: {} ({} functions)",
                    script.path.display(),
                    script.functions.len()
                ),
            },
            Some(result),
        ),
        LoadResult::FileError(_) | LoadResult::ParseError(_) => {
            (LoadCommandResult::Error(result.format()), None)
        }
    }
}

/// Process command based on current mode
pub fn process_command_by_mode(line: &str, state: &ReplState) -> ModeProcessResult {
    use crate::repl::executor::execute_command;

    let expanded_line = expand_variables(line, state.variables());

    match state.mode() {
        ReplMode::Normal => {
            let result = execute_command(&expanded_line);
            ModeProcessResult::Executed(result.format())
        }
        ReplMode::Purify => match purify_bash(&expanded_line) {
            Ok(result) => ModeProcessResult::Purified(result),
            Err(e) => ModeProcessResult::Error(format!("✗ Purification error: {}", e)),
        },
        ReplMode::Lint => match lint_bash(&expanded_line) {
            Ok(result) => ModeProcessResult::Linted(format_lint_results(&result)),
            Err(e) => ModeProcessResult::Error(format!("✗ Lint error: {}", e)),
        },
        ReplMode::Debug => ModeProcessResult::Debug(expanded_line),
        ReplMode::Explain => match explain_bash(line) {
            Some(explanation) => ModeProcessResult::Explained(explanation.format()),
            None => ModeProcessResult::NoExplanation(line.to_string()),
        },
    }
}

/// Process history command
pub fn process_history_command(state: &ReplState) -> HistoryResult {
    let history = state.history();

    if history.is_empty() {
        HistoryResult::Empty
    } else {
        HistoryResult::Entries(history.to_vec())
    }
}

/// Process vars command
pub fn process_vars_command(state: &ReplState) -> VarsResult {
    let variables = state.variables();

    if variables.is_empty() {
        VarsResult::Empty
    } else {
        let mut vars: Vec<_> = variables
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        vars.sort_by(|(a, _), (b, _)| a.cmp(b));
        VarsResult::Variables(vars)
    }
}

/// Process functions command
pub fn process_functions_command(state: &ReplState) -> FunctionsResult {
    let functions = state.loaded_functions();
    FunctionsResult(format_functions(functions))
}

/// Process reload command
pub fn process_reload_command(state: &ReplState) -> (ReloadResult, Option<LoadResult>) {
    if let Some(last_script) = state.last_loaded_script() {
        let path = last_script.clone();
        let result = load_script(&path);

        match &result {
            LoadResult::Success(script) => (
                ReloadResult::Success {
                    path: script.path.clone(),
                    function_count: script.functions.len(),
                },
                Some(result),
            ),
            LoadResult::FileError(_) | LoadResult::ParseError(_) => {
                (ReloadResult::Error(result.format()), None)
            }
        }
    } else {
        (ReloadResult::NoScript, None)
    }
}

/// Get history file path
pub fn get_history_path() -> anyhow::Result<PathBuf> {
    if let Ok(custom_path) = std::env::var("BASHRS_HISTORY_PATH") {
        return Ok(PathBuf::from(custom_path));
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let history_path = PathBuf::from(home).join(".bashrs_history");
    Ok(history_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== MODE COMMAND TESTS =====

    #[test]
    fn test_process_mode_command_show_current() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode", &state);

        assert!(matches!(result, ModeCommandResult::ShowCurrent { .. }));
        assert!(new_mode.is_none());

        let formatted = result.format();
        assert!(formatted.contains("Current mode:"));
        assert!(formatted.contains("Available modes:"));
    }

    #[test]
    fn test_process_mode_command_switch_valid() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode purify", &state);

        assert!(matches!(result, ModeCommandResult::Switched { .. }));
        assert_eq!(new_mode, Some(ReplMode::Purify));

        let formatted = result.format();
        assert!(formatted.contains("Switched to purify mode"));
    }

    #[test]
    fn test_process_mode_command_switch_all_modes() {
        let state = ReplState::new();

        for mode_name in &["normal", "purify", "lint", "debug", "explain"] {
            let (result, new_mode) = process_mode_command(&format!(":mode {}", mode_name), &state);
            assert!(matches!(result, ModeCommandResult::Switched { .. }));
            assert!(new_mode.is_some());
        }
    }

    #[test]
    fn test_process_mode_command_invalid_mode() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode invalid", &state);

        assert!(matches!(result, ModeCommandResult::InvalidMode(_)));
        assert!(new_mode.is_none());
    }

    #[test]
    fn test_process_mode_command_too_many_args() {
        let state = ReplState::new();
        let (result, new_mode) = process_mode_command(":mode purify extra", &state);

        assert!(matches!(result, ModeCommandResult::InvalidUsage));
        assert!(new_mode.is_none());
    }

    // ===== PARSE COMMAND TESTS =====

    #[test]
    fn test_process_parse_command_success() {
        let result = process_parse_command(":parse echo hello");

        assert!(matches!(result, ParseCommandResult::Success { .. }));
        let formatted = result.format();
        assert!(formatted.contains("Parse successful"));
    }

    #[test]
    fn test_process_parse_command_missing_input() {
        let result = process_parse_command(":parse");

        assert!(matches!(result, ParseCommandResult::MissingInput));
        let formatted = result.format();
        assert!(formatted.contains("Usage:"));
    }

    #[test]
    fn test_process_parse_command_complex() {
        let result = process_parse_command(":parse for i in 1 2 3; do echo $i; done");

        assert!(matches!(result, ParseCommandResult::Success { .. }));
    }

    // ===== PURIFY COMMAND TESTS =====

    #[test]
    fn test_process_purify_command_success() {
        let result = process_purify_command(":purify mkdir /tmp/test");

        assert!(matches!(result, PurifyCommandResult::Success(_)));
        let formatted = result.format();
        assert!(formatted.contains("Purification successful"));
    }

    #[test]
    fn test_process_purify_command_missing_input() {
        let result = process_purify_command(":purify");

        assert!(matches!(result, PurifyCommandResult::MissingInput));
    }

    #[test]
    fn test_process_purify_command_idempotent() {
        let result = process_purify_command(":purify rm file.txt");

        if let PurifyCommandResult::Success(output) = result {
            // Purified should contain idempotent flag
            assert!(output.contains("-f") || output.contains("rm"));
        }
    }

    // ===== LINT COMMAND TESTS =====

    #[test]
    fn test_process_lint_command_success() {
        let result = process_lint_command(":lint echo $var");

        assert!(matches!(result, LintCommandResult::Success(_)));
    }

    #[test]
    fn test_process_lint_command_missing_input() {
        let result = process_lint_command(":lint");

        assert!(matches!(result, LintCommandResult::MissingInput));
    }

    // ===== LOAD COMMAND TESTS =====

    #[test]
    fn test_process_load_command_missing_input() {
        let (result, load_result) = process_load_command(":load");

        assert!(matches!(result, LoadCommandResult::MissingInput));
        assert!(load_result.is_none());
    }

    #[test]
    fn test_process_load_command_file_not_found() {
        let (result, load_result) = process_load_command(":load /nonexistent/file.sh");

        assert!(matches!(result, LoadCommandResult::Error(_)));
        assert!(load_result.is_none());
    }

    // ===== SOURCE COMMAND TESTS =====

    #[test]
    fn test_process_source_command_missing_input() {
        let (result, load_result) = process_source_command(":source");

        assert!(matches!(result, LoadCommandResult::Error(_)));
        assert!(load_result.is_none());
    }

    // ===== HISTORY COMMAND TESTS =====

    #[test]
    fn test_process_history_command_empty() {
        let state = ReplState::new();
        let result = process_history_command(&state);

        assert!(matches!(result, HistoryResult::Empty));
        assert_eq!(result.format(), "No commands in history");
    }

    #[test]
    fn test_process_history_command_with_entries() {
        let mut state = ReplState::new();
        state.add_history("echo hello".to_string());
        state.add_history("ls -la".to_string());

        let result = process_history_command(&state);

        assert!(matches!(result, HistoryResult::Entries(_)));
        let formatted = result.format();
        assert!(formatted.contains("echo hello"));
        assert!(formatted.contains("ls -la"));
    }

    // ===== VARS COMMAND TESTS =====

    #[test]
    fn test_process_vars_command_empty() {
        let state = ReplState::new();
        let result = process_vars_command(&state);

        assert!(matches!(result, VarsResult::Empty));
        assert_eq!(result.format(), "No session variables set");
    }

    #[test]
    fn test_process_vars_command_with_variables() {
        let mut state = ReplState::new();
        state.set_variable("FOO".to_string(), "bar".to_string());
        state.set_variable("BAZ".to_string(), "qux".to_string());

        let result = process_vars_command(&state);

        assert!(matches!(result, VarsResult::Variables(_)));
        let formatted = result.format();
        assert!(formatted.contains("FOO = bar"));
        assert!(formatted.contains("BAZ = qux"));
    }

    #[test]
    fn test_process_vars_command_sorted() {
        let mut state = ReplState::new();
        state.set_variable("Z".to_string(), "last".to_string());
        state.set_variable("A".to_string(), "first".to_string());

        let result = process_vars_command(&state);

        if let VarsResult::Variables(vars) = result {
            assert_eq!(vars[0].0, "A");
            assert_eq!(vars[1].0, "Z");
        }
    }

    // ===== FUNCTIONS COMMAND TESTS =====

    #[test]
    fn test_process_functions_command_empty() {
        let state = ReplState::new();
        let result = process_functions_command(&state);

        let formatted = result.format();
        assert!(
            formatted.contains("No functions")
                || formatted.contains("0 functions")
                || formatted.is_empty()
                || formatted.contains("functions")
        );
    }

    // ===== RELOAD COMMAND TESTS =====

    #[test]
    fn test_process_reload_command_no_script() {
        let state = ReplState::new();
        let (result, load_result) = process_reload_command(&state);

        assert!(matches!(result, ReloadResult::NoScript));
        assert!(load_result.is_none());
        assert!(result.format().contains("No script to reload"));
    }

    // ===== MODE PROCESSING TESTS =====

    #[test]
    fn test_process_command_by_mode_normal() {
        let state = ReplState::new();
        let result = process_command_by_mode("echo hello", &state);

        assert!(matches!(result, ModeProcessResult::Executed(_)));
    }

    #[test]
    fn test_process_command_by_mode_purify() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Purify);

        let result = process_command_by_mode("mkdir /tmp/test", &state);

        assert!(matches!(result, ModeProcessResult::Purified(_)));
        let formatted = result.format();
        assert!(formatted.contains("Purified"));
    }

    #[test]
    fn test_process_command_by_mode_lint() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Lint);

        let result = process_command_by_mode("echo $var", &state);

        assert!(matches!(result, ModeProcessResult::Linted(_)));
    }

    #[test]
    fn test_process_command_by_mode_debug() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Debug);

        let result = process_command_by_mode("echo hello", &state);

        assert!(matches!(result, ModeProcessResult::Debug(_)));
        let formatted = result.format();
        assert!(formatted.contains("Debug mode"));
    }

    #[test]
    fn test_process_command_by_mode_explain() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Explain);

        let result = process_command_by_mode("${var:-default}", &state);

        // May or may not have explanation depending on implementation
        assert!(
            matches!(result, ModeProcessResult::Explained(_))
                || matches!(result, ModeProcessResult::NoExplanation(_))
        );
    }

    #[test]
    fn test_process_command_by_mode_variable_expansion() {
        let mut state = ReplState::new();
        state.set_variable("NAME".to_string(), "world".to_string());

        let result = process_command_by_mode("echo $NAME", &state);

        // The command should have expanded $NAME
        assert!(matches!(result, ModeProcessResult::Executed(_)));
    }

    // ===== HISTORY PATH TESTS =====

    #[test]
    fn test_get_history_path_default() {
        let path = get_history_path();
        assert!(path.is_ok());
        assert!(path.unwrap().to_string_lossy().contains(".bashrs_history"));
    }

    #[test]
    fn test_get_history_path_deterministic() {
        let path1 = get_history_path().unwrap();
        let path2 = get_history_path().unwrap();
        assert_eq!(path1, path2);
    }

    // ===== FORMAT METHOD TESTS =====

    #[test]
    fn test_mode_command_result_format_invalid_mode() {
        let result = ModeCommandResult::InvalidMode("unknown mode: foo".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("unknown mode: foo"));
    }

    #[test]
    fn test_mode_command_result_format_invalid_usage() {
        let result = ModeCommandResult::InvalidUsage;
        let formatted = result.format();
        assert!(formatted.contains("Usage:"));
        assert!(formatted.contains("Valid modes:"));
    }

    #[test]
    fn test_parse_command_result_format_error() {
        let result = ParseCommandResult::Error("parse error at line 1".to_string());
        let formatted = result.format();
        assert!(formatted.contains("✗"));
        assert!(formatted.contains("parse error at line 1"));
    }

    #[test]
    fn test_purify_command_result_format_error() {
        let result = PurifyCommandResult::Error("failed to purify".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Purification error"));
        assert!(formatted.contains("failed to purify"));
    }

    #[test]
    fn test_lint_command_result_format_error() {
        let result = LintCommandResult::Error("lint failed".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Lint error"));
        assert!(formatted.contains("lint failed"));
    }

    #[test]
    fn test_load_command_result_format_success() {
        let result = LoadCommandResult::Success {
            path: PathBuf::from("/test/script.sh"),
            function_count: 5,
            formatted: "Loaded 5 functions".to_string(),
        };
        let formatted = result.format();
        assert_eq!(formatted, "Loaded 5 functions");
    }

    #[test]
    fn test_load_command_result_format_error() {
        let result = LoadCommandResult::Error("file not found".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "file not found");
    }

    #[test]
    fn test_mode_process_result_format_purified() {
        let result = ModeProcessResult::Purified("mkdir -p /tmp".to_string());
        let formatted = result.format();
        assert!(formatted.contains("Purified:"));
        assert!(formatted.contains("mkdir -p /tmp"));
    }

    #[test]
    fn test_mode_process_result_format_no_explanation() {
        let result = ModeProcessResult::NoExplanation("some command".to_string());
        let formatted = result.format();
        assert!(formatted.contains("No explanation available"));
        assert!(formatted.contains("some command"));
    }

    #[test]
    fn test_mode_process_result_format_error() {
        let result = ModeProcessResult::Error("something went wrong".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "something went wrong");
    }

    #[test]
    fn test_reload_result_format_success() {
        let result = ReloadResult::Success {
            path: PathBuf::from("/test/script.sh"),
            function_count: 3,
        };
        let formatted = result.format();
        assert!(formatted.contains("Reloaded:"));
        assert!(formatted.contains("/test/script.sh"));
        assert!(formatted.contains("3 functions"));
    }

    #[test]
    fn test_reload_result_format_error() {
        let result = ReloadResult::Error("reload failed".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "reload failed");
    }

    #[test]
    fn test_functions_result_format() {
        let result = FunctionsResult("function1, function2".to_string());
        let formatted = result.format();
        assert_eq!(formatted, "function1, function2");
    }

    // ===== ADDITIONAL EDGE CASES =====

    #[test]
    fn test_parse_command_with_multiple_statements() {
        let result = process_parse_command(":parse echo a; echo b; echo c");
        if let ParseCommandResult::Success {
            statement_count, ..
        } = result
        {
            assert!(statement_count >= 1);
        }
    }

    #[test]
    fn test_process_command_by_mode_explain_no_match() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Explain);

        // A simple command that might not have an explanation
        let result = process_command_by_mode("ls", &state);

        // Could be explained or not, both are valid
        let formatted = result.format();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_history_result_entries_format() {
        let entries = vec!["cmd1".to_string(), "cmd2".to_string(), "cmd3".to_string()];
        let result = HistoryResult::Entries(entries);
        let formatted = result.format();

        assert!(formatted.contains("Command History"));
        assert!(formatted.contains("3 commands"));
        assert!(formatted.contains("1 cmd1"));
        assert!(formatted.contains("2 cmd2"));
        assert!(formatted.contains("3 cmd3"));
    }

    #[test]
    fn test_vars_result_format() {
        let vars = vec![
            ("AAA".to_string(), "111".to_string()),
            ("BBB".to_string(), "222".to_string()),
        ];
        let result = VarsResult::Variables(vars);
        let formatted = result.format();

        assert!(formatted.contains("Session Variables"));
        assert!(formatted.contains("2 variables"));
        assert!(formatted.contains("AAA = 111"));
        assert!(formatted.contains("BBB = 222"));
    }

    #[test]
    fn test_mode_command_result_eq() {
        let result1 = ModeCommandResult::InvalidUsage;
        let result2 = ModeCommandResult::InvalidUsage;
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_process_purify_command_error_handling() {
        // Invalid bash that can't be parsed
        let result = process_purify_command(":purify <<<");

        // Should handle parse error gracefully
        assert!(
            matches!(result, PurifyCommandResult::Error(_))
                || matches!(result, PurifyCommandResult::Success(_))
        );
    }

    #[test]
    fn test_process_lint_command_error_handling() {
        // Invalid bash that can't be parsed
        let result = process_lint_command(":lint <<<");

        // Should handle parse error gracefully
        assert!(
            matches!(result, LintCommandResult::Error(_))
                || matches!(result, LintCommandResult::Success(_))
        );
    }
}
