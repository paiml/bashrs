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

    /// Test: REPL-007-MUTATION-002 - Greater than or equal (>=) operator
    ///
    /// This test catches mutants:
    /// - Line 228: replace + with - or * in `pos + 2` (value parsing)
    /// - Line 282: delete match arm ">=" (operator missing)
    ///
    /// RED phase: Write comprehensive test for >= operator
    #[test]
    fn test_REPL_007_MUTATION_002_greater_than_or_equal() {
        let mut vars = HashMap::new();

        // Test case 1: >= with value GREATER than threshold (should break)
        vars.insert("count".to_string(), "10".to_string());
        let bp_gt = Breakpoint::with_condition(5, "$count >= 5".to_string());
        assert!(
            bp_gt.should_break(&vars),
            "Should break when count (10) >= 5 (greater than threshold)"
        );

        // Test case 2: >= with value EQUAL to threshold (should break)
        vars.insert("count".to_string(), "5".to_string());
        let bp_eq = Breakpoint::with_condition(5, "$count >= 5".to_string());
        assert!(
            bp_eq.should_break(&vars),
            "Should break when count (5) >= 5 (equal to threshold)"
        );

        // Test case 3: >= with value LESS than threshold (should NOT break)
        vars.insert("count".to_string(), "3".to_string());
        let bp_lt = Breakpoint::with_condition(5, "$count >= 5".to_string());
        assert!(
            !bp_lt.should_break(&vars),
            "Should NOT break when count (3) >= 5 (less than threshold)"
        );

        // Test case 4: >= with multi-digit value (tests parsing of value_part)
        vars.insert("count".to_string(), "100".to_string());
        let bp_large = Breakpoint::with_condition(5, "$count >= 99".to_string());
        assert!(
            bp_large.should_break(&vars),
            "Should break when count (100) >= 99 (multi-digit value parsing)"
        );

        // Test case 5: >= boundary case (exactly at boundary)
        vars.insert("count".to_string(), "0".to_string());
        let bp_zero = Breakpoint::with_condition(5, "$count >= 0".to_string());
        assert!(
            bp_zero.should_break(&vars),
            "Should break when count (0) >= 0 (zero boundary)"
        );
    }

    /// Test: REPL-007-MUTATION-003 - Less than or equal (<=) operator
    ///
    /// This test catches mutants:
    /// - Line 232: replace + with - or * in `pos + 2` (value parsing)
    /// - Line 291: delete match arm "<=" (operator missing)
    ///
    /// RED phase: Write comprehensive test for <= operator
    #[test]
    fn test_REPL_007_MUTATION_003_less_than_or_equal() {
        let mut vars = HashMap::new();

        // Test case 1: <= with value LESS than threshold (should break)
        vars.insert("count".to_string(), "3".to_string());
        let bp_lt = Breakpoint::with_condition(5, "$count <= 5".to_string());
        assert!(
            bp_lt.should_break(&vars),
            "Should break when count (3) <= 5 (less than threshold)"
        );

        // Test case 2: <= with value EQUAL to threshold (should break)
        vars.insert("count".to_string(), "5".to_string());
        let bp_eq = Breakpoint::with_condition(5, "$count <= 5".to_string());
        assert!(
            bp_eq.should_break(&vars),
            "Should break when count (5) <= 5 (equal to threshold)"
        );

        // Test case 3: <= with value GREATER than threshold (should NOT break)
        vars.insert("count".to_string(), "10".to_string());
        let bp_gt = Breakpoint::with_condition(5, "$count <= 5".to_string());
        assert!(
            !bp_gt.should_break(&vars),
            "Should NOT break when count (10) <= 5 (greater than threshold)"
        );

        // Test case 4: <= with multi-digit value (tests parsing of value_part)
        vars.insert("count".to_string(), "50".to_string());
        let bp_large = Breakpoint::with_condition(5, "$count <= 99".to_string());
        assert!(
            bp_large.should_break(&vars),
            "Should break when count (50) <= 99 (multi-digit value parsing)"
        );

        // Test case 5: <= boundary case (exactly at boundary)
        vars.insert("count".to_string(), "0".to_string());
        let bp_zero = Breakpoint::with_condition(5, "$count <= 0".to_string());
        assert!(
            bp_zero.should_break(&vars),
            "Should break when count (0) <= 0 (zero boundary)"
        );

        // Test case 6: <= with negative comparison (edge case)
        vars.insert("count".to_string(), "-5".to_string());
        let bp_neg = Breakpoint::with_condition(5, "$count <= 0".to_string());
        assert!(
            bp_neg.should_break(&vars),
            "Should break when count (-5) <= 0 (negative value)"
        );
    }

    /// Test: REPL-007-MUTATION-004 - Less than (<) operator boundary conditions
    ///
    /// This test catches the mutant at line 277: replace `<` with `<=`
    ///
    /// RED phase: Write comprehensive boundary test for < operator
    #[test]
    fn test_REPL_007_MUTATION_004_less_than_boundary() {
        let mut vars = HashMap::new();

        // Test case 1: < with value LESS than threshold (should break)
        vars.insert("count".to_string(), "3".to_string());
        let bp_lt = Breakpoint::with_condition(5, "$count < 5".to_string());
        assert!(
            bp_lt.should_break(&vars),
            "Should break when count (3) < 5 (less than threshold)"
        );

        // Test case 2: < with value EQUAL to threshold (should NOT break)
        // This is the KEY boundary test that catches the mutant `<` → `<=`
        vars.insert("count".to_string(), "5".to_string());
        let bp_eq = Breakpoint::with_condition(5, "$count < 5".to_string());
        assert!(
            !bp_eq.should_break(&vars),
            "Should NOT break when count (5) < 5 (EQUAL to threshold) - catches `<` → `<=` mutant"
        );

        // Test case 3: < with value GREATER than threshold (should NOT break)
        vars.insert("count".to_string(), "10".to_string());
        let bp_gt = Breakpoint::with_condition(5, "$count < 5".to_string());
        assert!(
            !bp_gt.should_break(&vars),
            "Should NOT break when count (10) < 5 (greater than threshold)"
        );

        // Test case 4: < with multi-digit values (tests value parsing)
        vars.insert("count".to_string(), "50".to_string());
        let bp_large = Breakpoint::with_condition(5, "$count < 99".to_string());
        assert!(
            bp_large.should_break(&vars),
            "Should break when count (50) < 99 (multi-digit value parsing)"
        );

        // Test case 5: < boundary at zero
        vars.insert("count".to_string(), "0".to_string());
        let bp_zero = Breakpoint::with_condition(5, "$count < 0".to_string());
        assert!(
            !bp_zero.should_break(&vars),
            "Should NOT break when count (0) < 0 (zero boundary - equal case)"
        );
    }

    /// Test: REPL-007-002-002 - Conditional breakpoint evaluates to false
    #[test]
    fn test_REPL_007_002_conditional_false() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "5".to_string());

        // Create conditional breakpoint: break if $count > 10
        let bp = Breakpoint::with_condition(5, "$count > 10".to_string());

        assert!(
            !bp.should_break(&vars),
            "Should not break when count (5) <= 10"
        );
    }

    /// Test: REPL-007-002-003 - Conditional breakpoint with invalid syntax
    #[test]
    fn test_REPL_007_002_conditional_invalid() {
        let vars = HashMap::new();

        // Invalid condition (missing variable)
        let bp1 = Breakpoint::with_condition(5, "$missing > 10".to_string());
        assert!(
            !bp1.should_break(&vars),
            "Should not break with missing variable"
        );

        // Invalid condition (bad syntax)
        let bp2 = Breakpoint::with_condition(5, "invalid syntax".to_string());
        assert!(
            !bp2.should_break(&vars),
            "Should not break with invalid syntax"
        );
    }

    /// Test: REPL-007-002-004 - String equality condition
    #[test]
    fn test_REPL_007_002_string_equality() {
        let mut vars = HashMap::new();
        vars.insert("status".to_string(), "running".to_string());

        // Test == operator
        let bp_eq = Breakpoint::with_condition(5, "$status == running".to_string());
        assert!(
            bp_eq.should_break(&vars),
            "Should break when status == running"
        );

        // Test != operator
        let bp_ne = Breakpoint::with_condition(5, "$status != stopped".to_string());
        assert!(
            bp_ne.should_break(&vars),
            "Should break when status != stopped"
        );

        // Test != operator (false case)
        let bp_ne_false = Breakpoint::with_condition(5, "$status != running".to_string());
        assert!(
            !bp_ne_false.should_break(&vars),
            "Should not break when status == running (but checking !=)"
        );
    }

    /// Test: REPL-007-002-005 - Less than comparison
    #[test]
    fn test_REPL_007_002_less_than() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "5".to_string());

        // Test < operator (true)
        let bp_lt = Breakpoint::with_condition(5, "$count < 10".to_string());
        assert!(
            bp_lt.should_break(&vars),
            "Should break when count (5) < 10"
        );

        // Test < operator (false)
        let bp_lt_false = Breakpoint::with_condition(5, "$count < 3".to_string());
        assert!(
            !bp_lt_false.should_break(&vars),
            "Should not break when count (5) >= 3"
        );
    }

    /// Test: REPL-007-002-006 - Disabled conditional breakpoint
    #[test]
    fn test_REPL_007_002_disabled_conditional() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "15".to_string());

        // Create and disable conditional breakpoint
        let mut bp = Breakpoint::with_condition(5, "$count > 10".to_string());
        bp.disable();

        assert!(
            !bp.should_break(&vars),
            "Disabled breakpoint should not trigger even if condition is true"
        );
    }

    // ===== HIT-COUNT BREAKPOINTS (REPL-007-003) =====

    /// Test: REPL-007-003-001 - Hit-count breakpoint triggers after threshold
    #[test]
    fn test_REPL_007_003_hit_count_trigger() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 3); // Break after 3 hits

        // First two hits should not trigger
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break on hit 1"
        );
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break on hit 2"
        );

        // Third hit should trigger
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 3");

        // Subsequent hits should also trigger
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 4");
    }

    /// Test: REPL-007-003-002 - Hit-count breakpoint does not trigger before threshold
    #[test]
    fn test_REPL_007_003_hit_count_not_reached() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 5); // Break after 5 hits

        // First 4 hits should not trigger
        for i in 1..=4 {
            assert!(
                !bp.should_break_with_hit(&vars),
                "Should not break on hit {}",
                i
            );
        }

        // Fifth hit should trigger
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 5");
    }

    /// Test: REPL-007-003-003 - Hit-count resets correctly
    #[test]
    fn test_REPL_007_003_hit_count_reset() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 2); // Break after 2 hits

        // Hit twice
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break on hit 1"
        );
        assert!(bp.should_break_with_hit(&vars), "Should break on hit 2");

        // Reset hit count
        bp.reset_hit_count();

        // Should not trigger on first hit after reset
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break after reset"
        );
        assert!(
            bp.should_break_with_hit(&vars),
            "Should break on hit 2 after reset"
        );
    }

    /// Test: REPL-007-003-004 - Hit-count with condition (both must be true)
    #[test]
    fn test_REPL_007_003_hit_count_with_condition() {
        let mut vars = HashMap::new();
        vars.insert("count".to_string(), "15".to_string());

        let mut bp = Breakpoint::with_hit_count_and_condition(5, 2, "$count > 10".to_string());

        // First hit: condition true, but hit count not reached
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break: condition true but hit count = 1"
        );

        // Second hit: condition true and hit count reached
        assert!(
            bp.should_break_with_hit(&vars),
            "Should break: condition true and hit count = 2"
        );
    }

    /// Test: REPL-007-003-005 - Get current hit count
    #[test]
    fn test_REPL_007_003_get_hit_count() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 3);

        assert_eq!(bp.get_hit_count(), 0, "Initial hit count should be 0");

        bp.should_break_with_hit(&vars);
        assert_eq!(
            bp.get_hit_count(),
            1,
            "Hit count should be 1 after first hit"
        );

        bp.should_break_with_hit(&vars);
        assert_eq!(
            bp.get_hit_count(),
            2,
            "Hit count should be 2 after second hit"
        );

        bp.should_break_with_hit(&vars);
        assert_eq!(
            bp.get_hit_count(),
            3,
            "Hit count should be 3 after third hit"
        );
    }

    /// Test: REPL-007-003-006 - Disabled hit-count breakpoint never triggers
    #[test]
    fn test_REPL_007_003_disabled_hit_count() {
        let vars = HashMap::new();
        let mut bp = Breakpoint::with_hit_count(5, 2);
        bp.disable();

        // Even after reaching threshold, disabled breakpoint should not trigger
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break when disabled"
        );
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break when disabled"
        );
        assert!(
            !bp.should_break_with_hit(&vars),
            "Should not break when disabled"
        );
    }
}
