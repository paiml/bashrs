//! Corpus reporting: show entry, export, history display.

use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
use crate::models::{Config, Error, Result};
use crate::cli::logic::truncate_str;
use std::path::PathBuf;
use super::corpus_diff_commands::chrono_free_date;

pub(crate) fn corpus_show_entry(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Corpus entry '{id}' not found")))?;

    let runner = CorpusRunner::new(Config::default());
    let result = runner.run_single(entry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            println!(
                "{WHITE}Entry:{RESET} {CYAN}{}{RESET} ({})",
                entry.id, entry.name
            );
            println!(
                "{DIM}Format: {} | Tier: {:?}{RESET}",
                entry.format, entry.tier
            );
            println!("{DIM}Description: {}{RESET}", entry.description);
            println!();
            let s = result.score();
            let gc = grade_color(if s >= 90.0 {
                "A"
            } else if s >= 70.0 {
                "B"
            } else {
                "D"
            });
            println!("Score: {gc}{:.1}/100{RESET}", s);
            println!();
            let check = |b: bool| -> String { pass_fail(b) };
            println!(
                "  {WHITE}A  Transpilation{RESET} (30):  {}",
                check(result.transpiled)
            );
            println!(
                "  {WHITE}B1 Containment{RESET}  (10):  {}",
                check(result.output_contains)
            );
            println!(
                "  {WHITE}B2 Exact match{RESET}  ( 8):  {}",
                check(result.output_exact)
            );
            println!(
                "  {WHITE}B3 Behavioral{RESET}   ( 7):  {}",
                check(result.output_behavioral)
            );
            let cc = pct_color(result.coverage_ratio * 100.0);
            println!(
                "  {WHITE}C  Coverage{RESET}     (15):  {cc}{:.1}%{RESET}",
                result.coverage_ratio * 100.0
            );
            println!(
                "  {WHITE}D  Lint{RESET}         (10):  {}",
                check(result.lint_clean)
            );
            println!(
                "  {WHITE}E  Determinism{RESET}  (10):  {}",
                check(result.deterministic)
            );
            println!(
                "  {WHITE}F  Metamorphic{RESET}  ( 5):  {}",
                check(result.metamorphic_consistent)
            );
            println!(
                "  {WHITE}G  Cross-shell{RESET}  ( 5):  {}",
                check(result.cross_shell_agree)
            );
            println!("  Schema valid:          {}", check(result.schema_valid));
            if let Some(ref output) = result.actual_output {
                println!();
                println!("{DIM}Output:{RESET}");
                println!("{DIM}{}{RESET}", truncate_str(output, 500));
            }
            if let Some(ref err) = result.error {
                println!();
                println!("{BRIGHT_RED}Error:{RESET} {err}");
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Export per-entry corpus results as structured JSON (spec §10.3).

pub(crate) fn corpus_export(output: Option<&str>, filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let config = Config::default();
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(config);

    let score = match filter {
        Some(CorpusFormatArg::Bash) => runner.run_format(&registry, CorpusFormat::Bash),
        Some(CorpusFormatArg::Makefile) => runner.run_format(&registry, CorpusFormat::Makefile),
        Some(CorpusFormatArg::Dockerfile) => runner.run_format(&registry, CorpusFormat::Dockerfile),
        None => runner.run(&registry),
    };

    // Build export entries by joining registry metadata with results
    let results_map: std::collections::HashMap<&str, &crate::corpus::runner::CorpusResult> =
        score.results.iter().map(|r| (r.id.as_str(), r)).collect();

    #[derive(serde::Serialize)]
    struct ExportEntry<'a> {
        id: &'a str,
        name: &'a str,
        format: &'a CorpusFormat,
        tier: &'a crate::corpus::registry::CorpusTier,
        transpiled: bool,
        score: f64,
        grade: String,
        actual_output: &'a Option<String>,
        error: &'a Option<String>,
        lint_clean: bool,
        deterministic: bool,
        behavioral: bool,
        cross_shell: bool,
    }

    let entries: Vec<ExportEntry<'_>> = registry
        .entries
        .iter()
        .filter_map(|e| {
            let r = results_map.get(e.id.as_str())?;
            Some(ExportEntry {
                id: &e.id,
                name: &e.name,
                format: &e.format,
                tier: &e.tier,
                transpiled: r.transpiled,
                score: r.score(),
                grade: crate::corpus::registry::Grade::from_score(r.score()).to_string(),
                actual_output: &r.actual_output,
                error: &r.error,
                lint_clean: r.lint_clean,
                deterministic: r.deterministic,
                behavioral: r.output_behavioral,
                cross_shell: r.cross_shell_agree,
            })
        })
        .collect();

    #[derive(serde::Serialize)]
    struct ExportDocument<'a> {
        bashrs_version: &'a str,
        date: String,
        total: usize,
        aggregate_score: f64,
        aggregate_grade: String,
        entries: Vec<ExportEntry<'a>>,
    }

    let doc = ExportDocument {
        bashrs_version: env!("CARGO_PKG_VERSION"),
        date: chrono_free_date(),
        total: entries.len(),
        aggregate_score: score.score,
        aggregate_grade: score.grade.to_string(),
        entries,
    };

    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;

    match output {
        Some(path) => {
            std::fs::write(path, &json)
                .map_err(|e| Error::Internal(format!("Failed to write {path}: {e}")))?;
            eprintln!("Exported {} entries to {path}", doc.total);
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// Format a per-format pass/total column (e.g. "499/500" or "-" if no data).

pub(crate) fn fmt_pass_total(passed: usize, total: usize) -> String {
    if total > 0 {
        format!("{passed}/{total}")
    } else {
        "-".to_string()
    }
}

/// Compute a trend arrow by comparing two values.

pub(crate) fn trend_arrow(current: usize, previous: usize) -> &'static str {
    if current > previous {
        "↑"
    } else if current < previous {
        "↓"
    } else {
        "→"
    }
}

/// Print a single convergence history row (human-readable).

pub(crate) fn corpus_print_history_row(
    e: &crate::corpus::runner::ConvergenceEntry,
    prev: Option<&crate::corpus::runner::ConvergenceEntry>,
    has_format_data: bool,
    has_score_data: bool,
) {
    use crate::cli::color::*;
    let rate_pct = e.rate * 100.0;
    let rc = pct_color(rate_pct);
    let dc = delta_color(e.delta);
    let score_part = if has_score_data {
        let sc = pct_color(e.score);
        let gr = if e.grade.is_empty() {
            "-".to_string()
        } else {
            e.grade.clone()
        };
        format!("  {sc}{:>5.1}{RESET} {:>2}", e.score, gr)
    } else {
        String::new()
    };
    if has_format_data {
        let trend = match prev {
            Some(p) => format!(
                "{}{}{}",
                trend_arrow(e.bash_passed, p.bash_passed),
                trend_arrow(e.makefile_passed, p.makefile_passed),
                trend_arrow(e.dockerfile_passed, p.dockerfile_passed)
            ),
            None => "---".to_string(),
        };
        println!(
            "{:>4}  {:>10}  {:>5}/{:<5}  {rc}{:>5.1}%{RESET}  {dc}{score_part}  {:>9} {:>9} {:>9}  {DIM}{trend}{RESET}  {}",
            e.iteration, e.date, e.passed, e.total, rate_pct,
            fmt_pass_total(e.bash_passed, e.bash_total),
            fmt_pass_total(e.makefile_passed, e.makefile_total),
            fmt_pass_total(e.dockerfile_passed, e.dockerfile_total),
            e.notes
        );
    } else {
        println!(
            "{:>4}  {:>10}  {:>5}/{:<5}  {rc}{:>5.1}%{RESET}  {dc}{score_part}  {}",
            e.iteration, e.date, e.passed, e.total, rate_pct, e.notes
        );
    }
}


pub(crate) fn corpus_show_history(format: &CorpusOutputFormat, last: Option<usize>) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;

    if entries.is_empty() {
        println!("No convergence history. Run `bashrs corpus run --log` to create entries.");
        return Ok(());
    }

    let display: &[_] = match last {
        Some(n) if n < entries.len() => &entries[entries.len() - n..],
        _ => &entries,
    };

    // Detect if any entry has per-format data (spec §11.10.5)
    let has_format_data = display
        .iter()
        .any(|e| e.bash_total > 0 || e.makefile_total > 0 || e.dockerfile_total > 0);
    // Detect if any entry has V2 score data (spec §5.1)
    let has_score_data = display.iter().any(|e| e.score > 0.0);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!(
                "{BOLD}Convergence History ({} entries):{RESET}",
                entries.len()
            );
            let score_hdr = if has_score_data { "  Score Gr" } else { "" };
            if has_format_data {
                println!(
                    "{DIM}{:>4}  {:>10}  {:>5}/{:<5}  {:>6}  {:>8}{score_hdr}  {:>9} {:>9} {:>9}  {:<5}Notes{RESET}",
                    "Iter", "Date", "Pass", "Total", "Rate", "Delta",
                    "Bash", "Make", "Docker", "Trend"
                );
            } else {
                println!(
                    "{DIM}{:>4}  {:>10}  {:>5}/{:<5}  {:>6}  {:>8}{score_hdr}  Notes{RESET}",
                    "Iter", "Date", "Pass", "Total", "Rate", "Delta"
                );
            }
            for (i, e) in display.iter().enumerate() {
                let prev = if i > 0 { Some(&display[i - 1]) } else { None };
                corpus_print_history_row(e, prev, has_format_data, has_score_data);
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(display)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}


pub(crate) fn corpus_show_failures(
    format: &CorpusOutputFormat,
    filter: Option<&CorpusFormatArg>,
    dimension: Option<&str>,
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

    let failures: Vec<_> = score
        .results
        .iter()
        .filter(|r| {
            let has_any_failure = !r.transpiled
                || !r.output_contains
                || !r.output_exact
                || !r.output_behavioral
                || !r.lint_clean
                || !r.deterministic
                || !r.metamorphic_consistent
                || !r.cross_shell_agree
                || !r.schema_valid;
            if !has_any_failure {
                return false;
            }
            match dimension {
                Some("a") => !r.transpiled,
                Some("b1") => !r.output_contains,
                Some("b2") => !r.output_exact,
                Some("b3") => !r.output_behavioral,
                Some("d") => !r.lint_clean,
                Some("e") => !r.deterministic,
                Some("f") => !r.metamorphic_consistent,
                Some("g") => !r.cross_shell_agree,
                Some("schema") => !r.schema_valid,
                _ => true,
            }
        })
        .collect();

    corpus_print_failures(&failures, format)
}


pub(crate) fn corpus_print_failures(
    failures: &[&crate::corpus::runner::CorpusResult],
    format: &CorpusOutputFormat,
) -> Result<()> {
    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            if failures.is_empty() {
                println!("{GREEN}No failures found.{RESET}");
                return Ok(());
            }
            println!("{BRIGHT_RED}Failures ({} entries):{RESET}", failures.len());
            println!("{DIM}{:<8} {:>6}  Failing Dimensions{RESET}", "ID", "Score");
            for r in failures {
                let dims = corpus_failing_dims(r);
                let sc = r.score();
                let gc = grade_color(if sc >= 90.0 {
                    "A"
                } else if sc >= 70.0 {
                    "B"
                } else {
                    "D"
                });
                println!(
                    "{CYAN}{:<8}{RESET} {gc}{:>5.1}{RESET}  {RED}{}{RESET}",
                    r.id, sc, dims
                );
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(failures)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}


pub(crate) fn corpus_failing_dims(r: &crate::corpus::runner::CorpusResult) -> String {
    let mut dims = Vec::new();
    if !r.transpiled {
        dims.push("A");
    }
    if !r.output_contains {
        dims.push("B1");
    }
    if !r.output_exact {
        dims.push("B2");
    }
    if !r.output_behavioral {
        dims.push("B3");
    }
    if !r.lint_clean {
        dims.push("D");
    }
    if !r.deterministic {
        dims.push("E");
    }
    if !r.metamorphic_consistent {
        dims.push("F");
    }
    if !r.cross_shell_agree {
        dims.push("G");
    }
    if !r.schema_valid {
        dims.push("Schema");
    }
    dims.join(", ")
}
