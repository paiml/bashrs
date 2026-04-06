
use super::*;
use crate::corpus::registry::{CorpusEntry, CorpusFormat};
use crate::corpus::runner::CorpusResult;

fn make_entry(id: &str, tier: CorpusTier) -> CorpusEntry {
    CorpusEntry::new(
        id,
        "test",
        "test",
        CorpusFormat::Bash,
        tier,
        "fn main() {}",
        "expected",
    )
}

fn make_result(id: &str, transpiled: bool) -> CorpusResult {
    CorpusResult {
        id: id.to_string(),
        transpiled,
        output_contains: transpiled,
        output_exact: transpiled,
        output_behavioral: transpiled,
        has_test: true,
        coverage_ratio: 0.95,
        schema_valid: true,
        lint_clean: transpiled,
        deterministic: true,
        metamorphic_consistent: true,
        cross_shell_agree: transpiled,
        expected_output: None,
        actual_output: if transpiled {
            Some("expected".to_string())
        } else {
            None
        },
        error: if transpiled {
            None
        } else {
            Some("error".to_string())
        },
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    }
}

fn make_score(results: Vec<CorpusResult>) -> CorpusScore {
    use crate::corpus::registry::Grade;
    let total = results.len();
    let passed = results.iter().filter(|r| r.transpiled).count();
    let failed = total - passed;
    let rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        0.0
    };
    CorpusScore {
        total,
        passed,
        failed,
        rate,
        score: rate * 100.0,
        grade: Grade::from_score(rate * 100.0),
        format_scores: vec![],
        results,
    }
}

#[test]
fn test_analyze_empty() {
    let registry = CorpusRegistry::new();
    let score = make_score(vec![]);
    let analysis = analyze_tiers(&registry, &score);
    assert_eq!(analysis.tiers.len(), 5);
    assert_eq!(analysis.weighted_score, 0.0);
    assert_eq!(analysis.unweighted_score, 0.0);
}

#[test]
fn test_analyze_single_tier() {
    let mut registry = CorpusRegistry::new();
    registry
        .entries
        .push(make_entry("B-001", CorpusTier::Trivial));
    registry
        .entries
        .push(make_entry("B-002", CorpusTier::Trivial));

    let results = vec![make_result("B-001", true), make_result("B-002", true)];
    let score = make_score(results);
    let analysis = analyze_tiers(&registry, &score);

    let t1 = &analysis.tiers[0];
    assert_eq!(t1.total, 2);
    assert_eq!(t1.passed, 2);
    assert_eq!(t1.pass_rate, 1.0);
    assert_eq!(t1.weight, 1.0);
    assert!(t1.meets_target);
}

#[test]
fn test_analyze_mixed_tiers() {
    let mut registry = CorpusRegistry::new();
    registry
        .entries
        .push(make_entry("B-001", CorpusTier::Trivial));
    registry
        .entries
        .push(make_entry("B-002", CorpusTier::Production));

    let results = vec![
        make_result("B-001", true),
        make_result("B-002", false), // production failure
    ];
    let score = make_score(results);
    let analysis = analyze_tiers(&registry, &score);

    // Trivial: 1/1 pass, weight 1.0
    assert_eq!(analysis.tiers[0].passed, 1);
    assert_eq!(analysis.tiers[0].total, 1);

    // Production: 0/1 pass, weight 3.0
    assert_eq!(analysis.tiers[4].passed, 0);
    assert_eq!(analysis.tiers[4].total, 1);
    assert!(!analysis.tiers[4].meets_target);

    // Weighted score should be less than unweighted
    // because the heavier tier (production) failed
    assert!(analysis.weighted_score < analysis.unweighted_score);
}

#[test]
fn test_weight_delta_all_pass() {
    let mut registry = CorpusRegistry::new();
    registry
        .entries
        .push(make_entry("B-001", CorpusTier::Trivial));
    registry
        .entries
        .push(make_entry("B-002", CorpusTier::Production));

    let results = vec![make_result("B-001", true), make_result("B-002", true)];
    let score = make_score(results);
    let analysis = analyze_tiers(&registry, &score);

    // All pass → weighted == unweighted == 100%
    assert!((analysis.weighted_score - 100.0).abs() < 0.01);
    assert!((analysis.unweighted_score - 100.0).abs() < 0.01);
    assert!(analysis.weight_delta.abs() < 0.01);
}

#[test]
fn test_all_targets_met() {
    let mut registry = CorpusRegistry::new();
    registry
        .entries
        .push(make_entry("B-001", CorpusTier::Trivial));
    let results = vec![make_result("B-001", true)];
    let score = make_score(results);
    let analysis = analyze_tiers(&registry, &score);
    assert!(analysis.all_targets_met);
}

#[test]
fn test_targets_not_met() {
    let mut registry = CorpusRegistry::new();
    registry
        .entries
        .push(make_entry("B-001", CorpusTier::Trivial));
    let results = vec![make_result("B-001", false)];
    let score = make_score(results);
    let analysis = analyze_tiers(&registry, &score);
    assert!(!analysis.all_targets_met);
}

#[test]
fn test_tier_label() {
    assert_eq!(tier_label(CorpusTier::Trivial), "T1: Trivial");
    assert_eq!(tier_label(CorpusTier::Standard), "T2: Standard");
    assert_eq!(tier_label(CorpusTier::Complex), "T3: Complex");
    assert_eq!(tier_label(CorpusTier::Adversarial), "T4: Adversarial");
    assert_eq!(tier_label(CorpusTier::Production), "T5: Production");
}

#[test]
fn test_format_tier_weights_header() {
    let analysis = TierWeightedScore {
        tiers: vec![TierStats {
            tier: CorpusTier::Trivial,
            total: 10,
            passed: 10,
            failed: 0,
            weight: 1.0,
            pass_rate: 1.0,
            weighted_score: 10.0,
            target_rate: 1.0,
            meets_target: true,
        }],
        weighted_score: 100.0,
        unweighted_score: 100.0,
        weight_delta: 0.0,
        all_targets_met: true,
    };
    let report = format_tier_weights(&analysis);
    assert!(report.contains("Tier-Weighted"));
    assert!(report.contains("T1: Trivial"));
    assert!(report.contains("100.0%"));
    assert!(report.contains("Weighted Score"));
}

#[test]
fn test_format_tier_analysis_distribution() {
    let analysis = TierWeightedScore {
        tiers: vec![
            TierStats {
                tier: CorpusTier::Trivial,
                total: 50,
                passed: 50,
                failed: 0,
                weight: 1.0,
                pass_rate: 1.0,
                weighted_score: 50.0,
                target_rate: 1.0,
                meets_target: true,
            },
            TierStats {
                tier: CorpusTier::Standard,
                total: 30,
                passed: 30,
                failed: 0,
                weight: 1.5,
                pass_rate: 1.0,
                weighted_score: 45.0,
                target_rate: 0.99,
                meets_target: true,
            },
        ],
        weighted_score: 100.0,
        unweighted_score: 100.0,
        weight_delta: 0.0,
        all_targets_met: true,
    };
    let report = format_tier_analysis(&analysis);
    assert!(report.contains("Distribution"));
    assert!(report.contains("T1: Trivial"));
    assert!(report.contains("T2: Standard"));
    assert!(report.contains("Scoring Comparison"));
}

#[test]
fn test_format_tier_targets_pass() {
    let analysis = TierWeightedScore {
        tiers: vec![TierStats {
            tier: CorpusTier::Trivial,
            total: 10,
            passed: 10,
            failed: 0,
            weight: 1.0,
            pass_rate: 1.0,
            weighted_score: 10.0,
            target_rate: 1.0,
            meets_target: true,
        }],
        weighted_score: 100.0,
        unweighted_score: 100.0,
        weight_delta: 0.0,
        all_targets_met: true,
    };
    let report = format_tier_targets(&analysis);
    assert!(report.contains("PASS"));
    assert!(report.contains("ALL TARGETS MET"));
}

#[test]
fn test_format_tier_targets_fail() {
    let analysis = TierWeightedScore {
        tiers: vec![TierStats {
            tier: CorpusTier::Trivial,
            total: 10,
            passed: 5,
            failed: 5,
            weight: 1.0,
            pass_rate: 0.5,
            weighted_score: 5.0,
            target_rate: 1.0,
            meets_target: false,
        }],
        weighted_score: 50.0,
        unweighted_score: 50.0,
        weight_delta: 0.0,
        all_targets_met: false,
    };
    let report = format_tier_targets(&analysis);
    assert!(report.contains("FAIL"));
    assert!(report.contains("TARGETS NOT MET"));
    assert!(report.contains("BELOW TARGET"));
}

#[test]
fn test_format_tier_targets_risk_ranking() {
    let analysis = TierWeightedScore {
        tiers: vec![
            TierStats {
                tier: CorpusTier::Trivial,
                total: 100,
                passed: 100,
                failed: 0,
                weight: 1.0,
                pass_rate: 1.0,
                weighted_score: 100.0,
                target_rate: 1.0,
                meets_target: true,
            },
            TierStats {
                tier: CorpusTier::Adversarial,
                total: 10,
                passed: 10,
                failed: 0,
                weight: 2.5,
                pass_rate: 1.0,
                weighted_score: 25.0,
                target_rate: 0.95,
                meets_target: true,
            },
        ],
        weighted_score: 100.0,
        unweighted_score: 100.0,
        weight_delta: 0.0,
        all_targets_met: true,
    };
    let report = format_tier_targets(&analysis);
    assert!(report.contains("Risk ranking"));
    assert!(report.contains("COMFORTABLE"));
}

#[test]
fn test_weighted_score_heavier_failure() {
    // When a Production tier entry fails, weighted score drops more
    // than when a Trivial tier entry fails
    let mut reg1 = CorpusRegistry::new();
    reg1.entries.push(make_entry("B-001", CorpusTier::Trivial));
    reg1.entries.push(make_entry("B-002", CorpusTier::Trivial));

    let score1 = make_score(vec![
        make_result("B-001", true),
        make_result("B-002", false),
    ]);
    let analysis1 = analyze_tiers(&reg1, &score1);

    let mut reg2 = CorpusRegistry::new();
    reg2.entries.push(make_entry("B-001", CorpusTier::Trivial));
    reg2.entries
        .push(make_entry("B-002", CorpusTier::Production));

    let score2 = make_score(vec![
        make_result("B-001", true),
        make_result("B-002", false),
    ]);
    let analysis2 = analyze_tiers(&reg2, &score2);

    // Both have 50% unweighted
    assert!((analysis1.unweighted_score - 50.0).abs() < 0.01);
    assert!((analysis2.unweighted_score - 50.0).abs() < 0.01);

    // Weighted: failing production (3.0x) hurts more than failing trivial (1.0x)
    // reg1: T1 pass 1.0*1 + T1 fail 0.0*1 / (1.0+1.0) = 0.5 → 50%
    // reg2: T1 pass 1.0*1 / (1.0+3.0) = 0.25 → 25%
    assert!(analysis2.weighted_score < analysis1.weighted_score);
}

#[test]
fn test_empty_tier_skipped_in_targets() {
    let registry = CorpusRegistry::new();
    let score = make_score(vec![]);
    let analysis = analyze_tiers(&registry, &score);
    let report = format_tier_targets(&analysis);
    assert!(report.contains("EMPTY"));
}

#[test]
fn test_tier_stats_fields() {
    let mut registry = CorpusRegistry::new();
    registry
        .entries
        .push(make_entry("B-001", CorpusTier::Complex));
    registry
        .entries
        .push(make_entry("B-002", CorpusTier::Complex));
    registry
        .entries
        .push(make_entry("B-003", CorpusTier::Complex));

    let results = vec![
        make_result("B-001", true),
        make_result("B-002", true),
        make_result("B-003", false),
    ];
    let score = make_score(results);
    let analysis = analyze_tiers(&registry, &score);

    let complex = &analysis.tiers[2]; // T3 Complex
    assert_eq!(complex.total, 3);
    assert_eq!(complex.passed, 2);
    assert_eq!(complex.failed, 1);
    assert_eq!(complex.weight, 2.0);
    assert!((complex.pass_rate - 2.0 / 3.0).abs() < 0.001);
    assert!(!complex.meets_target); // 66.7% < 98%
}
