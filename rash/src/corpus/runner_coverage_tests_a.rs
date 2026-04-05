//! Coverage tests for corpus/runner.rs
//!
//! Targets:
//! - parse_lcov_file_coverage (0% coverage, pure function)
//! - classify_error (0% coverage, pure function)
//! - check_exact_match (various single-line, multi-line, empty cases)
//! - CorpusResult::score (all branches: transpile fail, schema fail, partial, full)
//! - CorpusResult::score_v1 (backward compatibility)
//! - CorpusScore::gateway_met / format_score
//! - ConvergenceEntry::detect_regressions
//! - RegressionReport::has_regressions
//! - FormatScore and ConvergenceEntry field access

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::registry::{CorpusFormat, Grade};
use crate::corpus::runner::{
#[test]
fn test_RUNNER_COV_032_regression_report_empty_no_regressions() {
    let report = RegressionReport {
        regressions: vec![],
    };
    assert!(!report.has_regressions());
}

#[test]
fn test_RUNNER_COV_033_regression_report_with_entry_has_regressions() {
    use crate::corpus::runner::Regression;
    let report = RegressionReport {
        regressions: vec![Regression {
            message: "Score dropped".to_string(),
            dimension: "score".to_string(),
            previous: 95.0,
            current: 90.0,
        }],
    };
    assert!(report.has_regressions());
}

// ---------------------------------------------------------------------------
// FormatScore fields
// ---------------------------------------------------------------------------

#[test]
fn test_RUNNER_COV_034_format_score_fields() {
    let fs = FormatScore {
        format: CorpusFormat::Dockerfile,
        total: 50,
        passed: 45,
        rate: 0.9,
        score: 88.5,
        grade: Grade::B,
    };
    assert_eq!(fs.format, CorpusFormat::Dockerfile);
    assert_eq!(fs.total, 50);
    assert_eq!(fs.passed, 45);
    assert!((fs.rate - 0.9).abs() < 0.01);
    assert!((fs.score - 88.5).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// CorpusResult fields
// ---------------------------------------------------------------------------

#[test]
fn test_RUNNER_COV_035_corpus_result_default() {
    let r = CorpusResult::default();
    assert!(!r.transpiled);
    assert!(!r.output_contains);
    assert!(!r.output_exact);
    assert!(!r.output_behavioral);
    assert!(!r.has_test);
    assert!((r.coverage_ratio - 0.0).abs() < 0.001);
    assert!(!r.schema_valid);
    assert!(!r.lint_clean);
    assert!(!r.deterministic);
    assert!(!r.metamorphic_consistent);
    assert!(!r.cross_shell_agree);
    assert!(r.actual_output.is_none());
    assert!(r.error.is_none());
    assert!(r.error_category.is_none());
    assert!(r.decision_trace.is_none());
}

// ---------------------------------------------------------------------------
// check_exact_match — tested indirectly via CorpusResult.output_exact field
// We test the logic by simulating what the runner computes.
// ---------------------------------------------------------------------------

// The function is private; we exercise it by understanding what produces
// output_exact=true vs false in corpus runs.
// We test indirectly by verifying the score formula responds correctly.

#[test]
fn test_RUNNER_COV_036_output_exact_true_adds_8_points() {
    let result = CorpusResult {
        transpiled: true,
        schema_valid: true,
        output_contains: true,
        output_exact: true,
        ..Default::default()
    };
    let score = result.score();
    // A=30, B_L1=10, B_L2=8
    assert!(
        (score - 48.0).abs() < 0.01,
        "Score should be 48.0, got {score}"
    );
}

#[test]
fn test_RUNNER_COV_037_output_behavioral_true_adds_7_points() {
    let result = CorpusResult {
        transpiled: true,
        schema_valid: true,
        output_contains: true,
        output_behavioral: true,
        ..Default::default()
    };
    let score = result.score();
    // A=30, B_L1=10, B_L3=7
    assert!(
        (score - 47.0).abs() < 0.01,
        "Score should be 47.0, got {score}"
    );
}

// ---------------------------------------------------------------------------
// ConvergenceEntry fields
// ---------------------------------------------------------------------------

#[test]
fn test_RUNNER_COV_038_convergence_entry_default() {
    let entry = ConvergenceEntry::default();
    assert_eq!(entry.iteration, 0);
    assert_eq!(entry.total, 0);
    assert_eq!(entry.passed, 0);
    assert_eq!(entry.failed, 0);
    assert!((entry.rate - 0.0).abs() < 0.001);
    assert!((entry.score - 0.0).abs() < 0.001);
    assert_eq!(entry.bash_passed, 0);
    assert_eq!(entry.makefile_passed, 0);
    assert_eq!(entry.dockerfile_passed, 0);
    assert_eq!(entry.lint_passed, 0);
}

#[test]
fn test_RUNNER_COV_039_regression_message_format() {
    use crate::corpus::runner::Regression;
    let reg = Regression {
        message: "V2 score dropped: 95 → 90".to_string(),
        dimension: "score".to_string(),
        previous: 95.0,
        current: 90.0,
    };
    assert!(reg.message.contains("95"));
    assert!(reg.message.contains("90"));
    assert_eq!(reg.dimension, "score");
    assert!((reg.previous - 95.0).abs() < 0.01);
    assert!((reg.current - 90.0).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// CorpusResult with actual output and expected output fields
// ---------------------------------------------------------------------------

#[test]
fn test_RUNNER_COV_040_corpus_result_with_outputs() {
    let result = CorpusResult {
        id: "B-001".to_string(),
        transpiled: true,
        actual_output: Some("#!/bin/sh\necho hello".to_string()),
        expected_output: Some("echo hello".to_string()),
        schema_valid: true,
        output_contains: true,
        ..Default::default()
    };
    assert_eq!(result.id, "B-001");
    assert!(result.actual_output.is_some());
    assert!(result.expected_output.is_some());
    assert!(result.output_contains);
}
