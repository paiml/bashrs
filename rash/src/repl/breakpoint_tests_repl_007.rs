#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-007-001-001 - Set a breakpoint
    #[test]
    fn test_REPL_007_001_set_breakpoint() {
        let mut manager = BreakpointManager::new();

        // Set breakpoint at line 10
        let added = manager.set_breakpoint(10);
        assert!(added, "Should return true when adding new breakpoint");

        // Verify breakpoint is set
        assert!(
            manager.is_breakpoint_hit(10),
            "Breakpoint at line 10 should be set"
        );
        assert_eq!(manager.count(), 1, "Should have 1 breakpoint");

        // Setting same breakpoint again should return false
        let added_again = manager.set_breakpoint(10);
        assert!(
            !added_again,
            "Should return false when breakpoint already exists"
        );
        assert_eq!(manager.count(), 1, "Should still have 1 breakpoint");
    }

    /// Test: REPL-007-001-002 - Hit a breakpoint
    #[test]
    fn test_REPL_007_001_hit_breakpoint() {
        let mut manager = BreakpointManager::new();

        // Set breakpoint at line 5
        manager.set_breakpoint(5);

        // Check various lines
        assert!(
            manager.is_breakpoint_hit(5),
            "Should hit breakpoint at line 5"
        );
        assert!(
            !manager.is_breakpoint_hit(4),
            "Should not hit breakpoint at line 4"
        );
        assert!(
            !manager.is_breakpoint_hit(6),
            "Should not hit breakpoint at line 6"
        );
        assert!(
            !manager.is_breakpoint_hit(100),
            "Should not hit breakpoint at line 100"
        );
    }

    /// Test: REPL-007-001-003 - Remove a breakpoint
    #[test]
    fn test_REPL_007_001_remove_breakpoint() {
        let mut manager = BreakpointManager::new();

        // Set breakpoint at line 15
        manager.set_breakpoint(15);
        assert!(
            manager.is_breakpoint_hit(15),
            "Breakpoint at line 15 should be set"
        );

        // Remove the breakpoint
        let removed = manager.remove_breakpoint(15);
        assert!(
            removed,
            "Should return true when removing existing breakpoint"
        );

        // Verify breakpoint is removed
        assert!(
            !manager.is_breakpoint_hit(15),
            "Breakpoint at line 15 should be removed"
        );
        assert_eq!(manager.count(), 0, "Should have 0 breakpoints");

        // Removing non-existent breakpoint should return false
        let removed_again = manager.remove_breakpoint(15);
        assert!(
            !removed_again,
            "Should return false when breakpoint doesn't exist"
        );
    }

    /// Test: REPL-007-001-004 - Multiple breakpoints
    #[test]
    fn test_REPL_007_001_multiple_breakpoints() {
        let mut manager = BreakpointManager::new();

        // Set multiple breakpoints
        manager.set_breakpoint(1);
        manager.set_breakpoint(5);
        manager.set_breakpoint(10);
        manager.set_breakpoint(15);

        assert_eq!(manager.count(), 4, "Should have 4 breakpoints");

        // Check all breakpoints are hit
        assert!(manager.is_breakpoint_hit(1));
        assert!(manager.is_breakpoint_hit(5));
        assert!(manager.is_breakpoint_hit(10));
        assert!(manager.is_breakpoint_hit(15));

        // Check non-breakpoint lines
        assert!(!manager.is_breakpoint_hit(2));
        assert!(!manager.is_breakpoint_hit(3));
    }

    /// Test: REPL-007-001-005 - Get all breakpoints
    #[test]
    fn test_REPL_007_001_get_breakpoints() {
        let mut manager = BreakpointManager::new();

        // Set breakpoints in random order
        manager.set_breakpoint(15);
        manager.set_breakpoint(5);
        manager.set_breakpoint(10);
        manager.set_breakpoint(1);

        // Get all breakpoints (should be sorted)
        let breakpoints = manager.get_breakpoints();
        assert_eq!(
            breakpoints,
            vec![1, 5, 10, 15],
            "Breakpoints should be sorted"
        );
    }

    /// Test: REPL-007-001-006 - Clear all breakpoints
    #[test]
    fn test_REPL_007_001_clear_all() {
        let mut manager = BreakpointManager::new();

        // Set multiple breakpoints
        manager.set_breakpoint(1);
        manager.set_breakpoint(5);
        manager.set_breakpoint(10);
        assert_eq!(manager.count(), 3, "Should have 3 breakpoints");

        // Clear all
        manager.clear_all();
        assert_eq!(manager.count(), 0, "Should have 0 breakpoints after clear");
        assert!(!manager.is_breakpoint_hit(1));
        assert!(!manager.is_breakpoint_hit(5));
        assert!(!manager.is_breakpoint_hit(10));
    }

    // ===== CONDITIONAL BREAKPOINTS (REPL-007-002) =====

    /// Test: REPL-007-002-001 - Conditional breakpoint evaluates to true
    #[test]
    fn test_REPL_007_002_conditional_true() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "15".to_string());

        // Create conditional breakpoint: break if $count > 10
        let bp = Breakpoint::with_condition(5, "$count > 10".to_string());

        assert!(bp.is_conditional(), "Should be a conditional breakpoint");
        assert!(bp.should_break(&vars), "Should break when count (15) > 10");
    }

    /// Test: REPL-007-MUTATION-001 - Unconditional breakpoint is not conditional
    ///
    /// This test catches the mutant: `replace is_conditional -> bool with true`
    ///
    /// RED phase: Write test that verifies unconditional breakpoints return false for is_conditional()
    #[test]
    fn test_REPL_007_MUTATION_001_unconditional_is_not_conditional() {
        // ARRANGE: Create simple (unconditional) breakpoint
        let bp_simple = Breakpoint::new(10);

        // ASSERT: Simple breakpoint should NOT be conditional
        assert!(
            !bp_simple.is_conditional(),
            "Simple breakpoint (no condition) should return false for is_conditional()"
        );

        // ARRANGE: Create hit-count breakpoint (also unconditional)
        let bp_hit_count = Breakpoint::with_hit_count(20, 5);

        // ASSERT: Hit-count breakpoint (without condition) should NOT be conditional
        assert!(
            !bp_hit_count.is_conditional(),
            "Hit-count breakpoint (no condition) should return false for is_conditional()"
        );

        // ARRANGE: Create conditional breakpoint for contrast
        let bp_conditional = Breakpoint::with_condition(30, "$x > 5".to_string());

        // ASSERT: Conditional breakpoint SHOULD be conditional
        assert!(
            bp_conditional.is_conditional(),
            "Conditional breakpoint should return true for is_conditional()"
        );

        // ARRANGE: Create hit-count + condition breakpoint
        let bp_both = Breakpoint::with_hit_count_and_condition(40, 3, "$y < 10".to_string());

        // ASSERT: Hit-count + condition should be conditional
        assert!(
            bp_both.is_conditional(),
            "Hit-count + condition breakpoint should return true for is_conditional()"
        );
    }

}

#[cfg(test)]
mod breakpoint_tests_extracted_tests_repl_007 {
    use super::*;
    include!("breakpoint_tests_extracted_tests_repl_007.rs");
}
