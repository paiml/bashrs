//! Corpus metrics: top-k ranking, format comparison, stability, version, rate, distribution, trace, and suspicious detection.

use super::corpus_compare_commands::percentile;
use super::corpus_decision_commands::score_impact_color;
use super::corpus_diag_commands::dim_format_rate;
use super::corpus_failure_commands::result_fail_dims;
use crate::models::{Config, Error, Result};

pub(crate) fn corpus_topk(limit: usize) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_topk_with(&CorpusRegistry::load_full(), limit)
}

/// PMAT-257: body of `corpus_topk`, split so a test can pass a small
/// synthetic registry instead of running the full ~18k-entry corpus.
pub(crate) fn corpus_topk_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    limit: usize,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    // Collect entries sorted by fewest failures (worst first)
    let mut entries_with_dims: Vec<(&str, &str, usize, Vec<&str>)> = registry
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            score.results.get(i).map(|r| {
                let fails = result_fail_dims(r);
                let pass_count = 9 - fails.len();
                (entry.id.as_str(), entry.name.as_str(), pass_count, fails)
            })
        })
        .collect();

    // Sort: fewest passing dims first (worst entries first)
    entries_with_dims.sort_by(|a, b| a.2.cmp(&b.2));

    println!("{BOLD}Top-K Entries by Dimension Pass Count{RESET} (worst first)");
    println!();
    println!(
        "  {BOLD}{:<10} {:>5} {:<30} Failures{RESET}",
        "ID", "Pass", "Name"
    );

    for (id, name, pass_count, fails) in entries_with_dims.iter().take(limit) {
        let truncated_name: String = name.chars().take(28).collect();
        let color = if *pass_count == 9 {
            GREEN
        } else if *pass_count >= 7 {
            YELLOW
        } else {
            RED
        };
        let fail_str = if fails.is_empty() {
            "-".to_string()
        } else {
            fails.join(",")
        };
        println!(
            "  {CYAN}{:<10}{RESET} {color}{:>4}/9{RESET} {:<30} {DIM}{fail_str}{RESET}",
            id, pass_count, truncated_name
        );
    }

    Ok(())
}

/// Side-by-side format comparison.
pub(crate) fn corpus_format_cmp() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_format_cmp_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_format_cmp`, split for testability (see
/// `corpus_topk_with`).
pub(crate) fn corpus_format_cmp_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    println!("{BOLD}Format Comparison{RESET}");
    println!();

    let metrics = [
        "Total",
        "Passed",
        "Failed",
        "Rate",
        "Score",
        "Grade",
        "A (Transpile)",
        "B1 (Contains)",
        "B2 (Exact)",
        "B3 (Behavioral)",
        "D (Lint)",
        "E (Determinism)",
        "F (Metamorphic)",
        "G (Cross-shell)",
    ];

    println!(
        "  {BOLD}{:<18} {:>12} {:>12} {:>12}{RESET}",
        "Metric", "Bash", "Makefile", "Dockerfile"
    );

    for fs in &score.format_scores {
        // Handled below per-metric
        let _ = fs;
    }

    // Gather per-format dim rates
    let formats = [
        ("Bash", crate::corpus::registry::CorpusFormat::Bash),
        ("Makefile", crate::corpus::registry::CorpusFormat::Makefile),
        (
            "Dockerfile",
            crate::corpus::registry::CorpusFormat::Dockerfile,
        ),
    ];

    for (m_idx, metric) in metrics.iter().enumerate() {
        print!("  {CYAN}{:<18}{RESET}", metric);
        for (_, fmt) in &formats {
            let fs = score.format_scores.iter().find(|f| f.format == *fmt);
            let val = match m_idx {
                0 => format!("{}", fs.map_or(0, |f| f.total)),
                1 => format!("{}", fs.map_or(0, |f| f.passed)),
                2 => format!("{}", fs.map_or(0, |f| f.total - f.passed)),
                3 => format!("{:.1}%", fs.map_or(0.0, |f| f.rate * 100.0)),
                4 => format!("{:.1}", fs.map_or(0.0, |f| f.score)),
                5 => fs
                    .map_or_else(|| "?".to_string(), |f| format!("{}", f.grade))
                    .clone(),
                d @ 6..=13 => {
                    let dim_idx = d - 6;
                    let rate = dim_format_rate(registry, &score.results, *fmt, dim_idx);
                    format!("{:.1}%", rate)
                }
                _ => "-".to_string(),
            };
            let color = match m_idx {
                3 | 4 | 6..=13 => {
                    let num: f64 = val.trim_end_matches('%').parse().unwrap_or(100.0);
                    pct_color(num)
                }
                _ => "",
            };
            print!("{color}{:>12}{RESET}", val);
        }
        println!();
    }

    Ok(())
}

/// Stability index: ratio of entries never failing across iterations.
pub(crate) fn corpus_stability() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_stability_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_stability`, split for testability (see
/// `corpus_topk_with`).
pub(crate) fn corpus_stability_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    // Current stability: entries with zero failures
    let stable = score
        .results
        .iter()
        .filter(|r| result_fail_dims(r).is_empty())
        .count();
    let total = score.results.len();
    let stability = if total > 0 {
        stable as f64 / total as f64 * 100.0
    } else {
        100.0
    };

    println!("{BOLD}Stability Index{RESET}");
    println!();

    let color = pct_color(stability);
    println!("  Stable entries: {color}{stable}/{total} ({stability:.1}%){RESET}");
    println!("  Unstable entries: {}", total - stable);

    // Per-format stability
    println!();
    println!(
        "  {BOLD}{:<14} {:>8} {:>8} {:>8}{RESET}",
        "Format", "Stable", "Total", "Rate"
    );

    let formats = [
        ("Bash", crate::corpus::registry::CorpusFormat::Bash),
        ("Makefile", crate::corpus::registry::CorpusFormat::Makefile),
        (
            "Dockerfile",
            crate::corpus::registry::CorpusFormat::Dockerfile,
        ),
    ];

    for (name, fmt) in &formats {
        let fmt_entries: Vec<_> = registry
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.format == *fmt)
            .collect();
        let fmt_total = fmt_entries.len();
        let fmt_stable = fmt_entries
            .iter()
            .filter(|(i, _)| {
                score
                    .results
                    .get(*i)
                    .is_some_and(|r| result_fail_dims(r).is_empty())
            })
            .count();
        let fmt_rate = if fmt_total > 0 {
            fmt_stable as f64 / fmt_total as f64 * 100.0
        } else {
            100.0
        };
        let fc = pct_color(fmt_rate);
        println!(
            "  {CYAN}{:<14}{RESET} {:>8} {:>8} {fc}{:>7.1}%{RESET}",
            name, fmt_stable, fmt_total, fmt_rate
        );
    }

    // Stability assessment
    println!();
    let assessment = if stability >= 99.9 {
        format!("{GREEN}EXCELLENT{RESET} — near-perfect stability")
    } else if stability >= 99.0 {
        format!("{GREEN}GOOD{RESET} — high stability")
    } else if stability >= 95.0 {
        format!("{YELLOW}MODERATE{RESET} — some instability")
    } else {
        format!("{RED}POOR{RESET} — significant instability")
    };
    println!("  Assessment: {assessment}");

    Ok(())
}

/// Corpus version and metadata info.
pub(crate) fn corpus_version() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_version_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_version`, split for testability (see
/// `corpus_topk_with`).
pub(crate) fn corpus_version_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;

    let bash_count = registry
        .entries
        .iter()
        .filter(|e| e.format == crate::corpus::registry::CorpusFormat::Bash)
        .count();
    let make_count = registry
        .entries
        .iter()
        .filter(|e| e.format == crate::corpus::registry::CorpusFormat::Makefile)
        .count();
    let dock_count = registry
        .entries
        .iter()
        .filter(|e| e.format == crate::corpus::registry::CorpusFormat::Dockerfile)
        .count();

    println!("{BOLD}Corpus Version{RESET}");
    println!();
    println!("  Spec version:  2.1.0");
    println!("  Scoring:       V2 (9 dimensions, 100-point scale)");
    println!("  Total entries: {}", registry.entries.len());
    println!("  Bash:          {bash_count} (B-001..B-{bash_count:03})");
    println!("  Makefile:      {make_count} (M-001..M-{make_count:03})");
    println!("  Dockerfile:    {dock_count} (D-001..D-{dock_count:03})");
    println!("  CLI commands:  71");
    println!("  Dimensions:    A(30) B1(10) B2(8) B3(7) C(15) D(10) E(10) F(5) G(5)");

    Ok(())
}

/// Simple pass rate display per format.
pub(crate) fn corpus_rate() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_rate_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_rate`, split for testability (see
/// `corpus_topk_with`).
pub(crate) fn corpus_rate_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    println!("{BOLD}Pass Rates{RESET}");
    println!();

    for fs in &score.format_scores {
        let rate = fs.rate * 100.0;
        let color = pct_color(rate);
        println!(
            "  {CYAN}{:<12}{RESET} {color}{:>4}/{:<4} {:.1}%{RESET}",
            format!("{}", fs.format),
            fs.passed,
            fs.total,
            rate
        );
    }

    println!();
    let total_rate = score.rate * 100.0;
    let tc = pct_color(total_rate);
    println!(
        "  {BOLD}Total{RESET}        {tc}{:>4}/{:<4} {:.1}%{RESET}",
        score.passed, score.total, total_rate
    );

    Ok(())
}

/// Distribution of entries by timing buckets.
pub(crate) fn corpus_dist() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_dist_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_dist`, split for testability (see
/// `corpus_topk_with`).
pub(crate) fn corpus_dist_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let runner = CorpusRunner::new(Config::default());

    let mut timings: Vec<f64> = Vec::new();
    for entry in &registry.entries {
        let start = Instant::now();
        let _result = runner.run_single(entry);
        timings.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let buckets = [
        ("< 1ms", 0.0, 1.0),
        ("1-5ms", 1.0, 5.0),
        ("5-10ms", 5.0, 10.0),
        ("10-20ms", 10.0, 20.0),
        ("20-50ms", 20.0, 50.0),
        ("50-100ms", 50.0, 100.0),
        ("100ms+", 100.0, f64::MAX),
    ];

    println!(
        "{BOLD}Timing Distribution{RESET} ({} entries)",
        timings.len()
    );
    println!();

    let max_count = buckets
        .iter()
        .map(|(_, lo, hi)| timings.iter().filter(|t| **t >= *lo && **t < *hi).count())
        .max()
        .unwrap_or(1);

    for (label, lo, hi) in &buckets {
        let count = timings.iter().filter(|t| **t >= *lo && **t < *hi).count();
        let bar_len = if max_count > 0 {
            count * 40 / max_count
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        let pct = count as f64 / timings.len().max(1) as f64 * 100.0;
        let color = if count == 0 { DIM } else { CYAN };
        println!(
            "  {color}{label:<10}{RESET} {CYAN}{bar}{RESET} {BOLD}{count:>4}{RESET} ({pct:.1}%)"
        );
    }

    // Summary stats
    let total: f64 = timings.iter().sum();
    let mean = total / timings.len().max(1) as f64;
    timings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = percentile(&timings, 50.0);

    println!();
    println!(
        "  {DIM}Mean: {mean:.1}ms | Median: {median:.1}ms | Total: {:.1}s{RESET}",
        total / 1000.0
    );

    Ok(())
}

// PMAT-257: every top-level handler in this file builds a CorpusRunner and
// scores the whole ~18k-entry corpus (`CorpusRegistry::load_full()`), so each
// was split into a `pub(crate) fn <name>() -> Result<()>` thin wrapper that
// loads the real registry, and a `pub(crate) fn <name>_with(registry, ...)`
// that does the actual work against a caller-supplied registry. Tests below
// call the `_with` variants against a tiny three-entry fixture (one real
// corpus entry per format) so a run completes in milliseconds instead of
// transpiling the entire corpus.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};

    /// Three real (id, input, expected_output) triples, one per format,
    /// copied from corpus_data.jsonl (B-001, M-001, D-001).
    fn fixture_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "variable-assignment",
            "Simple string variable assignment",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { let greeting = \"hello\"; } ",
            "greeting='hello'",
        ));
        registry.add(CorpusEntry::new(
            "M-001",
            "simple-variable",
            "Single variable assignment",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "fn main() { let cc = \"gcc\"; }",
            "CC := gcc",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "basic-from",
            "Basic FROM instruction with pinned tag",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "fn main() { from_image(\"alpine\", \"3.18\"); } fn from_image(i: &str, t: &str) {}",
            "FROM alpine:3.18",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_topk_with_reports_every_entry() {
        let registry = fixture_registry();
        corpus_topk_with(&registry, 10).expect("top-k over three fixture entries must succeed");
        // A limit of zero must not panic on `.take(0)`.
        corpus_topk_with(&registry, 0).expect("a zero limit is not an error");
    }

    #[test]
    fn test_PMAT257_cov_format_cmp_with_covers_all_three_formats() {
        let registry = fixture_registry();
        corpus_format_cmp_with(&registry)
            .expect("format comparison over one entry per format must succeed");
    }

    #[test]
    fn test_PMAT257_cov_format_cmp_with_empty_registry_has_no_entries() {
        // Every `fs.map_or(0, ...)` branch (no FormatScore found) is only hit
        // when a format has zero entries.
        let registry = CorpusRegistry::new();
        corpus_format_cmp_with(&registry).expect("an empty registry must still print a table");
    }

    #[test]
    fn test_PMAT257_cov_stability_with_reports_per_format_rates() {
        let registry = fixture_registry();
        corpus_stability_with(&registry).expect("stability over the fixture must succeed");
    }

    #[test]
    fn test_PMAT257_cov_stability_with_empty_registry_is_not_a_divide_by_zero() {
        let registry = CorpusRegistry::new();
        corpus_stability_with(&registry).expect("an empty registry must not panic");
    }

    #[test]
    fn test_PMAT257_cov_version_with_counts_each_format() {
        let registry = fixture_registry();
        corpus_version_with(&registry).expect("version info over the fixture must succeed");
    }

    #[test]
    fn test_PMAT257_cov_rate_with_reports_totals() {
        let registry = fixture_registry();
        corpus_rate_with(&registry).expect("pass rates over the fixture must succeed");
    }

    #[test]
    fn test_PMAT257_cov_dist_with_buckets_the_fixture_timings() {
        let registry = fixture_registry();
        corpus_dist_with(&registry).expect("timing distribution over the fixture must succeed");
    }

    #[test]
    fn test_PMAT257_cov_dist_with_empty_registry_has_no_timings() {
        let registry = CorpusRegistry::new();
        corpus_dist_with(&registry).expect("an empty registry must not divide by zero");
    }
}

include!("corpus_metrics_commands_corpus.rs");
