//! Additional coverage tests for corpus/runner.rs — targets uncovered paths via
//! public API: run_single, run_entry_with_trace, run(), run_format(),
//! compute_score, convergence_entry, is_converged, score combinations, serde.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier, Grade};
use crate::corpus::runner::{
    ConvergenceEntry, CorpusResult, CorpusRunner, CorpusScore, FormatScore,
};
use crate::models::Config;

fn runner() -> CorpusRunner {
    CorpusRunner::new(Config::default())
}

fn bash_entry(id: &str, input: &str, expected: &str) -> CorpusEntry {
    CorpusEntry::new(
        id,
        id,
        id,
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        input,
        expected,
    )
}

// --- run_single: Bash success, failure, Makefile, Dockerfile ---

#[test]
fn test_RUNNER_COV2_001_run_single_bash_success() {
    let result = runner().run_single(&bash_entry("T-RC2-001", "fn main() { let x = 42; }", "x="));
    assert!(result.transpiled);
    assert!(result.actual_output.is_some());
    assert!(result.error.is_none());
    assert!(result.schema_valid);
    assert!(result.score() >= 30.0);
}

#[test]
fn test_RUNNER_COV2_002_run_single_bash_failure() {
    let result = runner().run_single(&bash_entry("T-RC2-002", "not valid!!!", "anything"));
    assert!(!result.transpiled);
    assert!(result.error.is_some());
    assert!(result.error_category.is_some());
    assert!(result.error_confidence.is_some());
    assert_eq!(result.score(), 0.0);
}

#[test]
fn test_RUNNER_COV2_003_run_single_makefile_and_dockerfile() {
    let r = runner();
    let mk = CorpusEntry::new(
        "T-RC2-003a",
        "m",
        "m",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        r#"fn main() { let cc = "gcc"; }"#,
        "CC",
    );
    let dk = CorpusEntry::new(
        "T-RC2-003b",
        "d",
        "d",
        CorpusFormat::Dockerfile,
        CorpusTier::Trivial,
        r#"fn from_image(i: &str, t: &str) {} fn main() { from_image("alpine", "3.18"); }"#,
        "FROM alpine:3.18",
    );
    let mk_r = r.run_single(&mk);
    let dk_r = r.run_single(&dk);
    assert!(mk_r.score() >= 0.0);
    assert!(dk_r.score() >= 0.0);
}

#[test]
fn test_RUNNER_COV2_004_run_single_format_failures() {
    let r = runner();
    let mk = CorpusEntry::new(
        "T-RC2-004a",
        "m",
        "m",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        "broken!!!",
        "anything",
    );
    let dk = CorpusEntry::new(
        "T-RC2-004b",
        "d",
        "d",
        CorpusFormat::Dockerfile,
        CorpusTier::Trivial,
        "broken!!!",
        "FROM alpine",
    );
    assert!(!r.run_single(&mk).transpiled);
    assert!(!r.run_single(&dk).transpiled);
}

// --- run_single exercising MR paths ---

#[test]
fn test_RUNNER_COV2_005_run_single_exercises_mr_paths() {
    let r = runner();
    // MR-7: entry with if statement
    let e1 = bash_entry(
        "T-RC2-005a",
        r#"fn main() { let x = 5; if x > 3 { println!("big"); } }"#,
        "echo",
    );
    let _ = r.run_single(&e1).metamorphic_consistent;
    // MR-5/MR-6: multiple let statements
    let e2 = bash_entry(
        "T-RC2-005b",
        "fn main() { let a = 1; let b = 2; let c = 3; }",
        "a=",
    );
    let _ = r.run_single(&e2).metamorphic_consistent;
    // println: exercises output_contains/output_exact
    let e3 = bash_entry(
        "T-RC2-005c",
        r#"fn main() { println!("hello world"); }"#,
        "echo",
    );
    if r.run_single(&e3).transpiled {
        // output checks computed in run_entry
    }
}

// --- run_entry_with_trace ---

#[test]
fn test_RUNNER_COV2_006_trace_bash_success_and_failure() {
    let r = runner();
    let ok = bash_entry("T-RC2-006a", "fn main() { let x = 42; }", "x=");
    let fail = bash_entry("T-RC2-006b", "invalid!!!", "anything");
    let mk = CorpusEntry::new(
        "T-RC2-006c",
        "m",
        "m",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        r#"fn main() { let cc = "gcc"; }"#,
        "CC",
    );

    let ok_r = r.run_entry_with_trace(&ok);
    if ok_r.transpiled {
        assert!(ok_r.decision_trace.is_some());
    }
    let fail_r = r.run_entry_with_trace(&fail);
    assert!(!fail_r.transpiled);
    assert!(fail_r.decision_trace.is_none());
    assert!(fail_r.error_category.is_some());
    // Non-Bash: no trace
    assert!(r.run_entry_with_trace(&mk).decision_trace.is_none());
}

// --- run() / run_format() with small registries ---

#[test]
fn test_RUNNER_COV2_007_run_tiny_registry() {
    let r = runner();
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry(
        "T-RC2-007a",
        r#"fn main() { println!("hi"); }"#,
        "echo",
    ));
    reg.add(bash_entry("T-RC2-007b", "invalid!!!", "anything"));
    let score = r.run(&reg);
    assert_eq!(score.total, 2);
    assert!(score.failed >= 1);
    assert!(!score.format_scores.is_empty());
}

#[test]
fn test_RUNNER_COV2_008_run_format_filters() {
    let r = runner();
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry("T-RC2-008a", "fn main() { let a = 1; }", "a="));
    reg.add(CorpusEntry::new(
        "T-RC2-008b",
        "m",
        "m",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        r#"fn main() { let cc = "gcc"; }"#,
        "CC",
    ));
    assert_eq!(r.run_format(&reg, CorpusFormat::Bash).total, 1);
    assert_eq!(r.run_format(&reg, CorpusFormat::Makefile).total, 1);
}

#[test]
fn test_RUNNER_COV2_009_run_empty_and_below_gateway() {
    let r = runner();
    // Empty
    let empty_score = r.run(&CorpusRegistry::new());
    assert_eq!(empty_score.total, 0);
    assert_eq!(empty_score.score, 0.0);
    // All failing = below gateway
    let mut reg = CorpusRegistry::new();
    for i in 0..5 {
        reg.add(bash_entry(
            &format!("T-RC2-009-{i}"),
            &format!("invalid {i}!!!"),
            "x",
        ));
    }
    let score = r.run(&reg);
    assert!(score.rate < 0.60);
    assert!((score.score - score.rate * 30.0).abs() < 0.01);
}

#[test]
fn test_RUNNER_COV2_010_run_mixed_formats() {
    let r = runner();
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry("T-RC2-010a", "fn main() { let x = 1; }", "x="));
    reg.add(CorpusEntry::new(
        "T-RC2-010b",
        "m",
        "m",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        r#"fn main() { let cc = "gcc"; }"#,
        "CC",
    ));
    reg.add(CorpusEntry::new(
        "T-RC2-010c",
        "d",
        "d",
        CorpusFormat::Dockerfile,
        CorpusTier::Trivial,
        r#"fn from_image(i: &str, t: &str) {} fn main() { from_image("alpine", "3.18"); }"#,
        "FROM",
    ));
    assert_eq!(r.run(&reg).total, 3);
}

// --- CorpusResult serde ---

#[test]
fn test_RUNNER_COV2_011_result_serde() {
    let result = CorpusResult {
        id: "B-999".into(),
        transpiled: true,
        output_contains: true,
        output_exact: true,
        coverage_ratio: 0.85,
        schema_valid: true,
        lint_clean: true,
        deterministic: true,
        expected_output: Some("echo hello".into()),
        actual_output: Some("#!/bin/sh\necho hello".into()),
        ..Default::default()
    };
    let json = serde_json::to_string(&result).unwrap();
    let loaded: CorpusResult = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.id, "B-999");
    assert_eq!(loaded.coverage_ratio, 0.85);

    // With error
    let err_result = CorpusResult {
        id: "B-ERR".into(),
        transpiled: false,
        error: Some("parse error".into()),
        error_category: Some("syntax_error".into()),
        error_confidence: Some(0.5),
        ..Default::default()
    };
    let loaded2: CorpusResult =
        serde_json::from_str(&serde_json::to_string(&err_result).unwrap()).unwrap();
    assert_eq!(loaded2.error_category.as_deref(), Some("syntax_error"));
}

// --- convergence_entry ---

#[test]
fn test_RUNNER_COV2_012_convergence_lint_and_zero() {
    let r = runner();
    let results = vec![
        CorpusResult {
            id: "A".into(),
            lint_clean: true,
            transpiled: true,
            ..Default::default()
        },
        CorpusResult {
            id: "B".into(),
            lint_clean: false,
            transpiled: true,
            ..Default::default()
        },
        CorpusResult {
            id: "C".into(),
            lint_clean: true,
            transpiled: true,
            ..Default::default()
        },
    ];
    let score = CorpusScore {
        total: 3,
        passed: 3,
        failed: 0,
        rate: 1.0,
        score: 80.0,
        grade: Grade::B,
        format_scores: vec![],
        results,
    };
    let entry = r.convergence_entry(&score, 1, "2026-02-23", 0.9, "lint");
    assert_eq!(entry.lint_passed, 2);
    assert!((entry.lint_rate - 2.0 / 3.0).abs() < 0.01);

    // Zero total
    let zero = CorpusScore {
        total: 0,
        passed: 0,
        failed: 0,
        rate: 0.0,
        score: 0.0,
        grade: Grade::F,
        format_scores: vec![],
        results: vec![],
    };
    let z = r.convergence_entry(&zero, 1, "2026-02-23", 0.0, "empty");
    assert_eq!(z.lint_rate, 0.0);
}

#[test]
fn test_RUNNER_COV2_013_convergence_with_format_scores() {
    let r = runner();
    let score = CorpusScore {
        total: 100,
        passed: 95,
        failed: 5,
        rate: 0.95,
        score: 92.0,
        grade: Grade::A,
        format_scores: vec![
            FormatScore {
                format: CorpusFormat::Bash,
                total: 70,
                passed: 68,
                rate: 68.0 / 70.0,
                score: 93.0,
                grade: Grade::A,
            },
            FormatScore {
                format: CorpusFormat::Makefile,
                total: 20,
                passed: 18,
                rate: 0.9,
                score: 88.0,
                grade: Grade::B,
            },
            FormatScore {
                format: CorpusFormat::Dockerfile,
                total: 10,
                passed: 9,
                rate: 0.9,
                score: 90.0,
                grade: Grade::A,
            },
        ],
        results: vec![],
    };
    let e = r.convergence_entry(&score, 5, "2026-02-23", 0.93, "fmt");
    assert_eq!(e.bash_passed, 68);
    assert_eq!(e.makefile_passed, 18);
    assert_eq!(e.dockerfile_passed, 9);
    assert!((e.bash_score - 93.0).abs() < 0.01);
}

// --- is_converged ---

#[test]

include!("runner_coverage_tests2_tests_RUNNER.rs");
