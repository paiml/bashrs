#[cfg(test)]
mod transformation_explanation_tests {
    use super::*;

    // ===== REPL-013-001: TRANSFORMATION EXPLANATION TESTS (RED PHASE) =====

    #[test]
    fn test_REPL_013_001_transformation_category_display() {
        // ARRANGE: Create categories
        let idempotency = TransformationCategory::Idempotency;
        let determinism = TransformationCategory::Determinism;
        let safety = TransformationCategory::Safety;

        // ASSERT: Categories are distinct
        assert_ne!(idempotency, determinism);
        assert_ne!(determinism, safety);
        assert_ne!(safety, idempotency);
    }

    #[test]
    fn test_REPL_013_001_transformation_explanation_new() {
        // ARRANGE & ACT: Create transformation explanation
        let explanation = TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir -p",
            "mkdir /tmp",
            "mkdir -p /tmp",
            "Added -p flag",
            "Prevents failure if exists",
        );

        // ASSERT: All fields set correctly
        assert_eq!(explanation.category, TransformationCategory::Idempotency);
        assert_eq!(explanation.title, "mkdir -p");
        assert_eq!(explanation.original, "mkdir /tmp");
        assert_eq!(explanation.transformed, "mkdir -p /tmp");
        assert_eq!(explanation.what_changed, "Added -p flag");
        assert_eq!(explanation.why_it_matters, "Prevents failure if exists");
        assert_eq!(explanation.line_number, None);
    }

    #[test]
    fn test_REPL_013_001_transformation_with_line_number() {
        // ARRANGE & ACT: Create with line number
        let explanation = TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            "echo $var",
            "echo \"$var\"",
            "Added quotes",
            "Prevents splitting",
        )
        .with_line_number(42);

        // ASSERT: Line number set
        assert_eq!(explanation.line_number, Some(42));
    }

    #[test]
    fn test_REPL_013_001_explain_mkdir_p_detailed() {
        // ARRANGE: Code that needs mkdir -p
        let original = "mkdir /tmp/test";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect mkdir -p transformation
        assert!(result.is_ok());
        let explanations = result.unwrap();
        assert_eq!(explanations.len(), 1);
        assert_eq!(
            explanations[0].category,
            TransformationCategory::Idempotency
        );
        assert_eq!(explanations[0].title, "mkdir → mkdir -p");
        assert!(explanations[0].what_changed.contains("-p flag"));
    }

    #[test]
    fn test_REPL_013_001_format_empty_report() {
        // ARRANGE: Empty transformations
        let transformations: Vec<TransformationExplanation> = vec![];

        // ACT: Format report
        let report = format_transformation_report(&transformations);

        // ASSERT: Should return "no transformations" message
        assert!(report.contains("No transformations"));
        assert!(report.contains("already purified"));
    }
}

#[cfg(test)]
mod transformation_explanation_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_013_001_explanation_new_never_panics(
            title in ".{0,100}",
            original in ".{0,200}",
            transformed in ".{0,200}",
            what in ".{0,200}",
            why in ".{0,300}",
        ) {
            // Should never panic creating explanations
            let _explanation = TransformationExplanation::new(
                TransformationCategory::Idempotency,
                title,
                original,
                transformed,
                what,
                why
            );
        }

        #[test]
        fn prop_REPL_013_001_format_report_never_panics(
            count in 0usize..10,
        ) {
            let transformations: Vec<TransformationExplanation> = (0..count)
                .map(|i| {
                    TransformationExplanation::new(
                        TransformationCategory::Idempotency,
                        format!("Transform {}", i),
                        "original",
                        "transformed",
                        "what changed",
                        "why it matters"
                    )
                })
                .collect();

            let report = format_transformation_report(&transformations);

            // Should contain result for count cases
            if count == 0 {
                prop_assert!(report.contains("No transformations"));
            } else {
                prop_assert!(report.contains("Transformation Report"));
            }
        }

        #[test]
        fn prop_REPL_013_001_explain_detailed_never_panics(
            input in ".*{0,500}",
        ) {
            // Should never panic on any input
            let _ = explain_purification_changes_detailed(&input);
        }

        #[test]
        fn prop_REPL_013_001_line_numbers_always_positive(
            line in 1usize..1000,
        ) {
            let explanation = TransformationExplanation::new(
                TransformationCategory::Safety,
                "test",
                "a",
                "b",
                "c",
                "d"
            )
            .with_line_number(line);

            prop_assert_eq!(explanation.line_number, Some(line));
        }
    }

    // ===== REPL-013-002: SAFETY RATIONALE TESTS (RED PHASE) =====

    #[cfg(test)]
    mod safety_rationale_tests {
        use super::*;

        #[test]
        fn test_REPL_013_002_safety_idempotency() {
            // ARRANGE: mkdir transformation
            let rationale = generate_idempotency_rationale("mkdir → mkdir -p");

            // ASSERT: Has failure elimination
            assert!(!rationale.failures_eliminated.is_empty());
            assert!(rationale
                .failures_eliminated
                .iter()
                .any(|f| f.contains("already exists")));

            // ASSERT: High severity
            assert_eq!(rationale.severity, SafetySeverity::High);

            // ASSERT: Has impact description
            assert!(rationale.impact_without_fix.contains("re-run"));
        }

        #[test]
        fn test_REPL_013_002_safety_determinism() {
            // ARRANGE: $RANDOM removal
            let rationale = generate_determinism_rationale("Remove $RANDOM");

            // ASSERT: Has vulnerability prevention
            assert!(!rationale.vulnerabilities_prevented.is_empty());
            assert!(rationale
                .vulnerabilities_prevented
                .iter()
                .any(|v| v.contains("reproducible") || v.contains("audit")));

            // ASSERT: Critical severity (reproducibility is critical)
            assert_eq!(rationale.severity, SafetySeverity::Critical);

            // ASSERT: Has impact description
            assert!(rationale.impact_without_fix.contains("unpredictable"));
        }

        #[test]
        fn test_REPL_013_002_safety_injection() {
            // ARRANGE: Variable quoting transformation
            let rationale = generate_safety_rationale("Quote variables");

            // ASSERT: Has vulnerability prevention
            assert!(rationale
                .vulnerabilities_prevented
                .iter()
                .any(|v| v.contains("injection")));

            // ASSERT: Has attack vectors
            assert!(!rationale.attack_vectors_closed.is_empty());
            assert!(rationale
                .attack_vectors_closed
                .iter()
                .any(|a| a.contains("metacharacters") || a.contains("execution")));

            // ASSERT: Critical severity (injection is critical)
            assert_eq!(rationale.severity, SafetySeverity::Critical);

            // ASSERT: Impact mentions attacks
            assert!(
                rationale
                    .impact_without_fix
                    .to_lowercase()
                    .contains("attack")
                    || rationale
                        .impact_without_fix
                        .to_lowercase()
                        .contains("inject")
            );
        }

        #[test]
        fn test_REPL_013_002_rationale_builder() {
            // ARRANGE & ACT: Build rationale with fluent API
            let rationale = SafetyRationale::new()
                .add_vulnerability("SQL injection")
                .add_vulnerability("XSS attack")
                .add_failure("Script crashes")
                .add_attack_vector("Malicious input")
                .with_impact("Data breach")
                .with_severity(SafetySeverity::Critical);

            // ASSERT: All fields populated
            assert_eq!(rationale.vulnerabilities_prevented.len(), 2);
            assert_eq!(rationale.failures_eliminated.len(), 1);
            assert_eq!(rationale.attack_vectors_closed.len(), 1);
            assert_eq!(rationale.impact_without_fix, "Data breach");
            assert_eq!(rationale.severity, SafetySeverity::Critical);
        }

        #[test]
        fn test_REPL_013_002_explanation_with_rationale() {
            // ARRANGE: Create rationale
            let rationale = SafetyRationale::new()
                .add_failure("Non-idempotent")
                .with_severity(SafetySeverity::High);

            // ACT: Add to explanation
            let explanation = TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir -p",
                "mkdir /tmp",
                "mkdir -p /tmp",
                "Added -p",
                "Prevents failure",
            )
            .with_safety_rationale(rationale.clone());

            // ASSERT: Rationale attached
            assert_eq!(explanation.safety_rationale, rationale);
            assert_eq!(explanation.safety_rationale.severity, SafetySeverity::High);
        }

        #[test]
        fn test_REPL_013_002_format_rationale() {
            // ARRANGE: Create rationale
            let rationale = SafetyRationale::new()
                .add_vulnerability("Injection")
                .add_failure("Crash")
                .add_attack_vector("Malicious input")
                .with_impact("Data loss")
                .with_severity(SafetySeverity::Critical);

            // ACT: Format
            let formatted = format_safety_rationale(&rationale);

            // ASSERT: All sections present
            assert!(formatted.contains("CRITICAL"));
            assert!(formatted.contains("Vulnerabilities Prevented"));
            assert!(formatted.contains("Injection"));
            assert!(formatted.contains("Failures Eliminated"));
            assert!(formatted.contains("Crash"));
            assert!(formatted.contains("Attack Vectors Closed"));
            assert!(formatted.contains("Malicious input"));
            assert!(formatted.contains("Impact Without Fix"));
            assert!(formatted.contains("Data loss"));
        }
    }

    // ===== REPL-013-002: SAFETY RATIONALE PROPERTY TESTS =====

    #[cfg(test)]
    mod safety_rationale_property_tests {
        use super::*;
        #[allow(unused_imports)] // Used by proptest! macro
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_REPL_013_002_rationale_builder_never_panics(
                vuln_count in 0usize..5,
                failure_count in 0usize..5,
                attack_count in 0usize..5,
            ) {
                let mut rationale = SafetyRationale::new();

                for i in 0..vuln_count {
                    rationale = rationale.add_vulnerability(format!("vuln_{}", i));
                }

                for i in 0..failure_count {
                    rationale = rationale.add_failure(format!("failure_{}", i));
                }

                for i in 0..attack_count {
                    rationale = rationale.add_attack_vector(format!("attack_{}", i));
                }

                // Should never panic
                prop_assert_eq!(rationale.vulnerabilities_prevented.len(), vuln_count);
                prop_assert_eq!(rationale.failures_eliminated.len(), failure_count);
                prop_assert_eq!(rationale.attack_vectors_closed.len(), attack_count);
            }

            #[test]
            fn prop_REPL_013_002_format_never_panics(
                impact in ".*{0,200}",
            ) {
                let rationale = SafetyRationale::new()
                    .with_impact(impact)
                    .with_severity(SafetySeverity::Medium);

                // Should never panic
                let _ = format_safety_rationale(&rationale);
            }

            #[test]
            fn prop_REPL_013_002_severity_always_valid(
                severity_index in 0usize..4,
            ) {
                let severity = match severity_index {
                    0 => SafetySeverity::Critical,
                    1 => SafetySeverity::High,
                    2 => SafetySeverity::Medium,
                    _ => SafetySeverity::Low,
                };

                let rationale = SafetyRationale::new()
                    .with_severity(severity.clone());

                prop_assert_eq!(rationale.severity, severity);
            }
        }
    }
}

// ===== REPL-013-003: ALTERNATIVE SUGGESTIONS TESTS (RED PHASE) =====


include!("purifier_transforms_tests_inline_incl2.rs");
