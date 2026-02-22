#![allow(dead_code)]
//! Corpus convergence logic: sparkline trends, bar charts, tier classification,
//! convergence log summaries, lint gap computation, and regression formatting.
//! All functions are stateless and free of I/O side effects.

use std::collections::HashMap;
use crate::cli::corpus_score_logic::{classify_difficulty, tier_label};

/// Trend direction for sparkline display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SparklineTrend { Up, Down, Flat }

/// Computed sparkline data from convergence entries.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SparklineData {
    pub sparkline: String,
    pub first: f64,
    pub last: f64,
    pub trend: SparklineTrend,
}

/// Result of classifying all corpus entries by tier.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TierClassificationResult {
    pub total: usize,
    /// Per-tier counts (index 0 unused, 1-5 for tiers).
    pub tier_counts: [u32; 6],
    /// Per-format tier counts keyed by format prefix ("B", "M", "D").
    pub format_tiers: HashMap<String, [u32; 6]>,
}

/// A single tier row for human-readable display.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TierClassifyRow {
    pub tier: u8, pub label: &'static str, pub count: u32, pub pct: f64, pub bar: String,
}

/// A single tier entry for JSON serialization.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TierJsonEntry {
    pub tier: u8, pub label: String, pub count: u32,
}

/// Per-format tier breakdown for display.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FormatTierSummary {
    pub key: String, pub label: &'static str, pub parts: Vec<String>,
}

/// Computed convergence log summary fields.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ConvergenceLogSummary {
    pub iteration: u32,
    pub prev_rate: f64,
}

/// Lint rate gap computation result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LintGapResult {
    pub lint_pct: f64,
    /// Gap between overall rate and lint rate, if significant (> 0.1%).
    pub gap: Option<f64>,
}

/// Formatted regression messages from a regression report.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RegressionMessages {
    pub has_regressions: bool,
    pub messages: Vec<String>,
}

/// Format a bar chart string for a percentage value (0.0-100.0).
/// Returns `width` characters using filled and empty block characters.
#[must_use]
pub(crate) fn stats_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

/// Compute sparkline data from a sequence of score values.
/// Returns `None` if scores is empty.
#[must_use]
pub(crate) fn compute_sparkline_data(scores: &[f64]) -> Option<SparklineData> {
    if scores.is_empty() {
        return None;
    }
    let bars = ['\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}',
                '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
    let min = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.1);
    let sparkline: String = scores.iter().map(|&s| {
        let idx = (((s - min) / range) * 7.0).round() as usize;
        bars[idx.min(7)]
    }).collect();
    let first = scores.first().copied().unwrap_or(0.0);
    let last = scores.last().copied().unwrap_or(0.0);
    let trend = if last > first { SparklineTrend::Up }
        else if last < first { SparklineTrend::Down }
        else { SparklineTrend::Flat };
    Some(SparklineData { sparkline, first, last, trend })
}

/// Classify all corpus entries and aggregate tier counts.
/// Each entry is `(id, input)` where `id` determines the format prefix.
#[must_use]
pub(crate) fn classify_all_entries(entries: &[(&str, &str)]) -> TierClassificationResult {
    let mut tier_counts = [0u32; 6];
    let mut format_tiers: HashMap<String, [u32; 6]> = HashMap::new();
    for (id, input) in entries {
        let (tier, _) = classify_difficulty(input);
        tier_counts[tier as usize] += 1;
        let fmt_key = id.chars().next().unwrap_or('?').to_string();
        format_tiers.entry(fmt_key).or_insert([0u32; 6])[tier as usize] += 1;
    }
    TierClassificationResult { total: entries.len(), tier_counts, format_tiers }
}

/// Build display rows for tier classification (tiers 1-5) with bar chart.
#[must_use]
pub(crate) fn tier_classify_rows(result: &TierClassificationResult, bar_width: usize) -> Vec<TierClassifyRow> {
    (1..=5u8).map(|t| {
        let count = result.tier_counts[t as usize];
        let pct = if result.total == 0 { 0.0 }
            else { count as f64 / result.total as f64 * 100.0 };
        TierClassifyRow { tier: t, label: tier_label(t), count, pct, bar: stats_bar(pct, bar_width) }
    }).collect()
}

/// Build JSON-serializable tier entries from classification results.
#[must_use]
pub(crate) fn tier_json_entries(result: &TierClassificationResult) -> Vec<TierJsonEntry> {
    (1..=5u8).map(|t| TierJsonEntry {
        tier: t, label: tier_label(t).to_string(), count: result.tier_counts[t as usize],
    }).collect()
}

/// Build per-format tier breakdown summaries for display.
#[must_use]
pub(crate) fn format_tier_summaries(result: &TierClassificationResult) -> Vec<FormatTierSummary> {
    [("B", "Bash"), ("M", "Makefile"), ("D", "Dockerfile")].iter().filter_map(|&(key, label)| {
        result.format_tiers.get(key).and_then(|ft| {
            let parts: Vec<String> = (1..=5u8)
                .filter(|&t| ft[t as usize] > 0)
                .map(|t| format!("T{t}:{}", ft[t as usize])).collect();
            if parts.is_empty() { None }
            else { Some(FormatTierSummary { key: key.to_string(), label, parts }) }
        })
    }).collect()
}

/// Compute convergence log summary: next iteration number and previous rate.
#[must_use]
pub(crate) fn convergence_log_summary(previous_count: usize, prev_last_rate: Option<f64>) -> ConvergenceLogSummary {
    ConvergenceLogSummary {
        iteration: previous_count as u32 + 1,
        prev_rate: prev_last_rate.unwrap_or(0.0),
    }
}

/// Compute lint rate gap. Returns `None` if `lint_passed` is 0.
#[must_use]
pub(crate) fn compute_lint_gap(lint_passed: usize, _total: usize, lint_rate: f64, overall_rate: f64) -> Option<LintGapResult> {
    if lint_passed == 0 { return None; }
    let lint_pct = lint_rate * 100.0;
    let gap_val = ((overall_rate - lint_rate) * 100.0).abs();
    Some(LintGapResult { lint_pct, gap: if gap_val > 0.1 { Some(gap_val) } else { None } })
}

/// Format per-format breakdown parts for convergence display.
/// Returns empty vec if all totals are zero.
#[must_use]
pub(crate) fn format_per_format_parts(
    bp: usize, bt: usize, mp: usize, mt: usize, dp: usize, dt: usize,
) -> Vec<String> {
    if bt == 0 && mt == 0 && dt == 0 { return Vec::new(); }
    let f = |name: &str, p: usize, t: usize| if t > 0 { format!("{name} {p}/{t}") } else { String::new() };
    [f("Bash", bp, bt), f("Make", mp, mt), f("Docker", dp, dt)]
        .into_iter().filter(|s| !s.is_empty()).collect()
}

/// Build regression messages from a slice of message strings.
#[must_use]
pub(crate) fn build_regression_messages(msgs: &[String]) -> RegressionMessages {
    RegressionMessages { has_regressions: !msgs.is_empty(), messages: msgs.to_vec() }
}

/// Format lint rate display. Returns `(lint_pct_str, gap_str)`.
#[must_use]
pub(crate) fn format_lint_display(lint_passed: usize, total: usize, gap: &LintGapResult) -> (String, String) {
    let lint_str = format!("{:.1}% ({}/{})", gap.lint_pct, lint_passed, total);
    let gap_str = match gap.gap {
        Some(g) => format!("(gap: {g:.1}%)"),
        None => String::new(),
    };
    (lint_str, gap_str)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // stats_bar
    #[test] fn test_CORPUS_CONV_001_stats_bar_zero_pct() {
        assert_eq!(stats_bar(0.0, 10), "\u{2591}".repeat(10));
    }
    #[test] fn test_CORPUS_CONV_002_stats_bar_full_pct() {
        assert_eq!(stats_bar(100.0, 10), "\u{2588}".repeat(10));
    }
    #[test] fn test_CORPUS_CONV_003_stats_bar_half_pct() {
        let b = stats_bar(50.0, 10);
        assert_eq!(b.chars().filter(|&c| c == '\u{2588}').count(), 5);
        assert_eq!(b.chars().filter(|&c| c == '\u{2591}').count(), 5);
    }
    #[test] fn test_CORPUS_CONV_004_stats_bar_width_zero() {
        assert!(stats_bar(50.0, 0).is_empty());
    }
    #[test] fn test_CORPUS_CONV_005_stats_bar_width_one() {
        assert_eq!(stats_bar(100.0, 1), "\u{2588}");
    }
    #[test] fn test_CORPUS_CONV_006_stats_bar_over_100() {
        assert!(stats_bar(150.0, 10).chars().filter(|&c| c == '\u{2588}').count() >= 10);
    }
    #[test] fn test_CORPUS_CONV_042_stats_bar_width_16() {
        let b = stats_bar(75.0, 16);
        assert_eq!(b.chars().count(), 16);
        assert_eq!(b.chars().filter(|&c| c == '\u{2588}').count(), 12);
    }
    // sparkline
    #[test] fn test_CORPUS_CONV_007_sparkline_empty() {
        assert!(compute_sparkline_data(&[]).is_none());
    }
    #[test] fn test_CORPUS_CONV_008_sparkline_single() {
        let d = compute_sparkline_data(&[50.0]).expect("data");
        assert_eq!((d.sparkline.chars().count(), d.first, d.last, d.trend),
            (1, 50.0, 50.0, SparklineTrend::Flat));
    }
    #[test] fn test_CORPUS_CONV_009_sparkline_ascending() {
        let d = compute_sparkline_data(&[10.0, 20.0, 30.0, 40.0, 50.0]).expect("data");
        assert_eq!((d.trend, d.first, d.last, d.sparkline.chars().count()),
            (SparklineTrend::Up, 10.0, 50.0, 5));
    }
    #[test] fn test_CORPUS_CONV_010_sparkline_descending() {
        let d = compute_sparkline_data(&[90.0, 80.0, 70.0]).expect("data");
        assert_eq!((d.trend, d.first, d.last), (SparklineTrend::Down, 90.0, 70.0));
    }
    #[test] fn test_CORPUS_CONV_011_sparkline_flat() {
        let d = compute_sparkline_data(&[50.0, 50.0, 50.0]).expect("data");
        assert_eq!(d.trend, SparklineTrend::Flat);
        let c: Vec<char> = d.sparkline.chars().collect();
        assert!(c.windows(2).all(|w| w[0] == w[1]));
    }
    #[test] fn test_CORPUS_CONV_012_sparkline_chars_ordered() {
        let c: Vec<char> = compute_sparkline_data(&[0.0, 50.0, 100.0])
            .expect("data").sparkline.chars().collect();
        assert!(c[0] < c[2]);
    }
    #[test] fn test_CORPUS_CONV_043_sparkline_two_values() {
        let d = compute_sparkline_data(&[10.0, 90.0]).expect("data");
        assert_eq!(d.trend, SparklineTrend::Up);
        let c: Vec<char> = d.sparkline.chars().collect();
        assert!(c[0] < c[1]);
    }
    // classify_all_entries
    #[test] fn test_CORPUS_CONV_013_classify_empty() {
        let r = classify_all_entries(&[]);
        assert_eq!((r.total, r.tier_counts), (0, [0; 6]));
        assert!(r.format_tiers.is_empty());
    }
    #[test] fn test_CORPUS_CONV_014_classify_single() {
        let r = classify_all_entries(&[("B-001", "echo hello")]);
        assert_eq!((r.total, r.tier_counts.iter().sum::<u32>()), (1, 1));
    }
    #[test] fn test_CORPUS_CONV_015_classify_format_prefixes() {
        let r = classify_all_entries(&[("B-1","echo"),("M-1","all:"),("D-1","FROM")]);
        assert_eq!(r.total, 3);
        assert!(r.format_tiers.contains_key("B") && r.format_tiers.contains_key("M")
            && r.format_tiers.contains_key("D"));
    }
    #[test] fn test_CORPUS_CONV_016_classify_counts_sum() {
        let r = classify_all_entries(&[("B-1","echo"),("B-2","for i in 1 2; do echo $i; done"),
            ("B-3","if true; then echo yes; fi")]);
        assert_eq!(r.tier_counts.iter().sum::<u32>(), r.total as u32);
    }
    #[test] fn test_CORPUS_CONV_017_classify_format_tiers_match() {
        let r = classify_all_entries(&[("B-1","echo"),("B-2","for i in 1 2; do echo $i; done"),
            ("M-1","all:")]);
        let b: u32 = r.format_tiers.get("B").map_or(0, |f| f.iter().sum());
        let m: u32 = r.format_tiers.get("M").map_or(0, |f| f.iter().sum());
        assert_eq!(b + m, r.total as u32);
    }
    #[test] fn test_CORPUS_CONV_044_classify_complex() {
        let inp = "for i in 0..10 {\n    if i % 2 == 0 {\n        unsafe { exec(\"cmd\") }\n    }\n}\nfn helper() { loop { break; } }";
        let t: u8 = (1..=5u8).find(|&t| classify_all_entries(&[("B-100", inp)])
            .tier_counts[t as usize] > 0).expect("tier");
        assert!(t >= 3, "got {t}");
    }
    // tier_classify_rows
    #[test] fn test_CORPUS_CONV_018_rows_always_five() {
        assert_eq!(tier_classify_rows(&classify_all_entries(&[]), 16).len(), 5);
    }
    #[test] fn test_CORPUS_CONV_019_rows_labels() {
        let l: Vec<&str> = tier_classify_rows(&classify_all_entries(&[("B-1","echo")]), 16)
            .iter().map(|r| r.label).collect();
        assert_eq!(l, ["Trivial","Standard","Complex","Adversarial","Production"]);
    }
    #[test] fn test_CORPUS_CONV_020_rows_pct_sum() {
        let s: f64 = tier_classify_rows(&classify_all_entries(
            &[("B-1","echo"),("B-2","for i in 1; do echo; done")]), 16)
            .iter().map(|r| r.pct).sum();
        assert!((s - 100.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_CONV_021_rows_bar_width() {
        for r in &tier_classify_rows(&classify_all_entries(&[("B-1","echo")]), 10) {
            assert_eq!(r.bar.chars().count(), 10);
        }
    }
    // tier_json_entries
    #[test] fn test_CORPUS_CONV_022_json_five() {
        assert_eq!(tier_json_entries(&classify_all_entries(&[])).len(), 5);
    }
    #[test] fn test_CORPUS_CONV_023_json_tiers() {
        let t: Vec<u8> = tier_json_entries(&classify_all_entries(&[("B-1","echo")]))
            .iter().map(|e| e.tier).collect();
        assert_eq!(t, [1, 2, 3, 4, 5]);
    }
    #[test] fn test_CORPUS_CONV_024_json_labels() {
        let e = tier_json_entries(&classify_all_entries(&[]));
        assert_eq!((e[0].label.as_str(), e[4].label.as_str()), ("Trivial", "Production"));
    }
    // format_tier_summaries
    #[test] fn test_CORPUS_CONV_025_summaries_empty() {
        assert!(format_tier_summaries(&classify_all_entries(&[])).is_empty());
    }
    #[test] fn test_CORPUS_CONV_026_summaries_bash_only() {
        let s = format_tier_summaries(&classify_all_entries(&[("B-1","echo"),("B-2","echo")]));
        assert_eq!((s.len(), s[0].key.as_str(), s[0].label), (1, "B", "Bash"));
    }
    #[test] fn test_CORPUS_CONV_027_summaries_parts() {
        let s = format_tier_summaries(&classify_all_entries(&[("B-1","echo")]));
        assert!(!s.is_empty() && !s[0].parts.is_empty() && s[0].parts[0].starts_with('T'));
    }
    // convergence_log_summary
    #[test] fn test_CORPUS_CONV_028_log_first() {
        let s = convergence_log_summary(0, None);
        assert_eq!((s.iteration, s.prev_rate), (1, 0.0));
    }
    #[test] fn test_CORPUS_CONV_029_log_subsequent() {
        let s = convergence_log_summary(5, Some(0.95));
        assert_eq!(s.iteration, 6); assert!((s.prev_rate - 0.95).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_CONV_045_log_large() {
        let s = convergence_log_summary(999, Some(0.999));
        assert_eq!(s.iteration, 1000); assert!((s.prev_rate - 0.999).abs() < 1e-9);
    }
    // compute_lint_gap
    #[test] fn test_CORPUS_CONV_030_lint_no_data() {
        assert!(compute_lint_gap(0, 100, 0.0, 0.9).is_none());
    }
    #[test] fn test_CORPUS_CONV_031_lint_significant() {
        let r = compute_lint_gap(80, 100, 0.80, 0.90).expect("data");
        assert!((r.lint_pct - 80.0).abs() < 1e-9 && (r.gap.unwrap() - 10.0).abs() < 1e-9);
    }
    #[test] fn test_CORPUS_CONV_032_lint_negligible() {
        let r = compute_lint_gap(90, 100, 0.90, 0.90).expect("data");
        assert!((r.lint_pct - 90.0).abs() < 1e-9 && r.gap.is_none());
    }
    #[test] fn test_CORPUS_CONV_033_lint_boundary() {
        assert!(compute_lint_gap(90, 100, 0.899, 0.90).expect("data").gap.expect("gap") > 0.1);
    }
    // format_per_format_parts
    #[test] fn test_CORPUS_CONV_034_parts_all_zero() {
        assert!(format_per_format_parts(0, 0, 0, 0, 0, 0).is_empty());
    }
    #[test] fn test_CORPUS_CONV_035_parts_bash_only() {
        let p = format_per_format_parts(90, 100, 0, 0, 0, 0);
        assert_eq!((p.len(), p[0].as_str()), (1, "Bash 90/100"));
    }
    #[test] fn test_CORPUS_CONV_036_parts_all_formats() {
        assert_eq!(format_per_format_parts(90,100,45,50,28,30),
            ["Bash 90/100", "Make 45/50", "Docker 28/30"]);
    }
    #[test] fn test_CORPUS_CONV_037_parts_skips_zero() {
        assert_eq!(format_per_format_parts(90, 100, 0, 0, 28, 30),
            ["Bash 90/100", "Docker 28/30"]);
    }
    // regression messages
    #[test] fn test_CORPUS_CONV_038_regression_empty() {
        let r = build_regression_messages(&[]);
        assert!(!r.has_regressions && r.messages.is_empty());
    }
    #[test] fn test_CORPUS_CONV_039_regression_present() {
        let r = build_regression_messages(&["B-001: drop".into(), "B-002: lint".into()]);
        assert!(r.has_regressions && r.messages.len() == 2);
    }
    // format_lint_display
    #[test] fn test_CORPUS_CONV_040_lint_no_gap() {
        let (l, g) = format_lint_display(90, 100, &LintGapResult { lint_pct: 90.0, gap: None });
        assert_eq!(l, "90.0% (90/100)"); assert!(g.is_empty());
    }
    #[test] fn test_CORPUS_CONV_041_lint_with_gap() {
        let (l, g) = format_lint_display(80, 100, &LintGapResult { lint_pct: 80.0, gap: Some(10.0) });
        assert_eq!((l.as_str(), g.as_str()), ("80.0% (80/100)", "(gap: 10.0%)"));
    }
}
