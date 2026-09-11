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
    use crate::corpus::registry::CorpusRegistry;
    corpus_decisions_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_decisions`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_decisions_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;
    use std::collections::HashMap;

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
    use crate::corpus::registry::CorpusRegistry;
    corpus_patterns_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_patterns`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_patterns_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::pattern_store::mine_patterns;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let store = mine_patterns(registry, &runner);

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
    use crate::corpus::registry::CorpusRegistry;
    corpus_pattern_query_with(&CorpusRegistry::load_full(), signal)
}

/// PMAT-257: body of `corpus_pattern_query`, split so a test can pass a
/// small synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_pattern_query_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    signal: &str,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::pattern_store::mine_patterns;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let store = mine_patterns(registry, &runner);

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
    use crate::corpus::registry::CorpusRegistry;
    corpus_fix_suggest_with(&CorpusRegistry::load_full(), id)
}

/// PMAT-257: body of `corpus_fix_suggest`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_fix_suggest_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    id: &str,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::pattern_store::{classify_failure_signals, mine_patterns, suggest_fixes};
    use crate::corpus::runner::CorpusRunner;

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

    let store = mine_patterns(registry, &runner);
    let suggestions = suggest_fixes(id, registry, &runner, &store);

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

// PMAT-257: coverage for the corpus decision-analysis handlers. `corpus_decisions`,
// `corpus_patterns`, `corpus_pattern_query`, and `corpus_fix_suggest` each build a
// `CorpusRunner` and score entries out of the real `CorpusRegistry::load_full()`
// (18,000+ entries) -- too slow for a unit test. Each was already split into a
// `*_with(registry, ...)` twin that takes the registry as a parameter, matching
// `corpus_compare_commands.rs`. `score_impact_color` and `accumulate_decision_stats`
// never touch the registry or a runner, so they're covered directly.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};

    fn tiny_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "hello-bash",
            "PMAT-257 fixture",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            r#"fn main() { let greeting = "hello"; }"#,
            "greeting='hello'",
        ));
        registry.add(CorpusEntry::new(
            "M-001",
            "hello-makefile",
            "PMAT-257 fixture",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "all:\n\techo hello\n",
            "all:",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "hello-dockerfile",
            "PMAT-257 fixture",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18\nWORKDIR /app\n",
            "FROM alpine:3.18",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_score_impact_color_high() {
        let (label, color) = score_impact_color(0.9);
        assert!(label.contains("HIGH"));
        assert_eq!(color, crate::cli::color::RED);
    }

    #[test]
    fn test_PMAT257_cov_score_impact_color_medium() {
        let (label, color) = score_impact_color(0.6);
        assert!(label.contains("MEDIUM"));
        assert_eq!(color, crate::cli::color::YELLOW);
    }

    #[test]
    fn test_PMAT257_cov_score_impact_color_low() {
        let (label, color) = score_impact_color(0.1);
        assert!(label.contains("LOW"));
        assert_eq!(color, crate::cli::color::DIM);
    }

    #[test]
    fn test_PMAT257_cov_accumulate_decision_stats_no_trace() {
        let result = crate::corpus::runner::CorpusResult {
            decision_trace: None,
            ..Default::default()
        };
        let mut stats = std::collections::HashMap::new();
        assert!(!accumulate_decision_stats(&result, &mut stats));
        assert!(stats.is_empty());
    }

    #[test]
    fn test_PMAT257_cov_accumulate_decision_stats_with_trace() {
        use crate::emitter::trace::TranspilerDecision;

        let trace = vec![TranspilerDecision {
            decision_type: "ir_dispatch".to_string(),
            choice: "Let".to_string(),
            ir_node: "Let".to_string(),
        }];

        let passing = crate::corpus::runner::CorpusResult {
            transpiled: true,
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            decision_trace: Some(trace.clone()),
            ..Default::default()
        };
        let mut stats = std::collections::HashMap::new();
        assert!(accumulate_decision_stats(&passing, &mut stats));
        let entry = stats.get("ir_dispatch:Let").expect("stat recorded");
        assert_eq!(*entry, (1, 1, 0));

        let failing = crate::corpus::runner::CorpusResult {
            transpiled: true,
            output_contains: false,
            decision_trace: Some(trace),
            ..Default::default()
        };
        assert!(accumulate_decision_stats(&failing, &mut stats));
        let entry = stats.get("ir_dispatch:Let").expect("stat recorded");
        assert_eq!(*entry, (2, 1, 1));
    }

    #[test]
    fn test_PMAT257_cov_decisions_with_tiny_registry() {
        corpus_decisions_with(&tiny_registry()).expect("decisions run over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_patterns_with_tiny_registry() {
        corpus_patterns_with(&tiny_registry()).expect("patterns run over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_pattern_query_with_no_match() {
        corpus_pattern_query_with(&tiny_registry(), "PMAT257_no_such_signal")
            .expect("pattern query runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_fix_suggest_with_known_id() {
        corpus_fix_suggest_with(&tiny_registry(), "B-001")
            .expect("fix suggest runs for a known entry");
    }

    #[test]
    fn test_PMAT257_cov_fix_suggest_with_unknown_id() {
        let err = corpus_fix_suggest_with(&tiny_registry(), "NOPE-999");
        assert!(err.is_err());
    }
}
