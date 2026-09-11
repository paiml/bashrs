//! Corpus weight, format reporting, budget analysis, entropy, and TODO tracking.

use super::corpus_failure_commands::result_fail_dims;
use super::corpus_ranking_commands::classify_category;
use crate::cli::args::CorpusOutputFormat;
use crate::models::{Config, Result};

pub(crate) fn corpus_weight() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    print_weight_analysis(&score.results);
    Ok(())
}

/// Pure: print the V2 scoring weight table for a set of already-scored results.
///
/// PMAT-257: split out of `corpus_weight()` so the table itself (dimension
/// weights, per-dimension rate/points, totals) is testable with a handful of
/// synthetic `CorpusResult`s instead of a real `CorpusRunner::run()` over the
/// full corpus.
pub(crate) fn print_weight_analysis(results: &[crate::corpus::runner::CorpusResult]) {
    use crate::cli::color::*;
    let n = results.len().max(1) as f64;

    let dims: &[(&str, f64, usize)] = &[
        (
            "A  Transpilation",
            30.0,
            results.iter().filter(|r| r.transpiled).count(),
        ),
        (
            "B1 Containment",
            10.0,
            results.iter().filter(|r| r.output_contains).count(),
        ),
        (
            "B2 Exact match",
            8.0,
            results.iter().filter(|r| r.output_exact).count(),
        ),
        (
            "B3 Behavioral",
            7.0,
            results.iter().filter(|r| r.output_behavioral).count(),
        ),
        ("C  Coverage", 15.0, 0), // handled separately
        (
            "D  Lint clean",
            10.0,
            results.iter().filter(|r| r.lint_clean).count(),
        ),
        (
            "E  Deterministic",
            10.0,
            results.iter().filter(|r| r.deterministic).count(),
        ),
        (
            "F  Metamorphic",
            5.0,
            results.iter().filter(|r| r.metamorphic_consistent).count(),
        ),
        (
            "G  Cross-shell",
            5.0,
            results.iter().filter(|r| r.cross_shell_agree).count(),
        ),
    ];

    let c_avg: f64 = results.iter().map(|r| r.coverage_ratio).sum::<f64>() / n;

    println!("{BOLD}V2 Scoring Weight Analysis{RESET} (100-point scale)");
    println!();
    println!(
        "  {BOLD}{:<18} {:>6} {:>6} {:>8} {:>8} {:>7}{RESET}",
        "Dimension", "Weight", "Rate", "Points", "Max", "Eff%"
    );

    let mut total_pts = 0.0f64;
    for (label, weight, pass) in dims {
        let (rate, pts) = if label.starts_with('C') {
            (c_avg * 100.0, c_avg * weight)
        } else {
            let r = *pass as f64 / n * 100.0;
            (r, *pass as f64 / n * weight)
        };
        total_pts += pts;
        let rc = pct_color(rate);
        let eff = pts / weight * 100.0;
        let ec = if eff >= 99.0 {
            GREEN
        } else if eff >= 90.0 {
            YELLOW
        } else {
            RED
        };
        println!("  {CYAN}{:<18}{RESET} {:>5.0}  {rc}{:>5.1}%{RESET}  {:>7.1}  {:>7.0}  {ec}{:>5.1}%{RESET}",
            label, weight, rate, pts, weight, eff);
    }
    println!();
    println!(
        "  {WHITE}Total: {:.1}/100{RESET}  (spec max: 100.0)",
        total_pts
    );
}

/// Detailed per-format quality report with dimension breakdown.
pub(crate) fn corpus_format_report(output_format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let formats = [
        ("Bash", CorpusFormat::Bash),
        ("Makefile", CorpusFormat::Makefile),
        ("Dockerfile", CorpusFormat::Dockerfile),
    ];

    match output_format {
        CorpusOutputFormat::Human => {
            for (name, fmt) in &formats {
                let entries: Vec<_> = registry
                    .entries
                    .iter()
                    .filter(|e| e.format == *fmt)
                    .collect();
                let results: Vec<_> = entries.iter().map(|e| runner.run_single(e)).collect();
                print_format_report_block(name, &results);
            }
        }
        CorpusOutputFormat::Json => {
            println!("{{}}"); // placeholder for JSON format
        }
    }
    Ok(())
}

/// Pure: print one format's entries/passed/dimension-breakdown block.
///
/// PMAT-257: split out of `corpus_format_report()`'s Human branch so the
/// dimension table is testable with synthetic `CorpusResult`s rather than a
/// `CorpusRunner::run_single()` sweep over a whole format's real entries.
pub(crate) fn print_format_report_block(
    name: &str,
    results: &[crate::corpus::runner::CorpusResult],
) {
    use crate::cli::color::*;
    let n = results.len();
    let fails = results
        .iter()
        .filter(|r| !result_fail_dims(r).is_empty())
        .count();

    println!(
        "{BOLD}{name}{RESET} ({n} entries, {GREEN}{}{RESET} passed, {}{}{})",
        n - fails,
        if fails > 0 { RED } else { GREEN },
        fails,
        RESET
    );

    // Per-dimension breakdown
    let dim_data = [
        (
            "A  Transpile",
            results.iter().filter(|r| r.transpiled).count(),
        ),
        (
            "B1 Contain",
            results.iter().filter(|r| r.output_contains).count(),
        ),
        (
            "B2 Exact",
            results.iter().filter(|r| r.output_exact).count(),
        ),
        (
            "B3 Behav",
            results.iter().filter(|r| r.output_behavioral).count(),
        ),
        ("D  Lint", results.iter().filter(|r| r.lint_clean).count()),
        (
            "E  Determ",
            results.iter().filter(|r| r.deterministic).count(),
        ),
        (
            "F  Meta",
            results.iter().filter(|r| r.metamorphic_consistent).count(),
        ),
        (
            "G  XShell",
            results.iter().filter(|r| r.cross_shell_agree).count(),
        ),
    ];
    for (dim, pass) in &dim_data {
        let rate = *pass as f64 / n.max(1) as f64 * 100.0;
        let rc = pct_color(rate);
        println!("  {CYAN}{dim:<12}{RESET} {rc}{pass}/{n}{RESET} ({rc}{rate:.1}%{RESET})");
    }
    println!();
}

/// Time budget analysis: time spent per format and per tier.
pub(crate) fn corpus_budget() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let mut format_time: std::collections::HashMap<String, (f64, usize)> =
        std::collections::HashMap::new();
    let mut tier_time: std::collections::HashMap<String, (f64, usize)> =
        std::collections::HashMap::new();

    for entry in &registry.entries {
        let start = Instant::now();
        let _ = runner.run_single(entry);
        let ms = start.elapsed().as_secs_f64() * 1000.0;

        let fmt = format!("{}", entry.format);
        let e = format_time.entry(fmt).or_insert((0.0, 0));
        e.0 += ms;
        e.1 += 1;

        let tier = format!("{:?}", entry.tier);
        let e = tier_time.entry(tier).or_insert((0.0, 0));
        e.0 += ms;
        e.1 += 1;
    }

    print_budget_report(&format_time, &tier_time, registry.entries.len());
    Ok(())
}

/// Pure: print the by-format / by-tier time budget tables.
///
/// PMAT-257: split out of `corpus_budget()` so the aggregation/sort/percent
/// logic is testable with a couple of synthetic `(total_ms, count)` entries
/// instead of timing a `CorpusRunner::run_single()` sweep over the whole
/// registry.
pub(crate) fn print_budget_report(
    format_time: &std::collections::HashMap<String, (f64, usize)>,
    tier_time: &std::collections::HashMap<String, (f64, usize)>,
    total_entries: usize,
) {
    use crate::cli::color::*;

    let total_ms: f64 = format_time.values().map(|(t, _)| t).sum();

    println!("{BOLD}Time Budget Analysis{RESET}");
    println!("{DIM}  Total: {total_ms:.0}ms across {total_entries} entries{RESET}");
    println!();

    // By format
    println!("  {BOLD}By Format:{RESET}");
    let mut fmt_sorted: Vec<_> = format_time.iter().collect();
    fmt_sorted.sort_by(|a, b| {
        b.1 .0
            .partial_cmp(&a.1 .0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (fmt, (time, count)) in &fmt_sorted {
        let pct = time / total_ms * 100.0;
        let avg = time / *count as f64;
        let color = if pct > 50.0 { YELLOW } else { DIM };
        println!("    {CYAN}{:<12}{RESET} {color}{:>8.0}ms{RESET} ({pct:>5.1}%)  {DIM}{count} entries, avg {avg:.1}ms{RESET}",
            fmt, time);
    }

    // By tier
    println!();
    println!("  {BOLD}By Tier:{RESET}");
    let tier_order = [
        "Trivial",
        "Standard",
        "Complex",
        "Adversarial",
        "Production",
    ];
    for tier_name in &tier_order {
        if let Some((time, count)) = tier_time.get(*tier_name) {
            let pct = time / total_ms * 100.0;
            let avg = time / *count as f64;
            let color = if pct > 30.0 { YELLOW } else { DIM };
            println!("    {CYAN}{:<14}{RESET} {color}{:>8.0}ms{RESET} ({pct:>5.1}%)  {DIM}{count} entries, avg {avg:.1}ms{RESET}",
                tier_name, time);
        }
    }
}

/// Information entropy of construct distribution.
pub(crate) fn corpus_entropy() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let total = registry.entries.len() as f64;

    // Category distribution entropy
    let mut cat_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for entry in &registry.entries {
        let cat = classify_category(&entry.name);
        *cat_counts.entry(cat).or_insert(0) += 1;
    }

    let mut h_cat = 0.0f64;
    for count in cat_counts.values() {
        let p = *count as f64 / total;
        if p > 0.0 {
            h_cat -= p * p.log2();
        }
    }
    let max_h_cat = (cat_counts.len() as f64).log2();
    let cat_norm = if max_h_cat > 0.0 {
        h_cat / max_h_cat
    } else {
        0.0
    };

    // Tier distribution entropy
    let mut tier_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for entry in &registry.entries {
        *tier_counts.entry(format!("{:?}", entry.tier)).or_insert(0) += 1;
    }
    let mut h_tier = 0.0f64;
    for count in tier_counts.values() {
        let p = *count as f64 / total;
        if p > 0.0 {
            h_tier -= p * p.log2();
        }
    }
    let max_h_tier = (tier_counts.len() as f64).log2();
    let tier_norm = if max_h_tier > 0.0 {
        h_tier / max_h_tier
    } else {
        0.0
    };

    // Format distribution entropy
    let mut fmt_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entry in &registry.entries {
        *fmt_counts.entry(format!("{}", entry.format)).or_insert(0) += 1;
    }
    let mut h_fmt = 0.0f64;
    for count in fmt_counts.values() {
        let p = *count as f64 / total;
        if p > 0.0 {
            h_fmt -= p * p.log2();
        }
    }
    let max_h_fmt = (fmt_counts.len() as f64).log2();
    let fmt_norm = if max_h_fmt > 0.0 {
        h_fmt / max_h_fmt
    } else {
        0.0
    };

    println!("{BOLD}Corpus Diversity (Shannon Entropy){RESET}");
    println!();
    println!(
        "  {BOLD}{:<18} {:>6} {:>8} {:>8} {:>8}{RESET}",
        "Distribution", "H", "H_max", "Norm", "Rating"
    );

    let rating = |norm: f64| -> (&str, &str) {
        if norm >= 0.8 {
            (GREEN, "Diverse")
        } else if norm >= 0.5 {
            (YELLOW, "Moderate")
        } else {
            (RED, "Clustered")
        }
    };

    let (cc, cr) = rating(cat_norm);
    println!(
        "  {CYAN}{:<18}{RESET} {:>5.2}  {:>7.2}  {:>7.2}  {cc}{cr}{RESET}",
        "Category", h_cat, max_h_cat, cat_norm
    );

    let (tc, tr) = rating(tier_norm);
    println!(
        "  {CYAN}{:<18}{RESET} {:>5.2}  {:>7.2}  {:>7.2}  {tc}{tr}{RESET}",
        "Tier", h_tier, max_h_tier, tier_norm
    );

    let (fc, fr) = rating(fmt_norm);
    println!(
        "  {CYAN}{:<18}{RESET} {:>5.2}  {:>7.2}  {:>7.2}  {fc}{fr}{RESET}",
        "Format", h_fmt, max_h_fmt, fmt_norm
    );

    println!();
    println!("  {DIM}H=Shannon entropy (bits), Norm=H/H_max (0=uniform, 1=max diversity){RESET}");
    Ok(())
}

/// Auto-generate improvement suggestions from current state.
pub(crate) fn corpus_todo() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let suggestions = build_todo_suggestions(
        &score.results,
        &registry.entries,
        &score.format_scores,
        score.score,
    );
    print_todo_suggestions(&suggestions);
    Ok(())
}

/// Pure: derive the (priority, message) suggestion list from already-scored
/// results.
///
/// PMAT-257: split out of `corpus_todo()` so the suggestion logic (failure
/// digest, coverage gate, category-diversity gate, per-format gate, score-cap
/// note) is testable with a handful of synthetic entries/results instead of a
/// `CorpusRunner::run()` over the full corpus.
pub(crate) fn build_todo_suggestions(
    results: &[crate::corpus::runner::CorpusResult],
    entries: &[crate::corpus::registry::CorpusEntry],
    format_scores: &[crate::corpus::runner::FormatScore],
    score: f64,
) -> Vec<(u8, String)> {
    let mut suggestions: Vec<(u8, String)> = Vec::new(); // (priority, suggestion)

    // Check for failures
    let failures: Vec<_> = results
        .iter()
        .enumerate()
        .filter(|(_, r)| !result_fail_dims(r).is_empty())
        .collect();
    if !failures.is_empty() {
        let fail_ids: Vec<String> = failures
            .iter()
            .filter_map(|(i, _)| entries.get(*i).map(|e| e.id.clone()))
            .collect();
        suggestions.push((
            1,
            format!(
                "Fix {} failing entries: {}",
                fail_ids.len(),
                fail_ids.join(", ")
            ),
        ));
    }

    // Check coverage
    let c_avg: f64 =
        results.iter().map(|r| r.coverage_ratio).sum::<f64>() / results.len().max(1) as f64;
    if c_avg < 0.99 {
        suggestions.push((
            2,
            format!(
                "Improve coverage: {:.1}% → target 99%+ (run LCOV, add tests)",
                c_avg * 100.0
            ),
        ));
    }

    // Check category diversity
    let mut cat_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for entry in entries {
        *cat_counts
            .entry(classify_category(&entry.name))
            .or_insert(0) += 1;
    }
    let untagged = cat_counts.get("General").copied().unwrap_or(0);
    let pct_untagged = untagged as f64 / entries.len().max(1) as f64 * 100.0;
    if pct_untagged > 40.0 {
        suggestions.push((
            3,
            format!("Reduce unclassified entries: {untagged} ({pct_untagged:.0}%) are 'General'"),
        ));
    }

    // Check format balance
    for fs in format_scores {
        if fs.score < 99.5 {
            suggestions.push((
                2,
                format!(
                    "{} score {:.1}/100 — investigate dimension gaps",
                    fs.format, fs.score
                ),
            ));
        }
    }

    // Check if score is capped
    if (99.9..100.0).contains(&score) {
        suggestions.push((
            4,
            "Score at 99.9 — theoretical max limited by B-143 (unfixable shell semantics)"
                .to_string(),
        ));
    }

    suggestions
}

/// Pure: print a sorted suggestion list (or the "all clear" message).
pub(crate) fn print_todo_suggestions(suggestions: &[(u8, String)]) {
    use crate::cli::color::*;

    println!("{BOLD}Corpus Improvement Suggestions{RESET}");
    println!();

    if suggestions.is_empty() {
        println!("  {GREEN}No improvements needed — corpus is in excellent shape!{RESET}");
    } else {
        let mut sorted = suggestions.to_vec();
        sorted.sort_by_key(|(p, _)| *p);
        for (priority, suggestion) in &sorted {
            let pc = match priority {
                1 => BRIGHT_RED,
                2 => YELLOW,
                3 => CYAN,
                _ => DIM,
            };
            let label = match priority {
                1 => "P0",
                2 => "P1",
                3 => "P2",
                _ => "P3",
            };
            println!("  {pc}[{label}]{RESET} {suggestion}");
        }
    }
}

// PMAT-257: every top-level handler in this file builds a CorpusRunner and
// scores either the whole registry or a whole format's real entries by
// timing them one at a time (`corpus_budget`); a full run measured >2 minutes
// and a Dockerfile-only run_format still measured >60ms per handler call, so
// none of the five `pub(crate) fn corpus_*` entry points are called directly
// here. `corpus_weight`, `corpus_format_report`'s Human branch, `corpus_budget`,
// and `corpus_todo` were each split so their table/suggestion logic is a pure
// function of already-computed `CorpusResult`s — those pure functions are
// what's under test. Only `corpus_entropy` (no CorpusRunner at all) and
// `corpus_format_report`'s Json branch (a `runner`/`registry` it never uses)
// are safe to call directly.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    use crate::corpus::runner::{CorpusResult, FormatScore};

    fn fake_result(id: &str, all_pass: bool, coverage_ratio: f64) -> CorpusResult {
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
            coverage_ratio,
            ..Default::default()
        }
    }

    #[test]
    fn test_PMAT257_cov_print_weight_analysis_mixed_and_perfect() {
        // A mix of a passing and a failing entry exercises both the "Eff%"
        // color branches and a total strictly between 0 and 100.
        print_weight_analysis(&[
            fake_result("B-001", true, 1.0),
            fake_result("B-002", false, 0.0),
        ]);
        // Empty input must not divide by zero (n = results.len().max(1)).
        print_weight_analysis(&[]);
    }

    #[test]
    fn test_PMAT257_cov_print_format_report_block_pass_and_fail() {
        print_format_report_block(
            "Dockerfile",
            &[
                fake_result("D-001", true, 1.0),
                fake_result("D-002", false, 0.0),
            ],
        );
        print_format_report_block("Empty", &[]);
    }

    #[test]
    fn test_PMAT257_cov_format_report_json_branch_is_a_cheap_placeholder() {
        // The Json branch never touches the runner/registry it constructs;
        // this is the one top-level handler in the file safe to call whole.
        corpus_format_report(&CorpusOutputFormat::Json)
            .expect("the json placeholder branch always succeeds");
    }

    #[test]
    fn test_PMAT257_cov_print_budget_report_sorts_by_time_and_pct() {
        let mut format_time = std::collections::HashMap::new();
        format_time.insert("Bash".to_string(), (900.0, 9));
        format_time.insert("Dockerfile".to_string(), (100.0, 1));
        let mut tier_time = std::collections::HashMap::new();
        tier_time.insert("Production".to_string(), (1000.0, 10));
        print_budget_report(&format_time, &tier_time, 10);

        // Empty maps: total_ms is 0.0, so pct = time/total_ms is only ever
        // computed for entries that exist -- with no entries the by-format
        // and by-tier loops both simply produce no rows.
        print_budget_report(
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            0,
        );
    }

    #[test]
    fn test_PMAT257_cov_corpus_entropy_reads_the_real_registry() {
        // No CorpusRunner anywhere in this function -- just registry stats.
        corpus_entropy().expect("entropy over the real registry always succeeds");
    }

    fn fake_entry(id: &str, name: &str) -> CorpusEntry {
        CorpusEntry::new(
            id,
            name,
            "PMAT-257 fixture",
            CorpusFormat::Bash,
            CorpusTier::Standard,
            "fn main() {}",
            "#!/bin/sh",
        )
    }

    #[test]
    fn test_PMAT257_cov_build_todo_suggestions_all_clear() {
        // Named to land in a real category (not "General") so the
        // untagged-diversity gate does not fire on a single-entry corpus.
        let entries = vec![fake_entry("B-001", "config-basic")];
        let results = vec![fake_result("B-001", true, 1.0)];
        let suggestions = build_todo_suggestions(&results, &entries, &[], 100.0);
        assert!(
            suggestions.is_empty(),
            "a fully-passing, fully-covered, classified, uncapped corpus needs no suggestions: {suggestions:?}"
        );
        print_todo_suggestions(&suggestions);
    }

    #[test]
    fn test_PMAT257_cov_build_todo_suggestions_every_gate_fires() {
        // Failing entry (P0), low coverage (P1), all-"General" names (P2 via
        // untagged-pct), a format below 99.5 (P1), and a score sitting on the
        // 99.9 cap (P3) -- every suggestion branch in one pass.
        let entries = vec![fake_entry("B-001", "zzz-unclassified")];
        let results = vec![fake_result("B-001", false, 0.5)];
        let format_scores = vec![FormatScore {
            format: CorpusFormat::Bash,
            total: 1,
            passed: 0,
            rate: 0.0,
            score: 42.0,
            grade: crate::corpus::registry::Grade::F,
        }];
        let suggestions = build_todo_suggestions(&results, &entries, &format_scores, 99.95);
        let priorities: Vec<u8> = suggestions.iter().map(|(p, _)| *p).collect();
        assert!(
            priorities.contains(&1),
            "failing entries must be P0: {suggestions:?}"
        );
        assert!(
            priorities.contains(&2),
            "low coverage or format gap must be P1/P2: {suggestions:?}"
        );
        assert!(
            priorities.contains(&4),
            "a score of 99.95 must trip the score-cap note: {suggestions:?}"
        );
        print_todo_suggestions(&suggestions);
    }
}
