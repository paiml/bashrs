/// Decision trace for a single corpus entry.
pub(crate) fn corpus_trace(id: &str) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Corpus entry '{id}' not found")))?;

    let runner = CorpusRunner::new(Config::default());
    let result = runner.run_entry_with_trace(entry);

    let pass_fail = if result.transpiled {
        format!("{BRIGHT_GREEN}PASS{RESET}")
    } else {
        format!("{BRIGHT_RED}FAIL{RESET}")
    };

    println!("\n  {BOLD}Decision Trace for {CYAN}{id}{RESET} [{pass_fail}]");
    println!("  {DIM}{}{}", "─".repeat(60), RESET);

    match &result.decision_trace {
        Some(trace) if !trace.is_empty() => {
            println!(
                "  {DIM}{:<4}  {:<22}  {:<24}  {:<12}{RESET}",
                "#", "Decision Type", "Choice", "IR Node"
            );
            println!("  {DIM}{}{RESET}", "─".repeat(60));
            for (i, d) in trace.iter().enumerate() {
                println!(
                    "  {DIM}{:<4}{RESET}  {CYAN}{:<22}{RESET}  {WHITE}{:<24}{RESET}  {DIM}{:<12}{RESET}",
                    i + 1,
                    d.decision_type,
                    d.choice,
                    d.ir_node
                );
            }
            println!();
            println!("  {DIM}Total decisions: {}{RESET}", trace.len());
        }
        _ => {
            println!(
                "  {DIM}No decision trace available (non-Bash entry or transpilation failed){RESET}"
            );
            if let Some(err) = &result.error {
                println!("  {RED}Error: {err}{RESET}");
            }
        }
    }

    println!();
    Ok(())
}

/// Collect decision trace coverage data from all corpus entries.
pub(crate) fn collect_trace_coverage(
    registry: &crate::corpus::registry::CorpusRegistry,
    runner: &crate::corpus::runner::CorpusRunner,
) -> Vec<(String, bool, Vec<String>)> {
    registry
        .entries
        .iter()
        .filter_map(|entry| {
            let result = runner.run_entry_with_trace(entry);
            let passed = result.transpiled
                && result.output_contains
                && result.schema_valid
                && result.lint_clean
                && result.deterministic;
            let locations: Vec<String> = result
                .decision_trace
                .as_ref()
                .map(|t| {
                    t.iter()
                        .map(|d| format!("{}:{}", d.decision_type, d.choice))
                        .collect()
                })
                .unwrap_or_default();
            if locations.is_empty() {
                None
            } else {
                Some((entry.id.clone(), passed, locations))
            }
        })
        .collect()
}

/// Tarantula fault localization ranking across all corpus decisions.
pub(crate) fn corpus_suspicious(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::quality::sbfl::{localize_faults, SbflFormula};

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let coverage_data = collect_trace_coverage(&registry, &runner);

    let total = coverage_data.len();
    let passed = coverage_data.iter().filter(|(_, p, _)| *p).count();
    let failed = total - passed;

    if failed == 0 {
        println!(
            "\n  {BRIGHT_GREEN}All {total} traced entries pass — no suspicious decisions{RESET}\n"
        );
        return Ok(());
    }

    let rankings = localize_faults(&coverage_data, SbflFormula::Tarantula);

    println!(
        "\n  {BOLD}Tarantula Fault Localization{RESET}  ({total} entries: {BRIGHT_GREEN}{passed} pass{RESET}, {BRIGHT_RED}{failed} fail{RESET})"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));
    println!(
        "  {DIM}{:<36}  {:>14}  {:>8}  {:>8}{RESET}",
        "Decision", "Suspiciousness", "Impact", "Priority"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for ranking in rankings.iter().take(limit) {
        let (impact, color) = score_impact_color(ranking.score);
        println!(
            "  {:<36}  {color}{:>14.4}{RESET}  {:>8}  {DIM}{:>8}{RESET}",
            ranking.location,
            ranking.score,
            impact,
            format!("#{}", ranking.rank)
        );
    }

    if rankings.len() > limit {
        println!(
            "\n  {DIM}... and {} more (use --limit to show more){RESET}",
            rankings.len() - limit
        );
    }

    println!();
    Ok(())
}
