#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for corpus gate, metrics, and score printing helper functions.
//! Tests internal helpers WITHOUT running CorpusRunner::run().

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier, Grade};
use crate::corpus::runner::{CorpusResult, CorpusScore, FormatScore};

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
        actual_output: if all_pass { Some("echo hello".into()) } else { None },
        error: if all_pass { None } else { Some("transpile error".into()) },
        error_category: if all_pass { None } else { Some("parse_error".into()) },
        error_confidence: None,
        decision_trace: None,
    }
}

fn mock_result_custom(
    id: &str, transpiled: bool, contains: bool, exact: bool, behavioral: bool,
    lint: bool, deterministic: bool, metamorphic: bool, cross_shell: bool,
) -> CorpusResult {
    CorpusResult {
        id: id.to_string(), transpiled, output_contains: contains,
        output_exact: exact, output_behavioral: behavioral,
        has_test: true, coverage_ratio: 0.5, schema_valid: true,
        lint_clean: lint, deterministic, metamorphic_consistent: metamorphic,
        cross_shell_agree: cross_shell, expected_output: None,
        actual_output: Some("echo test".into()), error: None,
        error_category: None, error_confidence: None, decision_trace: None,
    }
}

fn mock_entry(id: &str, name: &str, format: CorpusFormat) -> CorpusEntry {
    CorpusEntry::new(
        id, name, "test desc", format, CorpusTier::Standard,
        "fn main() { println!(\"test\"); }", "echo test",
    )
}

// ── corpus_gate_commands tests ──────────────────────────────────────────────

#[test]
fn test_gate_print_check_pass() {
    use super::corpus_gate_commands::gate_print_check;
    gate_print_check("Score >= 95.0", true);
}

#[test]
fn test_gate_print_check_fail() {
    use super::corpus_gate_commands::gate_print_check;
    gate_print_check("Score >= 95.0 (actual: 80.0)", false);
}

// ── corpus_metrics_commands tests ───────────────────────────────────────────

#[test]
fn test_corpus_result_score_all_pass() {
    let r = mock_result("B-001", true);
    let s = r.score();
    // A(30) + B1(10) + B2(8) + B3(7) + C(0.95*15=14.25) + D(10) + E(10) + F(5) + G(5) = 99.25
    assert!((s - 99.25).abs() < 0.01, "Expected 99.25, got {s}");
}

#[test]
fn test_corpus_result_score_all_fail() {
    let r = mock_result("B-001", false);
    assert!((r.score() - 0.0).abs() < 0.01);
}

#[test]
fn test_corpus_result_score_partial() {
    let r = mock_result_custom("B-001", true, true, false, false, true, false, true, false);
    let s = r.score();
    // A(30) + B1(10) + B2(0, exact=false) + B3(0, contains but behavioral=false => 0? No,
    // b3 = if contains && behavioral -> 7 else 0. contains=true, behavioral=false => 0)
    // C(0.5*15=7.5) + D(10) + E(0) + F(5) + G(0) = 62.5
    assert!((s - 62.5).abs() < 0.01, "Expected 62.5, got {s}");
}

#[test]
fn test_corpus_result_score_schema_invalid() {
    let mut r = mock_result("B-001", true);
    r.schema_valid = false;
    assert!((r.score() - 0.0).abs() < 0.01);
}

#[test]
fn test_result_fail_dims_all_pass() {
    use super::corpus_failure_commands::result_fail_dims;
    let r = mock_result("B-001", true);
    assert!(result_fail_dims(&r).is_empty());
}

#[test]
fn test_result_fail_dims_all_fail() {
    use super::corpus_failure_commands::result_fail_dims;
    let r = mock_result("B-001", false);
    let dims = result_fail_dims(&r);
    assert_eq!(dims.len(), 8);
    assert!(dims.contains(&"A"));
    assert!(dims.contains(&"G"));
}

#[test]
fn test_result_fail_dims_mixed() {
    use super::corpus_failure_commands::result_fail_dims;
    let r = mock_result_custom("B-001", true, true, false, true, false, true, true, true);
    let dims = result_fail_dims(&r);
    assert_eq!(dims.len(), 2); // B2 and D
    assert!(dims.contains(&"B2"));
    assert!(dims.contains(&"D"));
}

#[test]
fn test_count_dimension_failures() {
    use super::corpus_failure_commands::count_dimension_failures;
    let results = vec![
        mock_result("B-001", true),
        mock_result("B-002", false),
        mock_result_custom("B-003", true, true, false, true, true, true, true, true),
    ];
    let dims = count_dimension_failures(&results);
    // "A Transpilation": 1 fail (B-002)
    assert!(dims.iter().any(|(name, count)| name.contains("Transpilation") && *count == 1));
}

#[test]
fn test_score_impact_color_high() {
    use super::corpus_decision_commands::score_impact_color;
    let (label, _color) = score_impact_color(0.9);
    assert!(label.contains("HIGH"));
}

#[test]
fn test_score_impact_color_medium() {
    use super::corpus_decision_commands::score_impact_color;
    let (label, _color) = score_impact_color(0.6);
    assert!(label.contains("MEDIUM"));
}

#[test]
fn test_score_impact_color_low() {
    use super::corpus_decision_commands::score_impact_color;
    let (label, _color) = score_impact_color(0.2);
    assert!(label.contains("LOW"));
}

#[test]
fn test_accumulate_decision_stats_no_trace() {
    use super::corpus_decision_commands::accumulate_decision_stats;
    let r = mock_result("B-001", true);
    let mut stats = std::collections::HashMap::new();
    let had_trace = accumulate_decision_stats(&r, &mut stats);
    assert!(!had_trace);
    assert!(stats.is_empty());
}

#[test]
fn test_accumulate_decision_stats_with_trace() {
    use super::corpus_decision_commands::accumulate_decision_stats;
    use crate::emitter::trace::TranspilerDecision;
    let mut r = mock_result("B-001", true);
    r.decision_trace = Some(vec![
        TranspilerDecision {
            decision_type: "emit_type".to_string(),
            choice: "posix_sh".to_string(),
            ir_node: "FunctionDef".to_string(),
        },
        TranspilerDecision {
            decision_type: "emit_type".to_string(),
            choice: "posix_sh".to_string(),
            ir_node: "Assignment".to_string(),
        },
    ]);
    let mut stats = std::collections::HashMap::new();
    let had_trace = accumulate_decision_stats(&r, &mut stats);
    assert!(had_trace);
    assert!(stats.contains_key("emit_type:posix_sh"));
    let (total, pass, fail) = stats["emit_type:posix_sh"];
    assert_eq!(total, 2);
    assert_eq!(pass, 2);
    assert_eq!(fail, 0);
}

#[test]
fn test_accumulate_decision_stats_failed_entry() {
    use super::corpus_decision_commands::accumulate_decision_stats;
    use crate::emitter::trace::TranspilerDecision;
    let mut r = mock_result("B-001", false);
    r.transpiled = true;
    r.output_contains = false; // causes "passed" to be false
    r.decision_trace = Some(vec![
        TranspilerDecision {
            decision_type: "branch".to_string(),
            choice: "if_else".to_string(),
            ir_node: "If".to_string(),
        },
    ]);
    let mut stats = std::collections::HashMap::new();
    accumulate_decision_stats(&r, &mut stats);
    let (total, pass, fail) = stats["branch:if_else"];
    assert_eq!(total, 1);
    assert_eq!(pass, 0);
    assert_eq!(fail, 1);
}

// ── corpus_diag_commands tests ──────────────────────────────────────────────

#[test]
fn test_result_dim_pass_all_dimensions() {
    use super::corpus_diag_commands::result_dim_pass;
    let r = mock_result("B-001", true);
    for dim_idx in 0..8 {
        assert!(result_dim_pass(&r, dim_idx), "dim {dim_idx} should pass");
    }
}

#[test]
fn test_result_dim_pass_all_fail() {
    use super::corpus_diag_commands::result_dim_pass;
    let r = mock_result("B-001", false);
    for dim_idx in 0..8 {
        assert!(!result_dim_pass(&r, dim_idx), "dim {dim_idx} should fail");
    }
}

#[test]
fn test_result_dim_pass_specific() {
    use super::corpus_diag_commands::result_dim_pass;
    let r = mock_result_custom("B-001", true, false, true, false, true, false, true, false);
    assert!(result_dim_pass(&r, 0));  // transpiled
    assert!(!result_dim_pass(&r, 1)); // output_contains
    assert!(result_dim_pass(&r, 2));  // output_exact
    assert!(!result_dim_pass(&r, 3)); // output_behavioral
    assert!(result_dim_pass(&r, 4));  // lint_clean
    assert!(!result_dim_pass(&r, 5)); // deterministic
    assert!(result_dim_pass(&r, 6));  // metamorphic
    assert!(!result_dim_pass(&r, 7)); // cross_shell
}

#[test]
fn test_dim_format_rate_all_pass() {
    use super::corpus_diag_commands::dim_format_rate;
    let registry = crate::corpus::registry::CorpusRegistry {
        entries: vec![
            mock_entry("B-001", "t1", CorpusFormat::Bash),
            mock_entry("B-002", "t2", CorpusFormat::Bash),
        ],
    };
    let results = vec![mock_result("B-001", true), mock_result("B-002", true)];
    let rate = dim_format_rate(&registry, &results, CorpusFormat::Bash, 0);
    assert!((rate - 100.0).abs() < 0.01);
}

#[test]
fn test_dim_format_rate_half_pass() {
    use super::corpus_diag_commands::dim_format_rate;
    let registry = crate::corpus::registry::CorpusRegistry {
        entries: vec![
            mock_entry("B-001", "t1", CorpusFormat::Bash),
            mock_entry("B-002", "t2", CorpusFormat::Bash),
        ],
    };
    let results = vec![mock_result("B-001", true), mock_result("B-002", false)];
    let rate = dim_format_rate(&registry, &results, CorpusFormat::Bash, 0);
    assert!((rate - 50.0).abs() < 0.01);
}

#[test]
fn test_dim_format_rate_no_entries_returns_100() {
    use super::corpus_diag_commands::dim_format_rate;
    let registry = crate::corpus::registry::CorpusRegistry {
        entries: vec![mock_entry("B-001", "t1", CorpusFormat::Bash)],
    };
    let results = vec![mock_result("B-001", true)];
    // Looking for Makefile format when only Bash exists
    let rate = dim_format_rate(&registry, &results, CorpusFormat::Makefile, 0);
    assert!((rate - 100.0).abs() < 0.01);
}

#[test]
fn test_dim_format_rate_different_dims() {
    use super::corpus_diag_commands::dim_format_rate;
    let registry = crate::corpus::registry::CorpusRegistry {
        entries: vec![mock_entry("B-001", "t1", CorpusFormat::Bash)],
    };
    let r = mock_result_custom("B-001", true, true, false, true, false, true, false, true);
    let results = vec![r];
    // dim 0 (transpiled) = true => 100%
    assert!((dim_format_rate(&registry, &results, CorpusFormat::Bash, 0) - 100.0).abs() < 0.01);
    // dim 2 (output_exact) = false => 0%
    assert!((dim_format_rate(&registry, &results, CorpusFormat::Bash, 2) - 0.0).abs() < 0.01);
    // dim 4 (lint_clean) = false => 0%
    assert!((dim_format_rate(&registry, &results, CorpusFormat::Bash, 4) - 0.0).abs() < 0.01);
}

// ── corpus_ranking_commands tests ───────────────────────────────────────────

#[test]
fn test_sparkline_str_empty() {
    use super::corpus_ranking_commands::sparkline_str;
    assert_eq!(sparkline_str(&[]), "");
}

#[test]
fn test_sparkline_str_single() {
    use super::corpus_ranking_commands::sparkline_str;
    let result = sparkline_str(&[50.0]);
    assert_eq!(result.len(), 3); // one unicode char (3 bytes)
}

#[test]
fn test_sparkline_str_ascending() {
    use super::corpus_ranking_commands::sparkline_str;
    let result = sparkline_str(&[0.0, 25.0, 50.0, 75.0, 100.0]);
    assert!(!result.is_empty());
    assert_eq!(result.chars().count(), 5);
}

#[test]
fn test_sparkline_str_flat() {
    use super::corpus_ranking_commands::sparkline_str;
    let result = sparkline_str(&[99.0, 99.0, 99.0]);
    // All same → all full blocks
    assert_eq!(result.chars().count(), 3);
}

#[test]
fn test_classify_category_config() {
    use super::corpus_ranking_commands::classify_category;
    assert_eq!(classify_category("config-parser"), "Config (A)");
}

#[test]
fn test_classify_category_general() {
    use super::corpus_ranking_commands::classify_category;
    assert_eq!(classify_category("random-name-xyz"), "General");
}

// ── CorpusScore/FormatScore tests ───────────────────────────────────────────

#[test]
fn test_corpus_score_gateway_met() {
    let score = CorpusScore {
        total: 100, passed: 80, failed: 20, rate: 0.8,
        score: 80.0, grade: Grade::B,
        format_scores: vec![], results: vec![],
    };
    assert!(score.gateway_met());
}

#[test]
fn test_corpus_score_gateway_not_met() {
    let score = CorpusScore {
        total: 100, passed: 50, failed: 50, rate: 0.5,
        score: 50.0, grade: Grade::F,
        format_scores: vec![], results: vec![],
    };
    assert!(!score.gateway_met());
}

#[test]
fn test_corpus_score_format_score_lookup() {
    let score = CorpusScore {
        total: 10, passed: 10, failed: 0, rate: 1.0,
        score: 99.0, grade: Grade::APlus,
        format_scores: vec![
            FormatScore { format: CorpusFormat::Bash, total: 10, passed: 10, rate: 1.0, score: 99.0, grade: Grade::APlus },
        ],
        results: vec![],
    };
    assert!(score.format_score(CorpusFormat::Bash).is_some());
    assert!(score.format_score(CorpusFormat::Makefile).is_none());
}

#[test]
fn test_grade_from_score_all_thresholds() {
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(97.0), Grade::APlus);
    assert_eq!(Grade::from_score(96.9), Grade::A);
    assert_eq!(Grade::from_score(90.0), Grade::A);
    assert_eq!(Grade::from_score(89.9), Grade::B);
    assert_eq!(Grade::from_score(80.0), Grade::B);
    assert_eq!(Grade::from_score(79.9), Grade::C);
    assert_eq!(Grade::from_score(70.0), Grade::C);
    assert_eq!(Grade::from_score(69.9), Grade::D);
    assert_eq!(Grade::from_score(60.0), Grade::D);
    assert_eq!(Grade::from_score(59.9), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
}

#[test]
fn test_grade_display() {
    assert_eq!(Grade::APlus.to_string(), "A+");
    assert_eq!(Grade::A.to_string(), "A");
    assert_eq!(Grade::B.to_string(), "B");
    assert_eq!(Grade::C.to_string(), "C");
    assert_eq!(Grade::D.to_string(), "D");
    assert_eq!(Grade::F.to_string(), "F");
}
