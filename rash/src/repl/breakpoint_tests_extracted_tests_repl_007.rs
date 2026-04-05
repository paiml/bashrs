
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

