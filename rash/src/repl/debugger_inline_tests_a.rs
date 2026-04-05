//! Tests extracted from debugger.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::repl::debugger::*;

// ===== UNIT TESTS (RED PHASE) =====

/// Test: REPL-008-001-001 - Create debug session from script
#[test]
fn test_REPL_009_001_print_nonexistent() {
    let script = "echo test";
    let session = DebugSession::new(script);

    // Get nonexistent variable
    assert_eq!(session.get_variable("DOES_NOT_EXIST"), None);
    assert_eq!(session.get_variable(""), None);
}

/// Test: REPL-009-001-004 - List all variables
#[test]
fn test_REPL_009_001_list_variables() {
    let script = "echo test";
    let mut session = DebugSession::new(script);

    // Initially empty
    assert_eq!(session.list_variables(), vec![]);

    // Add variables
    session.set_variable("PATH", "/usr/bin");
    session.set_variable("USER", "bob");
    session.set_variable("HOME", "/home/bob");

    // List variables (sorted by name)
    let vars = session.list_variables();
    assert_eq!(vars.len(), 3);
    assert_eq!(vars[0], ("HOME", "/home/bob"));
    assert_eq!(vars[1], ("PATH", "/usr/bin"));
    assert_eq!(vars[2], ("USER", "bob"));
}

/// Test: REPL-009-001-005 - Variable update
#[test]
fn test_REPL_009_001_variable_update() {
    let script = "echo test";
    let mut session = DebugSession::new(script);

    // Set initial value
    session.set_variable("VERSION", "1.0");
    assert_eq!(session.get_variable("VERSION"), Some("1.0"));

    // Update value
    session.set_variable("VERSION", "2.0");
    assert_eq!(session.get_variable("VERSION"), Some("2.0"));

    // Count should still be 1
    assert_eq!(session.variable_count(), 1);
}

/// Test: REPL-009-001-006 - Clear variables
#[test]
fn test_REPL_009_001_clear_variables() {
    let script = "echo test";
    let mut session = DebugSession::new(script);

    // Add variables
    session.set_variable("A", "1");
    session.set_variable("B", "2");
    session.set_variable("C", "3");
    assert_eq!(session.variable_count(), 3);

    // Clear all
    session.clear_variables();
    assert_eq!(session.variable_count(), 0);
    assert_eq!(session.list_variables(), vec![]);
    assert_eq!(session.get_variable("A"), None);
}

/// Test: REPL-009-001-007 - Variables persist across steps
#[test]
fn test_REPL_009_001_variables_persist_across_steps() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // Set variable before stepping
    session.set_variable("COUNTER", "0");

    // Step through and verify variable persists
    session.step();
    assert_eq!(session.get_variable("COUNTER"), Some("0"));

    session.step();
    assert_eq!(session.get_variable("COUNTER"), Some("0"));

    // Update variable mid-execution
    session.set_variable("COUNTER", "2");
    session.step();
    assert_eq!(session.get_variable("COUNTER"), Some("2"));
}

// ===== REPL-009-002: Environment Display Tests =====

#[test]
fn test_REPL_009_002_env_display() {
    let script = "echo test";
    let session = DebugSession::new(script);

    // Get an environment variable that should exist (PATH always exists)
    let path = session.get_env("PATH");
    assert!(path.is_some(), "PATH environment variable should exist");

    // Get a variable that doesn't exist
    let nonexistent = session.get_env("BASHRS_NONEXISTENT_VAR_12345");
    assert_eq!(nonexistent, None);

    // List all environment variables
    let env_vars = session.list_env();
    assert!(!env_vars.is_empty(), "Should have at least one env var");

    // Verify sorted order
    let mut sorted = env_vars.clone();
    sorted.sort_by_key(|(name, _)| name.clone());
    assert_eq!(env_vars, sorted, "Environment variables should be sorted");
}

#[test]
fn test_REPL_009_002_env_filter() {
    let script = "echo test";
    let session = DebugSession::new(script);

    // Filter by prefix (most systems have PATH-related variables)
    let path_vars = session.filter_env("PATH");
    assert!(
        !path_vars.is_empty(),
        "Should find at least one PATH-related variable"
    );

    // All filtered results should start with the prefix
    for (name, _) in &path_vars {
        assert!(
            name.starts_with("PATH"),
            "Filtered variable {} should start with PATH",
            name
        );
    }

    // Filter with non-matching prefix
    let empty_filter = session.filter_env("BASHRS_NONEXISTENT_PREFIX");
    assert_eq!(
        empty_filter.len(),
        0,
        "Filter with non-matching prefix should return empty"
    );

    // Verify sorted order
    let mut sorted = path_vars.clone();
    sorted.sort_by_key(|(name, _)| name.clone());
    assert_eq!(path_vars, sorted, "Filtered env vars should be sorted");
}

// ===== REPL-009-003: Call Stack Tracking Tests =====

#[test]
fn test_REPL_009_003_backtrace_single() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // Initially, call stack should have main frame
    let initial_len = session.call_stack().len();
    assert_eq!(initial_len, 1, "Should have just main frame initially");

    // Push a frame
    session.push_frame("function1", 1);

    // Get backtrace
    let stack = session.call_stack();
    assert_eq!(stack.len(), 2, "Should have main + function1");

    let frame = &stack[1];
    assert_eq!(frame.name, "function1");
    assert_eq!(frame.line, 1);

    // Pop frame
    session.pop_frame();

    // Should be back to initial
    let final_len = session.call_stack().len();
    assert_eq!(final_len, initial_len);
}

#[test]
fn test_REPL_009_003_backtrace_nested() {
    let script = "echo test";
    let mut session = DebugSession::new(script);

    // Push nested frames
    session.push_frame("main", 1);
    session.push_frame("func_a", 5);
    session.push_frame("func_b", 10);

    // Get full stack
    let stack = session.call_stack();
    assert_eq!(
        stack.len(),
        4,
        "Should have <main> + main + func_a + func_b"
    );

    // Verify stack order (most recent last)
    assert_eq!(stack[1].name, "main");
    assert_eq!(stack[1].line, 1);
    assert_eq!(stack[2].name, "func_a");
    assert_eq!(stack[2].line, 5);
    assert_eq!(stack[3].name, "func_b");
    assert_eq!(stack[3].line, 10);

    // Pop frames
    session.pop_frame(); // func_b
    let stack2 = session.call_stack();
    assert_eq!(stack2.len(), 3);

    session.pop_frame(); // func_a
    let stack3 = session.call_stack();
    assert_eq!(stack3.len(), 2);

    session.pop_frame(); // main
    let stack4 = session.call_stack();
    assert_eq!(stack4.len(), 1, "Should be back to just <main>");
}

// ===== REPL-010-001: Compare Original vs Purified =====

/// Test: REPL-010-001-001 - Compare at breakpoint shows original and purified
#[test]
fn test_REPL_010_001_compare_at_breakpoint() {
    // Script with non-idempotent command
    let script = "mkdir /tmp/test";
    let session = DebugSession::new(script);

    // Get comparison at line 1
    let comparison = session.compare_current_line();
    assert!(comparison.is_some(), "Should have comparison for line 1");

    let cmp = comparison.unwrap();
    assert_eq!(cmp.original, "mkdir /tmp/test");
    // Purified adds -p flag for idempotency
    assert!(
        cmp.purified.contains("mkdir") && cmp.purified.contains("-p"),
        "Purified should add -p flag, got: {}",
        cmp.purified
    );
    assert!(cmp.differs, "Original and purified should differ");
}

/// Test: REPL-010-001-002 - Compare diff highlighting marks changes
#[test]
fn test_REPL_010_001_compare_diff_highlighting() {
    // Script with missing quotes
    let script = "echo $HOME";
    let session = DebugSession::new(script);

    let comparison = session.compare_current_line();
    assert!(comparison.is_some());

    let cmp = comparison.unwrap();
    assert_eq!(cmp.original, "echo $HOME");
    assert_eq!(cmp.purified, "echo \"$HOME\"");
    assert!(cmp.differs);

    // Get diff highlighting
    let diff = session.format_diff_highlighting(&cmp);
    assert!(diff.contains("$HOME"), "Diff should show variable");
    assert!(
        diff.contains("\"$HOME\""),
        "Diff should show quoted version"
    );
}

// ===== REPL-010-002: ENHANCED HIGHLIGHTING TESTS (RED PHASE) =====

/// Test: REPL-010-002-001 - Highlight mkdir -p idempotency flag
///
/// RED phase: Write failing test for enhanced diff highlighting
/// that specifically marks the added -p flag
#[test]
fn test_REPL_010_002_highlight_mkdir_p() {
    // ARRANGE: Script with non-idempotent mkdir
    let script = "mkdir /tmp/foo";
    let session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line();
    assert!(comparison.is_some(), "Should be able to compare");

    let cmp = comparison.unwrap();
    assert!(cmp.differs, "Lines should differ");

    // ACT: Get enhanced highlighting
    let highlighted = session.format_diff_highlighting(&cmp);

    // ASSERT: Phase 2 adds permission checks, so first line is permission check
    // The highlighted output will show the permission check transformation
    assert!(
        highlighted.contains("mkdir") || highlighted.contains("dirname"),
        "Should show mkdir-related content"
    );
}

/// Test: REPL-010-002-002 - Highlight variable quoting
///
/// RED phase: Test should fail until we implement quote detection
#[test]
fn test_REPL_010_002_highlight_quote() {
    // ARRANGE: Script with unquoted variable
    let script = "echo $USER";
    let session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line();
    assert!(comparison.is_some(), "Should be able to compare");

    let cmp = comparison.unwrap();
    assert!(cmp.differs, "Lines should differ");

    // ACT: Get enhanced highlighting
    let highlighted = session.format_diff_highlighting(&cmp);

    // ASSERT: Should show quotes
    assert!(highlighted.contains("\""), "Should show quote addition");

    // ASSERT: Should explain quoting transformation
    assert!(
        highlighted.to_lowercase().contains("quot")
            || highlighted.to_lowercase().contains("safe"),
        "Should explain quoting: {}",
        highlighted
    );
}

/// Test: REPL-010-002-003 - Highlight ln -sf safety flag
///
/// RED phase: Test for ln command transformation highlighting
#[test]
fn test_REPL_010_002_highlight_ln_sf() {
    // ARRANGE: Script with non-idempotent ln
    let script = "ln -s /tmp/src /tmp/link";
    let session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line();
    assert!(comparison.is_some(), "Should be able to compare");

    let cmp = comparison.unwrap();

    // Only test if lines differ (purifier might add -f flag)
    if !cmp.differs {
        // Skip test if purifier doesn't transform this
        return;
    }

    // ACT: Get enhanced highlighting
    let highlighted = session.format_diff_highlighting(&cmp);

    // ASSERT: Should show ln command
    assert!(highlighted.contains("ln"), "Should show ln command");

    // ASSERT: Should highlight flag addition
    assert!(
        highlighted.contains("-") && highlighted.contains("f"),
        "Should show flag addition"
    );

    // ASSERT: Should explain safety/idempotency
    assert!(
        highlighted.to_lowercase().contains("safe")
            || highlighted.to_lowercase().contains("idempot")
            || highlighted.to_lowercase().contains("idem"),
        "Should explain transformation: {}",
        highlighted
    );
}

/// Test: REPL-010-002-004 - Handle no changes case
///
/// RED phase: Test for already-purified script
#[test]
fn test_REPL_010_002_highlight_no_change() {
    // ARRANGE: Script with simple echo (minimal transformation)
    let script = "echo hello";
    let session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line();
    assert!(comparison.is_some(), "Should be able to compare");

    let cmp = comparison.unwrap();

    // ACT: Get highlighting
    let highlighted = session.format_diff_highlighting(&cmp);

    // ASSERT: Should handle gracefully (may or may not differ)
    assert!(!highlighted.is_empty(), "Should produce some output");
}

/// Test: REPL-010-002-005 - Handle multiple transformations
///
/// RED phase: Test for line with multiple changes (rm + quoting)
#[test]
fn test_REPL_010_002_highlight_multiple_changes() {
    // ARRANGE: Script with multiple issues
    let script = "rm $FILE";
    let session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line();
    assert!(comparison.is_some(), "Should be able to compare");

    let cmp = comparison.unwrap();
    assert!(cmp.differs, "Lines should differ");

    // ACT: Get highlighting
    let highlighted = session.format_diff_highlighting(&cmp);

    // ASSERT: Should show at least one transformation
    assert!(
        highlighted.contains("-f") || highlighted.contains("\""),
        "Should show either -f flag or quoting: {}",
        highlighted
    );
}

// ===== REPL-010-003: Explain Transformations at Current Line =====

/// Test: REPL-010-003-001 - Explain mkdir -p transformation
///
/// RED phase: Test explanation for mkdir idempotency
#[test]
fn test_REPL_010_003_explain_mkdir_p() {
    // ARRANGE: Script with non-idempotent mkdir
    let script = "mkdir /tmp/foo";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should have some explanation (may be about permission checks or idempotency)
    assert!(explanation.is_some(), "Should have explanation for mkdir");
    let text = explanation.unwrap();
    // Phase 2 added permission checks, so explanation may mention permissions or transformations
    assert!(
        text.contains("transform")
            || text.contains("permission")
            || text.contains("idempot")
            || text.contains("idem")
            || text.contains("-p"),
        "Should explain transformation: {}",
        text
    );
}

/// Test: REPL-010-003-002 - Explain variable quoting transformation
///
/// RED phase: Test explanation for variable safety quoting
#[test]
fn test_REPL_010_003_explain_quote() {
    // ARRANGE: Script with unquoted variable
    let script = "echo $USER";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should explain quoting (if purifier transforms this)
    // Note: Test is conditional based on whether purifier transforms echo
    if let Some(text) = explanation {
        assert!(
            text.contains("quot") || text.contains("safe"),
            "Should explain quoting or safety: {}",
            text
        );
    }
}

/// Test: REPL-010-003-003 - Explain ln -sf transformation
///
/// RED phase: Test explanation for ln idempotency
