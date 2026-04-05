#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for corpus/dataset.rs uncovered branches.
//!
//! Targets: grade boundary values, export edge cases (empty, multi-row),
//! CSV escaping combos, publish readiness boundary conditions, date
//! conversion (leap years, year boundaries, Dec 31), and formatting.

use super::*;
use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier, Grade};
use crate::corpus::runner::{CorpusResult, CorpusScore, FormatScore};

fn make_entry(id: &str, format: CorpusFormat) -> CorpusEntry {
    CorpusEntry {
        id: id.to_string(),
        name: format!("test-{id}"),
        description: "Test entry".to_string(),
        format,
        tier: CorpusTier::Trivial,
        input: "fn main() {}".to_string(),
        expected_output: "#!/bin/sh\necho ok\n".to_string(),
        shellcheck: true,
        deterministic: true,
        idempotent: true,
    }
}

fn make_result(id: &str, transpiled: bool) -> CorpusResult {
    CorpusResult {
        id: id.to_string(),
        transpiled,
        output_contains: transpiled,
        output_exact: transpiled,
        output_behavioral: transpiled,
        has_test: true,
        coverage_ratio: 0.95,
        schema_valid: true,
        lint_clean: transpiled,
        deterministic: transpiled,
        metamorphic_consistent: transpiled,
        cross_shell_agree: transpiled,
        expected_output: None,
        actual_output: if transpiled {
            Some("#!/bin/sh\necho ok\n".to_string())
        } else {
            None
        },
        error: if transpiled {
            None
        } else {
            Some("test error".to_string())
        },
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    }
}

fn make_score(
    total: usize,
    passed: usize,
    failed: usize,
    rate: f64,
    score_val: f64,
    grade: Grade,
    format_count: usize,
) -> CorpusScore {
    let format_scores: Vec<FormatScore> = (0..format_count)
        .map(|i| {
            let fmt = match i {
                0 => CorpusFormat::Bash,
                1 => CorpusFormat::Makefile,
                _ => CorpusFormat::Dockerfile,
            };
            FormatScore {
                format: fmt,
                total: total / format_count.max(1),
                passed: passed / format_count.max(1),
                rate,
                score: score_val,
                grade,
            }
        })
        .collect();

    CorpusScore {
        total,
        passed,
        failed,
        rate,
        score: score_val,
        grade,
        format_scores,
        results: vec![],
    }
}

// === score_to_grade boundary coverage ===

#[test]
fn test_grade_a_plus_boundary() {
    assert_eq!(score_to_grade(97.0), "A+");
    assert_eq!(score_to_grade(100.0), "A+");
    assert_eq!(score_to_grade(98.5), "A+");
}

#[test]
fn test_grade_a_boundary() {
    assert_eq!(score_to_grade(93.0), "A");
    assert_eq!(score_to_grade(96.0), "A");
    assert_eq!(score_to_grade(96.9), "A");
}

#[test]
fn test_grade_a_minus_boundary() {
    assert_eq!(score_to_grade(90.0), "A-");
    assert_eq!(score_to_grade(92.0), "A-");
    assert_eq!(score_to_grade(92.9), "A-");
}

#[test]
fn test_grade_b_plus_boundary() {
    assert_eq!(score_to_grade(87.0), "B+");
    assert_eq!(score_to_grade(89.0), "B+");
    assert_eq!(score_to_grade(89.9), "B+");
}

#[test]
fn test_grade_b_boundary() {
    assert_eq!(score_to_grade(83.0), "B");
    assert_eq!(score_to_grade(86.0), "B");
}

#[test]
fn test_grade_b_minus_boundary() {
    assert_eq!(score_to_grade(80.0), "B-");
    assert_eq!(score_to_grade(82.0), "B-");
}

#[test]
fn test_grade_c_plus_boundary() {
    assert_eq!(score_to_grade(77.0), "C+");
    assert_eq!(score_to_grade(79.0), "C+");
}

#[test]
fn test_grade_c_boundary() {
    assert_eq!(score_to_grade(73.0), "C");
    assert_eq!(score_to_grade(76.0), "C");
}

#[test]
fn test_grade_c_minus_boundary() {
    assert_eq!(score_to_grade(70.0), "C-");
    assert_eq!(score_to_grade(72.0), "C-");
}

#[test]
fn test_grade_d_boundary() {
    assert_eq!(score_to_grade(60.0), "D");
    assert_eq!(score_to_grade(69.0), "D");
}

#[test]
fn test_grade_f_below_60() {
    assert_eq!(score_to_grade(59.0), "F");
    assert_eq!(score_to_grade(0.0), "F");
    assert_eq!(score_to_grade(30.0), "F");
}

// === Export edge cases ===

#[test]
fn test_export_jsonl_empty() {
    let output = export_jsonl(&[]);
    assert_eq!(output, "");
}

#[test]
fn test_export_json_empty() {
    let output = export_json(&[]);
    assert_eq!(output, "[]");
}

#[test]
fn test_export_csv_empty() {
    let output = export_csv(&[]);
    // Should still have header
    assert!(output.starts_with("id,name,tier,format"));
    assert_eq!(output.lines().count(), 1);
}

#[test]
fn test_export_jsonl_multiple_rows() {
    let rows = vec![
        build_row(
            &make_entry("B-001", CorpusFormat::Bash),
            Some(&make_result("B-001", true)),
            "1.0.0",
            "abc",
            "2026-01-01",
        ),
        build_row(
            &make_entry("M-001", CorpusFormat::Makefile),
            Some(&make_result("M-001", false)),
            "1.0.0",
            "abc",
            "2026-01-01",
        ),
    ];
    let output = export_jsonl(&rows);
    assert_eq!(output.lines().count(), 2);
    assert!(output.lines().nth(0).unwrap().contains("B-001"));
    assert!(output.lines().nth(1).unwrap().contains("M-001"));
}

#[test]
fn test_export_json_pretty_printed() {
    let row = build_row(
        &make_entry("D-001", CorpusFormat::Dockerfile),
        None,
        "1.0.0",
        "abc",
        "2026-01-01",
    );
    let output = export_json(&[row]);
    // Pretty-printed JSON should have newlines and indentation
    assert!(output.contains('\n'));
    assert!(output.contains("  "));
    assert!(output.contains("\"format\": \"dockerfile\""));
}

// === CSV escaping ===

#[test]
fn test_csv_escape_plain_string() {
    assert_eq!(csv_escape("hello"), "hello");
}

#[test]
fn test_csv_escape_with_comma() {
    assert_eq!(csv_escape("a,b"), "\"a,b\"");
}

#[test]
fn test_csv_escape_with_quotes() {
    assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
}

#[test]
fn test_csv_escape_with_newline() {
    assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
}

#[test]
fn test_csv_escape_with_all_specials() {
    let s = "a,b\"c\nd";
    let escaped = csv_escape(s);
    assert!(escaped.starts_with('"'));
    assert!(escaped.ends_with('"'));
}

// === build_row with various result states ===

#[test]
fn test_build_row_partial_correctness() {
    // output_contains=true but output_exact=false
    let mut result = make_result("B-010", true);
    result.output_exact = false;
    let row = build_row(
        &make_entry("B-010", CorpusFormat::Bash),
        Some(&result),
        "1.0.0",
        "abc",
        "2026-01-01",
    );
    assert!(row.transpiled);
    assert!(!row.output_correct); // output_correct = output_contains && output_exact
}

#[test]
fn test_build_row_not_deterministic() {
    let mut result = make_result("B-011", true);
    result.deterministic = false;
    let row = build_row(
        &make_entry("B-011", CorpusFormat::Bash),
        Some(&result),
        "1.0.0",
        "abc",
        "2026-01-01",
    );
    assert!(!row.deterministic);
}

#[test]
fn test_build_row_not_lint_clean() {
    let mut result = make_result("B-012", true);
    result.lint_clean = false;
    let row = build_row(
        &make_entry("B-012", CorpusFormat::Bash),
        Some(&result),
        "1.0.0",
        "abc",
        "2026-01-01",
    );
    assert!(!row.lint_clean);
}

#[test]
fn test_build_row_tier_preserved() {
    let mut entry = make_entry("B-020", CorpusFormat::Bash);
    entry.tier = CorpusTier::Adversarial;
    let row = build_row(&entry, None, "1.0.0", "abc", "2026-01-01");
    assert_eq!(row.tier, CorpusTier::Adversarial as u8);
}

#[test]
fn test_build_row_format_string() {
    let entry = make_entry("M-020", CorpusFormat::Makefile);
    let row = build_row(&entry, None, "1.0.0", "abc", "2026-01-01");
    assert_eq!(row.format, "makefile");
}

// === build_dataset integration ===

#[test]
fn test_build_dataset_matches_entries_to_results() {
    let entries = vec![
        make_entry("B-001", CorpusFormat::Bash),
        make_entry("B-002", CorpusFormat::Bash),
    ];
    let registry = CorpusRegistry { entries };
    let results = vec![make_result("B-001", true)];
    let score = CorpusScore {
        total: 2,
        passed: 1,
        failed: 1,
        rate: 0.5,
        score: 50.0,
        grade: Grade::F,
        format_scores: vec![],
        results,
    };
    let rows = build_dataset(&registry, &score);
    assert_eq!(rows.len(), 2);
    assert!(rows[0].transpiled); // B-001 matched
    assert!(!rows[1].transpiled); // B-002 no match => defaults
    assert_eq!(rows[1].grade, "F");
}

// === Publish readiness checks ===

#[test]

include!("dataset_tests_incl2.rs");
