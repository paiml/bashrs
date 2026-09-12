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
    corpus_converged_with_log(
        min_rate,
        max_delta,
        min_stable,
        &PathBuf::from(".quality/convergence.log"),
    )
}

/// PMAT-259: body of `corpus_converged`, split so the convergence log comes
/// from the caller. Without this the verdict depends on whichever log happens
/// to sit in the process's working directory, and a test asserting "no log"
/// passes or fails according to the repository's own state rather than the
/// code's behaviour.
pub(crate) fn corpus_converged_with_log(
    min_rate: f64,
    max_delta: f64,
    min_stable: usize,
    log_path: &std::path::Path,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let entries = CorpusRunner::load_convergence_log(log_path)
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
            entries.last().map_or(0, |e| e.iteration),
            entries.last().map_or(0, |e| e.total),
            entries.last().map_or(0.0, |e| e.score)
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

pub(crate) fn converged_no_regressions(
    entries: &[crate::corpus::runner::ConvergenceEntry],
    n: usize,
) -> bool {
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

// PMAT-261: coverage for the convergence check. Every path is a log written
// into a TempDir, so no test depends on whichever `.quality/convergence.log`
// the checkout happens to carry -- the defect PMAT-259 fixed.
#[cfg(test)]
mod pmat261_cov_tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// One convergence-log line. `rate` and `delta` are fractions, as the
    /// checks divide their percentage thresholds by 100.
    fn line(iteration: u32, rate: f64, delta: f64, passed: usize, total: usize) -> String {
        format!(
            r#"{{"iteration":{iteration},"date":"2026-09-12","total":{total},"passed":{passed},"failed":{},"rate":{rate},"delta":{delta},"notes":"fixture"}}"#,
            total - passed
        )
    }

    fn log_with(dir: &TempDir, lines: &[String]) -> std::path::PathBuf {
        let p = dir.path().join("convergence.log");
        let mut f = std::fs::File::create(&p).expect("log created");
        for l in lines {
            writeln!(f, "{l}").expect("line written");
        }
        p
    }

    #[test]
    fn test_PMAT261_cov_converged_missing_log_is_an_error() {
        let dir = TempDir::new().unwrap();
        let err = corpus_converged_with_log(99.0, 0.5, 3, &dir.path().join("absent.log"))
            .expect_err("a missing log cannot be converged");
        assert!(matches!(err, Error::Internal(_)));
    }

    #[test]
    fn test_PMAT261_cov_converged_too_few_iterations_is_an_error() {
        let dir = TempDir::new().unwrap();
        let p = log_with(&dir, &[line(1, 0.99, 0.001, 99, 100)]);
        corpus_converged_with_log(90.0, 1.0, 3, &p)
            .expect_err("one iteration cannot satisfy a three-iteration window");
    }

    #[test]
    fn test_PMAT261_cov_converged_all_checks_pass() {
        let dir = TempDir::new().unwrap();
        let p = log_with(
            &dir,
            &[
                line(1, 0.99, 0.000, 99, 100),
                line(2, 0.99, 0.001, 99, 100),
                line(3, 0.99, 0.001, 99, 100),
            ],
        );
        corpus_converged_with_log(90.0, 1.0, 3, &p)
            .expect("a steady rate above the bar with no regression is converged");
    }

    #[test]
    fn test_PMAT261_cov_converged_rate_below_threshold_fails() {
        let dir = TempDir::new().unwrap();
        let p = log_with(
            &dir,
            &[
                line(1, 0.50, 0.000, 50, 100),
                line(2, 0.50, 0.001, 50, 100),
                line(3, 0.50, 0.001, 50, 100),
            ],
        );
        corpus_converged_with_log(90.0, 1.0, 3, &p)
            .expect_err("a rate under the threshold is not converged");
    }

    #[test]
    fn test_PMAT261_cov_converged_large_delta_fails() {
        let dir = TempDir::new().unwrap();
        let p = log_with(
            &dir,
            &[
                line(1, 0.99, 0.20, 99, 100),
                line(2, 0.99, 0.20, 99, 100),
                line(3, 0.99, 0.20, 99, 100),
            ],
        );
        corpus_converged_with_log(90.0, 1.0, 3, &p)
            .expect_err("a delta far above the bar is not stable");
    }

    #[test]
    fn test_PMAT261_cov_converged_regression_between_iterations_fails() {
        let dir = TempDir::new().unwrap();
        let p = log_with(
            &dir,
            &[
                line(1, 0.99, 0.000, 99, 100),
                line(2, 0.99, 0.001, 99, 100),
                line(3, 0.91, 0.001, 91, 100),
            ],
        );
        corpus_converged_with_log(90.0, 1.0, 3, &p)
            .expect_err("a drop in passing entries is a regression");
    }

    #[test]
    fn test_PMAT261_cov_converged_print_check_both_branches() {
        converged_print_check("a passing check", true);
        converged_print_check("a failing check", false);
    }

    #[test]
    fn test_PMAT261_cov_names_similar_matches_and_rejects() {
        assert!(names_similar("cron-install-job", "cron-install-job"));
        assert!(!names_similar("cron-install-job", "perl-extract-ip"));
    }
}
