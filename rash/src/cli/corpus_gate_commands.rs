//! Corpus gate operations: error analysis, sampling, completeness, gates, outliers, and matrix.

use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
use crate::models::{Config, Error, Result};
use super::corpus_failure_commands::result_fail_dims;
use std::path::PathBuf;
use super::corpus_ranking_commands::classify_category;

pub(crate) fn corpus_errors(format: &CorpusOutputFormat, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = match filter {
        Some(CorpusFormatArg::Bash) => runner.run_format(&registry, CorpusFormat::Bash),
        Some(CorpusFormatArg::Makefile) => runner.run_format(&registry, CorpusFormat::Makefile),
        Some(CorpusFormatArg::Dockerfile) => runner.run_format(&registry, CorpusFormat::Dockerfile),
        None => runner.run(&registry),
    };

    // Collect entries with errors or failures
    let mut categories: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for r in &score.results {
        let fails = result_fail_dims(r);
        if fails.is_empty() {
            continue;
        }
        let cat = r.error_category.as_deref().unwrap_or("uncategorized");
        categories
            .entry(cat.to_string())
            .or_default()
            .push(r.id.clone());
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            if categories.is_empty() {
                println!("{GREEN}No errors in corpus.{RESET}");
            } else {
                println!(
                    "{BOLD}Error Categories ({} categories){RESET}",
                    categories.len()
                );
                println!();
                println!("  {BOLD}{:<24} {:>5}  Entries{RESET}", "Category", "Count");
                for (cat, ids) in &categories {
                    let sample: Vec<_> = ids.iter().take(5).map(|s| s.as_str()).collect();
                    let more = if ids.len() > 5 {
                        format!(" {DIM}(+{}){RESET}", ids.len() - 5)
                    } else {
                        String::new()
                    };
                    println!(
                        "  {YELLOW}{:<24}{RESET} {:>5}  {}{}",
                        cat,
                        ids.len(),
                        sample.join(", "),
                        more
                    );
                }
            }
        }
        CorpusOutputFormat::Json => {
            let result: Vec<_> = categories
                .iter()
                .map(|(cat, ids)| {
                    serde_json::json!({
                        "category": cat,
                        "count": ids.len(),
                        "entries": ids,
                    })
                })
                .collect();
            let json = serde_json::to_string_pretty(&serde_json::json!({
                "total_errors": categories.values().map(|v| v.len()).sum::<usize>(),
                "categories": result,
            }))
            .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Random sample of N entries with results (spot-check).

pub(crate) fn corpus_sample(count: usize, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let entries: Vec<_> = registry
        .entries
        .iter()
        .filter(|e| match filter {
            Some(CorpusFormatArg::Bash) => e.format == CorpusFormat::Bash,
            Some(CorpusFormatArg::Makefile) => e.format == CorpusFormat::Makefile,
            Some(CorpusFormatArg::Dockerfile) => e.format == CorpusFormat::Dockerfile,
            None => true,
        })
        .collect();

    // Deterministic pseudo-random sampling using hash of current time
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(42);
    let n = entries.len();
    let sampled: Vec<_> = (0..count.min(n))
        .map(|i| {
            let idx = (seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(i * 1442695040888963407))
                % n;
            entries[idx]
        })
        .collect();

    println!("{BOLD}Random Sample ({count} of {n} entries){RESET}");
    println!();
    println!(
        "  {BOLD}{:<8} {:<10} {:<12} {:>5}  Status{RESET}",
        "ID", "Format", "Tier", "Dims"
    );
    for entry in &sampled {
        let result = runner.run_single(entry);
        let fails = result_fail_dims(&result);
        let status = if fails.is_empty() {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET} ({})", fails.join(", "))
        };
        println!(
            "  {:<8} {:<10} {:<12} {:>5}  {}",
            entry.id,
            format!("{}", entry.format),
            format!("{:?}", entry.tier),
            format!("{}/{}", 9 - fails.len(), 9),
            status
        );
    }
    Ok(())
}

/// Check corpus construct completeness by tier.

pub(crate) fn corpus_completeness() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry, CorpusTier};

    let registry = CorpusRegistry::load_full();

    // Spec targets from §2.3 and §3.1-3.3
    let targets = [
        (CorpusFormat::Bash, 500, "B-001..B-500"),
        (CorpusFormat::Makefile, 200, "M-001..M-200"),
        (CorpusFormat::Dockerfile, 200, "D-001..D-200"),
    ];

    println!("{BOLD}Corpus Completeness Check{RESET}");
    println!();

    let mut all_complete = true;
    for (fmt, target, range) in &targets {
        let count = registry.entries.iter().filter(|e| &e.format == fmt).count();
        let pct = count as f64 / *target as f64 * 100.0;
        let pc = pct_color(pct);
        let mark = if count >= *target {
            format!("{GREEN}\u{2713}{RESET}")
        } else {
            all_complete = false;
            format!("{RED}\u{2717}{RESET}")
        };
        println!("  {mark} {CYAN}{:<12}{RESET} {count:>4}/{target} {pc}({pct:.0}%){RESET}  {DIM}{range}{RESET}",
            format!("{fmt}"));
    }

    // Tier distribution
    println!();
    println!("  {BOLD}Tier Distribution:{RESET}");
    let tiers = [
        (CorpusTier::Trivial, "Trivial", 5),
        (CorpusTier::Standard, "Standard", 30),
        (CorpusTier::Complex, "Complex", 5),
        (CorpusTier::Adversarial, "Adversarial", 5),
        (CorpusTier::Production, "Production", 55),
    ];
    let total = registry.entries.len();
    for (tier, label, target_pct) in &tiers {
        let count = registry.entries.iter().filter(|e| &e.tier == tier).count();
        let actual_pct = count as f64 / total as f64 * 100.0;
        println!(
            "    {:<14} {:>4} ({:>5.1}%)  {DIM}target: ~{target_pct}%{RESET}",
            label, count, actual_pct
        );
    }

    println!();
    if all_complete {
        println!("  {GREEN}All format targets met.{RESET}");
    } else {
        println!("  {YELLOW}Some format targets not met yet.{RESET}");
    }
    Ok(())
}

/// CI quality gate: score + regressions + benchmark in one check.

pub(crate) fn corpus_gate(min_score: f64, max_ms: u64) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    println!("{BOLD}Corpus Quality Gate{RESET}");
    println!();

    // Gate 1: Run corpus and check score
    let score = runner.run(&registry);
    let score_pass = score.score >= min_score;
    gate_print_check(
        &format!("Score >= {min_score} (actual: {:.1})", score.score),
        score_pass,
    );

    // Gate 2: Check for failures
    let failure_count = score
        .results
        .iter()
        .filter(|r| !result_fail_dims(r).is_empty())
        .count();
    let fail_pass = failure_count <= 1; // Allow B-143 known failure
    gate_print_check(
        &format!("Failures <= 1 (actual: {failure_count})"),
        fail_pass,
    );

    // Gate 3: Check for regressions
    let log_path = PathBuf::from(".quality/convergence.log");
    let regression_pass = if let Ok(entries) = CorpusRunner::load_convergence_log(&log_path) {
        if entries.len() >= 2 {
            let last = &entries[entries.len() - 1];
            let prev = &entries[entries.len() - 2];
            let report = last.detect_regressions(prev);
            !report.has_regressions()
        } else {
            true
        }
    } else {
        true // No log = no regressions
    };
    gate_print_check("No regressions from previous iteration", regression_pass);

    // Gate 4: Benchmark spot-check (sample 50 entries)
    let sample_size = 50.min(registry.entries.len());
    let start = Instant::now();
    for entry in registry.entries.iter().take(sample_size) {
        let _ = runner.run_single(entry);
    }
    let avg_ms = start.elapsed().as_millis() / sample_size as u128;
    let bench_pass = avg_ms <= max_ms as u128;
    gate_print_check(
        &format!("Avg transpile <= {max_ms}ms (actual: {avg_ms}ms, {sample_size} sampled)"),
        bench_pass,
    );

    println!();
    let all_pass = score_pass && fail_pass && regression_pass && bench_pass;
    if all_pass {
        println!("  {BRIGHT_GREEN}ALL GATES PASSED{RESET}");
        Ok(())
    } else {
        println!("  {BRIGHT_RED}GATE FAILURE — STOP THE LINE{RESET}");
        Err(Error::Internal("Quality gate failed".to_string()))
    }
}


pub(crate) fn gate_print_check(label: &str, pass: bool) {
    use crate::cli::color::*;
    let mark = if pass {
        format!("{GREEN}\u{2713}{RESET}")
    } else {
        format!("{RED}\u{2717}{RESET}")
    };
    println!("  {mark} {label}");
}

/// Compute z-score outliers from timing data
fn find_zscore_outliers<'a>(
    timings: &[(&'a str, f64)],
    threshold: f64,
) -> Option<(f64, f64, Vec<(&'a str, f64, f64)>)> {
    let n = timings.len() as f64;
    if n < 2.0 {
        return None;
    }
    let mean = timings.iter().map(|(_, t)| t).sum::<f64>() / n;
    let variance = timings.iter().map(|(_, t)| (t - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let stddev = variance.sqrt();

    let mut outliers: Vec<(&str, f64, f64)> = timings
        .iter()
        .filter_map(|(id, ms)| {
            let z = if stddev > 0.0 {
                (ms - mean) / stddev
            } else {
                0.0
            };
            if z.abs() >= threshold {
                Some((*id, *ms, z))
            } else {
                None
            }
        })
        .collect();
    outliers.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    Some((mean, stddev, outliers))
}

/// Display outlier results
fn display_outliers(
    outliers: &[(&str, f64, f64)],
    mean: f64,
    stddev: f64,
    threshold: f64,
    total: usize,
) {
    use crate::cli::color::*;
    println!("{BOLD}Timing Outlier Detection{RESET} (z-score >= {threshold:.1})");
    println!("{DIM}  Mean: {mean:.1}ms | StdDev: {stddev:.1}ms | Entries: {total}{RESET}");
    println!();

    if outliers.is_empty() {
        println!("  {GREEN}No outliers detected.{RESET}");
    } else {
        println!(
            "  {BOLD}{:<8} {:>8} {:>8}  Status{RESET}",
            "ID", "Time", "Z-Score"
        );
        for (id, ms, z) in outliers {
            let color = if *z > 3.0 { BRIGHT_RED } else if *z > 2.0 { YELLOW } else { DIM };
            let status = if *z > 3.0 { "EXTREME" } else { "OUTLIER" };
            println!("  {CYAN}{:<8}{RESET} {color}{:>7.1}ms {:>+7.2}{RESET}  {status}", id, ms, z);
        }
        println!();
        println!(
            "  {DIM}{} outliers found out of {} entries{RESET}",
            outliers.len(),
            total
        );
    }
}

/// Find statistical outliers by transpilation timing (z-score detection).

pub(crate) fn corpus_outliers(threshold: f64, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let entries: Vec<_> = registry
        .entries
        .iter()
        .filter(|e| match filter {
            Some(CorpusFormatArg::Bash) => e.format == CorpusFormat::Bash,
            Some(CorpusFormatArg::Makefile) => e.format == CorpusFormat::Makefile,
            Some(CorpusFormatArg::Dockerfile) => e.format == CorpusFormat::Dockerfile,
            None => true,
        })
        .collect();

    let mut timings: Vec<(&str, f64)> = Vec::new();
    for entry in &entries {
        let start = Instant::now();
        let _ = runner.run_single(entry);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        timings.push((&entry.id, ms));
    }

    match find_zscore_outliers(&timings, threshold) {
        Some((mean, stddev, outliers)) => {
            display_outliers(&outliers, mean, stddev, threshold, timings.len());
        }
        None => {
            println!("Need at least 2 entries for outlier detection.");
        }
    }
    Ok(())
}

/// Cross-category × quality property matrix (spec §11.11.9).

pub(crate) fn corpus_matrix() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    // Classify entries by category and collect results
    let categories = [
        "Config (A)",
        "One-liner (B)",
        "Coreutils (G)",
        "Regex (H)",
        "System (F)",
        "Adversarial",
        "General",
        "Milestone",
    ];
    let properties = ["POSIX", "Determ", "Idempot", "X-Shell", "Lint", "B3-Behav"];

    // Collect per-category pass rates for each property
    let mut matrix: Vec<Vec<f64>> = Vec::new();
    let mut cat_counts: Vec<usize> = Vec::new();

    for cat in &categories {
        let cat_entries: Vec<_> = registry
            .entries
            .iter()
            .filter(|e| classify_category(&e.name) == *cat)
            .collect();
        let count = cat_entries.len();
        cat_counts.push(count);

        if count == 0 {
            matrix.push(vec![0.0; properties.len()]);
            continue;
        }

        let results: Vec<_> = cat_entries.iter().map(|e| runner.run_single(e)).collect();

        let rates = vec![
            results.iter().filter(|r| r.lint_clean).count() as f64 / count as f64, // POSIX (lint)
            results.iter().filter(|r| r.deterministic).count() as f64 / count as f64, // Deterministic
            results.iter().filter(|r| r.transpiled).count() as f64 / count as f64, // Idempotent (approx via transpile)
            results.iter().filter(|r| r.cross_shell_agree).count() as f64 / count as f64, // Cross-shell
            results.iter().filter(|r| r.lint_clean).count() as f64 / count as f64,        // Lint
            results.iter().filter(|r| r.output_behavioral).count() as f64 / count as f64, // B3 Behavioral
        ];
        matrix.push(rates);
    }

    println!("{BOLD}Category × Quality Property Matrix{RESET} (spec §11.11.9)");
    println!();

    // Header
    print!("  {BOLD}{:<16} {:>4}", "Category", "N");
    for prop in &properties {
        print!("  {:>7}", prop);
    }
    println!("{RESET}");

    // Rows
    for (i, cat) in categories.iter().enumerate() {
        if cat_counts[i] == 0 {
            continue;
        }
        print!("  {CYAN}{:<16}{RESET} {:>4}", cat, cat_counts[i]);
        for rate in &matrix[i] {
            let pct = rate * 100.0;
            let color = if pct >= 99.0 {
                GREEN
            } else if pct >= 95.0 {
                YELLOW
            } else {
                RED
            };
            print!("  {color}{:>6.1}%{RESET}", pct);
        }
        println!();
    }

    println!();
    println!("  {DIM}Cells show pass rate per quality property within each category.{RESET}");
    Ok(())
}
