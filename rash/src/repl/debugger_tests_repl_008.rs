#[cfg(test)]
mod tests {
    use super::*;

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

}

include!("debugger_tests_extracted_tests_repl_008.rs");
