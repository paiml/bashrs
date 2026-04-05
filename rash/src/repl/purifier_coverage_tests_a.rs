//! Coverage tests for repl/purifier.rs — targeting uncovered branches in:
//!   - `explain_purification_changes_detailed`
//!   - `collect_change_explanations`
//!   - `generate_idempotency_alternatives`
//!   - `format_purified_lint_result_with_context` (violation branch)
//!
//! Each test targets a specific uncovered branch identified by the coverage
//! report.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
    #[test]
    fn test_explain_changes_fallback_message() {
        // Code that purifier changes but has no specific pattern match:
        // we just verify no panic and returns Ok
        let result = explain_purification_changes("echo $RANDOM");
        assert!(result.is_ok());
    }

    // ── format_transformation_report with line number ─────────────────────────

    #[test]
    fn test_format_transformation_report_with_line_number() {
        let explanation = TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove $RANDOM",
            "echo $RANDOM",
            "echo 42",
            "Removed RANDOM",
            "Determinism",
        )
        .with_line_number(7);

        let report = format_transformation_report(&[explanation]);
        assert!(report.contains("DETERMINISM"));
        assert!(report.contains("Line: 7"));
        assert!(report.contains("Remove $RANDOM"));
    }

    #[test]
    fn test_format_transformation_report_multiple_entries() {
        let explanations = vec![
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir → mkdir -p",
                "mkdir /tmp",
                "mkdir -p /tmp",
                "Added -p",
                "Idempotent",
            ),
            TransformationExplanation::new(
                TransformationCategory::Safety,
                "Quote variables",
                "echo $v",
                "echo \"$v\"",
                "Added quotes",
                "Safety",
            ),
        ];
        let report = format_transformation_report(&explanations);
        assert!(report.contains("IDEMPOTENCY"));
        assert!(report.contains("SAFETY"));
    }

    // ── purify_and_lint integration ───────────────────────────────────────────

    #[test]
    fn test_purify_and_lint_mkdir_result() {
        let result = purify_and_lint("mkdir /tmp/test_dir");
        assert!(result.is_ok());
        let plr = result.unwrap();
        assert!(plr.purified_code.contains("mkdir"));
    }

    // ── explain_purification_changes_detailed coverage ──────────────────────

    #[test]
    fn test_explain_detailed_no_changes() {
        // Code that is already purified should return empty explanations
        let result = explain_purification_changes_detailed("echo hello");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        assert!(explanations.is_empty());
    }

    #[test]
    fn test_explain_detailed_mkdir_idempotency() {
        let result = explain_purification_changes_detailed("mkdir /tmp/test");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_mkdir = explanations.iter().any(|e| {
            matches!(e.category, TransformationCategory::Idempotency) && e.title.contains("mkdir")
        });
        assert!(has_mkdir, "Should detect mkdir → mkdir -p transformation");
    }

    #[test]
    fn test_explain_detailed_rm_idempotency() {
        let result = explain_purification_changes_detailed("rm /tmp/testfile");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_rm = explanations.iter().any(|e| {
            matches!(e.category, TransformationCategory::Idempotency) && e.title.contains("rm")
        });
        assert!(has_rm, "Should detect rm → rm -f transformation");
    }

    #[test]
    fn test_explain_detailed_variable_quoting_safety_category() {
        let result = explain_purification_changes_detailed("echo $HOME");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_quoting = explanations
            .iter()
            .any(|e| matches!(e.category, TransformationCategory::Safety));
        assert!(has_quoting, "Should detect variable quoting transformation");
    }

    #[test]
    fn test_explain_detailed_random_determinism() {
        let result = explain_purification_changes_detailed("echo $RANDOM");
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let has_random = explanations
            .iter()
            .any(|e| matches!(e.category, TransformationCategory::Determinism));
        assert!(
            has_random,
            "Should detect $RANDOM removal as determinism transformation"
        );
    }

    #[test]
    fn test_explain_detailed_ln_idempotency() {
        // Note: the purifier may or may not transform ln -s depending on
        // whether the transpiler handles it. We verify at minimum no error.
        let result = explain_purification_changes_detailed("ln -s /src /dest");
        assert!(result.is_ok());
        // If explanations are present, verify they make sense
        let explanations = result.unwrap();
        for e in &explanations {
            // All explanations should have non-empty fields
            assert!(!e.title.is_empty());
            assert!(!e.what_changed.is_empty());
        }
    }
}
