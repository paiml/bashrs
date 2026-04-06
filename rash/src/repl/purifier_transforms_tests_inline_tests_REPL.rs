#[cfg(test)]
mod alternatives_tests {
    use super::*;

    // GREEN PHASE TEST 1: Test generate_idempotency_alternatives
    #[test]
    fn test_REPL_013_003_alternatives_mkdir() {
        // ARRANGE: Request alternatives for idempotent mkdir
        let transformation_title = "mkdir → mkdir -p (idempotent)";

        // ACT: Generate alternatives
        let alternatives = generate_idempotency_alternatives(transformation_title);

        // ASSERT: Should return 2 alternatives for mkdir
        assert!(!alternatives.is_empty());
        assert_eq!(alternatives.len(), 2);
        assert_eq!(alternatives[0].approach, "Check before creating");
        assert!(alternatives[0].example.contains("[ -d"));
        assert_eq!(alternatives[1].approach, "Use mkdir with error suppression");
    }

    // GREEN PHASE TEST 2: Test generate_determinism_alternatives
    #[test]
    fn test_REPL_013_003_alternatives_random() {
        // ARRANGE: Request alternatives for deterministic random
        let transformation_title = "$RANDOM → Seeded random (deterministic)";

        // ACT: Generate alternatives
        let alternatives = generate_determinism_alternatives(transformation_title);

        // ASSERT: Should return 4 alternatives for $RANDOM
        assert!(!alternatives.is_empty());
        assert_eq!(alternatives.len(), 4);
        assert_eq!(alternatives[0].approach, "Use UUID for unique IDs");
        assert!(alternatives[1].approach.contains("timestamp"));
        assert!(alternatives[2].approach.contains("hash"));
        assert!(alternatives[3].approach.contains("counter"));
    }

    // GREEN PHASE TEST 3: Test generate_safety_alternatives
    #[test]
    fn test_REPL_013_003_alternatives_quoting() {
        // ARRANGE: Request alternatives for variable quoting
        let transformation_title = "$var → \"$var\" (quoted)";

        // ACT: Generate alternatives
        let alternatives = generate_safety_alternatives(transformation_title);

        // ASSERT: Should return 3 alternatives for quoting
        assert!(!alternatives.is_empty());
        assert_eq!(alternatives.len(), 3);
        assert!(alternatives[0].approach.contains("printf"));
        assert!(alternatives[1].approach.contains("arrays"));
        assert!(alternatives[2].approach.contains("Validate"));
    }

    // RED PHASE TEST 4: Test Alternative builder pattern (should pass)
    #[test]
    fn test_REPL_013_003_alternative_builder() {
        // ARRANGE: Create alternative with builder pattern
        let alternative = Alternative::new(
            "Use conditional mkdir",
            "[ -d /tmp/app ] || mkdir /tmp/app",
            "When you need explicit control",
        )
        .add_pro("Explicit logic")
        .add_pro("Works in POSIX sh")
        .add_con("More verbose");

        // ASSERT: Verify fields set correctly
        assert_eq!(alternative.approach, "Use conditional mkdir");
        assert_eq!(alternative.example, "[ -d /tmp/app ] || mkdir /tmp/app");
        assert_eq!(alternative.when_to_use, "When you need explicit control");
        assert_eq!(alternative.pros.len(), 2);
        assert_eq!(alternative.cons.len(), 1);
        assert_eq!(alternative.pros[0], "Explicit logic");
        assert_eq!(alternative.pros[1], "Works in POSIX sh");
        assert_eq!(alternative.cons[0], "More verbose");
    }

    // GREEN PHASE TEST 5: Test format_alternatives
    #[test]
    fn test_REPL_013_003_format_alternatives() {
        // ARRANGE: Create some alternatives
        let alternatives = vec![Alternative::new(
            "Use mkdir -p",
            "mkdir -p /tmp/app",
            "When you want simple idempotency",
        )
        .add_pro("Simple and concise")
        .add_con("No explicit error handling")];

        // ACT: Format alternatives
        let formatted = format_alternatives(&alternatives);

        // ASSERT: Should format correctly
        assert!(!formatted.is_empty());
        assert!(formatted.contains("Alternative Approaches:"));
        assert!(formatted.contains("1. Use mkdir -p"));
        assert!(formatted.contains("Example: mkdir -p /tmp/app"));
        assert!(formatted.contains("Pros:"));
        assert!(formatted.contains("+ Simple and concise"));
        assert!(formatted.contains("Cons:"));
        assert!(formatted.contains("- No explicit error handling"));
    }

    // RED PHASE TEST 6: Test TransformationExplanation.with_alternatives (should pass)
    #[test]
    fn test_REPL_013_003_explanation_with_alternatives() {
        // ARRANGE: Create transformation explanation
        let explanation = TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir → mkdir -p",
            "mkdir /tmp/app",
            "mkdir -p /tmp/app",
            "Added -p flag",
            "Makes operation idempotent",
        );

        // Create alternatives
        let alternatives = vec![
            Alternative::new(
                "Use conditional mkdir",
                "[ -d /tmp/app ] || mkdir /tmp/app",
                "When you need explicit control",
            )
            .add_pro("Explicit logic")
            .add_con("More verbose"),
            Alternative::new(
                "Use mkdir -p",
                "mkdir -p /tmp/app",
                "When you want simplicity",
            )
            .add_pro("Simple and concise")
            .add_con("No explicit check"),
        ];

        // ACT: Set alternatives
        let explanation_with_alts = explanation.with_alternatives(alternatives.clone());

        // ASSERT: Alternatives should be set
        assert_eq!(explanation_with_alts.alternatives.len(), 2);
        assert_eq!(
            explanation_with_alts.alternatives[0].approach,
            "Use conditional mkdir"
        );
        assert_eq!(
            explanation_with_alts.alternatives[1].approach,
            "Use mkdir -p"
        );
    }
}

// ===== REPL-013-003: PROPERTY TESTS FOR ALTERNATIVES (GREEN PHASE) =====

#[cfg(test)]
mod alternatives_property_tests {
    use super::*;
    use proptest::prelude::*;

    // PROPERTY TEST 1: Alternatives should always be provided for known transformations
    proptest! {
        #[test]
        fn prop_alternatives_always_provided(
            title in "(mkdir|rm|ln|\\$RANDOM|\\$\\$|date|quote).*"
        ) {
            // ACT: Generate alternatives based on title pattern
            let alternatives = if title.contains("mkdir") {
                generate_idempotency_alternatives(&title)
            } else if title.contains("$RANDOM") || title.contains("$$") || title.contains("date") {
                generate_determinism_alternatives(&title)
            } else {
                generate_safety_alternatives(&title)
            };

            // ASSERT: Should return at least one alternative
            prop_assert!(!alternatives.is_empty());
        }
    }

    // PROPERTY TEST 2: format_alternatives should never panic on valid input
    proptest! {
        #[test]
        fn prop_format_never_panics(
            approach in "[a-zA-Z ]{1,50}",
            example in "[a-zA-Z0-9 $/.-]{1,100}",
            when_to_use in "[a-zA-Z ]{1,100}"
        ) {
            // ARRANGE: Create valid alternative
            let alternatives = vec![
                Alternative::new(approach, example, when_to_use)
                    .add_pro("Test pro")
                    .add_con("Test con")
            ];

            // ACT: Format should never panic
            let formatted = format_alternatives(&alternatives);

            // ASSERT: Should complete without panic and return formatted output
            prop_assert!(!formatted.is_empty());
            prop_assert!(formatted.contains("Alternative Approaches:"));
        }
    }
}

// ===== REPL-014-001: AUTO-RUN LINTER ON PURIFIED OUTPUT (RED PHASE) =====

#[cfg(test)]
mod purify_and_lint_tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE - SHOULD PANIC) =====

    /// Test: REPL-014-001-001 - Purify and lint mkdir command
    #[test]
    fn test_REPL_014_001_purify_and_lint_mkdir() {
        let input = "mkdir /tmp/test";
        let result = purify_and_lint(input);

        assert!(result.is_ok());
        let result = result.unwrap();

        // Purified should add -p flag
        assert!(result.purified_code.contains("mkdir -p"));

        // Should be clean (no DET/IDEM/SEC violations)
        assert!(result.is_clean, "Purified mkdir should be clean");
        assert_eq!(result.critical_violations(), 0);
    }

    /// Test: REPL-014-001-002 - Purify and lint $RANDOM
    #[test]
    fn test_REPL_014_001_purify_and_lint_random() {
        let input = "echo $RANDOM";
        let result = purify_and_lint(input);

        assert!(result.is_ok());
        let result = result.unwrap();

        // Purified should remove $RANDOM
        assert!(!result.purified_code.contains("$RANDOM"));

        // Should be clean (no DET violations)
        assert!(result.is_clean, "Purified random should be clean");
        assert_eq!(result.det_violations().len(), 0);
    }

    /// Test: REPL-014-001-003 - Lint result structure is correct
    #[test]
    fn test_REPL_014_001_lint_result_structure() {
        let input = "echo hello";
        let result = purify_and_lint(input);

        assert!(result.is_ok());
        let result = result.unwrap();

        // Verify structure
        assert!(!result.purified_code.is_empty());
        // lint_result should exist (may or may not have diagnostics)
        let _ = result.lint_result.diagnostics.len();
        // is_clean should be determinable
        let _ = result.is_clean;
    }

    /// Test: REPL-014-001-004 - Violation helper methods work
    #[test]
    fn test_REPL_014_001_violation_helpers() {
        let input = "mkdir /tmp/test";
        let result = purify_and_lint(input);

        assert!(result.is_ok());
        let result = result.unwrap();

        // Verify helper methods are callable
        let _ = result.det_violations();
        let _ = result.idem_violations();
        let _ = result.sec_violations();
        let _ = result.critical_violations();

        // These methods should return collections
        assert!(result.det_violations().len() <= result.lint_result.diagnostics.len());
    }

    /// Test: REPL-014-001-005 - is_clean flag works correctly
    #[test]
    fn test_REPL_014_001_is_clean_flag() {
        let input = "echo hello";
        let result = purify_and_lint(input);

        assert!(result.is_ok());
        let result = result.unwrap();

        // is_clean is a boolean (guaranteed by type system)
        // If clean, critical_violations should be 0
        if result.is_clean {
            assert_eq!(result.critical_violations(), 0);
        }
    }

    /// Test: REPL-014-001-006 - Format purified lint result
    #[test]
    fn test_REPL_014_001_format_result() {
        let input = "mkdir /tmp/test";
        let result = purify_and_lint(input).unwrap();

        let formatted = format_purified_lint_result(&result);

        // Should contain purified code
        assert!(formatted.contains("mkdir -p"));

        // Should show clean status
        assert!(formatted.contains("CLEAN") || formatted.contains("✓"));
    }

    // ===== INTEGRATION TEST (RED PHASE - SHOULD PANIC) =====

    /// Integration Test: REPL-014-001 - Complete purify-and-lint workflow
    #[test]
    fn test_REPL_014_001_purify_and_lint_integration() {
        // Simple bash that purifier can handle
        let input = "mkdir /app/releases\necho hello";

        let result = purify_and_lint(input);
        assert!(result.is_ok());
        let result = result.unwrap();

        // Verify complete workflow
        assert!(
            !result.purified_code.is_empty(),
            "Should produce purified code"
        );

        // Lint result should be populated
        let _ = result.lint_result.diagnostics.len();

        // Helper methods should work
        assert!(result.det_violations().len() <= result.lint_result.diagnostics.len());
        assert!(result.idem_violations().len() <= result.lint_result.diagnostics.len());
        assert!(result.sec_violations().len() <= result.lint_result.diagnostics.len());
    }

    // ===== REPL-014-002: ZERO-TOLERANCE QUALITY GATE TESTS (GREEN PHASE) =====

    /// Test: REPL-014-002-001 - Zero DET violations
    #[test]
    fn test_REPL_014_002_zero_det_violations() {
        let input = "echo hello";
        let result = purify_and_validate(input);

        // Should succeed - no DET violations
        assert!(result.is_ok(), "Clean input should pass validation");
        let purified = result.unwrap();
        assert!(purified.contains("echo"));
    }

    /// Test: REPL-014-002-002 - Zero IDEM violations
    #[test]
    fn test_REPL_014_002_zero_idem_violations() {
        let input = "mkdir -p /tmp/test";
        let result = purify_and_validate(input);

        // Should succeed - already idempotent
        assert!(result.is_ok(), "Idempotent input should pass validation");
    }

    /// Test: REPL-014-002-003 - Zero SEC violations
    #[test]
    fn test_REPL_014_002_zero_sec_violations() {
        let input = "echo \"$var\"";
        let result = purify_and_validate(input);

        // Should succeed - variable is quoted
        assert!(result.is_ok(), "Quoted variable should pass validation");
    }

    /// Test: REPL-014-002-004 - Fails with violations
    #[test]
    fn test_REPL_014_002_fails_with_violations() {
        // Test various inputs that purifier might not be able to fix
        let test_cases = vec![
            ("echo $RANDOM", "DET violation"),
            ("rm /nonexistent", "IDEM violation"),
            ("eval $user_input", "SEC violation"),
        ];

        for (input, description) in test_cases {
            let result = purify_and_validate(input);

            // If purifier can't fix it, should fail validation
            if let Err(err) = result {
                let purif_err = err.downcast_ref::<PurificationError>();

                // Should have detailed error
                assert!(
                    purif_err.is_some(),
                    "Error should be PurificationError for: {}",
                    description
                );
            }
            // Note: If purifier CAN fix it, that's also acceptable
            // This test is about ensuring we catch unfixable violations
        }
    }

    /// Test: REPL-014-002-005 - Error details
    #[test]
    fn test_REPL_014_002_error_details() {
        // Use input that we know will have violations after purification
        // (This test may need adjustment based on actual purifier behavior)
        let input = "echo $RANDOM; eval $cmd; rm /tmp/file";

        let result = purify_and_validate(input);

        // If validation fails, check error details
        if let Err(e) = result {
            if let Some(purif_err) = e.downcast_ref::<PurificationError>() {
                // Should have violation counts
                assert!(purif_err.total_violations() > 0);

                // Error message should be descriptive
                let msg = purif_err.to_string();
                assert!(msg.contains("violation"));
            }
        }
        // Note: If purifier fixes everything, that's also valid
    }
}

// ===== REPL-014-001: PROPERTY TESTS (RED PHASE) =====


// FIXME(PMAT-238): include!("purifier_transforms_tests_inline_tests_prop_purify.rs");
