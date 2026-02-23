//! Corpus display: heatmap, dashboard, and search.

use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
use crate::models::{Config, Error, Result};
use super::corpus_failure_commands::result_fail_dims;
use std::path::PathBuf;

pub(crate) fn corpus_heatmap(limit: usize, filter: Option<&CorpusFormatArg>) -> Result<()> {
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

    // Sort: failures first (by # failing dims desc), then by ID
    let mut results: Vec<_> = score.results.iter().collect();
    results.sort_by(|a, b| {
        let a_fails = result_fail_dims(a).len();
        let b_fails = result_fail_dims(b).len();
        b_fails.cmp(&a_fails).then_with(|| a.id.cmp(&b.id))
    });
    let results: Vec<_> = results.into_iter().take(limit).collect();

    println!("{BOLD}Corpus Heatmap: Entry × Dimension (top {limit}){RESET}");
    println!();
    heatmap_print_header();
    for r in &results {
        heatmap_print_row(r);
    }
    println!();
    let total_fails = score
        .results
        .iter()
        .filter(|r| !result_fail_dims(r).is_empty())
        .count();
    println!(
        "  {DIM}Showing {}/{} entries ({} with failures){RESET}",
        results.len(),
        score.results.len(),
        total_fails
    );
    Ok(())
}


pub(crate) fn heatmap_print_header() {
    use crate::cli::color::*;
    println!(
        "  {BOLD}{:<8} {:>2} {:>3} {:>3} {:>3}  {:>2} {:>2} {:>2} {:>2}{RESET}",
        "ID", "A", "B1", "B2", "B3", "D", "E", "F", "G"
    );
}


pub(crate) fn heatmap_print_row(r: &crate::corpus::runner::CorpusResult) {
    use crate::cli::color::*;
    let g = |pass: bool| -> String {
        if pass {
            format!("{GREEN}\u{2713}{RESET}")
        } else {
            format!("{RED}\u{2717}{RESET}")
        }
    };
    println!(
        "  {:<8} {:>2} {:>3} {:>3} {:>3}  {:>2} {:>2} {:>2} {:>2}",
        r.id,
        g(r.transpiled),
        g(r.output_contains),
        g(r.output_exact),
        g(r.output_behavioral),
        g(r.lint_clean),
        g(r.deterministic),
        g(r.metamorphic_consistent),
        g(r.cross_shell_agree)
    );
}

/// Compact multi-corpus convergence dashboard (spec §11.10.5).
pub(crate) fn corpus_dashboard() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    // Score box
    let gc = grade_color(&format!("{}", score.grade));
    println!("{DIM}\u{256d}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{256e}{RESET}");
    println!("{DIM}\u{2502}{RESET}  {BOLD}Dashboard{RESET}: {BOLD}{WHITE}{:.1}/100{RESET} {gc}{}{RESET}  {DIM}({} entries){RESET}  {DIM}\u{2502}{RESET}",
        score.score, score.grade, score.results.len());
    println!("{DIM}\u{2570}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{256f}{RESET}");
    println!();

    // Per-format breakdown
    dashboard_print_formats(&score);

    // Failures summary
    let failures: Vec<_> = score
        .results
        .iter()
        .filter(|r| !result_fail_dims(r).is_empty())
        .collect();
    if failures.is_empty() {
        println!("  {GREEN}No failures.{RESET}");
    } else {
        println!("  {BOLD}Failures ({}):{RESET}", failures.len());
        for r in failures.iter().take(5) {
            let dims = result_fail_dims(r).join(", ");
            println!("    {RED}{:<8}{RESET} {DIM}\u{2192}{RESET} {dims}", r.id);
        }
        if failures.len() > 5 {
            println!("    {DIM}... and {} more{RESET}", failures.len() - 5);
        }
    }
    println!();

    // Recent convergence history (last 3)
    let log_path = PathBuf::from(".quality/convergence.log");
    if let Ok(entries) = CorpusRunner::load_convergence_log(&log_path) {
        if !entries.is_empty() {
            dashboard_print_history(&entries);
        }
    }
    Ok(())
}


pub(crate) fn dashboard_print_formats(score: &crate::corpus::runner::CorpusScore) {
    use crate::cli::color::*;
    println!("  {BOLD}Format Breakdown:{RESET}");
    for fs in &score.format_scores {
        let gc = grade_color(&format!("{}", fs.grade));
        let pc = pass_count(fs.passed, fs.total);
        println!(
            "    {CYAN}{:<12}{RESET} {BOLD}{WHITE}{:.1}/100{RESET} {gc}{}{RESET} \u{2014} {pc}",
            fs.format, fs.score, fs.grade
        );
    }
    println!();
}


pub(crate) fn dashboard_print_history(entries: &[crate::corpus::runner::ConvergenceEntry]) {
    use crate::cli::color::*;
    let recent: Vec<_> = entries.iter().rev().take(3).collect();
    println!("  {BOLD}Recent History (last {}):{RESET}", recent.len());
    for e in recent.iter().rev() {
        let sc = pct_color(e.score);
        let dc = delta_color(e.delta);
        println!(
            "    {DIM}#{:<3}{RESET} {}{:.1}/100{RESET} {dc} {DIM}{}{RESET}",
            e.iteration, sc, e.score, e.notes
        );
    }
    println!();
}

/// Display search results in human-readable format
fn display_search_human(
    matches: &[&crate::corpus::registry::CorpusEntry],
    pattern: &str,
) {
    use crate::cli::color::*;
    if matches.is_empty() {
        println!("No entries matching \"{pattern}\".");
        return;
    }
    println!(
        "{BOLD}Search results for \"{pattern}\" ({} matches):{RESET}",
        matches.len()
    );
    println!();
    for e in matches {
        let fmt = format!("{}", e.format);
        println!(
            "  {CYAN}{:<8}{RESET} {DIM}[{:<10}]{RESET} {BOLD}{}{RESET}",
            e.id, fmt, e.name
        );
        if !e.description.is_empty() {
            let desc = if e.description.len() > 72 {
                format!("{}...", &e.description[..69])
            } else {
                e.description.clone()
            };
            println!("           {DIM}{desc}{RESET}");
        }
    }
}

/// Search corpus entries by ID, name, or description pattern.
pub(crate) fn corpus_search(
    pattern: &str,
    format: &CorpusOutputFormat,
    filter: Option<&CorpusFormatArg>,
) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};

    let registry = CorpusRegistry::load_full();
    let pat = pattern.to_lowercase();

    let matches: Vec<_> = registry
        .entries
        .iter()
        .filter(|e| {
            let format_match = match filter {
                Some(CorpusFormatArg::Bash) => e.format == CorpusFormat::Bash,
                Some(CorpusFormatArg::Makefile) => e.format == CorpusFormat::Makefile,
                Some(CorpusFormatArg::Dockerfile) => e.format == CorpusFormat::Dockerfile,
                None => true,
            };
            format_match
                && (e.id.to_lowercase().contains(&pat)
                    || e.name.to_lowercase().contains(&pat)
                    || e.description.to_lowercase().contains(&pat))
        })
        .collect();

    match format {
        CorpusOutputFormat::Human => display_search_human(&matches, pattern),
        CorpusOutputFormat::Json => {
            let results: Vec<_> = matches
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "name": e.name,
                        "description": e.description,
                        "format": format!("{}", e.format),
                        "tier": format!("{:?}", e.tier),
                    })
                })
                .collect();
            let json = serde_json::to_string_pretty(&serde_json::json!({
                "pattern": pattern,
                "count": matches.len(),
                "results": results,
            }))
            .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}
