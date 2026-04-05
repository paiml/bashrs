//! CITL Pattern Store (§11.10.2)
//!
//! Closes the feedback loop between transpiler decisions and downstream validation
//! failures (shellcheck, sh execution, dash cross-shell). Maps specific error signals
//! (B3 timeout, D lint fail, G dash fail) to the emitter decisions that caused them.
//!
//! Creates an `error → decision → fix` knowledge base that can suggest fixes for
//! new failures automatically using Tarantula fault localization.

use serde::{Deserialize, Serialize};

/// A mined pattern linking an error signal to a causal decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellFixPattern {
    /// Error signal category (e.g. "B3_behavioral_fail", "D_lint_fail")
    pub error_signal: String,
    /// Causal decision from the emitter trace (e.g. "assignment_value:single_quote")
    pub causal_decision: String,
    /// Inferred fix category from the decision type
    pub fix_type: String,
    /// Tarantula suspiciousness score (0.0-1.0)
    pub confidence: f64,
    /// Entry IDs that provide evidence for this pattern
    pub evidence_ids: Vec<String>,
}

/// Collection of mined CITL fix patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStore {
    /// Mined fix patterns
    pub patterns: Vec<ShellFixPattern>,
    /// Total corpus entries analyzed
    pub total_entries: usize,
    /// Total failing entries
    pub total_failures: usize,
    /// Pattern store version
    pub version: String,
}

/// Classify a corpus result into its primary failure signal(s).
pub fn classify_failure_signals(result: &super::runner::CorpusResult) -> Vec<String> {
    let mut signals = Vec::new();
    if !result.transpiled {
        signals.push("A_transpile_fail".to_string());
        return signals; // A gate failure means nothing else is meaningful
    }
    if !result.output_contains {
        signals.push("B1_containment_fail".to_string());
    }
    if !result.output_exact {
        signals.push("B2_exact_fail".to_string());
    }
    if !result.output_behavioral {
        signals.push("B3_behavioral_fail".to_string());
    }
    if !result.lint_clean {
        signals.push("D_lint_fail".to_string());
    }
    if !result.cross_shell_agree {
        signals.push("G_cross_shell_fail".to_string());
    }
    signals
}

/// Derive a fix type category from a decision key (e.g. "assignment_value:single_quote").
fn derive_fix_type(decision_key: &str) -> String {
    let decision_type = decision_key.split(':').next().unwrap_or(decision_key);
    match decision_type {
        "assignment_value" => "quoting_strategy".to_string(),
        "ir_dispatch" => "ir_node_handling".to_string(),
        "string_emit" | "string_interpolation" => "string_handling".to_string(),
        "variable_expansion" => "expansion_strategy".to_string(),
        "command_substitution" => "substitution_strategy".to_string(),
        "redirect" | "redirect_emit" => "redirect_handling".to_string(),
        "pipe_emit" => "pipe_handling".to_string(),
        "arithmetic" | "arithmetic_emit" => "arithmetic_strategy".to_string(),
        "conditional" | "if_emit" => "conditional_handling".to_string(),
        "loop_emit" | "for_emit" | "while_emit" => "loop_handling".to_string(),
        "function_emit" => "function_handling".to_string(),
        _ => format!("{decision_type}_strategy"),
    }
}

/// Build a map of entry ID → decision trace locations from corpus results.
fn build_entry_locations(
    entry_results: &[(String, super::runner::CorpusResult)],
) -> std::collections::HashMap<String, Vec<String>> {
    entry_results
        .iter()
        .map(|(id, result)| {
            let locs = result
                .decision_trace
                .as_ref()
                .map(|t| {
                    t.iter()
                        .map(|d| format!("{}:{}", d.decision_type, d.choice))
                        .collect()
                })
                .unwrap_or_default();
            (id.clone(), locs)
        })
        .collect()
}

/// Group entry failures by error signal, returning (signal → [entry_ids], total_unique_failures).
fn group_failures_by_signal(
    entry_results: &[(String, super::runner::CorpusResult)],
) -> (std::collections::HashMap<String, Vec<String>>, usize) {
    let mut signal_failures: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut seen_failing = std::collections::HashSet::new();

    for (id, result) in entry_results {
        let signals = classify_failure_signals(result);
        if !signals.is_empty() {
            seen_failing.insert(id.clone());
            for signal in signals {
                signal_failures.entry(signal).or_default().push(id.clone());
            }
        }
    }

    let total = seen_failing.len();
    (signal_failures, total)
}

/// Run Tarantula localization for one signal and collect top-K patterns.
fn localize_signal_patterns(
    signal: &str,
    failing_ids: &[String],
    entry_results: &[(String, super::runner::CorpusResult)],
    entry_locations: &std::collections::HashMap<String, Vec<String>>,
) -> Vec<ShellFixPattern> {
    use crate::quality::sbfl::{localize_faults, SbflFormula};

    let failing_set: std::collections::HashSet<&String> = failing_ids.iter().collect();

    let coverage_data: Vec<(String, bool, Vec<String>)> = entry_results
        .iter()
        .filter_map(|(id, _)| {
            let locs = entry_locations.get(id)?;
            if locs.is_empty() {
                return None;
            }
            let passed_for_signal = !failing_set.contains(id);
            Some((id.clone(), passed_for_signal, locs.clone()))
        })
        .collect();

    if coverage_data.is_empty() {
        return Vec::new();
    }

    let rankings = localize_faults(&coverage_data, SbflFormula::Tarantula);

    rankings
        .iter()
        .take(5)
        .filter(|r| r.score > 0.0)
        .filter_map(|ranking| {
            let evidence: Vec<String> = failing_ids
                .iter()
                .filter(|id| {
                    entry_locations
                        .get(*id)
                        .is_some_and(|locs| locs.contains(&ranking.location))
                })
                .cloned()
                .collect();

            if evidence.is_empty() {
                return None;
            }

            Some(ShellFixPattern {
                error_signal: signal.to_string(),
                causal_decision: ranking.location.clone(),
                fix_type: derive_fix_type(&ranking.location),
                confidence: ranking.score,
                evidence_ids: evidence,
            })
        })
        .collect()
}

/// Mine CITL fix patterns from the full corpus using Tarantula fault localization.
///
/// For each failure signal, runs `localize_faults()` on the decision traces of
/// failing entries (for that signal) vs all passing entries, then takes the top-K
/// decisions as `ShellFixPattern` entries.
pub fn mine_patterns(
    registry: &super::registry::CorpusRegistry,
    runner: &super::runner::CorpusRunner,
) -> PatternStore {
    let entry_results: Vec<(String, super::runner::CorpusResult)> = registry
        .entries
        .iter()
        .map(|entry| (entry.id.clone(), runner.run_entry_with_trace(entry)))
        .collect();

    let total_entries = entry_results.len();
    let (signal_failures, total_failures) = group_failures_by_signal(&entry_results);

    if total_failures == 0 {
        return PatternStore {
            patterns: Vec::new(),
            total_entries,
            total_failures: 0,
            version: "1.0.0".to_string(),
        };
    }

    let entry_locations = build_entry_locations(&entry_results);

    let mut patterns: Vec<ShellFixPattern> = signal_failures
        .iter()
        .flat_map(|(signal, failing_ids)| {
            localize_signal_patterns(signal, failing_ids, &entry_results, &entry_locations)
        })
        .collect();

    patterns.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    PatternStore {
        patterns,
        total_entries,
        total_failures,
        version: "1.0.0".to_string(),
    }
}

/// Suggest fixes for a specific entry by matching its decision trace against a pattern store.
pub fn suggest_fixes(
    entry_id: &str,
    registry: &super::registry::CorpusRegistry,
    runner: &super::runner::CorpusRunner,
    store: &PatternStore,
) -> Vec<ShellFixPattern> {
    let entry = match registry.entries.iter().find(|e| e.id == entry_id) {
        Some(e) => e,
        None => return Vec::new(),
    };

    let result = runner.run_entry_with_trace(entry);
    let signals = classify_failure_signals(&result);

    if signals.is_empty() {
        return Vec::new(); // Entry passes — no fixes needed
    }

    let trace_decisions: std::collections::HashSet<String> = result
        .decision_trace
        .as_ref()
        .map(|t| {
            t.iter()
                .map(|d| format!("{}:{}", d.decision_type, d.choice))
                .collect()
        })
        .unwrap_or_default();

    // Filter patterns to those matching this entry's failure signals AND decision trace
    let mut suggestions: Vec<ShellFixPattern> = store
        .patterns
        .iter()
        .filter(|p| {
            signals.contains(&p.error_signal) && trace_decisions.contains(&p.causal_decision)
        })
        .cloned()
        .collect();

    // Deduplicate by causal_decision (keep highest confidence)
    suggestions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    suggestions.dedup_by(|a, b| a.causal_decision == b.causal_decision);

    suggestions
}

#[cfg(test)]
#[path = "pattern_store_tests_classify_fai.rs"]
mod tests_extracted;
