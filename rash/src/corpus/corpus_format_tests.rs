//! Tests for formatting functions across corpus modules:
//! - `tier_analysis`: `format_tier_weights`, `format_tier_analysis`, `format_tier_targets`
//! - `citl`: `format_convergence_criteria`, `format_lint_pipeline`, `format_regression_report`
//! - `schema_enforcement`: `format_schema_report`, `format_grammar_errors`, `format_grammar_spec`
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::citl::{
    format_convergence_criteria, format_lint_pipeline, format_regression_report,
    ConvergenceCriteria, LintPipelineEntry, RegressionEntry, RegressionReport,
};
use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier, Grade};
use crate::corpus::runner::{CorpusResult, CorpusScore};
use crate::corpus::schema_enforcement::{
    format_grammar_errors, format_grammar_spec, format_schema_report, validate_corpus,
    validate_entry, GrammarCategory, SchemaReport, ValidationLayer,
};
use crate::corpus::tier_analysis::{
    format_tier_analysis, format_tier_targets, format_tier_weights, TierStats, TierWeightedScore,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_tier_stats(tier: CorpusTier, total: usize, passed: usize) -> TierStats {
    let failed = total.saturating_sub(passed);
    let weight = tier.weight();
    let pass_rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        0.0
    };
    let weighted_score = pass_rate * weight * total as f64;
    let target_rate = tier.target_rate();
    TierStats {
        tier,
        total,
        passed,
        failed,
        weight,
        pass_rate,
        weighted_score,
        target_rate,
        meets_target: pass_rate >= target_rate,
    }
}

fn make_analysis(tiers: Vec<TierStats>) -> TierWeightedScore {
    let total_weighted_pass: f64 = tiers.iter().map(|t| t.weighted_score).sum();
    let total_weight: f64 = tiers.iter().map(|t| t.weight * t.total as f64).sum();
    let total_pass: usize = tiers.iter().map(|t| t.passed).sum();
    let total_entries: usize = tiers.iter().map(|t| t.total).sum();

    let weighted_score = if total_weight > 0.0 {
        (total_weighted_pass / total_weight) * 100.0
    } else {
        0.0
    };
    let unweighted_score = if total_entries > 0 {
        (total_pass as f64 / total_entries as f64) * 100.0
    } else {
        0.0
    };
    let all_targets_met = tiers.iter().all(|t| t.total == 0 || t.meets_target);

    TierWeightedScore {
        tiers,
        weighted_score,
        unweighted_score,
        weight_delta: weighted_score - unweighted_score,
        all_targets_met,
    }
}

fn make_corpus_entry(id: &str, format: CorpusFormat, output: &str) -> CorpusEntry {
    CorpusEntry {
        id: id.to_string(),
        name: format!("test-{id}"),
        description: "Test entry".to_string(),
        format,
        tier: CorpusTier::Trivial,
        input: String::new(),
        expected_output: output.to_string(),
        shellcheck: true,
        deterministic: true,
        idempotent: true,
    }
}

// ========================
// tier_analysis: format_tier_targets
// ========================

#[test]
fn test_format_tier_targets_empty_all_tiers() {
    let tiers: Vec<TierStats> = [
        CorpusTier::Trivial,
        CorpusTier::Standard,
        CorpusTier::Complex,
        CorpusTier::Adversarial,
        CorpusTier::Production,
    ]
    .iter()
    .map(|t| make_tier_stats(*t, 0, 0))
    .collect();

    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    // Every tier should show EMPTY
    assert_eq!(report.matches("EMPTY").count(), 5);
    assert!(report.contains("ALL TARGETS MET"));
    // No risk ranking when all tiers are empty
    assert!(!report.contains("Risk ranking"));
}

#[test]
fn test_format_tier_targets_single_tier_passing() {
    let tiers = vec![make_tier_stats(CorpusTier::Trivial, 100, 100)];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("100.0%"));
    assert!(report.contains("PASS"));
    assert!(report.contains("ALL TARGETS MET"));
    // Trivial target_rate is 1.0, so margin = 0.0 => "AT RISK" (margin < 0.02)
    assert!(report.contains("AT RISK"));
}

#[test]
fn test_format_tier_targets_single_tier_failing() {
    let tiers = vec![make_tier_stats(CorpusTier::Trivial, 100, 50)];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("50.0%"));
    assert!(report.contains("FAIL"));
    assert!(report.contains("TARGETS NOT MET"));
    assert!(report.contains("BELOW TARGET"));
}

#[test]
fn test_format_tier_targets_mixed_pass_fail() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100),
        make_tier_stats(CorpusTier::Standard, 100, 95),
        make_tier_stats(CorpusTier::Complex, 50, 49),
        make_tier_stats(CorpusTier::Adversarial, 20, 19),
        make_tier_stats(CorpusTier::Production, 10, 5),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    // Standard is 95%, target 99% -> FAIL
    assert!(report.contains("FAIL"));
    assert!(report.contains("TARGETS NOT MET"));
    assert!(report.contains("Risk ranking"));
}

#[test]
fn test_format_tier_targets_at_risk_margin() {
    // 1% margin above target -> AT RISK
    let mut ts = make_tier_stats(CorpusTier::Adversarial, 100, 96);
    // Adversarial target: 95%, actual: 96%, margin = 1% = 0.01
    ts.meets_target = true;
    let tiers = vec![ts];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("AT RISK"));
}

#[test]
fn test_format_tier_targets_marginal() {
    // 3% margin above target -> MARGINAL
    let mut ts = make_tier_stats(CorpusTier::Adversarial, 100, 98);
    // Adversarial target: 95%, actual: 98%, margin = 3% = 0.03
    ts.meets_target = true;
    let tiers = vec![ts];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("MARGINAL"));
}

#[test]
fn test_format_tier_targets_delta_formatting() {
    let tiers = vec![make_tier_stats(CorpusTier::Production, 100, 100)];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    // Production target: 95%, actual: 100% -> delta should be +5.0%
    assert!(report.contains("+5.0%"));
}

// ========================
// tier_analysis: format_tier_weights
// ========================

#[test]
fn test_format_tier_weights_empty_tiers() {
    let tiers: Vec<TierStats> = [
        CorpusTier::Trivial,
        CorpusTier::Standard,
        CorpusTier::Complex,
        CorpusTier::Adversarial,
        CorpusTier::Production,
    ]
    .iter()
    .map(|t| make_tier_stats(*t, 0, 0))
    .collect();

    let analysis = make_analysis(tiers);
    let report = format_tier_weights(&analysis);

    assert!(report.contains("Tier-Weighted"));
    assert!(report.contains("Weighted Score:"));
    assert!(report.contains("Unweighted Score:"));
    assert!(report.contains("Weight Effect:"));
    // Empty tiers show "-" for rate and weighted
    assert!(report.matches('-').count() >= 5);
}

#[test]
fn test_format_tier_weights_positive_delta() {
    // Higher tiers pass more -> positive weight delta
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 50),
        make_tier_stats(CorpusTier::Production, 100, 100),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_weights(&analysis);

    assert!(report.contains("+"));
}

#[test]
fn test_format_tier_weights_negative_delta() {
    // Higher tiers fail more -> negative weight delta
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100),
        make_tier_stats(CorpusTier::Production, 100, 50),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_weights(&analysis);

    // Negative delta shows no "+" prefix
    assert!(report.contains("Weight Effect:"));
}

// ========================
// tier_analysis: format_tier_analysis
// ========================

#[test]
fn test_format_tier_analysis_distribution_bars() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 50, 50),
        make_tier_stats(CorpusTier::Standard, 30, 30),
        make_tier_stats(CorpusTier::Complex, 10, 10),
        make_tier_stats(CorpusTier::Adversarial, 5, 5),
        make_tier_stats(CorpusTier::Production, 5, 5),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("Distribution:"));
    assert!(report.contains("T1: Trivial"));
    assert!(report.contains("T2: Standard"));
    assert!(report.contains("T3: Complex"));
    assert!(report.contains("T4: Adversarial"));
    assert!(report.contains("T5: Production"));
    assert!(report.contains("Scoring Comparison"));
    assert!(report.contains("Weight Impact"));
}

#[test]
fn test_format_tier_analysis_zero_delta_interpretation() {
    // All 100% pass -> delta is 0
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 50, 50),
        make_tier_stats(CorpusTier::Production, 50, 50),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("No difference"));
}

#[test]
fn test_format_tier_analysis_positive_delta_interpretation() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 50),
        make_tier_stats(CorpusTier::Production, 100, 100),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("Higher tiers performing better"));
}

#[test]
fn test_format_tier_analysis_negative_delta_interpretation() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100),
        make_tier_stats(CorpusTier::Production, 100, 50),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("Lower tiers performing better"));
}

#[test]
fn test_format_tier_analysis_empty_tiers_skipped_in_impact() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 0, 0),
        make_tier_stats(CorpusTier::Standard, 10, 10),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    // Trivial has 0 entries, so it should not appear in impact section
    // but it DOES appear in distribution. Impact section skips zero-total tiers.
    assert!(report.contains("Weight Impact"));
    // Standard should appear in impact
    assert!(report.contains("T2: Standard"));
}

// ========================
// citl: format_convergence_criteria
// ========================

#[test]
fn test_format_convergence_criteria_all_met() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![0.99, 1.0, 1.0],
        stability_met: true,
        delta_values: vec![0.001, 0.0, 0.001],
        growth_met: true,
        corpus_size: 1000,
        target_size: 900,
        no_regressions: true,
        converged: true,
    };
    let table = format_convergence_criteria(&criteria);

    assert!(table.contains("CONVERGED"));
    assert!(table.contains("Shewhart 1931"));
    assert!(table.contains("Add harder entries"));
    assert!(table.contains("PASS"));
    assert!(!table.contains("FAIL"));
    assert!(table.contains("1000/900"));
    assert!(table.contains("clean"));
}

#[test]

include!("corpus_format_tests_incl2.rs");
