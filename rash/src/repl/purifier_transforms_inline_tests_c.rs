//! Tests extracted from purifier_transforms.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::repl::purifier_transforms::*;

// ===== REPL-013-001: TRANSFORMATION EXPLANATION TESTS (RED PHASE) =====

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
} // mod purifier_cov_tests
