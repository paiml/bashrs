//! Advanced corpus commands: dependency graphs, impact analysis, blast radius, dedup, triage, and label rules.

use super::corpus_decision_commands::score_impact_color;
use super::corpus_metrics_commands::collect_trace_coverage;
use crate::models::{Config, Result};

pub(crate) fn corpus_graph() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_graph_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_graph`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_graph_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::graph_priority::{build_connectivity, connectivity_table};
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let coverage_data = collect_trace_coverage(registry, &runner);
    let total_entries = coverage_data.len();
    let conn = build_connectivity(&coverage_data);
    let table = connectivity_table(&conn);

    println!(
        "\n  {BOLD}Decision Connectivity Graph{RESET}  ({} decisions, {} traced entries)",
        table.len(),
        total_entries
    );
    println!("  {DIM}{}{RESET}", "─".repeat(78));
    println!(
        "  {DIM}{:<36}  {:>6}  {:<14}  Entries (sample){RESET}",
        "Decision", "Usage", "Connectivity"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(78));

    for row in &table {
        let bar_len = if total_entries > 0 {
            (row.usage_count * 12) / total_entries.max(1)
        } else {
            0
        };
        let bar: String = "\u{2588}".repeat(bar_len.clamp(1, 12));
        let conn_label = if row.is_high_connectivity {
            format!("{RED}HIGH{RESET}")
        } else {
            format!("{DIM}LOW{RESET}")
        };
        let sample: String = if row.entry_ids.len() <= 3 {
            row.entry_ids.join(", ")
        } else {
            format!(
                "{}, ... +{}",
                row.entry_ids[..2].join(", "),
                row.entry_ids.len() - 2
            )
        };
        println!(
            "  {:<36}  {:>6}  {:<14}  {DIM}{}{RESET}",
            row.decision,
            row.usage_count,
            format!("{bar}  {conn_label}"),
            sample
        );
    }

    println!();
    Ok(())
}

/// Impact-weighted decision priority combining suspiciousness × connectivity (§11.10.3).
pub(crate) fn corpus_impact(limit: usize) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_impact_with(&CorpusRegistry::load_full(), limit)
}

/// PMAT-257: body of `corpus_impact`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_impact_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    limit: usize,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::graph_priority::{build_connectivity, compute_graph_priorities};
    use crate::corpus::runner::CorpusRunner;
    use crate::quality::sbfl::{localize_faults, SbflFormula};

    let runner = CorpusRunner::new(Config::default());
    let coverage_data = collect_trace_coverage(registry, &runner);

    let total = coverage_data.len();
    let passed = coverage_data.iter().filter(|(_, p, _)| *p).count();
    let failed = total - passed;

    if failed == 0 {
        println!(
            "\n  {BRIGHT_GREEN}All {total} traced entries pass — no impact analysis needed{RESET}\n"
        );
        return Ok(());
    }

    let conn = build_connectivity(&coverage_data);
    let rankings = localize_faults(&coverage_data, SbflFormula::Tarantula);
    let analysis = compute_graph_priorities(&rankings, &conn, total);

    println!(
        "\n  {BOLD}Impact-Weighted Decision Priority{RESET}  ({} failing signals)",
        failed
    );
    println!("  {DIM}{}{RESET}", "─".repeat(80));
    println!(
        "  {DIM}{:<36}  {:>14}  {:>6}  {:>8}  {:>8}{RESET}",
        "Decision", "Suspiciousness", "Usage", "Priority", "Impact"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(80));

    for gp in analysis.priorities.iter().take(limit) {
        let (impact_label, color) = score_impact_color(gp.suspiciousness);
        let _ = impact_label; // use graph-level impact instead
        let impact_display = match gp.impact.as_str() {
            "HIGH" => format!("{RED}HIGH{RESET}"),
            "MEDIUM" => format!("{YELLOW}MEDIUM{RESET}"),
            _ => format!("{DIM}LOW{RESET}"),
        };
        println!(
            "  {:<36}  {color}{:>14.4}{RESET}  {:>6}  {:>8.2}  {:>8}",
            gp.decision, gp.suspiciousness, gp.usage_count, gp.priority, impact_display
        );
    }

    if analysis.priorities.len() > limit {
        println!(
            "\n  {DIM}... and {} more (use --limit to show more){RESET}",
            analysis.priorities.len() - limit
        );
    }

    println!();
    Ok(())
}

/// Show blast radius of fixing a specific decision (§11.10.3).
pub(crate) fn corpus_blast_radius(decision: &str) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_blast_radius_with(&CorpusRegistry::load_full(), decision)
}

/// PMAT-257: body of `corpus_blast_radius`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_blast_radius_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    decision: &str,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::graph_priority::build_connectivity;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let coverage_data = collect_trace_coverage(registry, &runner);
    let total_entries = coverage_data.len();
    let conn = build_connectivity(&coverage_data);

    let entry_ids = match conn.get(decision) {
        Some(ids) => {
            let mut v: Vec<String> = ids.iter().cloned().collect();
            v.sort();
            v
        }
        None => {
            println!("\n  {YELLOW}Decision '{decision}' not found in any traced entry{RESET}");
            println!("  {DIM}Use 'bashrs corpus graph' to see available decisions{RESET}\n");
            return Ok(());
        }
    };

    // Classify entries as passing or failing
    let pass_fail: std::collections::HashMap<String, bool> = coverage_data
        .iter()
        .map(|(id, passed, _)| (id.clone(), *passed))
        .collect();

    let mut passing = Vec::new();
    let mut failing = Vec::new();
    for id in &entry_ids {
        if pass_fail.get(id).copied().unwrap_or(true) {
            passing.push(id.clone());
        } else {
            failing.push(id.clone());
        }
    }

    let pct = if total_entries > 0 {
        (entry_ids.len() as f64 / total_entries as f64) * 100.0
    } else {
        0.0
    };

    println!("\n  {BOLD}Blast Radius for:{RESET} {CYAN}{decision}{RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(60));
    println!(
        "  Total entries using this decision: {WHITE}{}{RESET}",
        entry_ids.len()
    );

    if !failing.is_empty() {
        println!(
            "  Failing entries: {BRIGHT_RED}{}{RESET}",
            failing.join(", ")
        );
    }

    if !passing.is_empty() {
        let display = if passing.len() <= 8 {
            passing.join(", ")
        } else {
            format!("{}, ... +{}", passing[..6].join(", "), passing.len() - 6)
        };
        println!("  Passing entries: {BRIGHT_GREEN}{display}{RESET}");
    }

    println!(
        "\n  Impact: Fixing this decision affects {WHITE}{}/{total_entries}{RESET} entries ({WHITE}{pct:.1}%{RESET})",
        entry_ids.len()
    );
    println!();
    Ok(())
}

/// Deduplicated error view with counts and risk classification (§11.10.4).
pub(crate) fn corpus_dedup() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_dedup_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_dedup`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_dedup_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::error_dedup::deduplicate_errors;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let triage = deduplicate_errors(registry, &runner);

    println!(
        "\n  {BOLD}Deduplicated Errors{RESET}  ({DIM}{} raw → {} unique{RESET})",
        triage.total_raw, triage.total_unique
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    if triage.errors.is_empty() {
        println!("  {BRIGHT_GREEN}No errors in corpus.{RESET}\n");
        return Ok(());
    }

    println!(
        "  {DIM}{:<22}  {:<32}  {:>5}  {:<8}  Entries{RESET}",
        "Error Code", "Message (normalized)", "Count", "Risk"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for err in &triage.errors {
        let risk_color = match err.risk {
            crate::corpus::error_dedup::RiskLevel::High => RED,
            crate::corpus::error_dedup::RiskLevel::Medium => YELLOW,
            crate::corpus::error_dedup::RiskLevel::Low => DIM,
        };
        let msg_display = if err.message.len() > 30 {
            format!("{}...", &err.message[..27])
        } else {
            err.message.clone()
        };
        let entries_display: Vec<&str> = err.entry_ids.iter().take(5).map(|s| s.as_str()).collect();
        let more = if err.entry_ids.len() > 5 {
            format!(" (+{})", err.entry_ids.len() - 5)
        } else {
            String::new()
        };
        println!(
            "  {CYAN}{:<22}{RESET}  {:<32}  {:>5}  {risk_color}{:<8}{RESET}  {}{}",
            err.error_code,
            msg_display,
            err.count,
            format!("{}", err.risk),
            entries_display.join(", "),
            more,
        );
    }

    println!(
        "\n  {DIM}Summary: {RED}{} HIGH{RESET}{DIM}, {YELLOW}{} MEDIUM{RESET}{DIM}, {} LOW{RESET}",
        triage.high_count, triage.medium_count, triage.low_count
    );
    println!();
    Ok(())
}

/// Risk-prioritized fix backlog with weak supervision labels (§11.10.4).
pub(crate) fn corpus_triage() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_triage_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_triage`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_triage_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::error_dedup::deduplicate_errors;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let triage = deduplicate_errors(registry, &runner);

    println!(
        "\n  {BOLD}Risk-Prioritized Fix Backlog{RESET}  ({DIM}{} unique errors{RESET})",
        triage.total_unique
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    if triage.errors.is_empty() {
        println!("  {BRIGHT_GREEN}No errors to triage.{RESET}\n");
        return Ok(());
    }

    println!(
        "  {DIM}{:<8}  {:<8}  {:<24}  {:>5}  Fix Impact{RESET}",
        "Priority", "Risk", "Error Code", "Count"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for (i, err) in triage.errors.iter().enumerate() {
        let risk_color = match err.risk {
            crate::corpus::error_dedup::RiskLevel::High => RED,
            crate::corpus::error_dedup::RiskLevel::Medium => YELLOW,
            crate::corpus::error_dedup::RiskLevel::Low => DIM,
        };
        let fix_impact = match err.error_code.as_str() {
            "A_transpile_fail" => "transpilation",
            "B1_containment_fail" => "output correctness",
            "B2_exact_fail" => "exact match",
            "B3_behavioral_fail" => "behavioral correctness",
            "D_lint_fail" => "lint compliance",
            "G_cross_shell_fail" => "cross-shell compat",
            _ => "quality",
        };
        println!(
            "  {WHITE}#{:<7}{RESET}  {risk_color}{:<8}{RESET}  {CYAN}{:<24}{RESET}  {:>5}  {DIM}{}{RESET}",
            i + 1,
            format!("{}", err.risk),
            err.error_code,
            err.count,
            fix_impact,
        );
    }

    println!();
    Ok(())
}

/// Show programmatic labeling rules and match counts (§11.10.4).
pub(crate) fn corpus_label_rules() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_label_rules_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_label_rules`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_label_rules_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::error_dedup::count_rule_matches;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let rule_matches = count_rule_matches(registry, &runner);

    println!("\n  {BOLD}Programmatic Labeling Rules (Weak Supervision){RESET}");
    println!("  {DIM}{}{RESET}", "─".repeat(72));
    println!(
        "  {DIM}{:<14}  {:<34}  {:<8}  {:>7}{RESET}",
        "Rule", "Condition", "Risk", "Matches"
    );
    println!("  {DIM}{}{RESET}", "─".repeat(72));

    for (rule, count) in &rule_matches {
        let risk_color = match rule.risk {
            crate::corpus::error_dedup::RiskLevel::High => RED,
            crate::corpus::error_dedup::RiskLevel::Medium => YELLOW,
            crate::corpus::error_dedup::RiskLevel::Low => DIM,
        };
        let count_display = if *count > 0 {
            format!("{WHITE}{count}{RESET}")
        } else {
            format!("{DIM}0{RESET}")
        };
        println!(
            "  {CYAN}{:<14}{RESET}  {:<34}  {risk_color}{:<8}{RESET}  {:>7}",
            rule.name,
            rule.condition,
            format!("{}", rule.risk),
            count_display,
        );
    }

    println!();
    Ok(())
}

// PMAT-257: coverage for the advanced corpus handlers. These live inside the
// library, because the coverage gate measures `cargo llvm-cov --lib -p
// bashrs` and a test under rash/tests/ does not move that number at all.
// `corpus_graph`, `corpus_impact`, `corpus_blast_radius`, `corpus_dedup`,
// `corpus_triage`, and `corpus_label_rules` all build a `CorpusRunner` and
// score entries out of the real `CorpusRegistry::load_full()` (18,000+
// entries) -- too slow for a unit test as written. Each was split into a
// `*_with(registry, ...)` twin that takes the registry as a parameter,
// matching `corpus_compare_commands.rs`.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
    use crate::corpus::runner::CorpusRunner;

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

    /// A registry that also contains an entry whose `expected_contains`
    /// never matches the transpiled output, forcing a B2/containment
    /// failure so the failure-path branches of the handlers under test run.
    fn failing_registry() -> CorpusRegistry {
        let mut registry = tiny_registry();
        registry.add(CorpusEntry::new(
            "B-002",
            "broken-bash",
            "PMAT-257 fixture (forces a failure)",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            r#"fn main() { let greeting = "hello"; }"#,
            "this_string_never_appears_in_output",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_graph_with_tiny_registry() {
        corpus_graph_with(&tiny_registry()).expect("graph runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_impact_all_pass_early_return() {
        corpus_impact_with(&tiny_registry(), 5)
            .expect("impact early-returns when all traced entries pass");
    }

    #[test]
    fn test_PMAT257_cov_impact_with_failures() {
        corpus_impact_with(&failing_registry(), 5)
            .expect("impact prints the priority table when entries fail");
    }

    #[test]
    fn test_PMAT257_cov_blast_radius_unknown_decision() {
        corpus_blast_radius_with(&tiny_registry(), "no-such-decision")
            .expect("blast radius handles an unknown decision gracefully");
    }

    #[test]
    fn test_PMAT257_cov_blast_radius_known_decision() {
        let registry = failing_registry();
        let runner = CorpusRunner::new(Config::default());
        let coverage_data = collect_trace_coverage(&registry, &runner);
        let (_, _, locations) = coverage_data
            .first()
            .expect("at least one traced entry with decision locations");
        let decision = locations
            .first()
            .expect("at least one decision location")
            .clone();
        corpus_blast_radius_with(&registry, &decision)
            .expect("blast radius runs over a known decision");
    }

    #[test]
    fn test_PMAT257_cov_dedup_no_errors() {
        corpus_dedup_with(&tiny_registry()).expect("dedup runs when there are no errors");
    }

    #[test]
    fn test_PMAT257_cov_dedup_with_errors() {
        corpus_dedup_with(&failing_registry()).expect("dedup prints deduplicated error rows");
    }

    #[test]
    fn test_PMAT257_cov_triage_no_errors() {
        corpus_triage_with(&tiny_registry()).expect("triage runs when there are no errors");
    }

    #[test]
    fn test_PMAT257_cov_triage_with_errors() {
        corpus_triage_with(&failing_registry()).expect("triage prints the fix backlog");
    }

    #[test]
    fn test_PMAT257_cov_label_rules_with_tiny_registry() {
        corpus_label_rules_with(&tiny_registry()).expect("label rules run over a tiny registry");
    }
}
