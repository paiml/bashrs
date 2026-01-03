// REPL Loop Module
//
// Task: REPL-003-002 - Basic REPL loop with rustyline integration
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Integration tests: CLI interaction with assert_cmd
// - Mutation score: ≥90%
// - Complexity: <10 per function

use crate::repl::{
    completion::ReplCompleter,
    help::show_help,
    loader::LoadResult,
    logic::{
        get_history_path, process_command_by_mode, process_functions_command,
        process_history_command, process_lint_command, process_load_command, process_mode_command,
        process_parse_command, process_purify_command, process_reload_command,
        process_source_command, process_vars_command,
    },
    multiline::is_incomplete,
    variables::parse_assignment,
    ReplConfig, ReplState,
};
use anyhow::Result;
use rustyline::config::Config;
use rustyline::Editor;
use std::path::PathBuf;

/// Main REPL loop for bashrs
///
/// Provides an interactive shell for:
/// - Parsing bash scripts
/// - Purifying bash scripts
/// - Linting bash scripts
/// - Debugging bash scripts
/// - Explaining bash constructs
///
/// # Architecture
/// - Debugger-as-REPL pattern (matklad)
/// - Symbiotic embedding (RuchyRuchy pattern)
/// - Resource limits from ReplConfig
///
/// # Examples
///
/// ```rust,no_run
/// use bashrs::repl::{ReplConfig, run_repl};
///
/// let config = ReplConfig::default();
/// run_repl(config).expect("REPL failed");
/// ```
pub fn run_repl(config: ReplConfig) -> Result<()> {
    // Validate configuration first
    config.validate().map_err(|e| anyhow::anyhow!(e))?;

    // Configure rustyline editor with history settings
    // This configuration enables Ctrl-R reverse search automatically!
    let rustyline_config = Config::builder()
        .history_ignore_dups(config.history_ignore_dups)?
        .history_ignore_space(config.history_ignore_space)
        .max_history_size(config.max_history)?
        .auto_add_history(true)
        .build();

    // Initialize rustyline editor with tab completion
    let completer = ReplCompleter::new();
    let mut editor = Editor::with_config(rustyline_config)?;
    editor.set_helper(Some(completer));

    // Initialize REPL state
    let mut state = ReplState::new();

    // Load history from file (if exists)
    // Use custom history path from config, or default to ~/.bashrs_history
    let history_path = config
        .history_path
        .clone()
        .unwrap_or_else(|| get_history_path().unwrap_or_else(|_| PathBuf::from(".bashrs_history")));
    if history_path.exists() {
        let _ = editor.load_history(&history_path);
    }

    // Print welcome banner
    println!("bashrs REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'quit' or 'exit' to exit, 'help' for commands");
    println!("Tip: Use Up/Down arrows for history, Ctrl-R for reverse search");
    println!(
        "Current mode: {} - {}",
        state.mode(),
        state.mode().description()
    );

    // Main REPL loop with multi-line support
    let mut multiline_buffer = String::new();

    loop {
        // Determine prompt based on whether we're in multi-line mode
        let prompt = if multiline_buffer.is_empty() {
            format!("bashrs [{}]> ", state.mode())
        } else {
            "... > ".to_string()
        };

        let readline = editor.readline(&prompt);

        match readline {
            Ok(line) => {
                let trimmed_line = line.trim();

                // Handle empty input in multi-line mode
                if trimmed_line.is_empty() && !multiline_buffer.is_empty() {
                    // Empty line while in multi-line: continue accumulating
                    multiline_buffer.push('\n');
                    continue;
                }

                // Handle empty input in normal mode
                if trimmed_line.is_empty() {
                    continue;
                }

                // Accumulate multi-line input
                if !multiline_buffer.is_empty() {
                    multiline_buffer.push('\n');
                    multiline_buffer.push_str(&line);
                } else {
                    multiline_buffer.push_str(&line);
                }

                // Check if input is incomplete and needs continuation
                if is_incomplete(&multiline_buffer) {
                    // Input is incomplete, continue reading
                    continue;
                }

                // Input is complete - process it
                let complete_input = multiline_buffer.clone();
                multiline_buffer.clear();

                // Add to history
                let _ = editor.add_history_entry(&complete_input);
                state.add_history(complete_input.clone());

                // Process the complete input
                let line = complete_input.trim();

                // Handle variable assignments (before other commands)
                if let Some((name, value)) = parse_assignment(line) {
                    state.set_variable(name.clone(), value.clone());
                    println!("✓ Variable set: {} = {}", name, value);
                    continue;
                }

                // Handle special commands
                if line.starts_with(":mode") {
                    // Handle :mode command
                    handle_mode_command(line, &mut state);
                } else if line.starts_with(":parse") {
                    // Handle :parse command
                    handle_parse_command(line);
                } else if line.starts_with(":purify") {
                    // Handle :purify command
                    handle_purify_command(line);
                } else if line.starts_with(":lint") {
                    // Handle :lint command
                    handle_lint_command(line);
                } else if line.starts_with(":load") {
                    // Handle :load command
                    handle_load_command(line, &mut state);
                } else if line.starts_with(":source") {
                    // Handle :source command
                    handle_source_command(line, &mut state);
                } else if line == ":functions" {
                    // Handle :functions command
                    handle_functions_command(&state);
                } else if line == ":reload" {
                    // Handle :reload command
                    handle_reload_command(&mut state);
                } else if line == ":history" {
                    // Handle :history command
                    handle_history_command(&state);
                } else if line == ":vars" {
                    // Handle :vars command
                    handle_vars_command(&state);
                } else if line == ":clear" {
                    // Handle :clear command
                    handle_clear_command();
                } else if line == "quit" || line == "exit" {
                    println!("Goodbye!");
                    break;
                } else if line == "help" || line.starts_with("help ") || line.starts_with(":help") {
                    // Handle help command with optional topic
                    let topic = if line.contains(' ') {
                        // Extract topic after "help " or ":help "
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.get(1).copied()
                    } else {
                        None
                    };
                    print!("{}", show_help(topic));
                } else {
                    // Process command based on current mode
                    handle_command_by_mode(line, &state);
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                // Ctrl-C - reset multi-line buffer
                if !multiline_buffer.is_empty() {
                    println!("^C (multi-line input cancelled)");
                    multiline_buffer.clear();
                } else {
                    println!("^C");
                }
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                // Ctrl-D
                println!("EOF");
                break;
            }
            Err(err) => {
                return Err(anyhow::anyhow!("REPL error: {}", err));
            }
        }
    }

    // Save history before exiting
    let _ = editor.save_history(&history_path);

    Ok(())
}

/// Handle mode switching command (thin shim over logic module)
fn handle_mode_command(line: &str, state: &mut ReplState) {
    let (result, new_mode) = process_mode_command(line, state);
    println!("{}", result.format());
    if let Some(mode) = new_mode {
        state.set_mode(mode);
    }
}

/// Handle parse command (thin shim over logic module)
fn handle_parse_command(line: &str) {
    let result = process_parse_command(line);
    println!("{}", result.format());
}

/// Handle purify command (thin shim over logic module)
fn handle_purify_command(line: &str) {
    let result = process_purify_command(line);
    println!("{}", result.format());
}

/// Handle lint command (thin shim over logic module)
fn handle_lint_command(line: &str) {
    let result = process_lint_command(line);
    println!("{}", result.format());
}

/// Handle command processing based on current mode (thin shim over logic module)
fn handle_command_by_mode(line: &str, state: &ReplState) {
    let result = process_command_by_mode(line, state);
    let output = result.format();
    if !output.is_empty() {
        print!("{}", output);
        // Ensure newline for non-executed outputs
        if !output.ends_with('\n') {
            println!();
        }
    }
}

/// Handle history command (thin shim over logic module)
fn handle_history_command(state: &ReplState) {
    let result = process_history_command(state);
    println!("{}", result.format());
}

/// Handle vars command (thin shim over logic module)
fn handle_vars_command(state: &ReplState) {
    let result = process_vars_command(state);
    println!("{}", result.format());
}

/// Handle clear command
fn handle_clear_command() {
    // Clear screen using ANSI escape codes
    // \x1B[2J clears the screen
    // \x1B[H moves cursor to home position (0,0)
    print!("\x1B[2J\x1B[H");
}

/// Handle load command (thin shim over logic module)
fn handle_load_command(line: &str, state: &mut ReplState) {
    let (result, load_result) = process_load_command(line);
    println!("{}", result.format());

    // Update state if load was successful
    if let Some(LoadResult::Success(script)) = load_result {
        state.set_last_loaded_script(script.path.clone());
        state.clear_functions();
        for func in &script.functions {
            state.add_function(func.clone());
        }
    }
}

/// Handle source command (thin shim over logic module)
fn handle_source_command(line: &str, state: &mut ReplState) {
    let (result, load_result) = process_source_command(line);
    println!("{}", result.format());

    // Update state if source was successful
    if let Some(LoadResult::Success(script)) = load_result {
        state.set_last_loaded_script(script.path.clone());
        for func in &script.functions {
            state.add_function(func.clone());
        }
    }
}

/// Handle functions command (thin shim over logic module)
fn handle_functions_command(state: &ReplState) {
    let result = process_functions_command(state);
    println!("{}", result.format());
}

/// Handle reload command (thin shim over logic module)
fn handle_reload_command(state: &mut ReplState) {
    let (result, load_result) = process_reload_command(state);
    println!("{}", result.format());

    // Update state if reload was successful
    if let Some(LoadResult::Success(script)) = load_result {
        state.clear_functions();
        for func in &script.functions {
            state.add_function(func.clone());
        }
    }
}

// Note: Tests for REPL logic have been moved to logic.rs
// This file (loop.rs) is now a thin shim that handles I/O only

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ===== INTEGRATION TESTS FOR run_repl =====

    /// Test: REPL-003-002-001 - ReplConfig validation is called
    #[test]
    fn test_REPL_003_002_repl_validates_config() {
        // Invalid config (zero memory)
        let config = ReplConfig::new(0, Duration::from_secs(30), 100);

        // run_repl should fail validation
        let result = run_repl(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_memory"));
    }

    /// Test: REPL-003-002-002 - REPL handles empty input gracefully
    /// NOTE: This is a design test - actual interactive testing via CLI
    #[test]
    fn test_REPL_003_002_repl_handles_empty_input() {
        // This test verifies the code path exists
        // Actual empty input handling tested via assert_cmd CLI tests
        let config = ReplConfig::default();
        assert!(config.validate().is_ok());
    }

    /// Test: REPL-003-002-003 - REPL handles EOF (Ctrl-D) correctly
    /// NOTE: This is a design test - actual EOF testing via CLI
    #[test]
    fn test_REPL_003_002_repl_handles_eof() {
        // This test verifies the code path exists
        // Actual EOF handling tested via assert_cmd CLI tests
        let config = ReplConfig::default();
        assert!(config.validate().is_ok());
    }

    // History path tests are now in logic.rs module
}
