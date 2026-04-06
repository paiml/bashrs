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
use rustyline::history::DefaultHistory;
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
        let prompt = if multiline_buffer.is_empty() {
            format!("bashrs [{}]> ", state.mode())
        } else {
            "... > ".to_string()
        };

        match editor.readline(&prompt) {
            Ok(line) => {
                match process_repl_line(&line, &mut multiline_buffer, &mut state, &mut editor) {
                    LineAction::Continue => continue,
                    LineAction::Break => break,
                    LineAction::Next => {}
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                handle_interrupt(&mut multiline_buffer);
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
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

/// Result of processing a single REPL input line
enum LineAction {
    /// Continue to next iteration (skip remaining loop body)
    Continue,
    /// Break out of the REPL loop
    Break,
    /// Proceed normally (no special action)
    Next,
}

/// Process a single line of REPL input, handling multiline accumulation
fn process_repl_line(
    line: &str,
    multiline_buffer: &mut String,
    state: &mut ReplState,
    editor: &mut Editor<ReplCompleter, DefaultHistory>,
) -> LineAction {
    let trimmed_line = line.trim();

    // Handle empty input
    if trimmed_line.is_empty() {
        if !multiline_buffer.is_empty() {
            multiline_buffer.push('\n');
        }
        return LineAction::Continue;
    }

    // Accumulate multi-line input
    if !multiline_buffer.is_empty() {
        multiline_buffer.push('\n');
    }
    multiline_buffer.push_str(line);

    // Check if input is incomplete and needs continuation
    if is_incomplete(multiline_buffer) {
        return LineAction::Continue;
    }

    // Input is complete - process it
    let complete_input = multiline_buffer.clone();
    multiline_buffer.clear();

    // Add to history
    let _ = editor.add_history_entry(&complete_input);
    state.add_history(complete_input.clone());

    let trimmed = complete_input.trim();

    // Handle variable assignments (before other commands)
    if let Some((name, value)) = parse_assignment(trimmed) {
        state.set_variable(name.clone(), value.clone());
        println!("\u{2713} Variable set: {} = {}", name, value);
        return LineAction::Continue;
    }

    // Dispatch to command handler
    if dispatch_repl_command(trimmed, state) {
        return LineAction::Break;
    }

    LineAction::Next
}

/// Handle Ctrl-C interrupt in the REPL
fn handle_interrupt(multiline_buffer: &mut String) {
    if !multiline_buffer.is_empty() {
        println!("^C (multi-line input cancelled)");
        multiline_buffer.clear();
    } else {
        println!("^C");
    }
}

/// Dispatch a REPL command to the appropriate handler.
/// Returns `true` if the REPL should exit (quit/exit command).
fn dispatch_repl_command(line: &str, state: &mut ReplState) -> bool {
    // Handle colon commands
    if line.starts_with(':') {
        dispatch_colon_command(line, state);
        return false;
    }

    // Handle quit/exit
    if line == "quit" || line == "exit" {
        println!("Goodbye!");
        return true;
    }

    // Handle help
    if line == "help" || line.starts_with("help ") || line.starts_with(":help") {
        print!("{}", show_help(extract_help_topic(line)));
        return false;
    }

    // Default: process by current mode
    handle_command_by_mode(line, state);
    false
}

/// Dispatch colon-prefixed REPL commands (:mode, :parse, :purify, etc.)
fn dispatch_colon_command(line: &str, state: &mut ReplState) {
    // Extract the command name (first word after ':')
    let cmd = line.split_whitespace().next().unwrap_or("");
    match cmd {
        ":mode" => handle_mode_command(line, state),
        ":parse" => handle_parse_command(line),
        ":purify" => handle_purify_command(line),
        ":lint" => handle_lint_command(line),
        ":load" => handle_load_command(line, state),
        ":source" => handle_source_command(line, state),
        ":functions" => handle_functions_command(state),
        ":reload" => handle_reload_command(state),
        ":history" => handle_history_command(state),
        ":vars" => handle_vars_command(state),
        ":clear" => handle_clear_command(),
        ":help" => print!("{}", show_help(extract_help_topic(line))),
        _ => println!(
            "Unknown command: {}. Type 'help' for available commands.",
            cmd
        ),
    }
}

/// Extract the help topic from a help command line
fn extract_help_topic(line: &str) -> Option<&str> {
    if line.contains(' ') {
        let parts: Vec<&str> = line.split_whitespace().collect();
        parts.get(1).copied()
    } else {
        None
    }
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

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "loop_tests_repl_003.rs"]
// FIXME(PMAT-238): mod tests_extracted;
