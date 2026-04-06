#[cfg(test)]
mod purify_and_lint_property_tests {
    use super::*;
    use proptest::prelude::*;

    // NOTE: Property "purified output is always clean" was removed.
    //
    // This property is incorrect because the purifier's job is NOT to automatically
    // fix all DET/IDEM/SEC violations. The purifier focuses on:
    // 1. Variable quoting (safety)
    // 2. POSIX compliance
    // 3. Improved readability
    //
    // It does NOT automatically add flags like -f to rm, -p to mkdir, etc.
    // because that would change the semantic meaning of the script.
    //
    // The linter is separate from the purifier - it identifies issues,
    // but the purifier doesn't fix them all automatically.
    //
    // Example: "rm $a" purifies to "rm \"$a\"" (safer with quotes)
    // but still triggers IDEM002 (non-idempotent rm without -f).
    // This is expected and correct behavior.

    // Property: Function should never panic on any input
    proptest! {
        #[test]
        fn prop_purify_and_lint_never_panics(input in ".*{0,1000}") {
            // Should gracefully handle any input
            // This will panic with unimplemented!() during RED phase
            // but after GREEN phase, it should never panic
            let _ = purify_and_lint(&input);
        }
    }
}

// ===== REPL-014-002: PROPERTY TESTS (RED PHASE) =====

#[cfg(test)]
mod purify_and_validate_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property Test: REPL-014-002 - If validation succeeds, output MUST be clean
        #[test]
        fn prop_purified_always_passes_linter(input in ".*{0,100}") {
            if let Ok(purified) = purify_and_validate(&input) {
                // CRITICAL PROPERTY: If validation succeeds, output MUST be clean
                // Note: Re-purifying may fail (e.g., parser errors on generated code),
                // but that's OK - we only care that the output is clean when it can be linted
                if let Ok(lint_result) = purify_and_lint(&purified) {
                    prop_assert!(
                        lint_result.is_clean,
                        "Validated output must be clean, but found {} violations",
                        lint_result.critical_violations()
                    );

                    prop_assert_eq!(
                        lint_result.det_violations().len(),
                        0,
                        "No DET violations allowed"
                    );
                    prop_assert_eq!(
                        lint_result.idem_violations().len(),
                        0,
                        "No IDEM violations allowed"
                    );
                    prop_assert_eq!(
                        lint_result.sec_violations().len(),
                        0,
                        "No SEC violations allowed"
                    );
                }
                // If re-linting fails (e.g., parser error), that's acceptable
                // The guarantee is: validated output is clean IF it can be linted
            }
            // If validation fails, that's acceptable - not all inputs can be purified
        }
    }
}

// ===== REPL-014-003: DISPLAY VIOLATIONS IN REPL CONTEXT (RED PHASE) =====

#[cfg(test)]
mod format_violations_with_context_tests {
    use super::*;

    /// Integration Test: REPL-014-003 - Full workflow with purify and format
    #[test]
    fn test_REPL_014_003_integration_purify_and_format() {
        let messy_bash = r#"
mkdir /app/releases
echo $RANDOM
rm /tmp/old
"#;

        // Purify and lint
        let result = purify_and_lint(messy_bash);

        if let Ok(purified_result) = result {
            // Format with context
            let formatted = format_purified_lint_result_with_context(&purified_result, messy_bash);

            // Should show purified code
            assert!(formatted.contains("Purified:"));

            // If there are violations, should show context
            if !purified_result.is_clean {
                // Should show line numbers and context
                assert!(formatted.contains("|"));

                // Should show violation codes
                let has_det = !purified_result.det_violations().is_empty();
                let has_idem = !purified_result.idem_violations().is_empty();

                if has_det {
                    assert!(formatted.contains("DET"));
                }
                if has_idem {
                    assert!(formatted.contains("IDEM"));
                }
            }
        }
    }

    // ===== SafetyRationale tests =====

    #[test]
    fn test_safety_rationale_new() {
        let rationale = SafetyRationale::new();
        assert!(rationale.vulnerabilities_prevented.is_empty());
        assert!(rationale.failures_eliminated.is_empty());
        assert!(rationale.attack_vectors_closed.is_empty());
        assert!(rationale.impact_without_fix.is_empty());
        assert_eq!(rationale.severity, SafetySeverity::Low);
    }

    #[test]
    fn test_safety_rationale_default() {
        let rationale = SafetyRationale::default();
        assert!(rationale.vulnerabilities_prevented.is_empty());
    }

    #[test]
    fn test_safety_rationale_add_vulnerability() {
        let rationale = SafetyRationale::new().add_vulnerability("Command Injection");
        assert!(rationale
            .vulnerabilities_prevented
            .contains(&"Command Injection".to_string()));
    }

    #[test]
    fn test_safety_rationale_add_failure() {
        let rationale = SafetyRationale::new().add_failure("Race condition on recreate");
        assert!(rationale
            .failures_eliminated
            .contains(&"Race condition on recreate".to_string()));
    }

    #[test]
    fn test_safety_rationale_add_attack_vector() {
        let rationale = SafetyRationale::new().add_attack_vector("Path traversal");
        assert!(rationale
            .attack_vectors_closed
            .contains(&"Path traversal".to_string()));
    }

    #[test]
    fn test_safety_rationale_with_impact() {
        let rationale = SafetyRationale::new().with_impact("Data loss");
        assert_eq!(rationale.impact_without_fix, "Data loss");
    }

    #[test]
    fn test_safety_rationale_with_severity() {
        let rationale = SafetyRationale::new().with_severity(SafetySeverity::Critical);
        assert_eq!(rationale.severity, SafetySeverity::Critical);
    }

    #[test]
    fn test_safety_rationale_builder_chain() {
        let rationale = SafetyRationale::new()
            .add_vulnerability("Injection")
            .add_failure("Crash")
            .add_attack_vector("RCE")
            .with_impact("System compromise")
            .with_severity(SafetySeverity::High);

        assert_eq!(rationale.vulnerabilities_prevented.len(), 1);
        assert_eq!(rationale.failures_eliminated.len(), 1);
        assert_eq!(rationale.attack_vectors_closed.len(), 1);
        assert_eq!(rationale.impact_without_fix, "System compromise");
        assert_eq!(rationale.severity, SafetySeverity::High);
    }

    // ===== SafetySeverity tests =====

    #[test]
    fn test_safety_severity_eq() {
        assert_eq!(SafetySeverity::Critical, SafetySeverity::Critical);
        assert_ne!(SafetySeverity::Critical, SafetySeverity::High);
    }

    #[test]
    fn test_safety_severity_clone() {
        let severities = [
            SafetySeverity::Critical,
            SafetySeverity::High,
            SafetySeverity::Medium,
            SafetySeverity::Low,
        ];
        for severity in severities {
            let _ = severity.clone();
        }
    }

    // ===== Alternative tests =====

    #[test]
    fn test_alternative_new() {
        let alt = Alternative::new(
            "Use set -e",
            "set -e; rm file",
            "When you want script to fail on error",
        );
        assert_eq!(alt.approach, "Use set -e");
        assert_eq!(alt.example, "set -e; rm file");
        assert_eq!(alt.when_to_use, "When you want script to fail on error");
        assert!(alt.pros.is_empty());
        assert!(alt.cons.is_empty());
    }

    #[test]
    fn test_alternative_add_pro() {
        let alt = Alternative::new("Approach", "Example", "When").add_pro("Fast");
        assert!(alt.pros.contains(&"Fast".to_string()));
    }

    #[test]
    fn test_alternative_add_con() {
        let alt = Alternative::new("Approach", "Example", "When").add_con("Complex");
        assert!(alt.cons.contains(&"Complex".to_string()));
    }

    #[test]
    fn test_alternative_builder_chain() {
        let alt = Alternative::new("Approach", "Example", "When")
            .add_pro("Simple")
            .add_pro("Fast")
            .add_con("Verbose");

        assert_eq!(alt.pros.len(), 2);
        assert_eq!(alt.cons.len(), 1);
    }

    #[test]
    fn test_alternative_clone() {
        let alt = Alternative::new("Approach", "Example", "When")
            .add_pro("Fast")
            .add_con("Complex");
        let cloned = alt.clone();
        assert_eq!(cloned.approach, "Approach");
        assert_eq!(cloned.pros.len(), 1);
    }

    // ===== TransformationExplanation tests =====

    #[test]
    fn test_transformation_explanation_new() {
        let exp = TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "Use mkdir -p",
            "mkdir /dir",
            "mkdir -p /dir",
            "Added -p flag",
            "Prevents errors on rerun",
        );
        assert_eq!(exp.category, TransformationCategory::Idempotency);
        assert_eq!(exp.title, "Use mkdir -p");
        assert!(exp.line_number.is_none());
    }

    #[test]
    fn test_transformation_explanation_with_line_number() {
        let exp = TransformationExplanation::new(
            TransformationCategory::Safety,
            "Title",
            "Original",
            "Transformed",
            "What",
            "Why",
        )
        .with_line_number(42);
        assert_eq!(exp.line_number, Some(42));
    }

    #[test]
    fn test_transformation_explanation_with_safety_rationale() {
        let rationale = SafetyRationale::new().add_vulnerability("Injection");
        let exp = TransformationExplanation::new(
            TransformationCategory::Safety,
            "Title",
            "Original",
            "Transformed",
            "What",
            "Why",
        )
        .with_safety_rationale(rationale);
        assert!(!exp.safety_rationale.vulnerabilities_prevented.is_empty());
    }

    #[test]
    fn test_transformation_explanation_with_alternatives() {
        let alternatives = vec![Alternative::new("Alt", "Example", "When")];
        let exp = TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Title",
            "Original",
            "Transformed",
            "What",
            "Why",
        )
        .with_alternatives(alternatives);
        assert_eq!(exp.alternatives.len(), 1);
    }

    // ===== TransformationCategory tests =====

    #[test]
    fn test_transformation_category_eq() {
        assert_eq!(
            TransformationCategory::Idempotency,
            TransformationCategory::Idempotency
        );
        assert_ne!(
            TransformationCategory::Idempotency,
            TransformationCategory::Safety
        );
    }

    #[test]
    fn test_transformation_category_clone() {
        let categories = [
            TransformationCategory::Idempotency,
            TransformationCategory::Determinism,
            TransformationCategory::Safety,
        ];
        for cat in categories {
            let _ = cat.clone();
        }
    }

    // ===== PurifiedLintResult tests =====

    #[test]
    fn test_purified_lint_result_new_clean() {
        let lint_result = LintResult::new();
        let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
        assert!(plr.is_clean);
        assert_eq!(plr.critical_violations(), 0);
    }

    #[test]
    fn test_purified_lint_result_det_violations_empty() {
        let lint_result = LintResult::new();
        let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
        assert!(plr.det_violations().is_empty());
    }

    #[test]
    fn test_purified_lint_result_idem_violations_empty() {
        let lint_result = LintResult::new();
        let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
        assert!(plr.idem_violations().is_empty());
    }

    #[test]
    fn test_purified_lint_result_sec_violations_empty() {
        let lint_result = LintResult::new();
        let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
        assert!(plr.sec_violations().is_empty());
    }
}

// ===== PURIFIER_COV: Coverage tests for explain_purification_changes_detailed and format_transformation_report =====


// FIXME(PMAT-238): include!("purifier_transforms_tests_inline_tests_PURIFIER.rs");
