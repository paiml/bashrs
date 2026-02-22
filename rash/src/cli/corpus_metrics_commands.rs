//! Corpus metrics: top-k ranking, format comparison, stability, version, rate, distribution, trace, and suspicious detection.

use crate::models::{Config, Error, Result};
use super::corpus_failure_commands::result_fail_dims;
use super::corpus_decision_commands::score_impact_color;
use super::corpus_diag_commands::dim_format_rate;
use super::corpus_compare_commands::percentile;

pub(crate) fn corpus_topk(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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
                    .to_string(),
                d @ 6..=13 => {
                    let dim_idx = d - 6;
                    let rate = dim_format_rate(&registry, &score.results, *fmt, dim_idx);
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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
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

/// Decision trace for a single corpus entry.

pub(crate) fn corpus_trace(id: &str) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Corpus entry '{id}' not found")))?;

    let runner = CorpusRunner::new(Config::default());
    let result = runner.run_entry_with_trace(entry);

    let pass_fail = if result.transpiled {
        format!("{BRIGHT_GREEN}PASS{RESET}")
    } else {
        format!("{BRIGHT_RED}FAIL{RESET}")
    };

    println!("\n  {BOLD}Decision Trace for {CYAN}{id}{RESET} [{pass_fail}]");
    println!("  {DIM}{}{}", "─".repeat(60), RESET);

    match &result.decision_trace {
        Some(trace) if !trace.is_empty() => {
            println!(
                "  {DIM}{:<4}  {:<22}  {:<24}  {:<12}{RESET}",
                "#", "Decision Type", "Choice", "IR Node"
            );
            println!("  {DIM}{}{RESET}", "─".repeat(60));
            for (i, d) in trace.iter().enumerate() {
                println!(
                    "  {DIM}{:<4}{RESET}  {CYAN}{:<22}{RESET}  {WHITE}{:<24}{RESET}  {DIM}{:<12}{RESET}",
                    i + 1,
                    d.decision_type,
                    d.choice,
                    d.ir_node
                );
            }
            println!();
            println!("  {DIM}Total decisions: {}{RESET}", trace.len());
        }
        _ => {
            println!(
                "  {DIM}No decision trace available (non-Bash entry or transpilation failed){RESET}"
            );
            if let Some(err) = &result.error {
                println!("  {RED}Error: {err}{RESET}");
            }
        }
    }

    println!();
    Ok(())
}

/// Collect decision trace coverage data from all corpus entries.

pub(crate) fn collect_trace_coverage(
    registry: &crate::corpus::registry::CorpusRegistry,
    runner: &crate::corpus::runner::CorpusRunner,
) -> Vec<(String, bool, Vec<String>)> {
    registry
        .entries
        .iter()
        .filter_map(|entry| {
            let result = runner.run_entry_with_trace(entry);
            let passed = result.transpiled
                && result.output_contains
                && result.schema_valid
                && result.lint_clean
                && result.deterministic;
            let locations: Vec<String> = result
                .decision_trace
                .as_ref()
                .map(|t| {
                    t.iter()
                        .map(|d| format!("{}:{}", d.decision_type, d.choice))
                        .collect()
                })
                .unwrap_or_default();
            if locations.is_empty() {
                None
            } else {
                Some((entry.id.clone(), passed, locations))
            }
        })
        .collect()
}

/// Tarantula fault localization ranking across all corpus decisions.

pub(crate) fn corpus_suspicious(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::quality::sbfl::{localize_faults, SbflFormula};

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let coverage_data = collect_trace_coverage(&registry, &runner);

    let total = coverage_data.len();
    let passed = coverage_data.iter().filter(|(_, p, _)| *p).count();
    let failed = total - passed;

    if failed == 0 {
        println!(
            "\n  {BRIGHT_GREEN}All {total} traced entries pass — no suspicious decisions{RESET}\n"
        );
        return Ok(());
    }

    let rankings = localize_faults(&coverage_data, SbflFormula::Tarantula);

    println!(
        "\n  {BOLD}Tarantula Fault Localization{RESET}  ({total} entries: {BRIGHT_GREEN}{passed} pass{RESET}, {BRIGHT_RED}{failed} fail{RESET})"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));
    println!(
        "  {DIM}{:<36}  {:>14}  {:>8}  {:>8}{RESET}",
        "Decision", "Suspiciousness", "Impact", "Priority"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for ranking in rankings.iter().take(limit) {
        let (impact, color) = score_impact_color(ranking.score);
        println!(
            "  {:<36}  {color}{:>14.4}{RESET}  {:>8}  {DIM}{:>8}{RESET}",
            ranking.location,
            ranking.score,
            impact,
            format!("#{}", ranking.rank)
        );
    }

    if rankings.len() > limit {
        println!(
            "\n  {DIM}... and {} more (use --limit to show more){RESET}",
            rankings.len() - limit
        );
    }

    println!();
    Ok(())
}
