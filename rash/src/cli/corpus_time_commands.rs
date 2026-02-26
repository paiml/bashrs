//! Corpus temporal commands: timeline, drift analysis, slow entries, and tag management.

use crate::cli::args::CorpusFormatArg;
use crate::models::{Config, Error, Result};

pub(crate) fn corpus_timeline() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;
    use std::path::PathBuf;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to load convergence log: {e}")))?;

    if entries.is_empty() {
        println!("No convergence log entries found.");
        return Ok(());
    }

    println!("{BOLD}Corpus Growth Timeline{RESET}");
    println!();

    // Find max total for bar scaling
    let max_total = entries.iter().map(|e| e.total).max().unwrap_or(1) as f64;

    println!(
        "  {BOLD}{:<4} {:<12} {:>6} {:>6} {:>7} {:>7}  Growth Bar{RESET}",
        "Iter", "Date", "Total", "Pass", "Rate", "Score"
    );

    for entry in &entries {
        let bar_len = ((entry.total as f64 / max_total) * 30.0) as usize;
        let bar: String = "\u{2588}".repeat(bar_len);
        let empty: String = "\u{2591}".repeat(30 - bar_len);

        let rate_pct = entry.rate * 100.0;
        let rc = pct_color(rate_pct);
        let sc = if entry.score > 0.0 {
            format!("{:.1}", entry.score)
        } else {
            "-".to_string()
        };

        let delta_str = if entry.delta != 0.0 {
            let arrow = if entry.delta > 0.0 {
                "\u{2191}"
            } else {
                "\u{2193}"
            };
            format!(" {arrow}{:.1}%", entry.delta.abs() * 100.0)
        } else {
            String::new()
        };

        println!("  {:<4} {:<12} {:>6} {:>6} {rc}{:>6.1}%{RESET} {:>7}  {GREEN}{bar}{RESET}{DIM}{empty}{RESET}{delta_str}",
            entry.iteration, entry.date, entry.total, entry.passed,
            rate_pct, sc);
    }

    // Summary
    if entries.len() >= 2 {
        let first = &entries[0];
        let last = &entries[entries.len() - 1];
        let growth = last.total as i64 - first.total as i64;
        let iters = entries.len();
        println!();
        println!("  {DIM}Growth: +{growth} entries over {iters} iterations{RESET}");
        if last.score > 0.0 && first.score > 0.0 {
            let score_delta = last.score - first.score;
            let arrow = if score_delta >= 0.0 {
                "\u{2191}"
            } else {
                "\u{2193}"
            };
            println!(
                "  {DIM}Score: {:.1} → {:.1} ({arrow}{:.2}){RESET}",
                first.score,
                last.score,
                score_delta.abs()
            );
        }
    }
    Ok(())
}

/// Print format drift line for a single format dimension.
pub(crate) fn drift_print_format(
    name: &str,
    fp: usize,
    ft: usize,
    fs: f64,
    lp: usize,
    lt: usize,
    ls: f64,
) {
    use crate::cli::color::*;
    if ft == 0 && lt == 0 {
        return;
    }
    let first_rate = if ft > 0 {
        fp as f64 / ft as f64 * 100.0
    } else {
        0.0
    };
    let last_rate = if lt > 0 {
        lp as f64 / lt as f64 * 100.0
    } else {
        0.0
    };
    let rate_delta = last_rate - first_rate;
    let score_delta = ls - fs;
    let arrow_r = if rate_delta >= 0.0 {
        "\u{2191}"
    } else {
        "\u{2193}"
    };
    let rc = if rate_delta >= 0.0 { GREEN } else { RED };
    print!("    {CYAN}{name:<12}{RESET} rate: {rc}{arrow_r}{rate_delta:+.1}%{RESET}");
    if fs > 0.0 || ls > 0.0 {
        let arrow_s = if score_delta >= 0.0 {
            "\u{2191}"
        } else {
            "\u{2193}"
        };
        let sc = if score_delta >= 0.0 { GREEN } else { RED };
        print!("  score: {sc}{arrow_s}{score_delta:+.1}{RESET}");
    }
    println!("  ({lp}/{lt} passed)");
}

/// Detect per-dimension score drift across convergence iterations.
pub(crate) fn corpus_drift() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;
    use std::path::PathBuf;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to load convergence log: {e}")))?;

    if entries.len() < 2 {
        println!("Need at least 2 convergence iterations for drift detection.");
        return Ok(());
    }

    println!("{BOLD}Per-Dimension Drift Analysis{RESET}");
    println!();

    let scored: Vec<_> = entries.iter().filter(|e| e.score > 0.0).collect();
    if scored.len() < 2 {
        println!("  {DIM}Not enough scored iterations for drift analysis.{RESET}");
        return Ok(());
    }

    let first = scored[0];
    let last = scored[scored.len() - 1];
    println!("  {BOLD}Overall Score:{RESET}");
    let arrow = if last.score >= first.score {
        "\u{2191}"
    } else {
        "\u{2193}"
    };
    let color = if last.score >= first.score {
        GREEN
    } else {
        RED
    };
    println!(
        "    {:.1} \u{2192} {:.1} ({color}{arrow}{:.2}{RESET})",
        first.score,
        last.score,
        (last.score - first.score).abs()
    );
    println!();

    println!("  {BOLD}Per-Format Drift:{RESET}");
    drift_print_format(
        "Bash",
        first.bash_passed,
        first.bash_total,
        first.bash_score,
        last.bash_passed,
        last.bash_total,
        last.bash_score,
    );
    drift_print_format(
        "Makefile",
        first.makefile_passed,
        first.makefile_total,
        first.makefile_score,
        last.makefile_passed,
        last.makefile_total,
        last.makefile_score,
    );
    drift_print_format(
        "Dockerfile",
        first.dockerfile_passed,
        first.dockerfile_total,
        first.dockerfile_score,
        last.dockerfile_passed,
        last.dockerfile_total,
        last.dockerfile_score,
    );

    println!();
    println!("  {BOLD}Pass Rate History:{RESET}");
    for entry in &entries {
        let rate_pct = entry.rate * 100.0;
        let rc = pct_color(rate_pct);
        let lint_str = if entry.lint_rate > 0.0 {
            format!("  lint: {:.1}%", entry.lint_rate * 100.0)
        } else {
            String::new()
        };
        println!(
            "    iter {:<3} {rc}{:>6.1}%{RESET}  ({}/{} passed){lint_str}",
            entry.iteration, rate_pct, entry.passed, entry.total
        );
    }

    println!();
    if last.rate < first.rate {
        println!(
            "  {YELLOW}WARNING: Overall pass rate has decreased ({:.2}% \u{2192} {:.2}%){RESET}",
            first.rate * 100.0,
            last.rate * 100.0
        );
    } else {
        println!("  {GREEN}No negative drift detected.{RESET}");
    }
    Ok(())
}

/// Show entries sorted by transpilation time (slowest first).
pub(crate) fn corpus_slow(limit: usize, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::cli::color::*;
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

    // Time each entry
    let mut timings: Vec<(&str, &str, f64)> = Vec::new();
    for entry in &entries {
        let start = Instant::now();
        let _ = runner.run_single(entry);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        timings.push((&entry.id, &entry.name, ms));
    }

    // Sort by time descending
    timings.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let total_ms: f64 = timings.iter().map(|(_, _, ms)| ms).sum();
    let n = timings.len();

    println!("{BOLD}Slowest Corpus Entries{RESET} (top {limit} of {n})");
    println!(
        "{DIM}  Total: {total_ms:.0}ms | Avg: {:.1}ms{RESET}",
        total_ms / n as f64
    );
    println!();

    println!(
        "  {BOLD}{:<8} {:>8} {:>6}  Name{RESET}",
        "ID", "Time", "% Tot"
    );
    for (id, name, ms) in timings.iter().take(limit) {
        let pct = ms / total_ms * 100.0;
        let color = if *ms > 1000.0 {
            BRIGHT_RED
        } else if *ms > 100.0 {
            YELLOW
        } else {
            DIM
        };
        let name_short = if name.len() > 40 { &name[..40] } else { name };
        println!(
            "  {CYAN}{:<8}{RESET} {color}{:>7.1}ms{RESET} {:>5.1}%  {DIM}{name_short}{RESET}",
            id, ms, pct
        );
    }

    // Cumulative top-N percentage
    let top_n_ms: f64 = timings.iter().take(limit).map(|(_, _, ms)| ms).sum();
    let top_pct = top_n_ms / total_ms * 100.0;
    println!();
    println!("  {DIM}Top {limit} entries account for {top_pct:.1}% of total time.{RESET}");
    Ok(())
}

/// Show entries grouped by shell construct type.
pub(crate) fn corpus_tags() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();

    // Tag classification based on entry name/description keywords
    let tag_rules: &[(&str, &[&str])] = &[
        (
            "variable",
            &[
                "variable",
                "assignment",
                "var-",
                "let ",
                "readonly",
                "export",
            ],
        ),
        (
            "loop",
            &["loop", "for-", "while-", "until-", "iteration", "seq"],
        ),
        (
            "conditional",
            &["if-", "elif", "conditional", "ternary", "case-", "test-"],
        ),
        (
            "pipe",
            &["pipe", "pipeline", "redirect", "heredoc", "herestring"],
        ),
        (
            "arithmetic",
            &["arithmetic", "math", "calc", "expr", "integer", "modulo"],
        ),
        (
            "string",
            &["string", "concat", "substr", "trim", "quote", "escape"],
        ),
        (
            "function",
            &["function", "func-", "return-", "recursion", "scope"],
        ),
        ("array", &["array", "list", "assoc"]),
        (
            "process",
            &[
                "process",
                "subshell",
                "background",
                "trap",
                "signal",
                "exit",
            ],
        ),
        (
            "file-io",
            &["file", "read-", "write-", "mkdir", "chmod", "path", "temp"],
        ),
        ("regex", &["regex", "pattern", "glob", "match", "replace"]),
        (
            "security",
            &["injection", "sec-", "sanitize", "adversarial"],
        ),
    ];

    let mut tag_map: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    let mut untagged = Vec::new();

    for entry in &registry.entries {
        let name_lower = entry.name.to_lowercase();
        let desc_lower = entry.description.to_lowercase();
        let mut tagged = false;
        for (tag, keywords) in tag_rules {
            if keywords
                .iter()
                .any(|kw| name_lower.contains(kw) || desc_lower.contains(kw))
            {
                tag_map.entry(tag).or_default().push(&entry.id);
                tagged = true;
                break; // first match wins
            }
        }
        if !tagged {
            untagged.push(&entry.id);
        }
    }

    let total = registry.entries.len();
    println!("{BOLD}Corpus Construct Tags{RESET} ({total} entries)");
    println!();

    println!(
        "  {BOLD}{:<14} {:>5} {:>6}  Sample IDs{RESET}",
        "Tag", "Count", "% Tot"
    );
    for (tag, ids) in &tag_map {
        let pct = ids.len() as f64 / total as f64 * 100.0;
        let sample: Vec<_> = ids.iter().take(3).copied().collect();
        let sample_str = sample.join(", ");
        let more = if ids.len() > 3 {
            format!(", +{}", ids.len() - 3)
        } else {
            String::new()
        };
        println!(
            "  {CYAN}{:<14}{RESET} {:>5} {:>5.1}%  {DIM}{sample_str}{more}{RESET}",
            tag,
            ids.len(),
            pct
        );
    }
    if !untagged.is_empty() {
        let pct = untagged.len() as f64 / total as f64 * 100.0;
        println!(
            "  {DIM}{:<14} {:>5} {:>5.1}%  (no construct tag){RESET}",
            "untagged",
            untagged.len(),
            pct
        );
    }

    println!();
    let tagged_count: usize = tag_map.values().map(|v| v.len()).sum();
    println!(
        "  {DIM}{tagged_count} tagged / {total} total ({:.1}% tagged){RESET}",
        tagged_count as f64 / total as f64 * 100.0
    );
    Ok(())
}
