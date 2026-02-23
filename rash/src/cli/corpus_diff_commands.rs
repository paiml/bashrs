//! Corpus diff, report generation, and date utilities.

use crate::cli::args::CorpusOutputFormat;
use crate::models::{Config, Error, Result};
use std::path::PathBuf;
use super::corpus_report_commands::corpus_failing_dims;

pub(crate) fn corpus_show_diff(format: &CorpusOutputFormat, from: Option<u32>, to: Option<u32>) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;

    if entries.len() < 2 {
        return Err(Error::Validation(
            "Need at least 2 convergence entries to diff. Run `bashrs corpus run --log` multiple times.".to_string()
        ));
    }

    let from_entry = match from {
        Some(iter) => entries
            .iter()
            .find(|e| e.iteration == iter)
            .ok_or_else(|| {
                Error::Validation(format!("Iteration {iter} not found in convergence log"))
            })?,
        None => &entries[entries.len() - 2],
    };
    let to_entry = match to {
        Some(iter) => entries
            .iter()
            .find(|e| e.iteration == iter)
            .ok_or_else(|| {
                Error::Validation(format!("Iteration {iter} not found in convergence log"))
            })?,
        None => entries
            .last()
            .ok_or_else(|| Error::Validation("Empty convergence log".to_string()))?,
    };

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            println!(
                "{BOLD}Convergence Diff:{RESET} iteration {} → {}",
                from_entry.iteration, to_entry.iteration
            );
            println!();
            println!("  {DIM}{:>12}  {:>10}  {:>10}{RESET}", "", "From", "To");
            println!(
                "  {:>12}  {:>10}  {:>10}",
                "Date", from_entry.date, to_entry.date
            );
            println!(
                "  {:>12}  {:>10}  {:>10}",
                "Passed", from_entry.passed, to_entry.passed
            );
            println!(
                "  {:>12}  {:>10}  {:>10}",
                "Total", from_entry.total, to_entry.total
            );
            let from_pct = from_entry.rate * 100.0;
            let to_pct = to_entry.rate * 100.0;
            let frc = pct_color(from_pct);
            let trc = pct_color(to_pct);
            println!(
                "  {:>12}  {frc}{:>9.1}%{RESET}  {trc}{:>9.1}%{RESET}",
                "Rate", from_pct, to_pct
            );
            let rate_delta = to_entry.rate - from_entry.rate;
            let passed_delta = to_entry.passed as i64 - from_entry.passed as i64;
            println!();
            if rate_delta > 0.0 {
                println!(
                    "  {GREEN}Improvement: +{passed_delta} entries, +{:.4}% rate{RESET}",
                    rate_delta * 100.0
                );
            } else if rate_delta < 0.0 {
                println!(
                    "  {BRIGHT_RED}Regression: {passed_delta} entries, {:.4}% rate{RESET}",
                    rate_delta * 100.0
                );
            } else {
                println!("  {DIM}No change in pass rate.{RESET}");
            }
        }
        CorpusOutputFormat::Json => {
            let diff = serde_json::json!({
                "from": { "iteration": from_entry.iteration, "date": from_entry.date, "passed": from_entry.passed, "total": from_entry.total, "rate": from_entry.rate },
                "to": { "iteration": to_entry.iteration, "date": to_entry.date, "passed": to_entry.passed, "total": to_entry.total, "rate": to_entry.rate },
                "delta": { "passed": to_entry.passed as i64 - from_entry.passed as i64, "rate": to_entry.rate - from_entry.rate }
            });
            let json = serde_json::to_string_pretty(&diff)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}


pub(crate) fn corpus_generate_report(output: Option<&str>) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let date = chrono_free_date();
    let mut report = String::new();
    report.push_str("# V2 Corpus Quality Report\n\n");
    report.push_str(&format!("**Date**: {date}\n\n"));
    report.push_str(&format!(
        "## Score: {:.1}/100 ({})\n\n",
        score.score, score.grade
    ));

    // Summary table
    report.push_str("| Metric | Value |\n|--------|-------|\n");
    report.push_str(&format!("| Total entries | {} |\n", score.total));
    report.push_str(&format!("| Passed | {} |\n", score.passed));
    report.push_str(&format!("| Failed | {} |\n", score.failed));
    report.push_str(&format!("| Pass rate | {:.1}% |\n", score.rate * 100.0));
    report.push('\n');

    // Per-format breakdown
    report.push_str("## Format Breakdown\n\n");
    report.push_str("| Format | Score | Grade | Passed | Total |\n");
    report.push_str("|--------|-------|-------|--------|-------|\n");
    for fs in &score.format_scores {
        report.push_str(&format!(
            "| {} | {:.1}/100 | {} | {} | {} |\n",
            fs.format, fs.score, fs.grade, fs.passed, fs.total
        ));
    }
    report.push('\n');

    // Failures
    let failures: Vec<_> = score
        .results
        .iter()
        .filter(|r| {
            !r.transpiled
                || !r.output_behavioral
                || !r.cross_shell_agree
                || !r.lint_clean
                || !r.deterministic
                || !r.schema_valid
        })
        .collect();

    if failures.is_empty() {
        report.push_str("## Failures\n\nNone.\n\n");
    } else {
        report.push_str(&format!("## Failures ({})\n\n", failures.len()));
        report.push_str("| ID | Score | Failing Dimensions |\n");
        report.push_str("|----|-------|--------------------|\n");
        for r in &failures {
            let dims = corpus_failing_dims(r);
            report.push_str(&format!("| {} | {:.1} | {} |\n", r.id, r.score(), dims));
        }
        report.push('\n');
    }

    // Convergence history
    let log_path = PathBuf::from(".quality/convergence.log");
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    if !history.is_empty() {
        report.push_str("## Convergence History\n\n");
        report.push_str("| Iter | Date | Pass/Total | Rate | Delta |\n");
        report.push_str("|------|------|------------|------|-------|\n");
        let display = if history.len() > 10 {
            &history[history.len() - 10..]
        } else {
            &history
        };
        for e in display {
            report.push_str(&format!(
                "| {} | {} | {}/{} | {:.1}% | {:+.4} |\n",
                e.iteration,
                e.date,
                e.passed,
                e.total,
                e.rate * 100.0,
                e.delta
            ));
        }
        report.push('\n');
    }

    // V2 scoring formula reference
    report.push_str("## V2 Scoring Formula\n\n");
    report.push_str("| Dimension | Points | Description |\n");
    report.push_str("|-----------|--------|-------------|\n");
    report.push_str("| A | 30 | Transpilation succeeds |\n");
    report.push_str("| B1 | 10 | Output contains expected |\n");
    report.push_str("| B2 | 8 | Exact output match |\n");
    report.push_str("| B3 | 7 | Behavioral equivalence |\n");
    report.push_str("| C | 15 | LLVM coverage ratio |\n");
    report.push_str("| D | 10 | Lint clean |\n");
    report.push_str("| E | 10 | Deterministic output |\n");
    report.push_str("| F | 5 | Metamorphic consistency |\n");
    report.push_str("| G | 5 | Cross-shell agreement |\n");

    match output {
        Some(path) => {
            std::fs::write(path, &report)
                .map_err(|e| Error::Internal(format!("Failed to write report to {path}: {e}")))?;
            println!("Report written to {path}");
        }
        None => print!("{report}"),
    }
    Ok(())
}

/// Generate ISO 8601 date string without chrono dependency.
pub(crate) fn chrono_free_date() -> String {
    use std::process::Command;
    Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
