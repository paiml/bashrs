//! Corpus ranking: sparkline, top entries, categories, dimensions, and statistics.

use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
use crate::models::{Config, Error, Result};
use super::corpus_failure_commands::result_fail_dims;
use std::path::PathBuf;

pub(crate) fn corpus_sparkline() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;
    if entries.is_empty() {
        println!("No convergence history. Run `bashrs corpus run --log` first.");
        return Ok(());
    }

    let scores: Vec<f64> = entries.iter().map(|e| e.score).collect();
    let spark = sparkline_str(&scores);
    let first = scores.first().copied().unwrap_or(0.0);
    let last = scores.last().copied().unwrap_or(0.0);
    let sc = pct_color(last);

    println!("{BOLD}Score Trend{RESET} ({} iterations):", entries.len());
    println!("  {spark}  {sc}{last:.1}/100{RESET}");
    println!("  {DIM}{first:.1} \u{2192} {last:.1}{RESET}");

    // Per-format sparklines if available
    let bash_scores: Vec<f64> = entries.iter().map(|e| e.bash_score).collect();
    let make_scores: Vec<f64> = entries.iter().map(|e| e.makefile_score).collect();
    let dock_scores: Vec<f64> = entries.iter().map(|e| e.dockerfile_score).collect();

    if bash_scores.iter().any(|&s| s > 0.0) {
        println!("  {CYAN}bash:      {RESET} {}", sparkline_str(&bash_scores));
    }
    if make_scores.iter().any(|&s| s > 0.0) {
        println!("  {CYAN}makefile:  {RESET} {}", sparkline_str(&make_scores));
    }
    if dock_scores.iter().any(|&s| s > 0.0) {
        println!("  {CYAN}dockerfile:{RESET} {}", sparkline_str(&dock_scores));
    }
    Ok(())
}

/// Generate a sparkline string from a series of values.
pub(crate) fn sparkline_str(data: &[f64]) -> String {
    if data.is_empty() {
        return String::new();
    }
    let blocks = [
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    let min = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    data.iter()
        .map(|&v| {
            if range < 0.001 {
                blocks[7] // All same value → full block
            } else {
                let idx = ((v - min) / range * 7.0).round() as usize;
                blocks[idx.min(7)]
            }
        })
        .collect()
}

/// Show top/bottom entries ranked by failure count.
pub(crate) fn corpus_top(limit: usize, worst: bool, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::cli::color::*;
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

    let mut ranked: Vec<_> = score
        .results
        .iter()
        .map(|r| {
            let fail_count = result_fail_dims(r).len();
            (r, fail_count)
        })
        .collect();

    if worst {
        // Most failures first
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.id.cmp(&b.0.id)));
    } else {
        // Fewest failures first (best entries)
        ranked.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.id.cmp(&b.0.id)));
    }
    let ranked: Vec<_> = ranked.into_iter().take(limit).collect();

    let label = if worst { "Bottom" } else { "Top" };
    println!("{BOLD}{label} {limit} Entries (by failure count):{RESET}");
    println!();
    println!(
        "  {BOLD}{:<8} {:>5}  Failing Dimensions{RESET}",
        "ID", "Fails"
    );
    for (r, fail_count) in &ranked {
        let dims = result_fail_dims(r);
        let dim_str = if dims.is_empty() {
            format!("{GREEN}all pass{RESET}")
        } else {
            format!("{RED}{}{RESET}", dims.join(", "))
        };
        let fc = if *fail_count == 0 {
            format!("{GREEN}{:>5}{RESET}", fail_count)
        } else {
            format!("{RED}{:>5}{RESET}", fail_count)
        };
        println!("  {:<8} {}  {}", r.id, fc, dim_str);
    }
    Ok(())
}

/// Category classification rules: each entry is (&[keywords], category_label)
const CATEGORY_RULES: &[(&[&str], &str)] = &[
    (&["config", "bashrc", "profile", "alias", "xdg", "history"], "Config (A)"),
    (&["oneliner", "one-liner", "pipe-", "pipeline"], "One-liner (B)"),
    (&["coreutil", "reimpl"], "Coreutils (G)"),
    (&["regex", "pattern-match", "glob-match"], "Regex (H)"),
    (&["daemon", "cron", "startup", "service"], "System (F)"),
    (&["milestone"], "Milestone"),
    (&["adversarial", "injection", "fuzz"], "Adversarial"),
];

/// Classify entry into domain-specific category based on name/description (spec §11.11).
pub(crate) fn classify_category(name: &str) -> &'static str {
    let n = name.to_lowercase();
    for (keywords, category) in CATEGORY_RULES {
        if keywords.iter().any(|kw| n.contains(kw)) {
            return category;
        }
    }
    "General"
}

/// Show entries grouped by domain-specific category (spec §11.11).
pub(crate) fn corpus_categories(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let mut cats: std::collections::BTreeMap<&str, Vec<&str>> = std::collections::BTreeMap::new();
    for e in &registry.entries {
        let cat = classify_category(&e.name);
        cats.entry(cat).or_default().push(&e.id);
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Domain-Specific Categories (spec §11.11){RESET}");
            println!();
            println!(
                "  {BOLD}{:<18} {:>5}  Sample IDs{RESET}",
                "Category", "Count"
            );
            let total = registry.entries.len();
            for (cat, ids) in &cats {
                let sample: Vec<_> = ids.iter().take(5).copied().collect();
                let more = if ids.len() > 5 {
                    format!(" {DIM}(+{}){RESET}", ids.len() - 5)
                } else {
                    String::new()
                };
                let pct = ids.len() as f64 / total as f64 * 100.0;
                println!(
                    "  {CYAN}{:<18}{RESET} {:>5}  {DIM}({pct:>5.1}%){RESET}  {}{}",
                    cat,
                    ids.len(),
                    sample.join(", "),
                    more
                );
            }
            println!();
            println!(
                "  {DIM}Total: {total} entries in {} categories{RESET}",
                cats.len()
            );
        }
        CorpusOutputFormat::Json => {
            let result: Vec<_> = cats
                .iter()
                .map(|(cat, ids)| {
                    serde_json::json!({
                        "category": cat,
                        "count": ids.len(),
                        "ids": ids,
                    })
                })
                .collect();
            let json = serde_json::to_string_pretty(&serde_json::json!({
                "total": registry.entries.len(),
                "categories": result,
            }))
            .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Show per-dimension pass rates, weights, and point contributions.
pub(crate) fn corpus_dimensions(format: &CorpusOutputFormat, filter: Option<&CorpusFormatArg>) -> Result<()> {
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

    let total = score.results.len();
    let dims = compute_dimension_stats(&score.results, total);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}V2 Dimension Analysis ({total} entries){RESET}");
            println!();
            println!(
                "  {BOLD}{:<4} {:<16} {:>6} {:>6} {:>7}  {:>6} {:>6}{RESET}",
                "Dim", "Name", "Pass", "Fail", "Rate", "Weight", "Points"
            );
            for d in &dims {
                let rc = pct_color(d.rate * 100.0);
                println!(
                    "  {:<4} {:<16} {:>6} {:>6} {rc}{:>6.1}%{RESET}  {:>6.0} {:>6.1}",
                    d.code,
                    d.name,
                    d.pass,
                    d.fail,
                    d.rate * 100.0,
                    d.weight,
                    d.points
                );
            }
            let total_pts: f64 = dims.iter().map(|d| d.points).sum();
            let total_wt: f64 = dims.iter().map(|d| d.weight).sum();
            println!();
            println!(
                "  {BOLD}{:<4} {:<16} {:>6} {:>6} {:>7}  {:>6.0} {:>6.1}{RESET}",
                "", "Total", "", "", "", total_wt, total_pts
            );
        }
        CorpusOutputFormat::Json => {
            let result: Vec<_> = dims
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "code": d.code, "name": d.name,
                        "pass": d.pass, "fail": d.fail,
                        "rate": d.rate, "weight": d.weight, "points": d.points,
                    })
                })
                .collect();
            let json = serde_json::to_string_pretty(&serde_json::json!({
                "total_entries": total,
                "dimensions": result,
            }))
            .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}


pub(crate) struct DimStat {
    code: &'static str,
    name: &'static str,
    pass: usize,
    fail: usize,
    rate: f64,
    weight: f64,
    points: f64,
}


pub(crate) fn compute_dimension_stats(
    results: &[crate::corpus::runner::CorpusResult],
    total: usize,
) -> Vec<DimStat> {
    let count = |f: &dyn Fn(&crate::corpus::runner::CorpusResult) -> bool| -> usize {
        results.iter().filter(|r| f(r)).count()
    };
    let dim = |code: &'static str, name: &'static str, pass: usize, weight: f64| -> DimStat {
        let fail = total - pass;
        let rate = if total > 0 {
            pass as f64 / total as f64
        } else {
            0.0
        };
        let points = rate * weight;
        DimStat {
            code,
            name,
            pass,
            fail,
            rate,
            weight,
            points,
        }
    };
    vec![
        dim("A", "Transpilation", count(&|r| r.transpiled), 30.0),
        dim("B1", "Containment", count(&|r| r.output_contains), 10.0),
        dim("B2", "Exact match", count(&|r| r.output_exact), 8.0),
        dim("B3", "Behavioral", count(&|r| r.output_behavioral), 7.0),
        dim("C", "Coverage", total, 15.0), // coverage is always "pass" (ratio-based)
        dim("D", "Lint clean", count(&|r| r.lint_clean), 10.0),
        dim("E", "Deterministic", count(&|r| r.deterministic), 10.0),
        dim(
            "F",
            "Metamorphic",
            count(&|r| r.metamorphic_consistent),
            5.0,
        ),
        dim("G", "Cross-shell", count(&|r| r.cross_shell_agree), 5.0),
    ]
}
