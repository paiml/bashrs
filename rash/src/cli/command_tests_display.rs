#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for corpus display, visualization, and report formatting functions.
//! Tests internal helpers WITHOUT running CorpusRunner::run().

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier, Grade};
use crate::corpus::runner::{ConvergenceEntry, CorpusResult, CorpusScore, FormatScore};

// ── Mock data builders ──────────────────────────────────────────────────────

fn mock_result(id: &str, all_pass: bool) -> CorpusResult {
    CorpusResult {
        id: id.to_string(),
        transpiled: all_pass,
        output_contains: all_pass,
        output_exact: all_pass,
        output_behavioral: all_pass,
        has_test: true,
        coverage_ratio: if all_pass { 0.95 } else { 0.0 },
        schema_valid: true,
        lint_clean: all_pass,
        deterministic: all_pass,
        metamorphic_consistent: all_pass,
        cross_shell_agree: all_pass,
        expected_output: None,
        actual_output: if all_pass {
            Some("echo hello".into())
        } else {
            None
        },
        error: if all_pass {
            None
        } else {
            Some("transpile failed".into())
        },
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    }
}

fn mock_result_partial(id: &str) -> CorpusResult {
    CorpusResult {
        id: id.to_string(),
        transpiled: true,
        output_contains: true,
        output_exact: false,
        output_behavioral: false,
        has_test: true,
        coverage_ratio: 0.5,
        schema_valid: true,
        lint_clean: true,
        deterministic: false,
        metamorphic_consistent: true,
        cross_shell_agree: false,
        expected_output: None,
        actual_output: Some("echo partial".into()),
        error: None,
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    }
}

fn mock_entry(id: &str, name: &str, format: CorpusFormat) -> CorpusEntry {
    CorpusEntry::new(
        id,
        name,
        "test description",
        format,
        CorpusTier::Standard,
        "fn main() { println!(\"hello\"); }",
        "echo hello",
    )
}

fn mock_convergence_entry(iter: u32, score: f64, total: usize) -> ConvergenceEntry {
    ConvergenceEntry {
        iteration: iter,
        date: "2025-01-15".to_string(),
        total,
        passed: total - 1,
        failed: 1,
        rate: (total - 1) as f64 / total as f64,
        delta: 0.001,
        notes: format!("test iter {iter}"),
        bash_passed: 100,
        bash_total: 101,
        makefile_passed: 50,
        makefile_total: 50,
        dockerfile_passed: 30,
        dockerfile_total: 30,
        score,
        grade: "A+".to_string(),
        bash_score: score,
        makefile_score: 100.0,
        dockerfile_score: 100.0,
        lint_passed: total - 1,
        lint_rate: (total - 1) as f64 / total as f64,
    }
}

// ── corpus_viz_commands tests ───────────────────────────────────────────────

#[test]
fn test_grade_from_fail_count_all_grades() {
    use super::corpus_viz_commands::grade_from_fail_count;
    assert_eq!(grade_from_fail_count(0), "A+");
    assert_eq!(grade_from_fail_count(1), "A");
    assert_eq!(grade_from_fail_count(2), "B");
    assert_eq!(grade_from_fail_count(3), "C");
    assert_eq!(grade_from_fail_count(4), "C");
    assert_eq!(grade_from_fail_count(5), "D");
    assert_eq!(grade_from_fail_count(6), "D");
    assert_eq!(grade_from_fail_count(7), "F");
    assert_eq!(grade_from_fail_count(100), "F");
}

#[test]
fn test_schema_layer_counts_all_pass() {
    use super::corpus_viz_commands::schema_layer_counts;
    let results = vec![mock_result("B-001", true), mock_result("B-002", true)];
    let entries = vec![
        mock_entry("B-001", "test1", CorpusFormat::Bash),
        mock_entry("B-002", "test2", CorpusFormat::Bash),
    ];
    let indices: Vec<(usize, &CorpusEntry)> = entries.iter().enumerate().collect();
    let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
    assert_eq!(l1, 2);
    assert_eq!(l2, 2);
    assert_eq!(l3, 2);
    assert_eq!(l4, 2);
}

#[test]
fn test_schema_layer_counts_all_fail() {
    use super::corpus_viz_commands::schema_layer_counts;
    let results = vec![mock_result("B-001", false), mock_result("B-002", false)];
    let entries = vec![
        mock_entry("B-001", "t1", CorpusFormat::Bash),
        mock_entry("B-002", "t2", CorpusFormat::Bash),
    ];
    let indices: Vec<(usize, &CorpusEntry)> = entries.iter().enumerate().collect();
    let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
    assert_eq!(l1, 0);
    assert_eq!(l2, 0);
    assert_eq!(l3, 0);
    assert_eq!(l4, 0);
}

#[test]
fn test_schema_layer_counts_partial() {
    use super::corpus_viz_commands::schema_layer_counts;
    let results = vec![mock_result_partial("B-001")];
    let entries = vec![mock_entry("B-001", "t1", CorpusFormat::Bash)];
    let indices: Vec<(usize, &CorpusEntry)> = entries.iter().enumerate().collect();
    let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
    assert_eq!(l1, 1); // transpiled
    assert_eq!(l2, 1); // lint_clean
    assert_eq!(l3, 0); // deterministic=false
    assert_eq!(l4, 0); // behavioral=false
}

#[test]
fn test_schema_layer_counts_empty() {
    use super::corpus_viz_commands::schema_layer_counts;
    let results: Vec<CorpusResult> = vec![];
    let indices: Vec<(usize, &CorpusEntry)> = vec![];
    let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
    assert_eq!((l1, l2, l3, l4), (0, 0, 0, 0));
}

#[test]
fn test_schema_layer_counts_index_out_of_bounds() {
    use super::corpus_viz_commands::schema_layer_counts;
    let results = vec![mock_result("B-001", true)];
    let entry = mock_entry("B-005", "t5", CorpusFormat::Bash);
    // index 5 does not exist in results
    let indices: Vec<(usize, &CorpusEntry)> = vec![(5, &entry)];
    let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
    assert_eq!((l1, l2, l3, l4), (0, 0, 0, 0));
}

#[test]
fn test_history_chart_cell_renders_without_panic() {
    use super::corpus_viz_commands::history_chart_cell;
    // Just verify no panic for various inputs
    history_chart_cell(99.5, 9, 90.0, 10.0, 10);
    history_chart_cell(95.0, 5, 90.0, 10.0, 10);
    history_chart_cell(91.0, 1, 90.0, 10.0, 10);
    history_chart_cell(0.0, 5, 0.0, 10.0, 10); // score <= 0
    history_chart_cell(50.0, 0, 0.0, 100.0, 10);
}

// ── corpus_display_commands tests ───────────────────────────────────────────

#[test]
fn test_heatmap_print_header_no_panic() {
    use super::corpus_display_commands::heatmap_print_header;
    heatmap_print_header();
}

#[test]
fn test_heatmap_print_row_all_pass() {
    use super::corpus_display_commands::heatmap_print_row;
    let r = mock_result("B-001", true);
    heatmap_print_row(&r);
}

#[test]
fn test_heatmap_print_row_all_fail() {
    use super::corpus_display_commands::heatmap_print_row;
    let r = mock_result("B-002", false);
    heatmap_print_row(&r);
}

#[test]
fn test_heatmap_print_row_partial() {
    use super::corpus_display_commands::heatmap_print_row;
    let r = mock_result_partial("B-003");
    heatmap_print_row(&r);
}

#[test]
fn test_dashboard_print_formats_with_data() {
    use super::corpus_display_commands::dashboard_print_formats;
    let score = CorpusScore {
        total: 100,
        passed: 98,
        failed: 2,
        rate: 0.98,
        score: 99.0,
        grade: Grade::APlus,
        format_scores: vec![
            FormatScore {
                format: CorpusFormat::Bash,
                total: 60,
                passed: 59,
                rate: 0.983,
                score: 99.0,
                grade: Grade::APlus,
            },
            FormatScore {
                format: CorpusFormat::Makefile,
                total: 30,
                passed: 30,
                rate: 1.0,
                score: 100.0,
                grade: Grade::APlus,
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
    dashboard_print_formats(&score);
}

#[test]
fn test_dashboard_print_history_renders() {
    use super::corpus_display_commands::dashboard_print_history;
    let entries = vec![
        mock_convergence_entry(1, 95.0, 1000),
        mock_convergence_entry(2, 96.0, 1050),
        mock_convergence_entry(3, 99.2, 1100),
    ];
    dashboard_print_history(&entries);
}

#[test]
fn test_dashboard_print_history_single_entry() {
    use super::corpus_display_commands::dashboard_print_history;
    let entries = vec![mock_convergence_entry(1, 99.0, 500)];
    dashboard_print_history(&entries);
}

// ── corpus_report_commands tests ────────────────────────────────────────────

#[test]
fn test_fmt_pass_total_with_data() {
    use super::corpus_report_commands::fmt_pass_total;
    assert_eq!(fmt_pass_total(499, 500), "499/500");
    assert_eq!(fmt_pass_total(0, 100), "0/100");
}

#[test]
fn test_fmt_pass_total_zero() {
    use super::corpus_report_commands::fmt_pass_total;
    assert_eq!(fmt_pass_total(0, 0), "-");
}

#[test]
fn test_trend_arrow_variants() {
    use super::corpus_report_commands::trend_arrow;
    assert_eq!(trend_arrow(10, 5), "\u{2191}"); // up
    assert_eq!(trend_arrow(5, 10), "\u{2193}"); // down
    assert_eq!(trend_arrow(5, 5), "\u{2192}"); // same
}

#[test]
fn test_corpus_failing_dims_all_pass() {
    use super::corpus_report_commands::corpus_failing_dims;
    let r = mock_result("B-001", true);
    assert_eq!(corpus_failing_dims(&r), "");
}

#[test]
fn test_corpus_failing_dims_all_fail() {
    use super::corpus_report_commands::corpus_failing_dims;
    let r = mock_result("B-001", false);
    let dims = corpus_failing_dims(&r);
    assert!(dims.contains("A"));
    assert!(dims.contains("B1"));
    assert!(dims.contains("E"));
}

#[test]
fn test_corpus_failing_dims_partial() {
    use super::corpus_report_commands::corpus_failing_dims;
    let r = mock_result_partial("B-001");
    let dims = corpus_failing_dims(&r);
    assert!(!dims.contains("A")); // transpiled=true
    assert!(dims.contains("B2")); // output_exact=false
    assert!(dims.contains("B3")); // output_behavioral=false
    assert!(dims.contains("E")); // deterministic=false
    assert!(dims.contains("G")); // cross_shell_agree=false
}

#[test]
fn test_corpus_failing_dims_schema_invalid() {
    use super::corpus_report_commands::corpus_failing_dims;
    let mut r = mock_result("B-001", true);
    r.schema_valid = false;
    let dims = corpus_failing_dims(&r);
    assert!(dims.contains("Schema"));
}

#[test]
fn test_corpus_print_failures_empty() {
    use super::corpus_report_commands::corpus_print_failures;
    use crate::cli::args::CorpusOutputFormat;
    let failures: Vec<&CorpusResult> = vec![];
    let result = corpus_print_failures(&failures, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_failures_human_format() {
    use super::corpus_report_commands::corpus_print_failures;
    use crate::cli::args::CorpusOutputFormat;
    let r1 = mock_result("B-001", false);
    let r2 = mock_result_partial("B-002");
    let failures: Vec<&CorpusResult> = vec![&r1, &r2];
    let result = corpus_print_failures(&failures, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_failures_json_format() {
    use super::corpus_report_commands::corpus_print_failures;
    use crate::cli::args::CorpusOutputFormat;
    let r1 = mock_result("B-001", false);
    let failures: Vec<&CorpusResult> = vec![&r1];
    let result = corpus_print_failures(&failures, &CorpusOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_history_row_with_format_data() {
    use super::corpus_report_commands::corpus_print_history_row;
    let e = mock_convergence_entry(5, 99.0, 1000);
    let prev = mock_convergence_entry(4, 98.5, 980);
    corpus_print_history_row(&e, Some(&prev), true, true);
}

#[test]
fn test_corpus_print_history_row_without_format_data() {
    use super::corpus_report_commands::corpus_print_history_row;
    let e = mock_convergence_entry(1, 95.0, 500);
    corpus_print_history_row(&e, None, false, false);
}

#[test]
fn test_corpus_print_history_row_with_score_no_format() {
    use super::corpus_report_commands::corpus_print_history_row;
    let e = mock_convergence_entry(3, 97.5, 800);
    corpus_print_history_row(&e, None, false, true);
}

#[test]
fn test_corpus_print_history_row_empty_grade() {
    use super::corpus_report_commands::corpus_print_history_row;
    let mut e = mock_convergence_entry(2, 90.0, 600);
    e.grade = String::new();
    corpus_print_history_row(&e, None, false, true);
}

// ── corpus_score_print_commands tests ───────────────────────────────────────

#[test]
fn test_stats_bar_full() {
    use super::corpus_score_print_commands::stats_bar;
    let bar = stats_bar(100.0, 20);
    assert_eq!(bar.chars().filter(|c| *c == '\u{2588}').count(), 20);
}

#[test]
fn test_stats_bar_empty() {
    use super::corpus_score_print_commands::stats_bar;
    let bar = stats_bar(0.0, 20);
    assert_eq!(bar.chars().filter(|c| *c == '\u{2591}').count(), 20);
}

#[test]
fn test_stats_bar_half() {
    use super::corpus_score_print_commands::stats_bar;
    let bar = stats_bar(50.0, 20);
    assert!(bar.contains('\u{2588}'));
    assert!(bar.contains('\u{2591}'));
}

#[test]
fn test_corpus_stats_sparkline_trend_up() {
    use super::corpus_score_print_commands::corpus_stats_sparkline;
    let entries = vec![
        mock_convergence_entry(1, 90.0, 500),
        mock_convergence_entry(2, 95.0, 600),
        mock_convergence_entry(3, 99.0, 700),
    ];
    corpus_stats_sparkline(&entries);
}

#[test]
fn test_corpus_stats_sparkline_flat() {
    use super::corpus_score_print_commands::corpus_stats_sparkline;
    let entries = vec![
        mock_convergence_entry(1, 99.0, 500),
        mock_convergence_entry(2, 99.0, 500),
    ];
    corpus_stats_sparkline(&entries);
}

#[test]
fn test_corpus_print_score_human_no_failures() {
    use super::corpus_score_print_commands::corpus_print_score;
    use crate::cli::args::CorpusOutputFormat;
    let score = CorpusScore {
        total: 10,
        passed: 10,
        failed: 0,
        rate: 1.0,
        score: 99.5,
        grade: Grade::APlus,
        format_scores: vec![FormatScore {
            format: CorpusFormat::Bash,
            total: 10,
            passed: 10,
            rate: 1.0,
            score: 99.5,
            grade: Grade::APlus,
        }],
        results: (0..10)
            .map(|i| mock_result(&format!("B-{:03}", i + 1), true))
            .collect(),
    };
    let result = corpus_print_score(&score, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_score_human_with_failures() {
    use super::corpus_score_print_commands::corpus_print_score;
    use crate::cli::args::CorpusOutputFormat;
    let mut results: Vec<CorpusResult> = (0..8)
        .map(|i| mock_result(&format!("B-{:03}", i + 1), true))
        .collect();
    results.push(mock_result("B-009", false));
    results.push(mock_result("B-010", false));
    let score = CorpusScore {
        total: 10,
        passed: 8,
        failed: 2,
        rate: 0.8,
        score: 85.0,
        grade: Grade::B,
        format_scores: vec![FormatScore {
            format: CorpusFormat::Bash,
            total: 10,
            passed: 8,
            rate: 0.8,
            score: 85.0,
            grade: Grade::B,
        }],
        results,
    };
    let result = corpus_print_score(&score, &CorpusOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_print_score_json() {
    use super::corpus_score_print_commands::corpus_print_score;
    use crate::cli::args::CorpusOutputFormat;
    let score = CorpusScore {
        total: 2,
        passed: 2,
        failed: 0,
        rate: 1.0,
        score: 100.0,
        grade: Grade::APlus,
        format_scores: vec![],
        results: vec![mock_result("B-001", true), mock_result("B-002", true)],
    };
    let result = corpus_print_score(&score, &CorpusOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_corpus_load_save_last_run_returns_none_when_no_cache() {
    use super::corpus_score_print_commands::corpus_load_last_run;
    // This may return Some if there's a cache on disk, or None
    // We just ensure it doesn't panic
    let _ = corpus_load_last_run();
}

#[test]
fn test_corpus_cache_path_is_set() {
    use super::corpus_score_print_commands::CORPUS_CACHE_PATH;
    assert!(CORPUS_CACHE_PATH.contains("last-corpus-run"));
}
