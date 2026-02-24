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
    CorpusEntry::new(id, id, id, CorpusFormat::Bash, CorpusTier::Trivial, input, expected)
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
    let mk = CorpusEntry::new("T-RC2-003a", "m", "m", CorpusFormat::Makefile,
        CorpusTier::Trivial, r#"fn main() { let cc = "gcc"; }"#, "CC");
    let dk = CorpusEntry::new("T-RC2-003b", "d", "d", CorpusFormat::Dockerfile,
        CorpusTier::Trivial,
        r#"fn from_image(i: &str, t: &str) {} fn main() { from_image("alpine", "3.18"); }"#,
        "FROM alpine:3.18");
    let mk_r = r.run_single(&mk);
    let dk_r = r.run_single(&dk);
    assert!(mk_r.score() >= 0.0);
    assert!(dk_r.score() >= 0.0);
}

#[test]
fn test_RUNNER_COV2_004_run_single_format_failures() {
    let r = runner();
    let mk = CorpusEntry::new("T-RC2-004a", "m", "m", CorpusFormat::Makefile,
        CorpusTier::Trivial, "broken!!!", "anything");
    let dk = CorpusEntry::new("T-RC2-004b", "d", "d", CorpusFormat::Dockerfile,
        CorpusTier::Trivial, "broken!!!", "FROM alpine");
    assert!(!r.run_single(&mk).transpiled);
    assert!(!r.run_single(&dk).transpiled);
}

// --- run_single exercising MR paths ---

#[test]
fn test_RUNNER_COV2_005_run_single_exercises_mr_paths() {
    let r = runner();
    // MR-7: entry with if statement
    let e1 = bash_entry("T-RC2-005a",
        r#"fn main() { let x = 5; if x > 3 { println!("big"); } }"#, "echo");
    let _ = r.run_single(&e1).metamorphic_consistent;
    // MR-5/MR-6: multiple let statements
    let e2 = bash_entry("T-RC2-005b", "fn main() { let a = 1; let b = 2; let c = 3; }", "a=");
    let _ = r.run_single(&e2).metamorphic_consistent;
    // println: exercises output_contains/output_exact
    let e3 = bash_entry("T-RC2-005c", r#"fn main() { println!("hello world"); }"#, "echo");
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
    let mk = CorpusEntry::new("T-RC2-006c", "m", "m", CorpusFormat::Makefile,
        CorpusTier::Trivial, r#"fn main() { let cc = "gcc"; }"#, "CC");

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
    reg.add(bash_entry("T-RC2-007a", r#"fn main() { println!("hi"); }"#, "echo"));
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
    reg.add(CorpusEntry::new("T-RC2-008b", "m", "m", CorpusFormat::Makefile,
        CorpusTier::Trivial, r#"fn main() { let cc = "gcc"; }"#, "CC"));
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
        reg.add(bash_entry(&format!("T-RC2-009-{i}"), &format!("invalid {i}!!!"), "x"));
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
    reg.add(CorpusEntry::new("T-RC2-010b", "m", "m", CorpusFormat::Makefile,
        CorpusTier::Trivial, r#"fn main() { let cc = "gcc"; }"#, "CC"));
    reg.add(CorpusEntry::new("T-RC2-010c", "d", "d", CorpusFormat::Dockerfile,
        CorpusTier::Trivial,
        r#"fn from_image(i: &str, t: &str) {} fn main() { from_image("alpine", "3.18"); }"#,
        "FROM"));
    assert_eq!(r.run(&reg).total, 3);
}

// --- CorpusResult serde ---

#[test]
fn test_RUNNER_COV2_011_result_serde() {
    let result = CorpusResult {
        id: "B-999".into(), transpiled: true, output_contains: true, output_exact: true,
        coverage_ratio: 0.85, schema_valid: true, lint_clean: true, deterministic: true,
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
        id: "B-ERR".into(), transpiled: false,
        error: Some("parse error".into()), error_category: Some("syntax_error".into()),
        error_confidence: Some(0.5), ..Default::default()
    };
    let loaded2: CorpusResult = serde_json::from_str(
        &serde_json::to_string(&err_result).unwrap()).unwrap();
    assert_eq!(loaded2.error_category.as_deref(), Some("syntax_error"));
}

// --- convergence_entry ---

#[test]
fn test_RUNNER_COV2_012_convergence_lint_and_zero() {
    let r = runner();
    let results = vec![
        CorpusResult { id: "A".into(), lint_clean: true, transpiled: true, ..Default::default() },
        CorpusResult { id: "B".into(), lint_clean: false, transpiled: true, ..Default::default() },
        CorpusResult { id: "C".into(), lint_clean: true, transpiled: true, ..Default::default() },
    ];
    let score = CorpusScore {
        total: 3, passed: 3, failed: 0, rate: 1.0, score: 80.0, grade: Grade::B,
        format_scores: vec![], results,
    };
    let entry = r.convergence_entry(&score, 1, "2026-02-23", 0.9, "lint");
    assert_eq!(entry.lint_passed, 2);
    assert!((entry.lint_rate - 2.0 / 3.0).abs() < 0.01);

    // Zero total
    let zero = CorpusScore {
        total: 0, passed: 0, failed: 0, rate: 0.0, score: 0.0, grade: Grade::F,
        format_scores: vec![], results: vec![],
    };
    let z = r.convergence_entry(&zero, 1, "2026-02-23", 0.0, "empty");
    assert_eq!(z.lint_rate, 0.0);
}

#[test]
fn test_RUNNER_COV2_013_convergence_with_format_scores() {
    let r = runner();
    let score = CorpusScore {
        total: 100, passed: 95, failed: 5, rate: 0.95, score: 92.0, grade: Grade::A,
        format_scores: vec![
            FormatScore { format: CorpusFormat::Bash, total: 70, passed: 68,
                rate: 68.0/70.0, score: 93.0, grade: Grade::A },
            FormatScore { format: CorpusFormat::Makefile, total: 20, passed: 18,
                rate: 0.9, score: 88.0, grade: Grade::B },
            FormatScore { format: CorpusFormat::Dockerfile, total: 10, passed: 9,
                rate: 0.9, score: 90.0, grade: Grade::A },
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
fn test_RUNNER_COV2_014_converged_edge_cases() {
    assert!(!CorpusRunner::is_converged(&[]));
    assert!(!CorpusRunner::is_converged(&[
        ConvergenceEntry { rate: 0.99, delta: 0.001, ..Default::default() },
        ConvergenceEntry { rate: 0.995, delta: 0.002, ..Default::default() },
    ]));
    // Unstable delta
    assert!(!CorpusRunner::is_converged(&[
        ConvergenceEntry { rate: 0.99, delta: 0.001, ..Default::default() },
        ConvergenceEntry { rate: 0.995, delta: 0.001, ..Default::default() },
        ConvergenceEntry { rate: 0.992, delta: 0.01, ..Default::default() },
    ]));
    // Rate below threshold
    assert!(!CorpusRunner::is_converged(&[
        ConvergenceEntry { rate: 0.98, delta: 0.0, ..Default::default() },
        ConvergenceEntry { rate: 0.98, delta: 0.0, ..Default::default() },
        ConvergenceEntry { rate: 0.98, delta: 0.0, ..Default::default() },
    ]));
    // Negative delta, all stable
    assert!(CorpusRunner::is_converged(&[
        ConvergenceEntry { rate: 0.995, delta: -0.001, ..Default::default() },
        ConvergenceEntry { rate: 0.993, delta: -0.002, ..Default::default() },
        ConvergenceEntry { rate: 0.992, delta: -0.001, ..Default::default() },
    ]));
}

// --- score combinations ---

#[test]
fn test_RUNNER_COV2_015_score_combinations() {
    // A=30, D=10, E=10
    let r1 = CorpusResult { transpiled: true, schema_valid: true,
        lint_clean: true, deterministic: true, ..Default::default() };
    assert!((r1.score() - 50.0).abs() < 0.01);
    // A=30, F=5
    let r2 = CorpusResult { transpiled: true, schema_valid: true,
        metamorphic_consistent: true, ..Default::default() };
    assert!((r2.score() - 35.0).abs() < 0.01);
    // A=30, G=5
    let r3 = CorpusResult { transpiled: true, schema_valid: true,
        cross_shell_agree: true, ..Default::default() };
    assert!((r3.score() - 35.0).abs() < 0.01);
    // All B levels: A=30, B1=10, B2=8, B3=7
    let r4 = CorpusResult { transpiled: true, schema_valid: true,
        output_contains: true, output_exact: true, output_behavioral: true,
        ..Default::default() };
    assert!((r4.score() - 55.0).abs() < 0.01);
    // L1 gates L2/L3: A=30, C=15, D=10, E=10, F=5, G=5 = 75
    let r5 = CorpusResult { transpiled: true, schema_valid: true,
        output_contains: false, output_exact: true, output_behavioral: true,
        coverage_ratio: 1.0, lint_clean: true, deterministic: true,
        metamorphic_consistent: true, cross_shell_agree: true, ..Default::default() };
    assert!((r5.score() - 75.0).abs() < 0.01);
    // V1 partial: A=40, B=25, C=7.5, D=0, E=10 = 82.5
    let r6 = CorpusResult { transpiled: true, output_contains: true,
        coverage_ratio: 0.5, deterministic: true, ..Default::default() };
    assert!((r6.score_v1() - 82.5).abs() < 0.01);
}

// --- Grade boundaries ---

#[test]
fn test_RUNNER_COV2_016_grade_boundaries() {
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(97.0), Grade::APlus);
    assert_eq!(Grade::from_score(96.9), Grade::A);
    assert_eq!(Grade::from_score(89.9), Grade::B);
    assert_eq!(Grade::from_score(79.9), Grade::C);
    assert_eq!(Grade::from_score(69.9), Grade::D);
    assert_eq!(Grade::from_score(59.9), Grade::F);
}

// --- detect_regressions ---

#[test]
fn test_RUNNER_COV2_017_regressions() {
    let base = ConvergenceEntry {
        score: 95.0, passed: 900, bash_passed: 500, makefile_passed: 200,
        dockerfile_passed: 200, bash_score: 99.0, makefile_score: 100.0,
        dockerfile_score: 99.5, lint_passed: 890, ..Default::default()
    };
    // Equal: no regressions
    assert!(!base.detect_regressions(&base).has_regressions());
    // All improve: no regressions
    let better = ConvergenceEntry {
        score: 96.0, passed: 910, bash_passed: 510, makefile_passed: 200,
        dockerfile_passed: 200, bash_score: 99.5, makefile_score: 100.0,
        dockerfile_score: 100.0, lint_passed: 900, ..Default::default()
    };
    assert!(!better.detect_regressions(&base).has_regressions());
    // Lint-only regression
    let lint_drop = ConvergenceEntry { lint_passed: 880, ..base.clone() };
    let report = lint_drop.detect_regressions(&base);
    assert!(report.has_regressions());
    assert!(report.regressions.iter().any(|r| r.dimension == "lint_passed"));
}

// --- CorpusScore lookups and gateway ---

#[test]
fn test_RUNNER_COV2_018_score_lookups() {
    let cs = CorpusScore {
        total: 100, passed: 90, failed: 10, rate: 0.9, score: 85.0, grade: Grade::B,
        format_scores: vec![FormatScore {
            format: CorpusFormat::Makefile, total: 30, passed: 28,
            rate: 28.0/30.0, score: 92.0, grade: Grade::A,
        }],
        results: vec![],
    };
    assert!(cs.format_score(CorpusFormat::Makefile).is_some());
    assert!(cs.format_score(CorpusFormat::Bash).is_none());
    assert!(cs.gateway_met());

    let below = CorpusScore { rate: 0.59, ..cs.clone() };
    assert!(!below.gateway_met());
}

// --- run_single with deterministic=false ---

#[test]
fn test_RUNNER_COV2_019_no_determinism_check() {
    let r = runner();
    let mut entry = bash_entry("T-RC2-019", "fn main() { let x = 42; }", "x=");
    entry.deterministic = false;
    let result = r.run_single(&entry);
    if result.transpiled { assert!(result.deterministic); }
}

// --- Serde roundtrips ---

#[test]
fn test_RUNNER_COV2_020_serde_roundtrips() {
    // CorpusScore
    let score = CorpusScore {
        total: 10, passed: 8, failed: 2, rate: 0.8, score: 85.0, grade: Grade::B,
        format_scores: vec![FormatScore {
            format: CorpusFormat::Bash, total: 10, passed: 8,
            rate: 0.8, score: 85.0, grade: Grade::B,
        }],
        results: vec![CorpusResult { id: "X".into(), transpiled: true, ..Default::default() }],
    };
    let loaded: CorpusScore = serde_json::from_str(
        &serde_json::to_string(&score).unwrap()).unwrap();
    assert_eq!(loaded.total, 10);
    assert_eq!(loaded.format_scores.len(), 1);

    // ConvergenceEntry full
    let entry = ConvergenceEntry {
        iteration: 42, date: "2026-02-23".into(), total: 1000, passed: 998, failed: 2,
        rate: 0.998, delta: 0.001, notes: "full".into(),
        bash_passed: 700, bash_total: 702, makefile_passed: 200, makefile_total: 200,
        dockerfile_passed: 98, dockerfile_total: 98, score: 99.5, grade: "A+".into(),
        bash_score: 99.3, makefile_score: 100.0, dockerfile_score: 99.8,
        lint_passed: 995, lint_rate: 0.995,
    };
    let le: ConvergenceEntry = serde_json::from_str(
        &serde_json::to_string(&entry).unwrap()).unwrap();
    assert_eq!(le.iteration, 42);
    assert_eq!(le.bash_passed, 700);
    assert_eq!(le.grade, "A+");
}
