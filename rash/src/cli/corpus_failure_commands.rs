//! Corpus failure analysis: pareto, why-failed, and regression detection.

use super::corpus_entry_commands::truncate_line;
use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
use crate::models::{Config, Error, Result};
use std::path::PathBuf;

pub(crate) fn result_fail_dims(r: &crate::corpus::runner::CorpusResult) -> Vec<&'static str> {
    [
        (!r.transpiled, "A"),
        (!r.output_contains, "B1"),
        (!r.output_exact, "B2"),
        (!r.output_behavioral, "B3"),
        (!r.lint_clean, "D"),
        (!r.deterministic, "E"),
        (!r.metamorphic_consistent, "F"),
        (!r.cross_shell_agree, "G"),
    ]
    .iter()
    .filter_map(|(f, d)| if *f { Some(*d) } else { None })
    .collect()
}

/// Count failures per V2 dimension from corpus results.
pub(crate) fn count_dimension_failures(
    results: &[crate::corpus::runner::CorpusResult],
) -> Vec<(&'static str, usize)> {
    let dims = [
        (
            "A  Transpilation",
            results.iter().filter(|r| !r.transpiled).count(),
        ),
        (
            "B1 Containment",
            results.iter().filter(|r| !r.output_contains).count(),
        ),
        (
            "B2 Exact match",
            results.iter().filter(|r| !r.output_exact).count(),
        ),
        (
            "B3 Behavioral",
            results.iter().filter(|r| !r.output_behavioral).count(),
        ),
        (
            "D  Lint clean",
            results.iter().filter(|r| !r.lint_clean).count(),
        ),
        (
            "E  Deterministic",
            results.iter().filter(|r| !r.deterministic).count(),
        ),
        (
            "F  Metamorphic",
            results.iter().filter(|r| !r.metamorphic_consistent).count(),
        ),
        (
            "G  Cross-shell",
            results.iter().filter(|r| !r.cross_shell_agree).count(),
        ),
        ("Schema", results.iter().filter(|r| !r.schema_valid).count()),
    ];
    let mut sorted: Vec<_> = dims.into_iter().filter(|(_, c)| *c > 0).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

/// Print Pareto table rows with cumulative percentages.
pub(crate) fn pareto_print_table(sorted: &[(&str, usize)], total: usize, limit: usize) {
    use crate::cli::color::*;
    println!(
        "  {BOLD}{:<18} {:>5} {:>6} {:>6}  {:<20}{RESET}",
        "Dimension", "Count", "Pct", "Cum%", "Bar"
    );

    let mut cumulative = 0usize;
    for (i, (name, count)) in sorted.iter().take(limit).enumerate() {
        cumulative += count;
        let pct = *count as f64 / total as f64 * 100.0;
        let cum_pct = cumulative as f64 / total as f64 * 100.0;
        let bar_width = (pct / 100.0 * 16.0) as usize;
        let bar: String = "█".repeat(bar_width);
        let pad: String = "░".repeat(16 - bar_width);
        let color = if cum_pct <= 80.0 { BRIGHT_RED } else { YELLOW };
        let marker = if i == 0 { " ←vital few" } else { "" };
        println!(
            "  {color}{:<18} {:>5} {:>5.1}% {:>5.1}%  {bar}{pad}{RESET}{DIM}{marker}{RESET}",
            name, count, pct, cum_pct
        );
    }
}

/// Print affected entries summary (max 20).
pub(crate) fn pareto_print_affected(results: &[crate::corpus::runner::CorpusResult]) {
    use crate::cli::color::*;
    println!("  {BOLD}Affected entries:{RESET}");
    let mut shown = 0;
    let total_failing = results
        .iter()
        .filter(|r| !result_fail_dims(r).is_empty())
        .count();
    for r in results {
        let fails = result_fail_dims(r);
        if !fails.is_empty() {
            println!(
                "    {BRIGHT_RED}{:<8}{RESET} {DIM}fails:{RESET} {}",
                r.id,
                fails.join(", ")
            );
            shown += 1;
            if shown >= 20 && total_failing > shown {
                println!("    {DIM}... and {} more{RESET}", total_failing - shown);
                break;
            }
        }
    }
}

/// Pareto analysis: group failures by dimension, show 80/20 distribution (spec §11.10.4).
pub(crate) fn corpus_pareto_analysis(
    format: &CorpusOutputFormat,
    filter: Option<&CorpusFormatArg>,
    top: Option<usize>,
) -> Result<()> {
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

    let sorted = count_dimension_failures(&score.results);
    let total_failures: usize = sorted.iter().map(|(_, c)| c).sum();
    let limit = top.unwrap_or(sorted.len());

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Pareto Analysis: Corpus Failures by Dimension{RESET}");
            println!(
                "{DIM}Total entries: {}, Total dimension-failures: {}{RESET}",
                score.results.len(),
                total_failures
            );
            println!();

            if total_failures == 0 {
                println!("  {GREEN}No failures — perfect corpus!{RESET}");
                return Ok(());
            }

            pareto_print_table(&sorted, total_failures, limit);

            // Vital few insight
            println!();
            let vital_few: Vec<_> = sorted
                .iter()
                .scan(0usize, |acc, (name, count)| {
                    *acc += count;
                    Some((*name, *acc as f64 / total_failures as f64 * 100.0))
                })
                .take_while(|(_, cum)| *cum <= 80.0)
                .collect();
            if !vital_few.is_empty() {
                let names: Vec<_> = vital_few.iter().map(|(n, _)| n.trim()).collect();
                println!("  {BOLD}Vital few{RESET} (80/20): {}", names.join(", "));
                println!(
                    "  {DIM}Fix these {} dimension(s) to resolve ~80% of failures{RESET}",
                    names.len()
                );
            }

            println!();
            pareto_print_affected(&score.results);
        }
        CorpusOutputFormat::Json => {
            let json_dims: Vec<_> = sorted
                .iter()
                .take(limit)
                .scan(0usize, |acc, (name, count)| {
                    *acc += count;
                    Some(serde_json::json!({
                        "dimension": name.trim(),
                        "count": count,
                        "pct": *count as f64 / total_failures as f64 * 100.0,
                        "cumulative_pct": *acc as f64 / total_failures as f64 * 100.0,
                    }))
                })
                .collect();
            let result = serde_json::json!({
                "total_entries": score.results.len(),
                "total_failures": total_failures,
                "dimensions": json_dims,
            });
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Generate Five Whys root cause template for a failing corpus entry (spec §11.10.3).
pub(crate) fn corpus_why_failed(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Entry '{id}' not found")))?;

    let config = Config::default();
    let runner = CorpusRunner::new(config);
    let result = runner.run_single(entry);

    // Collect failing dimensions
    let failures: Vec<(&str, &str)> = [
        (
            !result.transpiled,
            (
                "A: Transpilation",
                "Parser/emitter cannot handle this construct",
            ),
        ),
        (
            !result.output_contains,
            ("B1: Containment", "Output missing expected content"),
        ),
        (
            !result.output_exact,
            ("B2: Exact match", "Output lines don't match expected"),
        ),
        (
            !result.output_behavioral,
            ("B3: Behavioral", "Shell execution fails or times out"),
        ),
        (
            !result.lint_clean,
            ("D: Lint clean", "Shellcheck/make -n reports errors"),
        ),
        (
            !result.deterministic,
            ("E: Deterministic", "Output varies between runs"),
        ),
        (
            !result.metamorphic_consistent,
            ("F: Metamorphic", "Metamorphic relation violated"),
        ),
        (
            !result.cross_shell_agree,
            ("G: Cross-shell", "sh and dash produce different output"),
        ),
    ]
    .iter()
    .filter_map(|(fail, info)| if *fail { Some(*info) } else { None })
    .collect();

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Five Whys: {id}{RESET}");
            println!("{DIM}Input:{RESET} {}", truncate_line(&entry.input, 70));
            println!();

            if failures.is_empty() {
                println!("  {GREEN}All dimensions pass — no failures to analyze.{RESET}");
                return Ok(());
            }

            println!("  {BRIGHT_RED}Failing dimensions:{RESET}");
            for (dim, hint) in &failures {
                println!("    {BRIGHT_RED}✗{RESET} {dim}: {DIM}{hint}{RESET}");
            }

            if let Some(err) = &result.error {
                println!();
                println!("  {BOLD}Error:{RESET} {}", truncate_line(err, 80));
            }

            if let Some(output) = &result.actual_output {
                println!();
                println!("  {BOLD}Actual output:{RESET}");
                for line in output.lines().take(5) {
                    println!("    {DIM}{}{RESET}", truncate_line(line, 70));
                }
            }

            // Five Whys template
            println!();
            println!("{BOLD}Root Cause Analysis (Five Whys){RESET}");
            println!("{DIM}Fill in each level to trace the root cause:{RESET}");
            println!();
            let primary = failures.first().map_or("Unknown", |(d, _)| *d);
            println!("  {BOLD}Why 1:{RESET} {id} fails dimension {primary}");
            println!("    → Because: ___");
            println!();
            println!("  {BOLD}Why 2:{RESET} Why does that happen?");
            println!("    → Because: ___");
            println!();
            println!("  {BOLD}Why 3:{RESET} Why does that happen?");
            println!("    → Because: ___");
            println!();
            println!("  {BOLD}Why 4:{RESET} Why does that happen?");
            println!("    → Because: ___");
            println!();
            println!("  {BOLD}Why 5:{RESET} Root cause");
            println!("    → Because: ___");
            println!();
            println!("  {BOLD}Countermeasure:{RESET} ___");
            println!("  {BOLD}Verification:{RESET} bashrs corpus check {id}");
        }
        CorpusOutputFormat::Json => {
            let result_json = serde_json::json!({
                "entry_id": id,
                "input": entry.input,
                "failures": failures.iter().map(|(d, h)| serde_json::json!({
                    "dimension": d,
                    "hint": h,
                })).collect::<Vec<_>>(),
                "error": result.error,
                "actual_output": result.actual_output,
                "five_whys": {
                    "why_1": format!("{id} fails dimension {}", failures.first().map_or("none", |(d, _)| *d)),
                    "why_2": "",
                    "why_3": "",
                    "why_4": "",
                    "why_5_root_cause": "",
                    "countermeasure": "",
                    "verification": format!("bashrs corpus check {id}"),
                },
            });
            let json = serde_json::to_string_pretty(&result_json)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Detect regressions between consecutive convergence log iterations (spec §5.3 Jidoka).
pub(crate) fn corpus_regressions(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;
    if entries.len() < 2 {
        println!("Need at least 2 convergence entries to detect regressions.");
        println!("Run `bashrs corpus run --log` multiple times first.");
        return Ok(());
    }

    let mut all_regressions = Vec::new();
    for pair in entries.windows(2) {
        let report = pair[1].detect_regressions(&pair[0]);
        if report.has_regressions() {
            all_regressions.push((pair[0].iteration, pair[1].iteration, report));
        }
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            if all_regressions.is_empty() {
                println!(
                    "{GREEN}No regressions detected across {} iterations.{RESET}",
                    entries.len()
                );
            } else {
                println!("{BOLD}Regressions Detected (Jidoka — spec §5.3){RESET}");
                println!();
                for (from, to, report) in &all_regressions {
                    println!("  {BRIGHT_RED}Iteration {from} → {to}:{RESET}");
                    for r in &report.regressions {
                        println!(
                            "    {RED}• {}{RESET}  ({} → {})",
                            r.message, r.previous, r.current
                        );
                    }
                }
                println!();
                println!(
                    "  {BRIGHT_RED}Total: {} regression(s) across {} transition(s){RESET}",
                    all_regressions
                        .iter()
                        .map(|(_, _, r)| r.regressions.len())
                        .sum::<usize>(),
                    all_regressions.len()
                );
            }
        }
        CorpusOutputFormat::Json => {
            let regressions: Vec<_> = all_regressions
                .iter()
                .map(|(from, to, report)| {
                    serde_json::json!({
                        "from_iteration": from,
                        "to_iteration": to,
                        "regressions": report.regressions.iter().map(|r| {
                            serde_json::json!({
                                "dimension": r.dimension,
                                "previous": r.previous,
                                "current": r.current,
                                "message": r.message,
                            })
                        }).collect::<Vec<_>>(),
                    })
                })
                .collect();
            let result = serde_json::json!({
                "iterations": entries.len(),
                "regression_count": all_regressions.iter().map(|(_, _, r)| r.regressions.len()).sum::<usize>(),
                "regressions": regressions,
            });
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}
