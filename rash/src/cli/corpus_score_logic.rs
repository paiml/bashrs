//! Corpus scoring, difficulty classification, and dimension analysis logic.
//!
//! This module contains pure logic functions used by corpus CLI commands for
//! computing percentiles, classifying entry difficulty, counting dimension
//! failures, and categorizing corpus entries by domain.
//!
//! All functions are stateless and free of I/O side effects.

use crate::corpus::runner::CorpusResult;

/// Compute a percentile value from a pre-sorted slice of f64 values.
///
/// Returns 0.0 for empty slices. Uses nearest-rank method.
///
/// # Arguments
/// * `sorted` - A sorted (ascending) slice of f64 values.
/// * `p` - Percentile to compute (0.0–100.0).
#[must_use]
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Compute arithmetic mean of a slice. Returns 0.0 for empty slices.
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Get the failing dimension labels for a corpus result.
///
/// Returns a vector of short dimension codes (e.g. `["A", "B1"]`) for each
/// failing dimension, in order: A, B1, B2, B3, D, E, F, G.
#[must_use]
pub fn result_fail_dims(r: &CorpusResult) -> Vec<&'static str> {
    [
        (!r.transpiled, "A"),
        (!r.output_contains, "B1"),
        (!r.output_exact, "B2"),
        (!r.output_behavioral, "B3"),
        (!r.lint_clean, "D"),
        (!r.deterministic, "E"),
        (!r.metamorphic_consistent, "F"),
        (!r.cross_shell_agree, "G"),
    ]
    .iter()
    .filter_map(|(f, d)| if *f { Some(*d) } else { None })
    .collect()
}

/// Count failures per V2 dimension from corpus results.
///
/// Returns a vector of `(dimension_label, count)` pairs sorted descending by
/// count. Only includes dimensions with at least one failure.
#[must_use]
pub fn count_dimension_failures(results: &[CorpusResult]) -> Vec<(&'static str, usize)> {
    let dims = [
        (
            "A  Transpilation",
            results.iter().filter(|r| !r.transpiled).count(),
        ),
        (
            "B1 Containment",
            results.iter().filter(|r| !r.output_contains).count(),
        ),
        (
            "B2 Exact match",
            results.iter().filter(|r| !r.output_exact).count(),
        ),
        (
            "B3 Behavioral",
            results.iter().filter(|r| !r.output_behavioral).count(),
        ),
        (
            "D  Lint clean",
            results.iter().filter(|r| !r.lint_clean).count(),
        ),
        (
            "E  Deterministic",
            results.iter().filter(|r| !r.deterministic).count(),
        ),
        (
            "F  Metamorphic",
            results.iter().filter(|r| !r.metamorphic_consistent).count(),
        ),
        (
            "G  Cross-shell",
            results.iter().filter(|r| !r.cross_shell_agree).count(),
        ),
        ("Schema", results.iter().filter(|r| !r.schema_valid).count()),
    ];
    let mut sorted: Vec<_> = dims.into_iter().filter(|(_, c)| *c > 0).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

/// Classify a corpus entry's difficulty based on input features (spec §2.3).
///
/// Returns `(tier, factors)` where tier is 1–5 and factors is a breakdown of
/// detected complexity indicators.
#[must_use]
pub fn classify_difficulty(input: &str) -> (u8, Vec<(&'static str, bool)>) {
    let lines: Vec<&str> = input.lines().collect();
    let line_count = lines.len();
    let has_fn = input.contains("fn ") && input.matches("fn ").count() > 1;
    let has_loop = input.contains("for ") || input.contains("while ") || input.contains("loop ");
    let has_pipe = input.contains('|');
    let has_if = input.contains("if ");
    let has_match = input.contains("match ");
    let has_nested = input.matches('{').count() > 3;
    let has_special = input.contains('\\') || input.contains("\\n") || input.contains("\\t");
    let has_unicode = !input.is_ascii();
    let has_unsafe = input.contains("unsafe") || input.contains("exec") || input.contains("eval");

    let mut factors = vec![
        (
            "Simple (single construct)",
            line_count <= 3 && !has_loop && !has_fn,
        ),
        ("Has loops", has_loop),
        ("Has multiple functions", has_fn),
        ("Has pipes/redirects", has_pipe),
        ("Has conditionals", has_if || has_match),
        ("Has deep nesting (>3)", has_nested),
        ("Has special chars/escapes", has_special),
        ("Has Unicode", has_unicode),
        ("Has unsafe/exec patterns", has_unsafe),
    ];

    let complexity: u32 = [
        has_loop as u32,
        has_fn as u32 * 2,
        has_pipe as u32,
        (has_if || has_match) as u32,
        has_nested as u32 * 2,
        has_special as u32,
        has_unicode as u32 * 2,
        has_unsafe as u32 * 3,
        (line_count > 10) as u32,
        (line_count > 30) as u32 * 2,
    ]
    .iter()
    .sum();

    let tier = match complexity {
        0..=1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        7..=9 => 4,
        _ => 5,
    };

    factors.push(("POSIX-safe (no bashisms)", !has_unsafe && !has_unicode));

    (tier, factors)
}

/// Return a human-readable label for a difficulty tier (1–5).
#[must_use]
pub fn tier_label(tier: u8) -> &'static str {
    match tier {
        1 => "Trivial",
        2 => "Standard",
        3 => "Complex",
        4 => "Adversarial",
        5 => "Production",
        _ => "Unknown",
    }
}

/// Classify entry into a domain-specific category based on its name (spec §11.11).
///
/// Returns a short category string such as `"Config (A)"`, `"One-liner (B)"`, etc.
#[must_use]
pub fn classify_category(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if n.contains("config")
        || n.contains("bashrc")
        || n.contains("profile")
        || n.contains("alias")
        || n.contains("xdg")
        || n.contains("history")
    {
        "Config (A)"
    } else if n.contains("oneliner")
        || n.contains("one-liner")
        || n.contains("pipe-")
        || n.contains("pipeline")
    {
        "One-liner (B)"
    } else if n.contains("coreutil") || n.contains("reimpl") {
        "Coreutils (G)"
    } else if n.contains("regex") || n.contains("pattern-match") || n.contains("glob-match") {
        "Regex (H)"
    } else if n.contains("daemon")
        || n.contains("cron")
        || n.contains("startup")
        || n.contains("service")
    {
        "System (F)"
    } else if n.contains("milestone") {
        "Milestone"
    } else if n.contains("adversarial") || n.contains("injection") || n.contains("fuzz") {
        "Adversarial"
    } else {
        "General"
    }
}

/// Truncate a string to the first line, with `"..."` suffix if truncated.
#[must_use]
pub fn truncate_line(s: &str, max_len: usize) -> String {
    let line = s.lines().next().unwrap_or(s);
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len])
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_result(
        transpiled: bool,
        output_contains: bool,
        output_exact: bool,
        output_behavioral: bool,
        lint_clean: bool,
        deterministic: bool,
        metamorphic_consistent: bool,
        cross_shell_agree: bool,
        schema_valid: bool,
    ) -> CorpusResult {
        CorpusResult {
            id: "B-001".to_string(),
            transpiled,
            output_contains,
            output_exact,
            output_behavioral,
            has_test: false,
            coverage_ratio: 0.0,
            schema_valid,
            lint_clean,
            deterministic,
            metamorphic_consistent,
            cross_shell_agree,
            expected_output: None,
            actual_output: None,
            error: None,
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    fn all_pass() -> CorpusResult {
        make_result(true, true, true, true, true, true, true, true, true)
    }

    fn all_fail() -> CorpusResult {
        make_result(false, false, false, false, false, false, false, false, false)
    }

    // ===== percentile tests =====

    #[test]
    fn test_CORPUS_SCORE_001_percentile_empty_returns_zero() {
        assert_eq!(percentile(&[], 50.0), 0.0);
    }

    #[test]
    fn test_CORPUS_SCORE_002_percentile_single_element() {
        assert_eq!(percentile(&[42.0], 50.0), 42.0);
        assert_eq!(percentile(&[42.0], 0.0), 42.0);
        assert_eq!(percentile(&[42.0], 100.0), 42.0);
    }

    #[test]
    fn test_CORPUS_SCORE_003_percentile_p50_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 50.0), 3.0);
    }

    #[test]
    fn test_CORPUS_SCORE_004_percentile_p0_p100_extremes() {
        let data = vec![1.0, 5.0, 10.0, 20.0, 100.0];
        assert_eq!(percentile(&data, 0.0), 1.0);
        assert_eq!(percentile(&data, 100.0), 100.0);
    }

    #[test]
    fn test_CORPUS_SCORE_006_percentile_p99_near_max() {
        let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        // For 100 elements p99 should be near 100
        assert!(percentile(&data, 99.0) >= 98.0);
    }

    // ===== mean tests =====

    #[test]
    fn test_CORPUS_SCORE_007_mean_empty_returns_zero() {
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_CORPUS_SCORE_008_mean_single_element() {
        assert_eq!(mean(&[7.0]), 7.0);
    }

    #[test]
    fn test_CORPUS_SCORE_009_mean_uniform_values() {
        assert_eq!(mean(&[4.0, 4.0, 4.0, 4.0]), 4.0);
    }

    #[test]
    fn test_CORPUS_SCORE_010_mean_varied_values() {
        let vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&vals) - 3.0).abs() < 1e-9);
    }

    // ===== result_fail_dims tests =====

    #[test]
    fn test_CORPUS_SCORE_011_result_fail_dims_all_pass_empty() {
        let r = all_pass();
        assert!(result_fail_dims(&r).is_empty());
    }

    #[test]
    fn test_CORPUS_SCORE_012_result_fail_dims_all_fail() {
        let r = all_fail();
        let dims = result_fail_dims(&r);
        assert!(dims.contains(&"A"));
        assert!(dims.contains(&"B1"));
        assert!(dims.contains(&"B2"));
        assert!(dims.contains(&"B3"));
        assert!(dims.contains(&"D"));
        assert!(dims.contains(&"E"));
        assert!(dims.contains(&"F"));
        assert!(dims.contains(&"G"));
    }

    #[test]
    fn test_CORPUS_SCORE_013_result_fail_dims_only_transpile_fails() {
        let mut r = all_pass();
        r.transpiled = false;
        let dims = result_fail_dims(&r);
        assert_eq!(dims, vec!["A"]);
    }

    #[test]
    fn test_CORPUS_SCORE_014_result_fail_dims_b1_b2_fail() {
        let mut r = all_pass();
        r.output_contains = false;
        r.output_exact = false;
        let dims = result_fail_dims(&r);
        assert!(dims.contains(&"B1"));
        assert!(dims.contains(&"B2"));
        assert!(!dims.contains(&"A"));
    }

    // ===== count_dimension_failures tests =====

    #[test]
    fn test_CORPUS_SCORE_015_count_dimension_failures_empty() {
        let result = count_dimension_failures(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_CORPUS_SCORE_016_count_dimension_failures_all_pass() {
        let results = vec![all_pass(), all_pass()];
        let result = count_dimension_failures(&results);
        assert!(result.is_empty());
    }

    #[test]
    fn test_CORPUS_SCORE_017_count_dimension_failures_some_fail() {
        let mut r1 = all_pass();
        r1.transpiled = false;
        let mut r2 = all_pass();
        r2.transpiled = false;
        r2.output_contains = false;
        let results = vec![r1, r2];
        let failures = count_dimension_failures(&results);
        // A = 2, B1 = 1, sorted descending
        assert_eq!(failures[0].0, "A  Transpilation");
        assert_eq!(failures[0].1, 2);
        assert_eq!(failures[1].0, "B1 Containment");
        assert_eq!(failures[1].1, 1);
    }

    #[test]
    fn test_CORPUS_SCORE_018_count_dimension_failures_sorted_descending() {
        let mut results = Vec::new();
        for _ in 0..5 {
            let mut r = all_pass();
            r.lint_clean = false; // D fails 5 times
            results.push(r);
        }
        for _ in 0..2 {
            let mut r = all_pass();
            r.transpiled = false; // A fails 2 times
            results.push(r);
        }
        let failures = count_dimension_failures(&results);
        // D should be first (5 > 2)
        assert_eq!(failures[0].0, "D  Lint clean");
        assert_eq!(failures[0].1, 5);
    }

    // ===== classify_difficulty tests =====

    #[test]
    fn test_CORPUS_SCORE_019_classify_difficulty_simple_code() {
        let (tier, _factors) = classify_difficulty("fn main() {\n  println!(\"hello\");\n}");
        assert!(tier <= 2, "Simple code should be tier 1 or 2, got {tier}");
    }

    #[test]
    fn test_CORPUS_SCORE_020_classify_difficulty_complex_code() {
        let complex = r#"
fn main() {
    for i in 0..10 {
        if i % 2 == 0 {
            unsafe { exec("rm -rf /") }
        }
    }
    let bytes: Vec<u8> = vec![0x41, 0x42, 0x43];
    for b in bytes { println!("{}", b); }
}
fn helper() { loop { break; } }
"#;
        let (tier, _) = classify_difficulty(complex);
        assert!(tier >= 3, "Complex code should be tier 3+, got {tier}");
    }

    #[test]
    fn test_CORPUS_SCORE_021_classify_difficulty_factors_returned() {
        let (_, factors) = classify_difficulty("fn main() { println!(\"hi\"); }");
        assert!(!factors.is_empty());
    }

    #[test]
    fn test_CORPUS_SCORE_022_classify_difficulty_unicode_raises_tier() {
        let with_unicode = "fn main() { println!(\"\u{1F600}\"); }";
        let without_unicode = "fn main() { println!(\"hello\"); }";
        let (t1, _) = classify_difficulty(with_unicode);
        let (t2, _) = classify_difficulty(without_unicode);
        assert!(t1 >= t2, "Unicode should raise or equal tier");
    }

    // ===== tier_label tests =====

    #[test]
    fn test_CORPUS_SCORE_023_tier_label_all_tiers() {
        assert_eq!(tier_label(1), "Trivial");
        assert_eq!(tier_label(2), "Standard");
        assert_eq!(tier_label(3), "Complex");
        assert_eq!(tier_label(4), "Adversarial");
        assert_eq!(tier_label(5), "Production");
        assert_eq!(tier_label(99), "Unknown");
    }

    // ===== classify_category tests =====

    #[test]
    fn test_CORPUS_SCORE_024_classify_category_all_known() {
        assert_eq!(classify_category("bashrc setup"), "Config (A)");
        assert_eq!(classify_category("pipeline-sort"), "One-liner (B)");
        assert_eq!(classify_category("coreutil-ls"), "Coreutils (G)");
        assert_eq!(classify_category("adversarial injection"), "Adversarial");
        assert_eq!(classify_category("daemon startup"), "System (F)");
        assert_eq!(classify_category("milestone 100"), "Milestone");
        assert_eq!(classify_category("regex pattern-match"), "Regex (H)");
        assert_eq!(classify_category("random test"), "General");
    }

    // ===== truncate_line tests =====

    #[test]
    fn test_CORPUS_SCORE_025_truncate_line_short_and_long() {
        assert_eq!(truncate_line("hello", 10), "hello");
        assert_eq!(truncate_line("hello world", 5), "hello...");
        assert_eq!(truncate_line("line1\nline2\nline3", 20), "line1");
        assert_eq!(truncate_line("hello", 5), "hello");
    }
}
