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


include!("logic_part2_incl2.rs");
