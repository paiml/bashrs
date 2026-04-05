//! Tests extracted from debugger.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::repl::debugger::*;

// ===== UNIT TESTS (RED PHASE) =====

/// Test: REPL-008-001-001 - Create debug session from script
    #[test]
    fn prop_filter_env_matches_prefix(
        prefix in "[A-Z_]{1,5}"
    ) {
        let script = "echo test";
        let session = DebugSession::new(script);

        let filtered = session.filter_env(&prefix);

        // All results must start with prefix
        for (name, _) in &filtered {
            prop_assert!(
                name.starts_with(&prefix),
                "Variable {} should start with prefix {}",
                name,
                prefix
            );
        }

        // Verify sorted order
        let mut sorted = filtered.clone();
        sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        prop_assert_eq!(filtered.clone(), sorted, "filter_env should return sorted results");

        // Verify determinism
        let second_filtered = session.filter_env(&prefix);
        prop_assert_eq!(filtered, second_filtered, "filter_env should be deterministic");
    }
}

// ===== REPL-009-003: Call Stack Tracking Property Tests =====

// Property: Call stack depth equals number of pushes minus pops
proptest! {
    #[test]
    fn prop_call_stack_depth_correct(
        num_pushes in 0usize..10,
        num_pops in 0usize..10
    ) {
        let script = "echo test";
        let mut session = DebugSession::new(script);

        // Initially has 1 frame (<main>)
        prop_assert_eq!(session.call_stack().len(), 1);

        // Push N frames
        for i in 0..num_pushes {
            session.push_frame(format!("func{}", i), i);
        }

        // Stack depth should be 1 + num_pushes
        let depth_after_push = session.call_stack().len();
        prop_assert_eq!(depth_after_push, 1 + num_pushes);

        // Pop M times (min(num_pops, num_pushes))
        let actual_pops = std::cmp::min(num_pops, num_pushes);
        for _ in 0..actual_pops {
            session.pop_frame();
        }

        // Stack depth should be 1 + num_pushes - actual_pops
        let expected_depth = 1 + num_pushes - actual_pops;
        let final_depth = session.call_stack().len();
        prop_assert_eq!(final_depth, expected_depth);

        // Try to pop more than available - should never go below 1
        for _ in 0..100 {
            session.pop_frame();
        }
        prop_assert_eq!(session.call_stack().len(), 1, "Stack should never go below 1 (main frame)");
    }
}

// ===== REPL-010-001: Compare Original vs Purified Property Tests =====

// Property: Comparison is deterministic (same result every time)
proptest! {
    #[test]
    fn prop_REPL_010_001_comparison_deterministic(
        cmd in "mkdir|rm|ln",
        path in "/tmp/[a-z]{1,10}"
    ) {
        let script = format!("{} {}", cmd, path);
        let session = DebugSession::new(&script);

        // Get comparison twice
        let first = session.compare_current_line();
        let second = session.compare_current_line();

        // Should be identical
        prop_assert_eq!(first, second, "Comparison should be deterministic");
    }
}

// Property: Comparison correctly identifies differences
proptest! {
    #[test]
    fn prop_REPL_010_001_differs_flag_correct(
        cmd in "mkdir|echo|rm"
    ) {
        let script = format!("{} /tmp/test", cmd);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            // differs flag should match actual string comparison
            let actual_differs = comparison.original != comparison.purified;
            prop_assert_eq!(
                comparison.differs,
                actual_differs,
                "differs flag should match actual comparison"
            );
        }
    }
}

// Property: Format diff highlighting never panics
proptest! {
    #[test]
    fn prop_REPL_010_001_diff_highlighting_valid(
        cmd in "[a-z]{1,10}",
        arg in "[a-z/]{1,20}"
    ) {
        let script = format!("{} {}", cmd, arg);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            // Should not panic
            let diff = session.format_diff_highlighting(&comparison);

            // Should contain original and purified if differs
            if comparison.differs {
                prop_assert!(diff.contains(&comparison.original) || diff.contains(&comparison.purified));
            }
        }
    }
}

// ===== REPL-010-002: Enhanced Highlighting Property Tests =====

// Property: Highlighting output always has valid format (non-empty, no panics)
proptest! {
    #[test]
    fn prop_REPL_010_002_highlight_always_valid_format(
        cmd in "[a-z]{1,10}",
        arg in "[a-z/$]{1,20}"
    ) {
        let script = format!("{} {}", cmd, arg);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            // Should not panic
            let highlighted = session.format_diff_highlighting(&comparison);

            // Should always produce non-empty output
            prop_assert!(!highlighted.is_empty(), "Output should not be empty");

            // Should always contain original line or "no changes"
            prop_assert!(
                highlighted.contains(&comparison.original) || highlighted.contains("no change"),
                "Output should contain original line or 'no changes'"
            );
        }
    }
}

// Property: If differs flag is true, output contains diff markers
proptest! {
    #[test]
    fn prop_REPL_010_002_differs_implies_diff_markers(
        cmd in "mkdir|rm|ln|echo"
    ) {
        let script = format!("{} /tmp/test", cmd);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            let highlighted = session.format_diff_highlighting(&comparison);

            if comparison.differs {
                // Should show diff with - and + markers
                prop_assert!(
                    highlighted.contains('-') && highlighted.contains('+'),
                    "Differing lines should show - and + markers: {}",
                    highlighted
                );
            } else {
                // Should not show diff markers when identical
                prop_assert!(
                    highlighted.contains("no change") || !highlighted.starts_with('-'),
                    "Non-differing lines should not start with -: {}",
                    highlighted
                );
            }
        }
    }
}

// Property: Detected transformations always have explanations
proptest! {
    #[test]
    fn prop_REPL_010_002_transformations_have_explanations(
        cmd in "mkdir|ln"
    ) {
        let script = format!("{} /tmp/test", cmd);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            let highlighted = session.format_diff_highlighting(&comparison);

            // If we detect common transformations (mkdir -p, ln -sf),
            // there should be an explanation
            if comparison.purified.contains("mkdir -p") && !comparison.original.contains("mkdir -p") {
                prop_assert!(
                    highlighted.contains("idempot") || highlighted.contains("idem") || highlighted.contains("-p"),
                    "mkdir -p transformation should have explanation: {}",
                    highlighted
                );
            }

            if comparison.purified.contains("-sf") && !comparison.original.contains("-sf") {
                prop_assert!(
                    highlighted.contains("idempot") || highlighted.contains("idem") || highlighted.contains("-f"),
                    "ln -sf transformation should have explanation: {}",
                    highlighted
                );
            }
        }
    }
}

// ===== REPL-010-003 PROPERTY TESTS =====

// Property: explain_current_line returns None when lines are identical
proptest! {
    #[test]
    fn prop_REPL_010_003_explain_none_when_identical(
        cmd in "mkdir -p|rm -f|ln -sf",
        arg in "[a-z/$]{1,20}"
    ) {
        // Already-purified commands should have no explanation
        let script = format!("{} {}", cmd, arg);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            if !comparison.differs {
                let explanation = session.explain_current_line();
                prop_assert!(
                    explanation.is_none(),
                    "Should have no explanation when lines identical, got: {:?}",
                    explanation
                );
            }
        }
    }
}

// Property: explain_current_line returns Some when lines differ
proptest! {
    #[test]
    fn prop_REPL_010_003_explain_some_when_differs(
        cmd in "mkdir|ln -s"
    ) {
        // Non-idempotent commands should have explanation
        let script = format!("{} /tmp/test", cmd);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            if comparison.differs {
                let explanation = session.explain_current_line();
                prop_assert!(
                    explanation.is_some(),
                    "Should have explanation when lines differ"
                );

                // Should contain non-empty explanation
                let text = explanation.unwrap();
                prop_assert!(
                    !text.is_empty(),
                    "Explanation should not be empty"
                );
            }
        }
    }
}

// Property: explain_current_line never panics on any input
proptest! {
    #[test]
    fn prop_REPL_010_003_explain_never_panics(
        cmd in "[a-z]{1,10}",
        arg in "[a-z/$]{1,20}"
    ) {
        let script = format!("{} {}", cmd, arg);
        let session = DebugSession::new(&script);

        // Should never panic, always return Some or None
        let result = session.explain_current_line();

        // If it returns Some, text should not be empty
        if let Some(text) = result {
            prop_assert!(
                !text.is_empty(),
                "Explanation text should not be empty"
            );
        }
    }
}

// ===== REPL-008-004 PROPERTY TESTS =====

// Property: finish() never increases call depth
proptest! {
    #[test]
    fn prop_REPL_008_004_finish_never_increases_depth(num_lines in 1usize..10) {
        let script = (0..num_lines)
            .map(|i| format!("echo line{}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let mut session = DebugSession::new(&script);

        // Simulate entering a function
        session.push_frame("test_function", 1);
        let initial_depth = session.call_depth();

        // finish() should never increase call depth
        let _ = session.finish();
        let final_depth = session.call_depth();

        prop_assert!(
            final_depth <= initial_depth,
            "finish() should never increase depth: initial={}, final={}",
            initial_depth,
            final_depth
        );
    }
}

// Property: finish() always returns valid result
proptest! {
    #[test]
    fn prop_REPL_008_004_finish_always_valid_result(num_lines in 1usize..10) {
        let script = (0..num_lines)
            .map(|i| format!("echo line{}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let mut session = DebugSession::new(&script);
        let result = session.finish();

        // Result must be one of these two variants
        prop_assert!(
            matches!(result, ContinueResult::Finished) ||
            matches!(result, ContinueResult::BreakpointHit(_)),
            "Result should be Finished or BreakpointHit, got: {:?}",
            result
        );
    }
}

// Property: finish() is deterministic
proptest! {
    #[test]
    fn prop_REPL_008_004_finish_deterministic(num_lines in 1usize..10) {
        let script = (0..num_lines)
            .map(|i| format!("echo line{}", i))
            .collect::<Vec<_>>()
            .join("\n");

        // Run finish() on two identical sessions
        let mut session1 = DebugSession::new(&script);
        let mut session2 = DebugSession::new(&script);

        let result1 = session1.finish();
        let result2 = session2.finish();

        // Should produce identical results
        prop_assert_eq!(result1, result2, "finish() should be deterministic");
        prop_assert_eq!(
            session1.is_finished(),
            session2.is_finished(),
            "Finished state should match"
        );
    }
}
} // mod property_tests
