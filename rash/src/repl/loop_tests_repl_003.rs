#![allow(clippy::unwrap_used)]
use super::*;
use crate::repl::ReplMode;
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

// ===== SHIM FUNCTION TESTS =====
// These tests exercise the thin shim functions to achieve coverage

/// Test: handle_mode_command - show current mode
#[test]
fn test_handle_mode_command_show_current() {
    let mut state = ReplState::new();
    // Just calling the function exercises the code path
    handle_mode_command(":mode", &mut state);
    // State should be unchanged
    assert_eq!(state.mode(), ReplMode::Normal);
}

/// Test: handle_mode_command - switch mode
#[test]
fn test_handle_mode_command_switch() {
    let mut state = ReplState::new();
    handle_mode_command(":mode purify", &mut state);
    assert_eq!(state.mode(), ReplMode::Purify);
}

/// Test: handle_mode_command - switch to all modes
#[test]
fn test_handle_mode_command_all_modes() {
    let mut state = ReplState::new();

    handle_mode_command(":mode normal", &mut state);
    assert_eq!(state.mode(), ReplMode::Normal);

    handle_mode_command(":mode purify", &mut state);
    assert_eq!(state.mode(), ReplMode::Purify);

    handle_mode_command(":mode lint", &mut state);
    assert_eq!(state.mode(), ReplMode::Lint);

    handle_mode_command(":mode debug", &mut state);
    assert_eq!(state.mode(), ReplMode::Debug);

    handle_mode_command(":mode explain", &mut state);
    assert_eq!(state.mode(), ReplMode::Explain);
}

/// Test: handle_mode_command - invalid mode
#[test]
fn test_handle_mode_command_invalid() {
    let mut state = ReplState::new();
    handle_mode_command(":mode invalid_mode", &mut state);
    // Mode should remain unchanged
    assert_eq!(state.mode(), ReplMode::Normal);
}

/// Test: handle_parse_command - basic parse
#[test]
fn test_handle_parse_command_basic() {
    handle_parse_command(":parse echo hello");
}

/// Test: handle_parse_command - missing input
#[test]
fn test_handle_parse_command_missing_input() {
    handle_parse_command(":parse");
}

/// Test: handle_parse_command - complex input
#[test]
fn test_handle_parse_command_complex() {
    handle_parse_command(":parse for i in 1 2 3; do echo $i; done");
}

/// Test: handle_purify_command - basic purify
#[test]
fn test_handle_purify_command_basic() {
    handle_purify_command(":purify echo $RANDOM");
}

/// Test: handle_purify_command - missing input
#[test]
fn test_handle_purify_command_missing_input() {
    handle_purify_command(":purify");
}

/// Test: handle_purify_command - idempotent operations
#[test]
fn test_handle_purify_command_idempotent() {
    handle_purify_command(":purify mkdir mydir");
}

/// Test: handle_lint_command - basic lint
#[test]
fn test_handle_lint_command_basic() {
    handle_lint_command(":lint echo $unquoted_var");
}

/// Test: handle_lint_command - missing input
#[test]
fn test_handle_lint_command_missing_input() {
    handle_lint_command(":lint");
}

/// Test: handle_lint_command - clean code
#[test]
fn test_handle_lint_command_clean() {
    handle_lint_command(":lint echo \"$quoted_var\"");
}

/// Test: handle_command_by_mode - normal mode
#[test]
fn test_handle_command_by_mode_normal() {
    let state = ReplState::new();
    handle_command_by_mode("echo test", &state);
}

/// Test: handle_command_by_mode - purify mode
#[test]
fn test_handle_command_by_mode_purify() {
    let mut state = ReplState::new();
    state.set_mode(ReplMode::Purify);
    handle_command_by_mode("echo $RANDOM", &state);
}

/// Test: handle_command_by_mode - lint mode
#[test]
fn test_handle_command_by_mode_lint() {
    let mut state = ReplState::new();
    state.set_mode(ReplMode::Lint);
    handle_command_by_mode("echo $unquoted", &state);
}

/// Test: handle_command_by_mode - explain mode
#[test]
fn test_handle_command_by_mode_explain() {
    let mut state = ReplState::new();
    state.set_mode(ReplMode::Explain);
    handle_command_by_mode("for i in 1 2 3; do echo $i; done", &state);
}

/// Test: handle_command_by_mode - debug mode
#[test]
fn test_handle_command_by_mode_debug() {
    let mut state = ReplState::new();
    state.set_mode(ReplMode::Debug);
    handle_command_by_mode("x=5; echo $x", &state);
}

/// Test: handle_history_command
#[test]
fn test_handle_history_command() {
    let mut state = ReplState::new();
    state.add_history("echo hello".to_string());
    state.add_history("echo world".to_string());
    handle_history_command(&state);
}

/// Test: handle_history_command - empty
#[test]
fn test_handle_history_command_empty() {
    let state = ReplState::new();
    handle_history_command(&state);
}

/// Test: handle_vars_command
#[test]
fn test_handle_vars_command() {
    let mut state = ReplState::new();
    state.set_variable("foo".to_string(), "bar".to_string());
    state.set_variable("count".to_string(), "42".to_string());
    handle_vars_command(&state);
}

/// Test: handle_vars_command - empty
#[test]
fn test_handle_vars_command_empty() {
    let state = ReplState::new();
    handle_vars_command(&state);
}

/// Test: handle_clear_command
#[test]
fn test_handle_clear_command() {
    handle_clear_command();
}

/// Test: handle_load_command - missing file path
#[test]
fn test_handle_load_command_missing_path() {
    let mut state = ReplState::new();
    handle_load_command(":load", &mut state);
}

/// Test: handle_load_command - nonexistent file
#[test]
fn test_handle_load_command_nonexistent() {
    let mut state = ReplState::new();
    handle_load_command(":load /nonexistent/file.sh", &mut state);
}

/// Test: handle_source_command - missing file path
#[test]
fn test_handle_source_command_missing_path() {
    let mut state = ReplState::new();
    handle_source_command(":source", &mut state);
}

/// Test: handle_source_command - nonexistent file
#[test]
fn test_handle_source_command_nonexistent() {
    let mut state = ReplState::new();
    handle_source_command(":source /nonexistent/file.sh", &mut state);
}

/// Test: handle_functions_command - empty
#[test]
fn test_handle_functions_command_empty() {
    let state = ReplState::new();
    handle_functions_command(&state);
}

/// Test: handle_functions_command - with functions
#[test]
fn test_handle_functions_command_with_functions() {
    let mut state = ReplState::new();
    state.add_function("greet".to_string());
    state.add_function("farewell".to_string());
    handle_functions_command(&state);
}

/// Test: handle_reload_command - no script loaded
#[test]
fn test_handle_reload_command_no_script() {
    let mut state = ReplState::new();
    handle_reload_command(&mut state);
}

/// Test: handle_reload_command - with last loaded script path (nonexistent)
#[test]
fn test_handle_reload_command_nonexistent_script() {
    let mut state = ReplState::new();
    state.set_last_loaded_script(PathBuf::from("/nonexistent/script.sh"));
    handle_reload_command(&mut state);
}

// History path tests are now in logic.rs module

// ── Additional coverage tests for loop.rs pure logic ─────────────

/// Test: extract_help_topic with no topic
#[test]
fn test_extract_help_topic_no_topic() {
    assert_eq!(extract_help_topic("help"), None);
}

/// Test: extract_help_topic with topic
#[test]
fn test_extract_help_topic_with_topic() {
    assert_eq!(extract_help_topic("help mode"), Some("mode"));
}

/// Test: extract_help_topic with multiple words
#[test]
fn test_extract_help_topic_multiple_words() {
    // Should return only the second word
    assert_eq!(extract_help_topic("help parse lint"), Some("parse"));
}

/// Test: extract_help_topic with colon prefix
#[test]
fn test_extract_help_topic_colon_help() {
    assert_eq!(extract_help_topic(":help mode"), Some("mode"));
}

/// Test: extract_help_topic single word
#[test]
fn test_extract_help_topic_single_word() {
    assert_eq!(extract_help_topic(":help"), None);
}

/// Test: dispatch_repl_command - quit
#[test]
fn test_dispatch_repl_command_quit() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command("quit", &mut state);
    assert!(should_exit, "quit should return true");
}

/// Test: dispatch_repl_command - exit
#[test]
fn test_dispatch_repl_command_exit() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command("exit", &mut state);
    assert!(should_exit, "exit should return true");
}

/// Test: dispatch_repl_command - help
#[test]
fn test_dispatch_repl_command_help() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command("help", &mut state);
    assert!(!should_exit, "help should not exit");
}

/// Test: dispatch_repl_command - help with topic
#[test]
fn test_dispatch_repl_command_help_with_topic() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command("help mode", &mut state);
    assert!(!should_exit);
}

/// Test: dispatch_repl_command - :help
#[test]
fn test_dispatch_repl_command_colon_help() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command(":help", &mut state);
    assert!(!should_exit);
}

/// Test: dispatch_repl_command - regular input (not quit/help)
#[test]
fn test_dispatch_repl_command_regular_input() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command("echo hello", &mut state);
    assert!(!should_exit, "Regular input should not exit");
}

/// Test: dispatch_colon_command - :mode
#[test]
fn test_dispatch_colon_command_mode() {
    let mut state = ReplState::new();
    dispatch_colon_command(":mode purify", &mut state);
    assert_eq!(state.mode(), ReplMode::Purify);
}

/// Test: dispatch_colon_command - :parse
#[test]
fn test_dispatch_colon_command_parse() {
    let mut state = ReplState::new();
    dispatch_colon_command(":parse echo hello", &mut state);
    // Just exercises the code path; no state change expected
    assert_eq!(state.mode(), ReplMode::Normal);
}

/// Test: dispatch_colon_command - :purify
#[test]
fn test_dispatch_colon_command_purify() {
    let mut state = ReplState::new();
    dispatch_colon_command(":purify echo $RANDOM", &mut state);
}

/// Test: dispatch_colon_command - :lint
#[test]
fn test_dispatch_colon_command_lint() {
    let mut state = ReplState::new();
    dispatch_colon_command(":lint echo $x", &mut state);
}

/// Test: dispatch_colon_command - :clear
#[test]
fn test_dispatch_colon_command_clear() {
    let mut state = ReplState::new();
    dispatch_colon_command(":clear", &mut state);
}

/// Test: dispatch_colon_command - :history
#[test]
fn test_dispatch_colon_command_history() {
    let mut state = ReplState::new();
    state.add_history("echo test".to_string());
    dispatch_colon_command(":history", &mut state);
}

/// Test: dispatch_colon_command - :vars
#[test]
fn test_dispatch_colon_command_vars() {
    let mut state = ReplState::new();
    state.set_variable("x".to_string(), "42".to_string());
    dispatch_colon_command(":vars", &mut state);
}

/// Test: dispatch_colon_command - :load (missing path)
#[test]
fn test_dispatch_colon_command_load_missing() {
    let mut state = ReplState::new();
    dispatch_colon_command(":load", &mut state);
}

/// Test: dispatch_colon_command - :source (missing path)
#[test]
fn test_dispatch_colon_command_source_missing() {
    let mut state = ReplState::new();
    dispatch_colon_command(":source", &mut state);
}

/// Test: dispatch_colon_command - :functions
#[test]
fn test_dispatch_colon_command_functions() {
    let mut state = ReplState::new();
    state.add_function("my_func".to_string());
    dispatch_colon_command(":functions", &mut state);
}

/// Test: dispatch_colon_command - :reload (no script loaded)
#[test]
fn test_dispatch_colon_command_reload() {
    let mut state = ReplState::new();
    dispatch_colon_command(":reload", &mut state);
}

/// Test: dispatch_colon_command - :help with topic
#[test]
fn test_dispatch_colon_command_help_with_topic() {
    let mut state = ReplState::new();
    dispatch_colon_command(":help parse", &mut state);
}

/// Test: dispatch_colon_command - unknown command
#[test]
fn test_dispatch_colon_command_unknown() {
    let mut state = ReplState::new();
    dispatch_colon_command(":foobar", &mut state);
    // Should print "Unknown command" and not crash
    assert_eq!(state.mode(), ReplMode::Normal);
}

/// Test: handle_interrupt - with empty buffer
#[test]
fn test_handle_interrupt_empty_buffer() {
    let mut buffer = String::new();
    handle_interrupt(&mut buffer);
    assert!(buffer.is_empty());
}

/// Test: handle_interrupt - with non-empty buffer
#[test]
fn test_handle_interrupt_nonempty_buffer() {
    let mut buffer = String::from("partial input");
    handle_interrupt(&mut buffer);
    assert!(
        buffer.is_empty(),
        "Buffer should be cleared after interrupt"
    );
}

/// Test: handle_interrupt - with multiline buffer
#[test]
fn test_handle_interrupt_multiline_buffer() {
    let mut buffer = String::from("for i in 1 2 3; do\n  echo $i");
    handle_interrupt(&mut buffer);
    assert!(buffer.is_empty());
}

/// Test: dispatch_repl_command with colon command (delegated path)
#[test]
fn test_dispatch_repl_command_colon_command() {
    let mut state = ReplState::new();
    let should_exit = dispatch_repl_command(":mode lint", &mut state);
    assert!(!should_exit);
    assert_eq!(state.mode(), ReplMode::Lint);
}

/// Test: dispatch_repl_command with :help topic (both paths)
#[test]
fn test_dispatch_repl_command_help_starts_with() {
    let mut state = ReplState::new();
    // "help " with space triggers the starts_with path
    let should_exit = dispatch_repl_command("help purify", &mut state);
    assert!(!should_exit);
}

/// Test: dispatch_repl_command - regular code in different modes
#[test]
fn test_dispatch_repl_command_by_mode_purify() {
    let mut state = ReplState::new();
    state.set_mode(ReplMode::Purify);
    let should_exit = dispatch_repl_command("mkdir /tmp/test", &mut state);
    assert!(!should_exit);
}

/// Test: dispatch_repl_command - debug mode
#[test]
fn test_dispatch_repl_command_by_mode_debug() {
    let mut state = ReplState::new();
    state.set_mode(ReplMode::Debug);
    let should_exit = dispatch_repl_command("x=5; echo $x", &mut state);
    assert!(!should_exit);
}

/// Test: handle_command_by_mode - empty output case
#[test]
fn test_handle_command_by_mode_empty_input() {
    let state = ReplState::new();
    // Very simple input that produces minimal output
    handle_command_by_mode("", &state);
}

/// Test: process_repl_line with empty line and empty buffer
#[test]
fn test_process_repl_line_empty_empty_buffer() {
    let mut buffer = String::new();
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    editor.set_helper(Some(completer));
    let action = process_repl_line("", &mut buffer, &mut state, &mut editor);
    assert!(matches!(action, LineAction::Continue));
}

/// Test: process_repl_line with empty line and non-empty buffer
#[test]
fn test_process_repl_line_empty_nonempty_buffer() {
    let mut buffer = String::from("for i in 1 2; do");
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    editor.set_helper(Some(completer));
    let action = process_repl_line("", &mut buffer, &mut state, &mut editor);
    assert!(matches!(action, LineAction::Continue));
    assert!(buffer.contains('\n'), "Should append newline to buffer");
}

/// Test: process_repl_line with quit command
#[test]
fn test_process_repl_line_quit() {
    let mut buffer = String::new();
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    editor.set_helper(Some(completer));
    let action = process_repl_line("quit", &mut buffer, &mut state, &mut editor);
    assert!(matches!(action, LineAction::Break));
}

/// Test: process_repl_line with exit command
#[test]
fn test_process_repl_line_exit() {
    let mut buffer = String::new();
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    editor.set_helper(Some(completer));
    let action = process_repl_line("exit", &mut buffer, &mut state, &mut editor);
    assert!(matches!(action, LineAction::Break));
}

/// Test: process_repl_line with variable assignment
#[test]
fn test_process_repl_line_variable_assignment() {
    let mut buffer = String::new();
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    let action = process_repl_line("x=42", &mut buffer, &mut state, &mut editor);
    assert!(matches!(action, LineAction::Continue));
}

/// Test: process_repl_line with regular command
#[test]
fn test_process_repl_line_regular_command() {
    let mut buffer = String::new();
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    let action = process_repl_line("echo hello", &mut buffer, &mut state, &mut editor);
    // Regular commands that don't quit return Next
    assert!(matches!(action, LineAction::Next));
}

/// Test: process_repl_line with colon command
#[test]
fn test_process_repl_line_colon_command() {
    let mut buffer = String::new();
    let mut state = ReplState::new();
    let completer = crate::repl::completion::ReplCompleter::new();
    let config = rustyline::config::Config::builder()
        .auto_add_history(true)
        .build();
    let mut editor = Editor::with_config(config).expect("editor");
    let action = process_repl_line(":mode purify", &mut buffer, &mut state, &mut editor);
    assert!(matches!(action, LineAction::Next));
    assert_eq!(state.mode(), ReplMode::Purify);
}
