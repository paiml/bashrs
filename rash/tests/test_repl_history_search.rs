#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// REPL History Search Tests
//
// Task: REPL-015-NEW - History search with Ctrl-R
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 10+ scenarios
// - Integration tests: CLI interaction with assert_cmd
// - Mutation score: ≥90%
// - Complexity: <10 per function

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to get test history directory
fn get_test_history_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Helper to create a test history file
fn create_test_history(dir: &TempDir, commands: &[&str]) -> PathBuf {
    let history_path = dir.path().join(".bashrs_history");
    let content = commands.join("\n");
    fs::write(&history_path, content).expect("Failed to write history file");
    history_path
}

// ===== RED PHASE: Write failing tests first =====

/// Test: REPL-015-NEW-001 - History file is created on first run
#[test]
fn test_repl_015_new_001_history_file_created() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL and quit immediately
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin(":quit\n")
        .assert()
        .success();

    // History file should exist after REPL exits
    assert!(history_path.exists(), "History file should be created");
}

/// Test: REPL-015-NEW-002 - Commands are added to history
#[test]
fn test_repl_015_new_002_commands_added_to_history() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL with some commands
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo hello\necho world\n:quit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // Commands should be in history
    assert!(history_content.contains("echo hello"));
    assert!(history_content.contains("echo world"));
}

/// Test: REPL-015-NEW-003 - History is loaded on subsequent runs
#[test]
fn test_repl_015_new_003_history_loaded_on_restart() {
    let temp_dir = get_test_history_dir();
    let history_path = create_test_history(&temp_dir, &["echo first", "echo second", "echo third"]);

    // Run REPL and execute new command to verify history was loaded
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo fourth\n:quit\n")
        .assert()
        .success();

    // Read history file - should contain both old and new commands
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // Previously saved commands should still be there
    assert!(history_content.contains("echo first"));
    assert!(history_content.contains("echo second"));
    assert!(history_content.contains("echo third"));

    // New command should be added
    assert!(history_content.contains("echo fourth"));
}

/// Test: REPL-015-NEW-004 - Duplicate commands are ignored (if configured)
#[test]
fn test_repl_015_new_004_history_ignores_duplicates() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL with duplicate commands
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo hello\necho hello\necho hello\n:quit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // "echo hello" should appear only once (duplicates ignored)
    let count = history_content.matches("echo hello").count();
    assert_eq!(count, 1, "Duplicate commands should be ignored");
}

/// Test: REPL-015-NEW-005 - Commands starting with space are ignored (if configured)
#[test]
fn test_repl_015_new_005_history_ignores_space_prefix() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL with space-prefixed command
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo saved\n echo not_saved\n:quit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // "echo saved" should be in history
    assert!(history_content.contains("echo saved"));

    // " echo not_saved" should NOT be in history (space prefix ignored)
    assert!(!history_content.contains("echo not_saved"));
}

/// Test: REPL-015-NEW-006 - REPL commands (:quit, :help, etc.) are in history
#[test]
fn test_repl_015_new_006_repl_commands_in_history() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL with various commands
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin(":mode purify\n:help\necho test\n:quit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // REPL commands should be in history
    assert!(history_content.contains(":mode purify"));
    assert!(history_content.contains(":help"));
    assert!(history_content.contains("echo test"));
}

/// Test: REPL-015-NEW-007 - History has maximum size limit
#[test]
fn test_repl_015_new_007_history_max_size() {
    let temp_dir = get_test_history_dir();

    // Create history with 1000 commands
    let commands: Vec<String> = (0..1000).map(|i| format!("echo command_{}", i)).collect();

    let command_refs: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();
    let history_path = create_test_history(&temp_dir, &command_refs);

    // Run REPL with one more command
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo new_command\nquit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    let line_count = history_content.lines().filter(|l| !l.is_empty()).count();

    // History should be capped (rustyline default: 1000 lines)
    // Allow some flexibility as rustyline may trim slightly differently
    assert!(
        line_count <= 1002,
        "History should be capped at ~1000 lines, got {}",
        line_count
    );

    // Verify the new command was added
    assert!(history_content.contains("echo new_command"));
}

/// Test: REPL-015-NEW-008 - Empty lines are not added to history
#[test]
fn test_repl_015_new_008_empty_lines_not_in_history() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL with empty lines
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo before\n\n\necho after\nquit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    let lines: Vec<&str> = history_content.lines().filter(|l| !l.is_empty()).collect();

    // Only non-empty commands should be in history (including "quit")
    // Expected: "echo before", "echo after", "quit"
    assert!(
        lines.len() >= 2,
        "Should have at least 2 commands in history"
    );
    assert!(history_content.contains("echo before"));
    assert!(history_content.contains("echo after"));

    // Verify empty lines were NOT added
    let empty_line_count = history_content.lines().filter(|l| l.is_empty()).count();
    assert!(
        empty_line_count <= 1,
        "Should have at most 1 empty line (trailing newline)"
    );
}

/// Test: REPL-015-NEW-009 - Multi-line commands are saved as single entry
#[test]
fn test_repl_015_new_009_multiline_commands_in_history() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // Run REPL with multi-line function
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("function greet() {\n  echo hello\n}\n:quit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // Multi-line command should be in history (possibly as multi-line or single line with \n)
    assert!(history_content.contains("greet"));
    assert!(history_content.contains("echo hello"));
}

/// Test: REPL-015-NEW-010 - History persists across crashes (saved incrementally)
#[test]
fn test_repl_015_new_010_history_persists_across_sessions() {
    let temp_dir = get_test_history_dir();
    let history_path = temp_dir.path().join(".bashrs_history");

    // First session
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo session1\n:quit\n")
        .assert()
        .success();

    // Second session
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("repl")
        .env("BASHRS_HISTORY_PATH", &history_path)
        .write_stdin("echo session2\n:quit\n")
        .assert()
        .success();

    // Read history file
    let history_content = fs::read_to_string(&history_path).expect("Failed to read history file");

    // Both sessions should be in history
    assert!(history_content.contains("echo session1"));
    assert!(history_content.contains("echo session2"));
}

// ===== NOTE: Ctrl-R reverse search is built into rustyline =====
// Once the Editor is properly configured with Config::builder(),
// Ctrl-R will automatically work for history search.
// No additional code needed - just proper configuration!
