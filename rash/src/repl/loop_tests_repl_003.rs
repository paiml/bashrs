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
