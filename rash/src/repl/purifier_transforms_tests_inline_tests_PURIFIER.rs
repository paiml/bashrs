#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(non_snake_case)]
mod purifier_cov_tests {
    use super::*;

    // --- explain_purification_changes_detailed tests ---

    #[test]
    fn test_PURIFIER_COV_001_explain_changes_empty_no_transformation() {
        // ARRANGE: Code that needs no purification (already clean)
        let original = "echo hello";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: No changes detected, returns empty vec
        assert!(result.is_ok());
        let explanations = result.unwrap();
        assert!(
            explanations.is_empty(),
            "Already-pure code should return empty explanations"
        );
    }

    #[test]
    fn test_PURIFIER_COV_002_explain_changes_with_determinism_random() {
        // ARRANGE: Code with $RANDOM (non-deterministic)
        let original = "x=$RANDOM";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect $RANDOM removal
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let random_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.category == TransformationCategory::Determinism)
            .filter(|e| e.title.contains("$RANDOM"))
            .collect();
        assert!(
            !random_explanations.is_empty(),
            "Should detect $RANDOM removal as Determinism transformation"
        );
        assert!(random_explanations[0].what_changed.contains("$RANDOM"));
        assert!(random_explanations[0]
            .why_it_matters
            .contains("unpredictable"));
    }

    #[test]
    fn test_PURIFIER_COV_003_explain_changes_with_determinism_seconds() {
        // ARRANGE: Code with $SECONDS (non-deterministic timestamp)
        let original = "elapsed=$SECONDS";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect $SECONDS as timestamp removal
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let timestamp_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.category == TransformationCategory::Determinism)
            .filter(|e| e.title.contains("timestamp"))
            .collect();
        assert!(
            !timestamp_explanations.is_empty(),
            "Should detect $SECONDS removal as Determinism/timestamp transformation"
        );
        assert!(timestamp_explanations[0]
            .what_changed
            .contains("time-based"));
        assert!(timestamp_explanations[0]
            .why_it_matters
            .contains("non-reproducible"));
    }

    #[test]
    fn test_PURIFIER_COV_004_explain_changes_with_idempotency_rm() {
        // ARRANGE: Code with rm that needs -f flag
        let original = "rm /tmp/file";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect rm → rm -f transformation
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let rm_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.category == TransformationCategory::Idempotency)
            .filter(|e| e.title.contains("rm"))
            .collect();
        assert!(
            !rm_explanations.is_empty(),
            "Should detect rm → rm -f as Idempotency transformation"
        );
        assert_eq!(rm_explanations[0].title, "rm → rm -f");
        assert!(rm_explanations[0].what_changed.contains("-f flag"));
        assert!(rm_explanations[0].why_it_matters.contains("safe to re-run"));
    }

    #[test]
    fn test_PURIFIER_COV_005_explain_changes_with_idempotency_mkdir() {
        // ARRANGE: Code with mkdir that needs -p flag
        let original = "mkdir /tmp/test";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect mkdir → mkdir -p transformation
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let mkdir_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.category == TransformationCategory::Idempotency)
            .filter(|e| e.title.contains("mkdir"))
            .collect();
        assert!(
            !mkdir_explanations.is_empty(),
            "Should detect mkdir → mkdir -p as Idempotency transformation"
        );
        assert_eq!(mkdir_explanations[0].title, "mkdir → mkdir -p");
        assert!(mkdir_explanations[0].what_changed.contains("-p flag"));
        assert!(mkdir_explanations[0]
            .why_it_matters
            .contains("safe to re-run"));
    }

    #[test]
    fn test_PURIFIER_COV_006_explain_changes_with_quoting_safety() {
        // ARRANGE: Code with unquoted variable
        let original = "echo $var";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect variable quoting as Safety transformation
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let safety_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.category == TransformationCategory::Safety)
            .collect();
        assert!(
            !safety_explanations.is_empty(),
            "Should detect unquoted variable as Safety transformation"
        );
        assert!(safety_explanations[0].title.contains("Quote"));
        assert!(safety_explanations[0].what_changed.contains("quotes"));
        assert!(safety_explanations[0].why_it_matters.contains("injection"));
    }

    #[test]
    fn test_PURIFIER_COV_007_explain_changes_mixed_multiple() {
        // ARRANGE: Code with multiple issues
        let original = "mkdir /tmp/test\nrm /tmp/file\necho $var\nx=$RANDOM";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect multiple transformations
        assert!(result.is_ok());
        let explanations = result.unwrap();
        assert!(
            explanations.len() >= 3,
            "Mixed input should produce at least 3 explanations, got {}",
            explanations.len()
        );

        // Check categories present
        let has_idempotency = explanations
            .iter()
            .any(|e| e.category == TransformationCategory::Idempotency);
        let has_determinism = explanations
            .iter()
            .any(|e| e.category == TransformationCategory::Determinism);
        let has_safety = explanations
            .iter()
            .any(|e| e.category == TransformationCategory::Safety);

        assert!(
            has_idempotency,
            "Should have at least one Idempotency transformation"
        );
        assert!(
            has_determinism,
            "Should have at least one Determinism transformation"
        );
        assert!(has_safety, "Should have at least one Safety transformation");
    }

    #[test]
    fn test_PURIFIER_COV_008_explain_changes_already_has_mkdir_p() {
        // ARRANGE: Code that already has mkdir -p (no transformation needed)
        let original = "mkdir -p /tmp/test";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should NOT detect mkdir transformation (already has -p)
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let mkdir_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.title.contains("mkdir"))
            .collect();
        assert!(
            mkdir_explanations.is_empty(),
            "Already-correct mkdir -p should not trigger a transformation"
        );
    }

    #[test]
    fn test_PURIFIER_COV_009_explain_changes_already_has_rm_f() {
        // ARRANGE: Code that already has rm -f (no transformation needed)
        let original = "rm -f /tmp/file";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should NOT detect rm transformation (already has -f)
        assert!(result.is_ok());
        let explanations = result.unwrap();
        let rm_explanations: Vec<_> = explanations
            .iter()
            .filter(|e| e.title.contains("rm"))
            .collect();
        assert!(
            rm_explanations.is_empty(),
            "Already-correct rm -f should not trigger a transformation"
        );
    }

    // --- format_transformation_report tests ---

    #[test]
    fn test_PURIFIER_COV_010_format_report_single_idempotency() {
        // ARRANGE: Single idempotency transformation
        let transformations = vec![TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir → mkdir -p",
            "mkdir /tmp/test",
            "mkdir -p /tmp/test",
            "Added -p flag",
            "Makes directory creation safe to re-run.",
        )];

        // ACT: Format the report
        let report = format_transformation_report(&transformations);

        // ASSERT: Report contains expected content
        assert!(report.contains("Transformation Report"));
        assert!(report.contains("IDEMPOTENCY"));
        assert!(report.contains("mkdir → mkdir -p"));
        assert!(report.contains("Added -p flag"));
        assert!(report.contains("safe to re-run"));
        assert!(report.contains("Original:"));
        assert!(report.contains("Transformed:"));
        assert!(report.contains("mkdir /tmp/test"));
        assert!(report.contains("mkdir -p /tmp/test"));
    }

    #[test]
    fn test_PURIFIER_COV_011_format_report_single_determinism() {
        // ARRANGE: Single determinism transformation
        let transformations = vec![TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove $RANDOM",
            "x=$RANDOM",
            "x=0",
            "Removed $RANDOM variable",
            "Non-deterministic values are unpredictable.",
        )];

        // ACT: Format the report
        let report = format_transformation_report(&transformations);

        // ASSERT: Report contains DETERMINISM category
        assert!(report.contains("Transformation Report"));
        assert!(report.contains("DETERMINISM"));
        assert!(report.contains("Remove $RANDOM"));
        assert!(report.contains("Removed $RANDOM variable"));
    }

    #[test]
    fn test_PURIFIER_COV_012_format_report_single_safety() {
        // ARRANGE: Single safety transformation
        let transformations = vec![TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            "echo $var",
            "echo \"$var\"",
            "Added quotes around variables",
            "Prevents injection attacks.",
        )];

        // ACT: Format the report
        let report = format_transformation_report(&transformations);

        // ASSERT: Report contains SAFETY category
        assert!(report.contains("Transformation Report"));
        assert!(report.contains("SAFETY"));
        assert!(report.contains("Quote variables"));
        assert!(report.contains("injection attacks"));
    }

    #[test]
    fn test_PURIFIER_COV_013_format_report_with_line_number() {
        // ARRANGE: Transformation with a line number
        let transformations = vec![TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "rm → rm -f",
            "rm /tmp/file",
            "rm -f /tmp/file",
            "Added -f flag",
            "Makes deletion safe to re-run.",
        )
        .with_line_number(7)];

        // ACT: Format the report
        let report = format_transformation_report(&transformations);

        // ASSERT: Report contains line number
        assert!(report.contains("Line: 7"));
    }

    #[test]
    fn test_PURIFIER_COV_014_format_report_multiple_transformations() {
        // ARRANGE: Multiple transformations across categories
        let transformations = vec![
            TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir → mkdir -p",
                "mkdir /tmp/test",
                "mkdir -p /tmp/test",
                "Added -p flag",
                "Safe to re-run.",
            ),
            TransformationExplanation::new(
                TransformationCategory::Determinism,
                "Remove $RANDOM",
                "x=$RANDOM",
                "x=0",
                "Removed $RANDOM",
                "Reproducible output.",
            ),
            TransformationExplanation::new(
                TransformationCategory::Safety,
                "Quote variables",
                "echo $var",
                "echo \"$var\"",
                "Added quotes",
                "Prevents injection.",
            ),
        ];

        // ACT: Format the report
        let report = format_transformation_report(&transformations);

        // ASSERT: Report contains all three categories
        assert!(report.contains("IDEMPOTENCY"));
        assert!(report.contains("DETERMINISM"));
        assert!(report.contains("SAFETY"));

        // ASSERT: Report has separator between transformations
        // (second and third should be separated from previous by double newline)
        assert!(report.contains("Transformation Report"));
        assert!(report.contains("===================="));
    }

    #[test]
    fn test_PURIFIER_COV_015_format_report_without_line_number() {
        // ARRANGE: Transformation without a line number
        let transformations = vec![TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove timestamps",
            "elapsed=$SECONDS",
            "elapsed=0",
            "Removed time-based values",
            "Non-reproducible across runs.",
        )];

        // ACT: Format the report
        let report = format_transformation_report(&transformations);

        // ASSERT: Report does NOT contain a "Line:" entry
        assert!(
            !report.contains("Line:"),
            "Report should not contain Line: when no line number is set"
        );
    }
}
