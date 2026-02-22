//! Corpus decision analysis: score impact, decision statistics, patterns, pattern queries, and fix suggestions.

use crate::models::{Config, Error, Result};

pub(crate) fn score_impact_color(score: f64) -> (String, &'static str) {
    use crate::cli::color::*;
    if score >= 0.8 {
        (format!("{RED}HIGH{RESET}"), RED)
    } else if score >= 0.5 {
        (format!("{YELLOW}MEDIUM{RESET}"), YELLOW)
    } else {
        (format!("{DIM}LOW{RESET}"), DIM)
    }
}

/// Accumulate per-decision pass/fail stats from a trace result.

pub(crate) fn accumulate_decision_stats(
    result: &crate::corpus::runner::CorpusResult,
    stats: &mut std::collections::HashMap<String, (usize, usize, usize)>,
) -> bool {
    let passed = result.transpiled
        && result.output_contains
        && result.schema_valid
        && result.lint_clean
        && result.deterministic;

    let trace = match &result.decision_trace {
        Some(t) => t,
        None => return false,
    };

    for d in trace {
        let key = format!("{}:{}", d.decision_type, d.choice);
        let entry = stats.entry(key).or_insert((0, 0, 0));
        entry.0 += 1;
        if passed {
            entry.1 += 1;
        } else {
            entry.2 += 1;
        }
    }

    !trace.is_empty()
}

/// Decision frequency and pass/fail correlation summary.

pub(crate) fn corpus_decisions() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::collections::HashMap;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let mut stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
    let mut total_entries = 0usize;
    let mut traced_entries = 0usize;

    for entry in &registry.entries {
        let result = runner.run_entry_with_trace(entry);
        total_entries += 1;
        if accumulate_decision_stats(&result, &mut stats) {
            traced_entries += 1;
        }
    }

    let mut sorted: Vec<_> = stats.into_iter().collect();
    sorted.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    println!(
        "\n  {BOLD}Decision Frequency Summary{RESET}  ({traced_entries}/{total_entries} entries traced)"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(78));
    println!(
        "  {DIM}{:<36}  {:>8}  {:>10}  {:>10}  {:>8}{RESET}",
        "Decision", "Count", "In Pass", "In Fail", "Fail %"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(78));

    for (key, (total, in_pass, in_fail)) in &sorted {
        let fail_pct = if *total > 0 {
            (*in_fail as f64 / *total as f64) * 100.0
        } else {
            0.0
        };
        let color = if fail_pct >= 50.0 {
            RED
        } else if fail_pct >= 20.0 {
            YELLOW
        } else {
            ""
        };
        let end = if color.is_empty() { "" } else { RESET };
        println!(
            "  {color}{:<36}  {:>8}  {:>10}  {:>10}  {:>7.1}%{end}",
            key, total, in_pass, in_fail, fail_pct
        );
    }

    println!("\n  {DIM}Total unique decisions: {}{RESET}", sorted.len());
    println!();
    Ok(())
}

/// Mine and display CITL fix patterns (§11.10.2).

pub(crate) fn corpus_patterns() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::pattern_store::mine_patterns;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let store = mine_patterns(&registry, &runner);

    println!(
        "\n  {BOLD}CITL Pattern Store{RESET}  ({} traced, {} failures)",
        store.total_entries, store.total_failures
    );
    println!("  {DIM}{}{RESET}", "─".repeat(76));

    if store.patterns.is_empty() {
        println!("  {BRIGHT_GREEN}No failure patterns — all entries pass{RESET}");
        println!();
        return Ok(());
    }

    println!(
        "  {DIM}{:<22}  {:<30}  {:>10}  {:<12}{RESET}",
        "Signal", "Decision", "Confidence", "Evidence"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(76));

    for pattern in &store.patterns {
        let (_, color) = score_impact_color(pattern.confidence);
        let evidence = if pattern.evidence_ids.len() <= 3 {
            pattern.evidence_ids.join(", ")
        } else {
            format!(
                "{}, ... +{}",
                pattern.evidence_ids[..2].join(", "),
                pattern.evidence_ids.len() - 2
            )
        };
        println!(
            "  {:<22}  {color}{:<30}{RESET}  {:>10.4}  {DIM}{:<12}{RESET}",
            pattern.error_signal, pattern.causal_decision, pattern.confidence, evidence
        );
    }

    println!(
        "\n  {DIM}Total patterns: {} (version {}){RESET}",
        store.patterns.len(),
        store.version
    );
    println!();
    Ok(())
}

/// Query CITL patterns for a specific error signal (§11.10.2).

pub(crate) fn corpus_pattern_query(signal: &str) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::pattern_store::mine_patterns;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let store = mine_patterns(&registry, &runner);

    let matching: Vec<_> = store
        .patterns
        .iter()
        .filter(|p| p.error_signal == signal)
        .collect();

    println!("\n  {BOLD}Patterns for:{RESET} {CYAN}{signal}{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    if matching.is_empty() {
        println!("  {DIM}No patterns found for signal '{signal}'{RESET}");
        println!("  {DIM}Known signals: A_transpile_fail, B1_containment_fail, B2_exact_fail,");
        println!("  B3_behavioral_fail, D_lint_fail, G_cross_shell_fail{RESET}");
        println!();
        return Ok(());
    }

    println!(
        "  {DIM}{:<30}  {:>10}  {:<16}  {:<12}{RESET}",
        "Decision", "Confidence", "Fix Type", "Evidence"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for pattern in &matching {
        let (_, color) = score_impact_color(pattern.confidence);
        let evidence = pattern.evidence_ids.join(", ");
        println!(
            "  {color}{:<30}{RESET}  {:>10.4}  {:<16}  {DIM}{:<12}{RESET}",
            pattern.causal_decision, pattern.confidence, pattern.fix_type, evidence
        );
    }

    println!();
    Ok(())
}

/// Suggest fixes for a failing corpus entry (§11.10.2).

pub(crate) fn corpus_fix_suggest(id: &str) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::pattern_store::{classify_failure_signals, mine_patterns, suggest_fixes};
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    // Verify entry exists
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Corpus entry '{id}' not found")))?;

    // Get current result to show failure signals
    let result = runner.run_entry_with_trace(entry);
    let signals = classify_failure_signals(&result);

    if signals.is_empty() {
        println!("\n  {BRIGHT_GREEN}{id} passes all checks — no fixes needed{RESET}\n");
        return Ok(());
    }

    let signal_list = signals.join(", ");
    println!("\n  {BOLD}Fix Suggestions for {CYAN}{id}{RESET} ({BRIGHT_RED}{signal_list}{RESET})");
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    let store = mine_patterns(&registry, &runner);
    let suggestions = suggest_fixes(id, &registry, &runner, &store);

    if suggestions.is_empty() {
        println!("  {DIM}No pattern-based suggestions available for this entry{RESET}");
        println!("  {DIM}(decision trace may not match any known failure patterns){RESET}");
        println!();
        return Ok(());
    }

    println!(
        "  {DIM}{:<4}  {:<30}  {:<18}  {:>10}{RESET}",
        "#", "Decision", "Fix Type", "Confidence"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for (i, suggestion) in suggestions.iter().enumerate() {
        let (_, color) = score_impact_color(suggestion.confidence);
        println!(
            "  {WHITE}#{:<3}{RESET}  {color}{:<30}{RESET}  {:<18}  {:>10.4}",
            i + 1,
            suggestion.causal_decision,
            suggestion.fix_type,
            suggestion.confidence
        );
    }

    println!();
    Ok(())
}
