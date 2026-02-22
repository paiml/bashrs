//! Corpus analysis logic: risk classification, drift metrics, decision stats, and impact scoring.
//!
//! This module contains pure logic functions for analyzing corpus run results:
//! risk classification of failures, per-dimension drift computations,
//! decision trace accumulation, and score-to-impact color mapping.
//!
//! All functions are stateless and free of I/O side effects.

use crate::corpus::runner::CorpusResult;

/// Map a dimension code to a risk level string.
///
/// Returns one of `"HIGH"`, `"MEDIUM"`, or `"LOW"`.
///
/// - `"A"` (transpilation failure) → `"HIGH"` — output cannot be used at all
/// - `"B3"` (behavioral failure) → `"HIGH"` — execution fails or hangs
/// - `"E"` (non-deterministic) → `"HIGH"` — unreliable output
/// - `"D"` (lint violations) → `"MEDIUM"` — quality issue
/// - `"G"` (cross-shell) → `"MEDIUM"` — portability issue
/// - `"F"` (metamorphic) → `"MEDIUM"` — consistency issue
/// - `"B1"`, `"B2"` → `"LOW"` — containment / cosmetic
#[must_use]
pub fn dimension_risk(dim: &str) -> &'static str {
    match dim {
        "A" => "HIGH",
        "B3" => "HIGH",
        "E" => "HIGH",
        "D" => "MEDIUM",
        "G" => "MEDIUM",
        "F" => "MEDIUM",
        "B1" | "B2" => "LOW",
        _ => "LOW",
    }
}

/// Collect classified failures from corpus results, optionally filtered by risk level.
///
/// Returns a vector of `(entry_id, dimension, risk_level)` triples. The `level_filter`,
/// when `Some`, restricts output to matching risk levels (case-insensitive).
#[must_use]
pub fn collect_risk_failures<'a>(
    results: &'a [CorpusResult],
    level_filter: Option<&str>,
    fail_dims_fn: &dyn Fn(&CorpusResult) -> Vec<&'static str>,
) -> Vec<(&'a str, &'static str, &'static str)> {
    let mut classified = Vec::new();
    for r in results {
        for dim in fail_dims_fn(r) {
            let risk = dimension_risk(dim);
            if level_filter.is_none_or(|f| risk.eq_ignore_ascii_case(f)) {
                classified.push((r.id.as_str(), dim, risk));
            }
        }
    }
    classified
}

/// Compute the pass rate delta between first and last scored convergence iterations.
///
/// Returns `(first_rate_pct, last_rate_pct, delta_pct)` as percentages (0.0–100.0).
/// Returns `(0.0, 0.0, 0.0)` if fewer than 2 scored entries are provided.
#[must_use]
pub fn compute_drift_rate(
    first_passed: usize,
    first_total: usize,
    last_passed: usize,
    last_total: usize,
) -> (f64, f64, f64) {
    let first_rate = if first_total > 0 {
        first_passed as f64 / first_total as f64 * 100.0
    } else {
        0.0
    };
    let last_rate = if last_total > 0 {
        last_passed as f64 / last_total as f64 * 100.0
    } else {
        0.0
    };
    let delta = last_rate - first_rate;
    (first_rate, last_rate, delta)
}

/// Determine the drift arrow character for a rate delta.
///
/// Returns `"\u{2191}"` (up) for positive delta, `"\u{2193}"` (down) for negative,
/// `"\u{2192}"` (right) for zero.
#[must_use]
pub fn drift_arrow(delta: f64) -> &'static str {
    if delta > 0.0 {
        "\u{2191}"
    } else if delta < 0.0 {
        "\u{2193}"
    } else {
        "\u{2192}"
    }
}

/// Map a suspiciousness score (0.0–1.0) to an impact label string.
///
/// - `>= 0.8` → `"HIGH"`
/// - `>= 0.5` → `"MEDIUM"`
/// - `< 0.5`  → `"LOW"`
#[must_use]
pub fn score_impact_label(score: f64) -> &'static str {
    if score >= 0.8 {
        "HIGH"
    } else if score >= 0.5 {
        "MEDIUM"
    } else {
        "LOW"
    }
}

/// Accumulate per-decision pass/fail statistics from a corpus result's trace.
///
/// Updates `stats` map keyed by `"decision_type:choice"` with
/// `(total_count, pass_count, fail_count)` tuples.
///
/// Returns `true` if the result had a non-empty decision trace.
pub fn accumulate_decision_stats(
    result: &CorpusResult,
    passed: bool,
    stats: &mut std::collections::HashMap<String, (usize, usize, usize)>,
) -> bool {
    let trace = match &result.decision_trace {
        Some(t) => t,
        None => return false,
    };

    for d in trace {
        let key = format!("{}:{}", d.decision_type, d.choice);
        let entry = stats.entry(key).or_insert((0, 0, 0));
        entry.0 += 1;
        if passed {
            entry.1 += 1;
        } else {
            entry.2 += 1;
        }
    }

    !trace.is_empty()
}

/// Compute whether a corpus result is overall "passing" for decision stats purposes.
///
/// A result passes if transpilation succeeded, output contains the expected string,
/// the schema is valid, lint is clean, and output is deterministic.
#[must_use]
pub fn result_passes_for_decisions(r: &CorpusResult) -> bool {
    r.transpiled && r.output_contains && r.schema_valid && r.lint_clean && r.deterministic
}

/// Count results whose suspiciousness score exceeds a threshold.
///
/// Suspiciousness is computed as `1.0 - pass_rate` for each result group.
/// This function counts how many entries have at least one failing dimension
/// with a score above `threshold`.
#[must_use]
pub fn count_suspicious(results: &[CorpusResult], threshold: f64) -> usize {
    results
        .iter()
        .filter(|r| {
            let fail_count = [
                !r.transpiled,
                !r.output_contains,
                !r.output_exact,
                !r.output_behavioral,
                !r.lint_clean,
                !r.deterministic,
                !r.metamorphic_consistent,
                !r.cross_shell_agree,
            ]
            .iter()
            .filter(|&&f| f)
            .count();
            let score = fail_count as f64 / 8.0;
            score > threshold
        })
        .count()
}

/// Compute a sparkline string from a sequence of score values.
///
/// Uses block characters `▁▂▃▄▅▆▇█` to represent relative magnitudes.
/// Returns an empty string if `scores` is empty.
#[must_use]
pub fn scores_to_sparkline(scores: &[f64]) -> String {
    if scores.is_empty() {
        return String::new();
    }
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let min = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.1);
    scores
        .iter()
        .map(|&s| {
            let idx = (((s - min) / range) * 7.0).round() as usize;
            bars[idx.min(7)]
        })
        .collect()
}

/// Compute cumulative percentage for Pareto chart rows.
///
/// Given a sorted list of `(label, count)` pairs (descending), returns a vector
/// of `(label, count, pct, cumulative_pct)` tuples.
#[must_use]
pub fn compute_pareto_rows<'a>(sorted: &'a [(&'a str, usize)], total: usize) -> Vec<(&'a str, usize, f64, f64)> {
    if total == 0 {
        return Vec::new();
    }
    let mut cumulative = 0usize;
    sorted
        .iter()
        .map(|(name, count)| {
            cumulative += count;
            let pct = *count as f64 / total as f64 * 100.0;
            let cum_pct = cumulative as f64 / total as f64 * 100.0;
            (*name, *count, pct, cum_pct)
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_result(id: &str, transpiled: bool, output_contains: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled,
            output_contains,
            output_exact: true,
            output_behavioral: true,
            has_test: false,
            coverage_ratio: 0.0,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            expected_output: None,
            actual_output: None,
            error: None,
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    fn all_pass(id: &str) -> CorpusResult {
        make_result(id, true, true)
    }

    // ===== dimension_risk tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_001_dimension_risk_a_is_high() {
        assert_eq!(dimension_risk("A"), "HIGH");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_002_dimension_risk_b3_is_high() {
        assert_eq!(dimension_risk("B3"), "HIGH");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_003_dimension_risk_e_is_high() {
        assert_eq!(dimension_risk("E"), "HIGH");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_004_dimension_risk_d_is_medium() {
        assert_eq!(dimension_risk("D"), "MEDIUM");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_005_dimension_risk_g_is_medium() {
        assert_eq!(dimension_risk("G"), "MEDIUM");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_006_dimension_risk_f_is_medium() {
        assert_eq!(dimension_risk("F"), "MEDIUM");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_007_dimension_risk_b1_b2_are_low() {
        assert_eq!(dimension_risk("B1"), "LOW");
        assert_eq!(dimension_risk("B2"), "LOW");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_008_dimension_risk_unknown_is_low() {
        assert_eq!(dimension_risk("Z"), "LOW");
        assert_eq!(dimension_risk(""), "LOW");
    }

    // ===== collect_risk_failures tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_009_collect_risk_failures_empty() {
        let fail_dims = |r: &CorpusResult| {
            crate::cli::corpus_score_logic::result_fail_dims(r)
        };
        let results = vec![all_pass("B-001")];
        let classified = collect_risk_failures(&results, None, &fail_dims);
        assert!(classified.is_empty());
    }

    #[test]
    fn test_CORPUS_ANALYSIS_010_collect_risk_failures_transpile_fail_is_high() {
        let fail_dims = |r: &CorpusResult| {
            crate::cli::corpus_score_logic::result_fail_dims(r)
        };
        let results = vec![make_result("B-001", false, true)];
        let classified = collect_risk_failures(&results, None, &fail_dims);
        assert_eq!(classified.len(), 1);
        assert_eq!(classified[0].0, "B-001");
        assert_eq!(classified[0].1, "A");
        assert_eq!(classified[0].2, "HIGH");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_011_collect_risk_failures_filter_by_level() {
        let fail_dims = |r: &CorpusResult| {
            crate::cli::corpus_score_logic::result_fail_dims(r)
        };
        let r1 = make_result("B-001", false, true); // A = HIGH
        let mut r2 = all_pass("B-002");
        r2.lint_clean = false; // D = MEDIUM
        let results = vec![r1, r2];
        let high_only = collect_risk_failures(&results, Some("HIGH"), &fail_dims);
        assert_eq!(high_only.len(), 1);
        assert_eq!(high_only[0].2, "HIGH");
    }

    // ===== compute_drift_rate tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_012_compute_drift_rate_improvement() {
        let (first, last, delta) = compute_drift_rate(50, 100, 90, 100);
        assert!((first - 50.0).abs() < 1e-9);
        assert!((last - 90.0).abs() < 1e-9);
        assert!((delta - 40.0).abs() < 1e-9);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_013_compute_drift_rate_regression() {
        let (_first, _last, delta) = compute_drift_rate(90, 100, 70, 100);
        assert!(delta < 0.0);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_014_compute_drift_rate_zero_totals() {
        let (first, last, delta) = compute_drift_rate(0, 0, 0, 0);
        assert_eq!(first, 0.0);
        assert_eq!(last, 0.0);
        assert_eq!(delta, 0.0);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_015_compute_drift_rate_stable() {
        let (_, _, delta) = compute_drift_rate(80, 100, 80, 100);
        assert_eq!(delta, 0.0);
    }

    // ===== drift_arrow and score_impact_label tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_016_drift_arrow_all_directions() {
        assert_eq!(drift_arrow(1.0), "\u{2191}");
        assert_eq!(drift_arrow(-1.0), "\u{2193}");
        assert_eq!(drift_arrow(0.0), "\u{2192}");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_017_score_impact_label_all_levels() {
        assert_eq!(score_impact_label(1.0), "HIGH");
        assert_eq!(score_impact_label(0.8), "HIGH");
        assert_eq!(score_impact_label(0.5), "MEDIUM");
        assert_eq!(score_impact_label(0.79), "MEDIUM");
        assert_eq!(score_impact_label(0.49), "LOW");
        assert_eq!(score_impact_label(0.0), "LOW");
    }

    // ===== count_suspicious tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_022_count_suspicious_none_above_threshold() {
        let results = vec![all_pass("B-001"), all_pass("B-002")];
        assert_eq!(count_suspicious(&results, 0.1), 0);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_023_count_suspicious_all_fail() {
        let mut r = all_pass("B-001");
        r.transpiled = false;
        r.output_contains = false;
        r.output_exact = false;
        r.output_behavioral = false;
        r.lint_clean = false;
        r.deterministic = false;
        r.metamorphic_consistent = false;
        r.cross_shell_agree = false;
        // 8/8 = 1.0 > any threshold
        assert_eq!(count_suspicious(&[r], 0.5), 1);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_024_count_suspicious_threshold_boundary() {
        let mut r = all_pass("B-001");
        r.transpiled = false;
        r.output_contains = false;
        r.output_exact = false;
        r.output_behavioral = false;
        // 4/8 = 0.5, threshold > 0.5 should not count
        assert_eq!(count_suspicious(&[r.clone()], 0.5), 0);
        // threshold <= 0.49 should count
        assert_eq!(count_suspicious(&[r], 0.49), 1);
    }

    // ===== scores_to_sparkline tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_025_scores_to_sparkline_empty() {
        assert_eq!(scores_to_sparkline(&[]), "");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_026_scores_to_sparkline_single() {
        let s = scores_to_sparkline(&[5.0]);
        assert_eq!(s.chars().count(), 1);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_027_scores_to_sparkline_ascending() {
        let ascending = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let s = scores_to_sparkline(&ascending);
        // Should have 5 chars, first < last in block progression
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars.len(), 5);
        assert!(chars[0] <= chars[4], "Ascending input should produce ascending sparkline");
    }

    #[test]
    fn test_CORPUS_ANALYSIS_028_scores_to_sparkline_uniform() {
        let uniform = vec![5.0, 5.0, 5.0];
        let s = scores_to_sparkline(&uniform);
        // All same value — all chars should be identical
        let chars: Vec<char> = s.chars().collect();
        assert!(chars.windows(2).all(|w| w[0] == w[1]));
    }

    // ===== compute_pareto_rows tests =====

    #[test]
    fn test_CORPUS_ANALYSIS_029_compute_pareto_rows_empty_total() {
        let sorted: Vec<(&str, usize)> = vec![("A", 5)];
        let rows = compute_pareto_rows(&sorted, 0);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_CORPUS_ANALYSIS_030_compute_pareto_rows_single_entry() {
        let sorted = vec![("A  Transpilation", 10)];
        let rows = compute_pareto_rows(&sorted, 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "A  Transpilation");
        assert_eq!(rows[0].1, 10);
        assert!((rows[0].2 - 100.0).abs() < 1e-9);
        assert!((rows[0].3 - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_031_compute_pareto_rows_cumulative_increases() {
        let sorted = vec![("A", 5), ("B1", 3), ("D", 2)];
        let rows = compute_pareto_rows(&sorted, 10);
        assert_eq!(rows.len(), 3);
        // Cumulative must be monotonically increasing
        for i in 1..rows.len() {
            assert!(rows[i].3 >= rows[i - 1].3);
        }
        // Last cumulative should be 100%
        assert!((rows[rows.len() - 1].3 - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_CORPUS_ANALYSIS_032_compute_pareto_rows_percentages_sum_to_100() {
        let sorted = vec![("A", 4), ("B", 3), ("C", 3)];
        let rows = compute_pareto_rows(&sorted, 10);
        let total_pct: f64 = rows.iter().map(|r| r.2).sum();
        assert!((total_pct - 100.0).abs() < 1e-6);
    }
}
