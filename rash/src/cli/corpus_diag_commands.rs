//! Corpus diagnostics: flaky detection, profiling, gap analysis, summary JSON, and audit.

use crate::models::{Config, Result};
use super::corpus_failure_commands::result_fail_dims;
use super::corpus_ranking_commands::classify_category;

pub(crate) fn corpus_flaky(threshold: f64) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let num_runs = 3;
    let mut timings: Vec<(&str, Vec<f64>, Vec<bool>)> = registry
        .entries
        .iter()
        .map(|e| (e.id.as_str(), Vec::new(), Vec::new()))
        .collect();

    // Run corpus multiple times to detect timing variance
    for _ in 0..num_runs {
        for (i, entry) in registry.entries.iter().enumerate() {
            let start = Instant::now();
            let result = runner.run_single(entry);
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            let passed = result_fail_dims(&result).is_empty();
            timings[i].1.push(ms);
            timings[i].2.push(passed);
        }
    }

    println!("{BOLD}Flaky Entry Detection{RESET} ({num_runs} runs, threshold CV > {threshold:.1})");
    println!();

    // Calculate coefficient of variation for each entry
    let mut flaky_entries: Vec<(&str, f64, f64, f64, bool)> = Vec::new();
    for (id, times, passes) in &timings {
        let n = times.len() as f64;
        let mean = times.iter().sum::<f64>() / n;
        let variance = times.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();
        let cv = if mean > 0.0 { stddev / mean } else { 0.0 };
        let result_flaky = passes.iter().any(|p| *p) && passes.iter().any(|p| !*p);

        if cv > threshold || result_flaky {
            flaky_entries.push((id, mean, stddev, cv, result_flaky));
        }
    }

    flaky_entries.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    if flaky_entries.is_empty() {
        println!("  {GREEN}No flaky entries detected{RESET}");
        println!("  {DIM}All entries have consistent timing (CV < {threshold:.1}) and stable results{RESET}");
    } else {
        println!(
            "  {BOLD}{:<10} {:>8} {:>8} {:>6} {:>8}{RESET}",
            "ID", "Mean(ms)", "Std(ms)", "CV", "Result"
        );
        for (id, mean, stddev, cv, result_flaky) in &flaky_entries {
            let rv = if *result_flaky {
                format!("{RED}FLAKY{RESET}")
            } else {
                format!("{DIM}stable{RESET}")
            };
            let cv_color = if *cv > 1.0 { RED } else { YELLOW };
            println!(
                "  {CYAN}{:<10}{RESET} {:>8.1} {:>8.1} {cv_color}{:>6.2}{RESET} {rv}",
                id, mean, stddev, cv
            );
        }
    }

    println!();
    println!("  {DIM}CV = coefficient of variation (stddev/mean). Higher = more variable.{RESET}");

    Ok(())
}

/// Corpus composition profile: tier, format, category breakdown.
pub(crate) fn corpus_profile() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    println!("{BOLD}Corpus Profile{RESET}");
    println!();

    // Format breakdown
    println!("  {BOLD}By Format:{RESET}");
    let mut fmt_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for entry in &registry.entries {
        *fmt_counts.entry(format!("{}", entry.format)).or_default() += 1;
    }
    for (fmt, count) in &fmt_counts {
        let pct = *count as f64 / registry.entries.len() as f64 * 100.0;
        println!("    {CYAN}{:<12}{RESET} {:>4} ({pct:.1}%)", fmt, count);
    }

    // Tier breakdown
    println!();
    println!("  {BOLD}By Tier:{RESET}");
    let mut tier_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for entry in &registry.entries {
        *tier_counts.entry(format!("{:?}", entry.tier)).or_default() += 1;
    }
    for (tier, count) in &tier_counts {
        let pct = *count as f64 / registry.entries.len() as f64 * 100.0;
        println!("    {CYAN}{:<14}{RESET} {:>4} ({pct:.1}%)", tier, count);
    }

    // Category breakdown
    println!();
    println!("  {BOLD}By Category:{RESET}");
    let mut cat_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for entry in &registry.entries {
        let cat = classify_category(&entry.name);
        *cat_counts.entry(cat.to_string()).or_default() += 1;
    }
    for (cat, count) in &cat_counts {
        let pct = *count as f64 / registry.entries.len() as f64 * 100.0;
        println!("    {CYAN}{:<20}{RESET} {:>4} ({pct:.1}%)", cat, count);
    }

    // Summary stats
    println!();
    println!("  {BOLD}Quality:{RESET}");
    println!("    Score:  {GREEN}{:.1}/100{RESET}", score.score);
    println!("    Grade:  {GREEN}{}{RESET}", score.grade);
    println!("    Pass:   {}/{}", score.passed, score.total);
    println!("    Fail:   {}", score.failed);

    Ok(())
}

/// Check if a result passes a specific dimension by index.
pub(crate) fn result_dim_pass(r: &crate::corpus::runner::CorpusResult, dim_idx: usize) -> bool {
    match dim_idx {
        0 => r.transpiled,
        1 => r.output_contains,
        2 => r.output_exact,
        3 => r.output_behavioral,
        4 => r.lint_clean,
        5 => r.deterministic,
        6 => r.metamorphic_consistent,
        _ => r.cross_shell_agree,
    }
}

/// Compute pass rate for a dimension filtered by format.
pub(crate) fn dim_format_rate(
    registry: &crate::corpus::registry::CorpusRegistry,
    results: &[crate::corpus::runner::CorpusResult],
    fmt: crate::corpus::registry::CorpusFormat,
    dim_idx: usize,
) -> f64 {
    let mut pass = 0usize;
    let mut total = 0usize;
    for (i, entry) in registry.entries.iter().enumerate() {
        if entry.format != fmt {
            continue;
        }
        if let Some(r) = results.get(i) {
            total += 1;
            if result_dim_pass(r, dim_idx) {
                pass += 1;
            }
        }
    }
    if total > 0 {
        pass as f64 / total as f64 * 100.0
    } else {
        100.0
    }
}

/// Find quality gaps: dimensions where specific formats underperform.
pub(crate) fn corpus_gaps() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let formats = [
        ("Bash", CorpusFormat::Bash),
        ("Makefile", CorpusFormat::Makefile),
        ("Dockerfile", CorpusFormat::Dockerfile),
    ];
    let dims = ["A", "B1", "B2", "B3", "D", "E", "F", "G"];

    println!("{BOLD}Quality Gaps{RESET} (per-format dimension pass rates)");
    println!();

    print!("  {BOLD}{:<5}", "Dim");
    for (name, _) in &formats {
        print!("{:>12}", name);
    }
    println!("   Gap{RESET}");

    for (d_idx, dim) in dims.iter().enumerate() {
        print!("  {CYAN}{:<5}{RESET}", dim);

        let mut rates: Vec<f64> = Vec::new();
        for (_, fmt) in &formats {
            let rate = dim_format_rate(&registry, &score.results, *fmt, d_idx);
            rates.push(rate);
            let color = pct_color(rate);
            print!("{color}{:>11.1}%{RESET}", rate);
        }

        let max_r = rates.iter().copied().fold(0.0f64, f64::max);
        let min_r = rates.iter().copied().fold(100.0f64, f64::min);
        let gap = max_r - min_r;
        let gc = if gap > 1.0 { YELLOW } else { DIM };
        println!("  {gc}{gap:>4.1}%{RESET}");
    }

    println!();
    println!("  {DIM}Gap = difference between best and worst format per dimension{RESET}");

    Ok(())
}

/// Compact JSON summary for CI/script consumption.
pub(crate) fn corpus_summary_json() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let json = serde_json::json!({
        "total": score.total,
        "passed": score.passed,
        "failed": score.failed,
        "rate": score.rate,
        "score": score.score,
        "grade": format!("{}", score.grade),
        "formats": score.format_scores.iter().map(|fs| {
            serde_json::json!({
                "format": format!("{}", fs.format),
                "total": fs.total,
                "passed": fs.passed,
                "rate": fs.rate,
                "score": fs.score,
                "grade": format!("{}", fs.grade),
            })
        }).collect::<Vec<_>>(),
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
    Ok(())
}

/// Full audit trail: entries, tests, build, lint status.
pub(crate) fn corpus_audit() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let start = Instant::now();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let elapsed = start.elapsed();

    println!("{BOLD}Corpus Audit{RESET}");
    println!();

    // Entry inventory
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

    println!("  {BOLD}Entries:{RESET}");
    println!("    Total:      {}", registry.entries.len());
    println!("    Bash:       {bash_count}");
    println!("    Makefile:   {make_count}");
    println!("    Dockerfile: {dock_count}");

    // Scoring results
    println!();
    println!("  {BOLD}Scoring:{RESET}");
    println!("    Score:  {GREEN}{:.1}/100{RESET}", score.score);
    println!("    Grade:  {GREEN}{}{RESET}", score.grade);
    println!("    Pass:   {GREEN}{}{RESET}/{}", score.passed, score.total);
    println!(
        "    Fail:   {}{}{RESET}",
        if score.failed > 0 { RED } else { GREEN },
        score.failed
    );

    // Dimension pass rates
    println!();
    println!("  {BOLD}Dimensions:{RESET}");
    let dims = ["A", "B1", "B2", "B3", "D", "E", "F", "G"];
    for (d_idx, dim) in dims.iter().enumerate() {
        let pass = score
            .results
            .iter()
            .filter(|r| result_dim_pass(r, d_idx))
            .count();
        let rate = pass as f64 / score.results.len().max(1) as f64 * 100.0;
        let color = pct_color(rate);
        println!(
            "    {:<3} {color}{:>4}/{:<4} {:>5.1}%{RESET}",
            dim, pass, score.total, rate
        );
    }

    // Performance
    println!();
    println!("  {BOLD}Performance:{RESET}");
    println!("    Run time: {:.1}s", elapsed.as_secs_f64());
    println!(
        "    Per entry: {:.1}ms",
        elapsed.as_secs_f64() * 1000.0 / score.total.max(1) as f64
    );

    // Convergence log
    let log_exists = std::path::Path::new(".quality/convergence.log").exists()
        || std::path::Path::new("../.quality/convergence.log").exists();
    println!();
    println!("  {BOLD}Infrastructure:{RESET}");
    println!(
        "    Convergence log: {}",
        if log_exists {
            format!("{GREEN}present{RESET}")
        } else {
            format!("{YELLOW}missing{RESET}")
        }
    );

    Ok(())
}
