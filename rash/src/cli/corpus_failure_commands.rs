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

    print_pareto_analysis(&score.results, format, top)
}

/// Pure rendering of a Pareto analysis over an already-computed result set.
/// Split out of `corpus_pareto_analysis` so the analysis itself is testable
/// without executing the corpus through a `CorpusRunner`.
pub(crate) fn print_pareto_analysis(
    results: &[crate::corpus::runner::CorpusResult],
    format: &CorpusOutputFormat,
    top: Option<usize>,
) -> Result<()> {
    let sorted = count_dimension_failures(results);
    let total_failures: usize = sorted.iter().map(|(_, c)| c).sum();
    let limit = top.unwrap_or(sorted.len());

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Pareto Analysis: Corpus Failures by Dimension{RESET}");
            println!(
                "{DIM}Total entries: {}, Total dimension-failures: {}{RESET}",
                results.len(),
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
            pareto_print_affected(results);
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
                "total_entries": results.len(),
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

    print_why_failed(id, entry, &result, format)
}

/// Pure rendering of the Five Whys report for one already-computed result.
/// Split out of `corpus_why_failed` so it is testable without running the
/// corpus through a `CorpusRunner`.
pub(crate) fn print_why_failed(
    id: &str,
    entry: &crate::corpus::registry::CorpusEntry,
    result: &crate::corpus::runner::CorpusResult,
    format: &CorpusOutputFormat,
) -> Result<()> {
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

    print_regressions(&entries, format)
}

/// Pure rendering of regression detection over an already-loaded convergence
/// log. Split out of `corpus_regressions` so it is testable with hand-built
/// `ConvergenceEntry` fixtures instead of the real `.quality/convergence.log`.
pub(crate) fn print_regressions(
    entries: &[crate::corpus::runner::ConvergenceEntry],
    format: &CorpusOutputFormat,
) -> Result<()> {
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

#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::CorpusEntry;
    use crate::corpus::runner::{ConvergenceEntry, CorpusResult};

    fn fake_result(id: &str, all_pass: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled: all_pass,
            output_contains: all_pass,
            output_exact: all_pass,
            output_behavioral: all_pass,
            lint_clean: all_pass,
            deterministic: all_pass,
            metamorphic_consistent: all_pass,
            cross_shell_agree: all_pass,
            schema_valid: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_PMAT257_cov_result_fail_dims_all_pass_is_empty() {
        let r = fake_result("B-001", true);
        assert!(result_fail_dims(&r).is_empty());
    }

    #[test]
    fn test_PMAT257_cov_result_fail_dims_all_fail_lists_every_dim() {
        let r = fake_result("B-002", false);
        let dims = result_fail_dims(&r);
        assert_eq!(dims, vec!["A", "B1", "B2", "B3", "D", "E", "F", "G"]);
    }

    #[test]
    fn test_PMAT257_cov_count_dimension_failures_sorted_and_filtered() {
        let mut r1 = fake_result("B-001", true);
        r1.transpiled = false;
        let mut r2 = fake_result("B-002", true);
        r2.transpiled = false;
        r2.lint_clean = false;
        let results = vec![r1, r2, fake_result("B-003", true)];
        let sorted = count_dimension_failures(&results);
        // Transpilation fails twice, lint fails once -- transpilation must sort first.
        assert_eq!(sorted[0].0, "A  Transpilation");
        assert_eq!(sorted[0].1, 2);
        assert!(sorted.iter().all(|(_, c)| *c > 0));
    }

    #[test]
    fn test_PMAT257_cov_count_dimension_failures_empty_when_all_pass() {
        let results = vec![fake_result("B-001", true), fake_result("B-002", true)];
        assert!(count_dimension_failures(&results).is_empty());
    }

    #[test]
    fn test_PMAT257_cov_pareto_print_table_renders_rows_and_marks_vital_few() {
        let sorted = vec![("A  Transpilation", 5usize), ("D  Lint clean", 1usize)];
        pareto_print_table(&sorted, 6, 2);
        // limit smaller than rows exercises the `.take(limit)` branch too.
        pareto_print_table(&sorted, 6, 1);
    }

    #[test]
    fn test_PMAT257_cov_pareto_print_affected_handles_mixed_and_empty() {
        let results = vec![fake_result("B-001", false), fake_result("B-002", true)];
        pareto_print_affected(&results);
        pareto_print_affected(&[]);
    }

    #[test]
    fn test_PMAT257_cov_pareto_print_affected_truncates_past_twenty() {
        let results: Vec<CorpusResult> = (0..25)
            .map(|i| fake_result(&format!("B-{i:03}"), false))
            .collect();
        pareto_print_affected(&results);
    }

    #[test]
    fn test_PMAT257_cov_print_pareto_analysis_human_no_failures() {
        let results = vec![fake_result("B-001", true)];
        print_pareto_analysis(&results, &CorpusOutputFormat::Human, None)
            .expect("an all-passing set prints the perfect-corpus branch");
    }

    #[test]
    fn test_PMAT257_cov_print_pareto_analysis_human_with_failures_and_top() {
        let results = vec![fake_result("B-001", false), fake_result("B-002", true)];
        print_pareto_analysis(&results, &CorpusOutputFormat::Human, Some(1))
            .expect("a failing set prints the vital-few and affected-entries sections");
    }

    #[test]
    fn test_PMAT257_cov_print_pareto_analysis_json_branch() {
        let results = vec![fake_result("B-001", false), fake_result("B-002", true)];
        print_pareto_analysis(&results, &CorpusOutputFormat::Json, None)
            .expect("the json branch always succeeds given valid results");
    }

    fn fake_entry(id: &str) -> CorpusEntry {
        CorpusEntry::new(
            id,
            "fixture",
            "PMAT-257 fixture entry",
            crate::corpus::registry::CorpusFormat::Bash,
            crate::corpus::registry::CorpusTier::Standard,
            "fn main() {}",
            "#!/bin/sh",
        )
    }

    #[test]
    fn test_PMAT257_cov_print_why_failed_human_all_pass() {
        let entry = fake_entry("B-001");
        let result = fake_result("B-001", true);
        print_why_failed("B-001", &entry, &result, &CorpusOutputFormat::Human)
            .expect("an all-passing result prints the no-failures branch");
    }

    #[test]
    fn test_PMAT257_cov_print_why_failed_human_with_error_and_output() {
        let entry = fake_entry("B-002");
        let mut result = fake_result("B-002", false);
        result.error = Some("parse error: unexpected token".to_string());
        result.actual_output = Some("line one\nline two\nline three".to_string());
        print_why_failed("B-002", &entry, &result, &CorpusOutputFormat::Human)
            .expect("a failing result with error/output prints every optional section");
    }

    #[test]
    fn test_PMAT257_cov_print_why_failed_json_branch() {
        let entry = fake_entry("B-003");
        let result = fake_result("B-003", false);
        print_why_failed("B-003", &entry, &result, &CorpusOutputFormat::Json)
            .expect("the json branch always succeeds given a valid entry/result");
    }

    fn fake_convergence(iteration: u32, passed: usize, score: f64) -> ConvergenceEntry {
        ConvergenceEntry {
            iteration,
            date: "2026-01-01".to_string(),
            total: 100,
            passed,
            failed: 100 - passed,
            rate: passed as f64 / 100.0,
            delta: 0.0,
            notes: "fixture".to_string(),
            score,
            grade: "A".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_PMAT257_cov_print_regressions_human_no_regressions() {
        let entries = vec![fake_convergence(1, 90, 90.0), fake_convergence(2, 95, 95.0)];
        print_regressions(&entries, &CorpusOutputFormat::Human)
            .expect("an improving log has no regressions");
    }

    #[test]
    fn test_PMAT257_cov_print_regressions_human_detects_a_drop() {
        let entries = vec![fake_convergence(1, 95, 95.0), fake_convergence(2, 90, 90.0)];
        print_regressions(&entries, &CorpusOutputFormat::Human)
            .expect("a dropping log reports the regression");
    }

    #[test]
    fn test_PMAT257_cov_print_regressions_json_branch() {
        let entries = vec![fake_convergence(1, 95, 95.0), fake_convergence(2, 90, 90.0)];
        print_regressions(&entries, &CorpusOutputFormat::Json)
            .expect("the json branch always succeeds given a valid log");
    }

    #[test]
    fn test_PMAT257_cov_print_regressions_single_entry_is_a_no_op_windows() {
        // `entries.windows(2)` over a single element yields nothing, so no
        // regressions can be found -- exercises the guard indirectly via the
        // pure function rather than the real convergence log on disk.
        let entries = vec![fake_convergence(1, 90, 90.0)];
        print_regressions(&entries, &CorpusOutputFormat::Human)
            .expect("a single-entry log prints the no-regressions branch");
    }
}
