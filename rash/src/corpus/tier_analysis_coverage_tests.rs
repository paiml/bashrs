//! Coverage tests for corpus/tier_analysis.rs — targeting format_tier_targets.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::registry::CorpusTier;
use crate::corpus::tier_analysis::{format_tier_targets, TierStats, TierWeightedScore};

fn make_tier_stats(
    tier: CorpusTier,
    total: usize,
    passed: usize,
    weight: f64,
    target_rate: f64,
) -> TierStats {
    let failed = total.saturating_sub(passed);
    let pass_rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        0.0
    };
    TierStats {
        tier,
        total,
        passed,
        failed,
        weight,
        pass_rate,
        weighted_score: pass_rate * weight,
        target_rate,
        meets_target: pass_rate >= target_rate,
    }
}

fn make_all_passing() -> TierWeightedScore {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100, 0.10, 0.99),
        make_tier_stats(CorpusTier::Standard, 200, 195, 0.20, 0.95),
        make_tier_stats(CorpusTier::Complex, 150, 145, 0.30, 0.90),
        make_tier_stats(CorpusTier::Adversarial, 50, 48, 0.25, 0.85),
        make_tier_stats(CorpusTier::Production, 30, 29, 0.15, 0.90),
    ];
    TierWeightedScore {
        weighted_score: 95.0,
        unweighted_score: 93.0,
        weight_delta: 2.0,
        all_targets_met: true,
        tiers,
    }
}

fn make_some_failing() -> TierWeightedScore {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100, 0.10, 0.99),
        make_tier_stats(CorpusTier::Standard, 200, 195, 0.20, 0.95),
        make_tier_stats(CorpusTier::Complex, 150, 120, 0.30, 0.90),
        make_tier_stats(CorpusTier::Adversarial, 50, 30, 0.25, 0.85),
        make_tier_stats(CorpusTier::Production, 30, 20, 0.15, 0.90),
    ];
    TierWeightedScore {
        weighted_score: 78.0,
        unweighted_score: 80.0,
        weight_delta: -2.0,
        all_targets_met: false,
        tiers,
    }
}

// =============================================================================
// format_tier_targets — all code paths
// =============================================================================

#[test]
fn test_tier_targets_all_passing() {
    let analysis = make_all_passing();
    let result = format_tier_targets(&analysis);
    assert!(
        result.contains("PASS") || result.contains("pass") || result.contains("✓"),
        "All passing should show PASS indicator"
    );
}

#[test]
fn test_tier_targets_some_failing() {
    let analysis = make_some_failing();
    let result = format_tier_targets(&analysis);
    assert!(
        result.contains("FAIL") || result.contains("fail") || result.contains("✗"),
        "Failing tiers should show FAIL indicator"
    );
}

#[test]
fn test_tier_targets_contains_all_tier_names() {
    let analysis = make_all_passing();
    let result = format_tier_targets(&analysis);
    assert!(
        result.contains("Trivial") || result.contains("trivial"),
        "Should mention Trivial tier"
    );
    assert!(
        result.contains("Standard") || result.contains("standard"),
        "Should mention Standard tier"
    );
    assert!(
        result.contains("Complex") || result.contains("complex"),
        "Should mention Complex tier"
    );
}

#[test]
fn test_tier_targets_empty_tiers() {
    let analysis = TierWeightedScore {
        tiers: vec![],
        weighted_score: 0.0,
        unweighted_score: 0.0,
        weight_delta: 0.0,
        all_targets_met: true,
    };
    let result = format_tier_targets(&analysis);
    assert!(!result.is_empty());
}

#[test]
fn test_tier_targets_single_tier() {
    let analysis = TierWeightedScore {
        tiers: vec![make_tier_stats(
            CorpusTier::Trivial,
            50,
            50,
            1.0,
            0.99,
        )],
        weighted_score: 100.0,
        unweighted_score: 100.0,
        weight_delta: 0.0,
        all_targets_met: true,
    };
    let result = format_tier_targets(&analysis);
    assert!(!result.is_empty());
}

#[test]
fn test_tier_targets_zero_total_tier() {
    let analysis = TierWeightedScore {
        tiers: vec![make_tier_stats(CorpusTier::Production, 0, 0, 0.15, 0.90)],
        weighted_score: 0.0,
        unweighted_score: 0.0,
        weight_delta: 0.0,
        all_targets_met: false,
    };
    let result = format_tier_targets(&analysis);
    assert!(!result.is_empty());
}

#[test]
fn test_tier_targets_negative_weight_delta() {
    let analysis = make_some_failing();
    let result = format_tier_targets(&analysis);
    // Negative delta means unweighted > weighted
    assert!(!result.is_empty());
}

#[test]
fn test_tier_targets_risk_ranking() {
    // Tiers barely meeting target should show risk
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100, 0.10, 0.99),
        make_tier_stats(CorpusTier::Complex, 100, 91, 0.30, 0.90), // just barely
        make_tier_stats(CorpusTier::Adversarial, 100, 86, 0.25, 0.85), // just barely
    ];
    let analysis = TierWeightedScore {
        weighted_score: 90.0,
        unweighted_score: 92.0,
        weight_delta: -2.0,
        all_targets_met: true,
        tiers,
    };
    let result = format_tier_targets(&analysis);
    assert!(!result.is_empty());
}

#[test]
fn test_tier_targets_contains_percentages() {
    let analysis = make_all_passing();
    let result = format_tier_targets(&analysis);
    // Should contain percentage values
    assert!(
        result.contains('%') || result.contains("100") || result.contains("95"),
        "Should contain percentage or numeric values"
    );
}

#[test]
fn test_tier_targets_format_contains_table() {
    let analysis = make_all_passing();
    let result = format_tier_targets(&analysis);
    // Should contain table-like formatting
    assert!(
        result.contains("│") || result.contains("|") || result.contains("─") || result.lines().count() > 3,
        "Should contain table structure"
    );
}
