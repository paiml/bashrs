
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

include!("debugger_tests_extracted_cont_tests_repl_008.rs");
