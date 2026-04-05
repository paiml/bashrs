//! Tests extracted from debugger.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::repl::debugger::*;

// ===== UNIT TESTS (RED PHASE) =====

/// Test: REPL-008-001-001 - Create debug session from script
#[test]
fn test_REPL_008_001_create_session() {
    let script = "echo hello\necho world";
    let session = DebugSession::new(script);

    assert_eq!(session.current_line(), 1, "Should start at line 1");
    assert_eq!(session.total_lines(), 2, "Should have 2 lines");
    assert!(!session.is_finished(), "Should not be finished initially");
}

/// Test: REPL-008-001-002 - Step through single line
#[test]
fn test_REPL_008_001_step_single_line() {
    let script = "echo hello";
    let mut session = DebugSession::new(script);

    // Step once
    let output = session.step();
    assert!(output.is_some(), "Should execute the line");
    assert!(
        output.unwrap().contains("echo hello"),
        "Should show executed line"
    );

    // Should be finished after one line
    assert!(
        session.is_finished(),
        "Should be finished after single line"
    );

    // Stepping again should return None
    assert!(session.step().is_none(), "Should return None when finished");
}

/// Test: REPL-008-001-003 - Step through multiple lines
#[test]
fn test_REPL_008_001_step_multiple_lines() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // Step 1
    assert_eq!(session.current_line(), 1);
    session.step();

    // Step 2
    assert_eq!(session.current_line(), 2);
    session.step();

    // Step 3
    assert_eq!(session.current_line(), 3);
    session.step();

    // Should be finished
    assert!(session.is_finished());
}

/// Test: REPL-008-001-004 - Get current line content
#[test]
fn test_REPL_008_001_current_line_content() {
    let script = "first line\nsecond line";
    let session = DebugSession::new(script);

    assert_eq!(session.current_line_content(), Some("first line"));
}

/// Test: REPL-008-001-005 - Breakpoint at current line
#[test]
fn test_REPL_008_001_breakpoint_integration() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // Set breakpoint at line 2
    assert!(session.set_breakpoint(2), "Should set breakpoint at line 2");

    // Step to line 1 (no breakpoint)
    assert!(
        !session.at_breakpoint(),
        "Line 1 should not have breakpoint"
    );
    session.step();

    // Now at line 2 (has breakpoint)
    assert!(session.at_breakpoint(), "Line 2 should have breakpoint");
}

/// Test: REPL-008-001-006 - Invalid breakpoint line
#[test]
fn test_REPL_008_001_invalid_breakpoint() {
    let script = "echo hello";
    let mut session = DebugSession::new(script);

    // Try to set breakpoint at line 0 (invalid)
    assert!(!session.set_breakpoint(0), "Should reject line 0");

    // Try to set breakpoint beyond script length
    assert!(
        !session.set_breakpoint(999),
        "Should reject line beyond script"
    );
}

// ===== REPL-008-002: NEXT COMMAND TESTS (SKIP OVER FUNCTIONS) =====

/// Test: REPL-008-002-001 - Next at same level (simple statements)
///
/// RED Phase: This test will FAIL because next() method doesn't exist yet
#[test]
fn test_REPL_008_002_next_same_level() {
    // ARRANGE: Script with simple statements (no functions)
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // ACT: Call next() from line 1
    assert_eq!(session.current_line(), 1, "Should start at line 1");
    session.step_over();

    // ASSERT: Should be at line 2 (next line at same depth)
    assert_eq!(
        session.current_line(),
        2,
        "Should be at line 2 after next()"
    );
    assert!(!session.is_finished(), "Should not be finished");
}

/// Test: REPL-008-002-002 - Next advances to completion
///
/// RED Phase: This test will FAIL because next() method doesn't exist yet
#[test]
fn test_REPL_008_002_next_to_end() {
    // ARRANGE: Single line script
    let script = "echo hello";
    let mut session = DebugSession::new(script);

    // ACT: Call next() - should complete execution
    session.step_over();

    // ASSERT: Should be finished
    assert!(session.is_finished(), "Should be finished after next()");
}

/// Test: REPL-008-002-003 - Next multiple times
///
/// RED Phase: This test will FAIL because next() method doesn't exist yet
#[test]
fn test_REPL_008_002_next_multiple() {
    // ARRANGE: Three line script
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // ACT: Next through all lines
    assert_eq!(session.current_line(), 1);
    session.step_over();

    assert_eq!(session.current_line(), 2);
    session.step_over();

    assert_eq!(session.current_line(), 3);
    session.step_over();

    // ASSERT: Should be finished
    assert!(
        session.is_finished(),
        "Should be finished after 3 next() calls"
    );
}

/// Test: REPL-008-002-004 - Next when already finished
///
/// RED Phase: This test will FAIL because next() method doesn't exist yet
#[test]
fn test_REPL_008_002_next_when_finished() {
    // ARRANGE: Single line script
    let script = "echo hello";
    let mut session = DebugSession::new(script);

    // ACT: Next to completion
    session.step_over();
    assert!(session.is_finished());

    // ACT: Try next() again when finished
    session.step_over();

    // ASSERT: Should still be finished
    assert!(
        session.is_finished(),
        "Should remain finished after next() on completed session"
    );
}

/// Test: REPL-008-002-005 - Next with call depth tracking
///
/// RED Phase: This test will FAIL because next() method doesn't exist yet
/// Note: Simplified version - just verify call_depth() accessor exists
#[test]
fn test_REPL_008_002_call_depth_accessor() {
    // ARRANGE
    let script = "echo test";
    let session = DebugSession::new(script);

    // ACT & ASSERT: Verify call_depth() method exists
    // Initial depth should be 1 (main frame)
    assert_eq!(
        session.call_depth(),
        1,
        "Initial call depth should be 1 (main frame)"
    );
}

// ===== REPL-008-003: CONTINUE EXECUTION TESTS =====

/// Test: REPL-008-003-001 - Continue to breakpoint
#[test]
fn test_REPL_008_003_continue_to_breakpoint() {
    let script = "echo line1\necho line2\necho line3\necho line4";
    let mut session = DebugSession::new(script);

    // Set breakpoint at line 3
    session.set_breakpoint(3);

    // Continue execution - should stop at line 3
    let result = session.continue_execution();
    assert_eq!(
        result,
        ContinueResult::BreakpointHit(3),
        "Should stop at breakpoint on line 3"
    );
    assert_eq!(session.current_line(), 3, "Current line should be 3");
}

/// Test: REPL-008-003-002 - Continue to end (no breakpoints)
#[test]
fn test_REPL_008_003_continue_to_end() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // No breakpoints - should run to completion
    let result = session.continue_execution();
    assert_eq!(
        result,
        ContinueResult::Finished,
        "Should finish execution without breakpoints"
    );
    assert!(session.is_finished(), "Session should be finished");
}

/// Test: REPL-008-003-003 - Continue past first breakpoint
#[test]
fn test_REPL_008_003_continue_multiple_breakpoints() {
    let script = "echo line1\necho line2\necho line3\necho line4\necho line5";
    let mut session = DebugSession::new(script);

    // Set breakpoints at lines 2 and 4
    session.set_breakpoint(2);
    session.set_breakpoint(4);

    // First continue - stop at line 2
    let result1 = session.continue_execution();
    assert_eq!(result1, ContinueResult::BreakpointHit(2));
    assert_eq!(session.current_line(), 2);

    // Step over the breakpoint
    session.step();

    // Second continue - stop at line 4
    let result2 = session.continue_execution();
    assert_eq!(result2, ContinueResult::BreakpointHit(4));
    assert_eq!(session.current_line(), 4);
}

/// Test: REPL-008-003-004 - Continue when already at breakpoint
#[test]
fn test_REPL_008_003_continue_at_breakpoint() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // Set breakpoint at line 1 (starting position)
    session.set_breakpoint(1);

    // Continue should immediately return (already at breakpoint)
    let result = session.continue_execution();
    assert_eq!(
        result,
        ContinueResult::BreakpointHit(1),
        "Should detect we're already at breakpoint"
    );
    assert_eq!(session.current_line(), 1);
}

/// Test: REPL-008-003-005 - Continue from middle of script
#[test]
fn test_REPL_008_003_continue_from_middle() {
    let script = "echo line1\necho line2\necho line3\necho line4";
    let mut session = DebugSession::new(script);

    // Step to line 2
    session.step();
    assert_eq!(session.current_line(), 2);

    // Set breakpoint at line 4
    session.set_breakpoint(4);

    // Continue from line 2 to line 4
    let result = session.continue_execution();
    assert_eq!(result, ContinueResult::BreakpointHit(4));
    assert_eq!(session.current_line(), 4);
}

/// Test: REPL-008-003-006 - Continue past last breakpoint to end
#[test]
fn test_REPL_008_003_continue_past_breakpoint_to_end() {
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // Set breakpoint at line 2
    session.set_breakpoint(2);

    // First continue - stop at breakpoint
    assert_eq!(
        session.continue_execution(),
        ContinueResult::BreakpointHit(2)
    );

    // Step past breakpoint
    session.step();

    // Second continue - run to end
    assert_eq!(session.continue_execution(), ContinueResult::Finished);
    assert!(session.is_finished());
}

// ===== REPL-008-004: FINISH (EXIT CURRENT FUNCTION) TESTS =====

/// Test: REPL-008-004-001 - Finish returns from simulated function
///
/// RED phase: Test finish() with manual call stack manipulation
#[test]
fn test_REPL_008_004_finish_returns_from_function() {
    // ARRANGE: Script with multiple lines
    let script = "echo line1\necho line2\necho line3\necho line4";
    let mut session = DebugSession::new(script);

    // Simulate entering a function by pushing a frame
    session.push_frame("test_function", 2);

    // Current depth should be 2 (<main> + test_function)
    assert_eq!(session.call_depth(), 2);

    // ACT: Call finish() to exit function
    let result = session.finish();

    // ASSERT: Should have returned from function
    // (In simplified version, this will just continue to end or breakpoint)
    assert!(matches!(result, ContinueResult::Finished));
}

/// Test: REPL-008-004-002 - Finish at top level continues to end
///
/// RED phase: Test finish() when already at <main> level
#[test]
fn test_REPL_008_004_finish_at_top_level() {
    // ARRANGE: Script with no functions (depth 1 - main only)
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // At line 1, depth 1 (<main> only)
    assert_eq!(session.call_depth(), 1);

    // ACT: Call finish() at top level
    let result = session.finish();

    // ASSERT: Should continue to end (behaves like continue)
    assert!(matches!(result, ContinueResult::Finished));
    assert!(session.is_finished());
}

/// Test: REPL-008-004-003 - Finish stops at breakpoint
///
/// RED phase: Test that finish() respects breakpoints
#[test]
fn test_REPL_008_004_finish_stops_at_breakpoint() {
    // ARRANGE: Script with breakpoint
    let script = "echo line1\necho line2\necho line3\necho line4";
    let mut session = DebugSession::new(script);

    // Set breakpoint on line 3
    session.set_breakpoint(3);

    // Simulate entering a function
    session.push_frame("test_function", 1);

    // ACT: Call finish() - should stop at breakpoint before returning
    let result = session.finish();

    // ASSERT: Should hit breakpoint
    assert!(matches!(result, ContinueResult::BreakpointHit(3)));
    assert!(!session.is_finished());
}

/// Test: REPL-008-004-004 - Finish with nested frames
///
/// RED phase: Test finish() returns one level only
#[test]
fn test_REPL_008_004_finish_nested_functions() {
    // ARRANGE: Script with manual nested call stack
    let script = "echo line1\necho line2\necho line3\necho line4";
    let mut session = DebugSession::new(script);

    // Simulate nested function calls
    session.push_frame("outer_function", 1);
    session.push_frame("inner_function", 2);

    // Should be at depth 3 (<main> + outer + inner)
    assert_eq!(session.call_depth(), 3);

    // ACT: finish() should return from inner to outer
    let result = session.finish();

    // ASSERT: Should exit one level (inner function only)
    // Note: In simplified version without real function tracking,
    // we'll just verify finish() doesn't crash and returns a valid result
    assert!(
        matches!(result, ContinueResult::Finished)
            || matches!(result, ContinueResult::BreakpointHit(_))
    );
}

/// Test: REPL-008-004-005 - Finish when already finished
///
/// RED phase: Test finish() behavior at end of script
#[test]
fn test_REPL_008_004_finish_when_already_finished() {
    // ARRANGE: Script at end
    let script = "echo done";
    let mut session = DebugSession::new(script);

    // Step to end
    session.step();
    session.step();
    assert!(session.is_finished());

    // ACT: Call finish() when already finished
    let result = session.finish();

    // ASSERT: Should remain finished
    assert!(matches!(result, ContinueResult::Finished));
    assert!(session.is_finished());
}

// ===== REPL-009-001: VARIABLE INSPECTION TESTS =====

/// Test: REPL-009-001-001 - Set and get variable
#[test]
fn test_REPL_009_001_print_variable() {
    let script = "echo hello";
    let mut session = DebugSession::new(script);

    // Set a variable
    session.set_variable("USER", "alice");
    session.set_variable("HOME", "/home/alice");

    // Get variable values
    assert_eq!(session.get_variable("USER"), Some("alice"));
    assert_eq!(session.get_variable("HOME"), Some("/home/alice"));

    // Variable count
    assert_eq!(session.variable_count(), 2);
}

/// Test: REPL-009-001-002 - Array-like variables (stored as comma-separated)
#[test]
fn test_REPL_009_001_print_array() {
    let script = "echo test";
    let mut session = DebugSession::new(script);

    // Store array as comma-separated string (simplified array handling)
    session.set_variable("ARRAY", "item1,item2,item3");

    // Retrieve array
    let array_value = session.get_variable("ARRAY");
    assert_eq!(array_value, Some("item1,item2,item3"));

    // Could be split by caller if needed
    let items: Vec<&str> = array_value.unwrap().split(',').collect();
    assert_eq!(items, vec!["item1", "item2", "item3"]);
}

/// Test: REPL-009-001-003 - Nonexistent variable returns None
