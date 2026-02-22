//! Corpus operations: duplication detection, convergence checks, and benchmarking.

use crate::cli::args::CorpusFormatArg;
use crate::models::{Config, Error, Result};
use std::path::PathBuf;

pub(crate) fn corpus_dupes() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let mut dupes: Vec<(&str, &str, &str)> = Vec::new();

    // Compare all pairs (O(n^2) but n=900 is fine)
    for i in 0..registry.entries.len() {
        for j in (i + 1)..registry.entries.len() {
            let a = &registry.entries[i];
            let b = &registry.entries[j];
            // Same format only
            if a.format != b.format {
                continue;
            }
            // Check name similarity
            if names_similar(&a.name, &b.name) {
                dupes.push((&a.id, &b.id, &a.name));
            }
        }
    }

    if dupes.is_empty() {
        println!("{GREEN}No potential duplicates found.{RESET}");
    } else {
        println!("{BOLD}Potential Duplicates ({} pairs):{RESET}", dupes.len());
        println!();
        for (a, b, name) in dupes.iter().take(20) {
            println!("  {YELLOW}{a}{RESET} \u{2194} {YELLOW}{b}{RESET}  {DIM}({name}){RESET}");
        }
        if dupes.len() > 20 {
            println!("  {DIM}... and {} more{RESET}", dupes.len() - 20);
        }
    }
    Ok(())
}

/// Check if two entry names are similar enough to flag as potential duplicates.

pub(crate) fn names_similar(a: &str, b: &str) -> bool {
    // Exact match (different IDs, same name)
    if a == b {
        return true;
    }
    // One is a prefix of the other (e.g., "variable" and "variable-assignment")
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    // Same normalized name after removing common suffixes
    let strip_suffix = |s: &str| -> String {
        s.trim_end_matches("-basic")
            .trim_end_matches("-simple")
            .trim_end_matches("-advanced")
            .to_string()
    };
    strip_suffix(&a_lower) == strip_suffix(&b_lower) && a_lower != b_lower
}

/// Check convergence criteria from spec §5.2.
/// Returns exit code 0 if converged, 1 if not.

pub(crate) fn corpus_converged(min_rate: f64, max_delta: f64, min_stable: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;

    if entries.len() < min_stable {
        println!(
            "{YELLOW}NOT CONVERGED{RESET}: need {min_stable} iterations, have {}",
            entries.len()
        );
        return Err(Error::Internal("Not converged".to_string()));
    }

    let recent: Vec<_> = entries.iter().rev().take(min_stable).collect();
    let rate_threshold = min_rate / 100.0;

    // Check 1: Rate >= threshold for min_stable consecutive iterations
    let all_above_rate = recent.iter().all(|e| e.rate >= rate_threshold);
    // Check 2: Delta < max_delta for min_stable consecutive iterations
    let all_stable = recent.iter().all(|e| e.delta.abs() < max_delta / 100.0);
    // Check 3: No regressions between consecutive entries
    let no_regressions = converged_no_regressions(&entries, min_stable);

    println!("{BOLD}Convergence Check (spec §5.2){RESET}");
    println!();
    converged_print_check(
        &format!("Rate >= {min_rate}% for {min_stable} iters"),
        all_above_rate,
    );
    converged_print_check(
        &format!("Delta < {max_delta}% for {min_stable} iters"),
        all_stable,
    );
    converged_print_check(
        &format!("No regressions in last {min_stable} iters"),
        no_regressions,
    );
    println!();

    if all_above_rate && all_stable && no_regressions {
        println!(
            "  {BRIGHT_GREEN}CONVERGED{RESET} at iteration {} ({} entries, {:.1}/100)",
            entries.last().map(|e| e.iteration).unwrap_or(0),
            entries.last().map(|e| e.total).unwrap_or(0),
            entries.last().map(|e| e.score).unwrap_or(0.0)
        );
        println!("  {DIM}Per spec §5.2: expand corpus with harder entries.{RESET}");
        Ok(())
    } else {
        println!("  {BRIGHT_RED}NOT CONVERGED{RESET}");
        Err(Error::Internal("Not converged".to_string()))
    }
}


pub(crate) fn converged_print_check(label: &str, pass: bool) {
    use crate::cli::color::*;
    let mark = if pass {
        format!("{GREEN}\u{2713}{RESET}")
    } else {
        format!("{RED}\u{2717}{RESET}")
    };
    println!("  {mark} {label}");
}


pub(crate) fn converged_no_regressions(entries: &[crate::corpus::runner::ConvergenceEntry], n: usize) -> bool {
    if entries.len() < 2 {
        return true;
    }
    let start = entries.len().saturating_sub(n);
    for pair in entries[start..].windows(2) {
        let report = pair[1].detect_regressions(&pair[0]);
        if report.has_regressions() {
            return false;
        }
    }
    true
}

/// Benchmark transpilation time per entry (spec §8.2).

pub(crate) fn corpus_benchmark(max_ms: u64, filter: Option<&CorpusFormatArg>) -> Result<()> {
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

    let mut timings: Vec<(String, u128)> = Vec::with_capacity(entries.len());
    let start_all = Instant::now();
    for entry in &entries {
        let t = Instant::now();
        let _ = runner.run_single(entry);
        let elapsed = t.elapsed().as_millis();
        timings.push((entry.id.clone(), elapsed));
    }
    let total_ms = start_all.elapsed().as_millis();

    // Sort by time descending
    timings.sort_by(|a, b| b.1.cmp(&a.1));

    let times: Vec<u128> = timings.iter().map(|(_, t)| *t).collect();
    let avg = times.iter().sum::<u128>() as f64 / times.len().max(1) as f64;
    let max_time = times.first().copied().unwrap_or(0);
    let min_time = times.last().copied().unwrap_or(0);
    let p95_idx = (times.len() as f64 * 0.05) as usize;
    let p95 = times.get(p95_idx).copied().unwrap_or(0);
    let violations: Vec<_> = timings
        .iter()
        .filter(|(_, t)| *t > max_ms as u128)
        .collect();

    println!(
        "{BOLD}Corpus Benchmark ({} entries, {}ms total){RESET}",
        entries.len(),
        total_ms
    );
    println!();
    println!("  {BOLD}Timing Statistics:{RESET}");
    println!("    Min:  {min_time}ms");
    println!("    Avg:  {avg:.1}ms");
    println!("    P95:  {p95}ms");
    println!("    Max:  {max_time}ms");
    println!();

    if violations.is_empty() {
        println!("  {GREEN}All entries under {max_ms}ms threshold.{RESET}");
    } else {
        println!(
            "  {BRIGHT_RED}{} entries exceed {max_ms}ms threshold:{RESET}",
            violations.len()
        );
        for (id, t) in violations.iter().take(10) {
            println!("    {RED}{id}{RESET}: {t}ms");
        }
    }

    // Top 5 slowest
    println!();
    println!("  {BOLD}Slowest 5:{RESET}");
    for (id, t) in timings.iter().take(5) {
        let tc = if *t > max_ms as u128 { RED } else { GREEN };
        println!("    {tc}{id}{RESET}: {t}ms");
    }
    Ok(())
}
