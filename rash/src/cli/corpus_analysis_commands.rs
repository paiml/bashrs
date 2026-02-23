//! Corpus analysis: summary, growth, coverage, and validation.

use crate::cli::args::CorpusOutputFormat;
use crate::models::{Config, Error, Result};
use super::corpus_failure_commands::result_fail_dims;
use std::path::PathBuf;

pub(crate) fn corpus_summary() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let failures: Vec<_> = score
        .results
        .iter()
        .filter(|r| !result_fail_dims(r).is_empty())
        .collect();
    let fail_ids: Vec<_> = failures.iter().map(|r| r.id.as_str()).collect();

    if failures.is_empty() {
        println!(
            "{} entries, {:.1}/100 {}, 0 failures",
            score.results.len(),
            score.score,
            score.grade
        );
    } else {
        println!(
            "{} entries, {:.1}/100 {}, {} failure(s) ({})",
            score.results.len(),
            score.score,
            score.grade,
            failures.len(),
            fail_ids.join(", ")
        );
    }
    Ok(())
}

/// Show corpus size growth over time from convergence log (spec §4).
pub(crate) fn corpus_growth(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;

    if entries.is_empty() {
        println!("No convergence history. Run `bashrs corpus run --log` first.");
        return Ok(());
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Corpus Growth (from convergence log){RESET}");
            println!(
                "{DIM}{:>4}  {:>10}  {:>5}  {:>6}  Notes{RESET}",
                "Iter", "Date", "Total", "Added"
            );

            let mut prev_total = 0;
            for e in &entries {
                let added = e.total.saturating_sub(prev_total);
                let added_str = if added > 0 {
                    format!("{GREEN}+{added}{RESET}")
                } else {
                    format!("{DIM}  0{RESET}")
                };
                println!(
                    "{:>4}  {:>10}  {:>5}  {}  {}",
                    e.iteration, e.date, e.total, added_str, e.notes
                );
                prev_total = e.total;
            }

            let first = entries.first().map(|e| e.total).unwrap_or(0);
            let last = entries.last().map(|e| e.total).unwrap_or(0);
            let growth = last.saturating_sub(first);
            println!();
            println!("  {BOLD}Total growth{RESET}: {first} → {last} ({GREEN}+{growth}{RESET} entries over {} iterations)",
                entries.len());
        }
        CorpusOutputFormat::Json => {
            let growth: Vec<_> = entries
                .windows(2)
                .map(|w| {
                    serde_json::json!({
                        "iteration": w[1].iteration,
                        "date": w[1].date,
                        "total": w[1].total,
                        "added": w[1].total.saturating_sub(w[0].total),
                    })
                })
                .collect();
            let result = serde_json::json!({
                "first_total": entries.first().map(|e| e.total),
                "last_total": entries.last().map(|e| e.total),
                "iterations": entries.len(),
                "growth": growth,
            });
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Show tier × format coverage matrix (spec §2.3).
pub(crate) fn corpus_coverage(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry, CorpusTier};

    let registry = CorpusRegistry::load_full();

    let tiers = [
        (CorpusTier::Trivial, "Trivial"),
        (CorpusTier::Standard, "Standard"),
        (CorpusTier::Complex, "Complex"),
        (CorpusTier::Adversarial, "Adversarial"),
        (CorpusTier::Production, "Production"),
    ];
    let count = |t: &CorpusTier, f: &CorpusFormat| -> usize {
        registry
            .entries
            .iter()
            .filter(|e| &e.tier == t && &e.format == f)
            .count()
    };

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Corpus Coverage: Tier × Format Matrix{RESET}");
            println!();
            println!(
                "  {BOLD}{:<14} {:>6} {:>9} {:>11}  {:>5}{RESET}",
                "Tier", "Bash", "Makefile", "Dockerfile", "Total"
            );

            let mut grand_total = 0;
            for (tier, label) in &tiers {
                let bash = count(tier, &CorpusFormat::Bash);
                let make = count(tier, &CorpusFormat::Makefile);
                let dock = count(tier, &CorpusFormat::Dockerfile);
                let total = bash + make + dock;
                grand_total += total;
                println!(
                    "  {:<14} {:>6} {:>9} {:>11}  {:>5}",
                    label, bash, make, dock, total
                );
            }
            println!(
                "  {DIM}{:<14} {:>6} {:>9} {:>11}  {:>5}{RESET}",
                "Total",
                count_format(&registry, &CorpusFormat::Bash),
                count_format(&registry, &CorpusFormat::Makefile),
                count_format(&registry, &CorpusFormat::Dockerfile),
                grand_total
            );
        }
        CorpusOutputFormat::Json => {
            let matrix: Vec<_> = tiers
                .iter()
                .map(|(tier, label)| {
                    serde_json::json!({
                        "tier": label,
                        "bash": count(tier, &CorpusFormat::Bash),
                        "makefile": count(tier, &CorpusFormat::Makefile),
                        "dockerfile": count(tier, &CorpusFormat::Dockerfile),
                    })
                })
                .collect();
            let result = serde_json::json!({
                "total": registry.entries.len(),
                "matrix": matrix,
            });
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}


pub(crate) fn count_format(
    registry: &crate::corpus::registry::CorpusRegistry,
    format: &crate::corpus::registry::CorpusFormat,
) -> usize {
    registry
        .entries
        .iter()
        .filter(|e| &e.format == format)
        .count()
}

/// Validate a single corpus entry and return issues found.
pub(crate) fn validate_corpus_entry(
    entry: &crate::corpus::registry::CorpusEntry,
    seen_ids: &mut std::collections::HashSet<String>,
) -> Vec<String> {
    use crate::corpus::registry::CorpusFormat;
    let mut issues = Vec::new();

    if !seen_ids.insert(entry.id.clone()) {
        issues.push("Duplicate ID".to_string());
    }
    let valid_prefix = match entry.format {
        CorpusFormat::Bash => entry.id.starts_with("B-"),
        CorpusFormat::Makefile => entry.id.starts_with("M-"),
        CorpusFormat::Dockerfile => entry.id.starts_with("D-"),
    };
    if !valid_prefix {
        issues.push(format!("ID prefix doesn't match format {:?}", entry.format));
    }
    if entry.name.is_empty() {
        issues.push("Empty name".to_string());
    }
    if entry.description.is_empty() {
        issues.push("Empty description".to_string());
    }
    if entry.input.is_empty() {
        issues.push("Empty input".to_string());
    }
    if entry.expected_output.is_empty() {
        issues.push("Empty expected_output".to_string());
    }
    if entry.format == CorpusFormat::Bash && !entry.input.contains("fn main()") {
        issues.push("Bash entry missing fn main()".to_string());
    }
    issues
}

/// Validate all corpus entries for metadata correctness (spec §2.3).
pub(crate) fn corpus_validate(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};

    let registry = CorpusRegistry::load_full();
    let mut seen_ids = std::collections::HashSet::new();
    let mut all_issues: Vec<(String, String)> = Vec::new();

    for entry in &registry.entries {
        for issue in validate_corpus_entry(entry, &mut seen_ids) {
            all_issues.push((entry.id.clone(), issue));
        }
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!(
                "{BOLD}Corpus Validation: {} entries{RESET}",
                registry.entries.len()
            );
            let bash_count = registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Bash)
                .count();
            let make_count = registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Makefile)
                .count();
            let dock_count = registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Dockerfile)
                .count();
            println!("  {DIM}Bash: {bash_count}, Makefile: {make_count}, Dockerfile: {dock_count}{RESET}");
            println!();
            if all_issues.is_empty() {
                println!("  {GREEN}All entries valid — no issues found.{RESET}");
            } else {
                println!("  {BRIGHT_RED}{} issue(s) found:{RESET}", all_issues.len());
                for (id, msg) in &all_issues {
                    println!("    {BRIGHT_RED}{id}{RESET}: {msg}");
                }
            }
        }
        CorpusOutputFormat::Json => {
            let result = serde_json::json!({
                "total_entries": registry.entries.len(),
                "issues_count": all_issues.len(),
                "issues": all_issues.iter().map(|(id, msg)| serde_json::json!({
                    "id": id, "issue": msg
                })).collect::<Vec<_>>(),
            });
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}
