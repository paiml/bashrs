//! Tier-Weighted Aggregate Scoring (§4.3)
//!
//! Implements tier-weighted scoring where higher-difficulty tiers
//! contribute more to the overall quality signal:
//!
//! - Tier 1 (Trivial): weight 1.0
//! - Tier 2 (Standard): weight 1.5
//! - Tier 3 (Complex): weight 2.0
//! - Tier 4 (Adversarial): weight 2.5
//! - Tier 5 (Production): weight 3.0
//!
//! Formula: Repo_Score = Σ(entry_pass × tier_weight) / Σ(tier_weight)

use crate::corpus::registry::{CorpusRegistry, CorpusTier};
use crate::corpus::runner::CorpusScore;

/// Per-tier statistics with weighting
#[derive(Debug, Clone)]
pub struct TierStats {
    pub tier: CorpusTier,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub weight: f64,
    pub pass_rate: f64,
    pub weighted_score: f64,
    pub target_rate: f64,
    pub meets_target: bool,
}

/// Aggregate tier-weighted scoring result
#[derive(Debug, Clone)]
pub struct TierWeightedScore {
    pub tiers: Vec<TierStats>,
    pub weighted_score: f64,
    pub unweighted_score: f64,
    pub weight_delta: f64,
    pub all_targets_met: bool,
}

/// All 5 tiers in order
const ALL_TIERS: &[CorpusTier] = &[
    CorpusTier::Trivial,
    CorpusTier::Standard,
    CorpusTier::Complex,
    CorpusTier::Adversarial,
    CorpusTier::Production,
];

/// Build a map from entry ID to CorpusTier
fn build_tier_map(registry: &CorpusRegistry) -> std::collections::HashMap<String, CorpusTier> {
    registry
        .entries
        .iter()
        .map(|e| (e.id.clone(), e.tier))
        .collect()
}

/// Compute tier-weighted analysis
pub fn analyze_tiers(registry: &CorpusRegistry, score: &CorpusScore) -> TierWeightedScore {
    let tier_map = build_tier_map(registry);

    let mut tier_counts: std::collections::HashMap<CorpusTier, (usize, usize)> =
        std::collections::HashMap::new();

    for result in &score.results {
        if let Some(&tier) = tier_map.get(&result.id) {
            let (total, passed) = tier_counts.entry(tier).or_insert((0, 0));
            *total += 1;
            if result.transpiled {
                *passed += 1;
            }
        }
    }

    let mut total_weighted_pass = 0.0;
    let mut total_weight = 0.0;
    let mut total_pass = 0usize;
    let mut total_entries = 0usize;

    let tiers: Vec<TierStats> = ALL_TIERS
        .iter()
        .map(|tier| {
            let (t, p) = tier_counts.get(tier).copied().unwrap_or((0, 0));
            let f = t.saturating_sub(p);
            let weight = tier.weight();
            let pass_rate = if t > 0 { p as f64 / t as f64 } else { 0.0 };
            let weighted = pass_rate * weight * t as f64;
            let target = tier.target_rate();

            total_weighted_pass += weighted;
            total_weight += weight * t as f64;
            total_pass += p;
            total_entries += t;

            TierStats {
                tier: *tier,
                total: t,
                passed: p,
                failed: f,
                weight,
                pass_rate,
                weighted_score: weighted,
                target_rate: target,
                meets_target: pass_rate >= target,
            }
        })
        .collect();

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

/// Format per-tier weight breakdown table
pub fn format_tier_weights(analysis: &TierWeightedScore) -> String {
    let mut out = String::new();
    let sep = "\u{2500}".repeat(78);

    out.push_str("Tier-Weighted Corpus Scoring (\u{00a7}4.3)\n");
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!(
        "{:<16} {:>6} {:>6} {:>6} {:>8} {:>10} {:>10}\n",
        "Tier", "Total", "Pass", "Fail", "Weight", "Rate", "Weighted"
    ));
    out.push_str(&sep);
    out.push('\n');

    for ts in &analysis.tiers {
        let rate_str = if ts.total > 0 {
            format!("{:.1}%", ts.pass_rate * 100.0)
        } else {
            "-".to_string()
        };
        let weighted_str = if ts.total > 0 {
            format!("{:.2}", ts.weighted_score)
        } else {
            "-".to_string()
        };
        out.push_str(&format!(
            "{:<16} {:>6} {:>6} {:>6} {:>8.1}x {:>10} {:>10}\n",
            tier_label(ts.tier),
            ts.total,
            ts.passed,
            ts.failed,
            ts.weight,
            rate_str,
            weighted_str,
        ));
    }

    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!(
        "Weighted Score:   {:.2}%\n",
        analysis.weighted_score
    ));
    out.push_str(&format!(
        "Unweighted Score: {:.2}%\n",
        analysis.unweighted_score
    ));
    let delta_sign = if analysis.weight_delta >= 0.0 {
        "+"
    } else {
        ""
    };
    out.push_str(&format!(
        "Weight Effect:    {}{:.2}%\n",
        delta_sign, analysis.weight_delta
    ));

    out
}

/// Format tier analysis with weighted vs unweighted comparison
pub fn format_tier_analysis(analysis: &TierWeightedScore) -> String {
    let mut out = String::new();
    let sep = "\u{2500}".repeat(70);

    out.push_str("Tier Difficulty Analysis (\u{00a7}4.3)\n");
    out.push_str(&sep);
    out.push('\n');

    // Show tier distribution
    let total: usize = analysis.tiers.iter().map(|t| t.total).sum();
    out.push_str("Distribution:\n");
    for ts in &analysis.tiers {
        let pct = if total > 0 {
            (ts.total as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let bar_len = (pct / 2.0) as usize;
        let bar = "\u{2588}".repeat(bar_len);
        out.push_str(&format!(
            "  {:<16} {:>4} ({:>5.1}%) {}\n",
            tier_label(ts.tier),
            ts.total,
            pct,
            bar,
        ));
    }

    // Weighted vs unweighted comparison
    out.push_str(&format!("\n{}\n", sep));
    out.push_str("Scoring Comparison:\n");
    out.push_str(&format!(
        "  Unweighted (flat):     {:.2}%  (all entries equal)\n",
        analysis.unweighted_score
    ));
    out.push_str(&format!(
        "  Weighted (\u{00a7}4.3):       {:.2}%  (harder tiers count more)\n",
        analysis.weighted_score
    ));

    let delta_sign = if analysis.weight_delta >= 0.0 {
        "+"
    } else {
        ""
    };
    let interpretation = if analysis.weight_delta.abs() < 0.01 {
        "No difference (uniform performance across tiers)"
    } else if analysis.weight_delta > 0.0 {
        "Higher tiers performing better than lower tiers"
    } else {
        "Lower tiers performing better than higher tiers"
    };
    out.push_str(&format!(
        "  Delta:                 {}{:.2}%  ({})\n",
        delta_sign, analysis.weight_delta, interpretation
    ));

    // Impact analysis: which tiers contribute most to weighted score
    out.push_str(&format!("\n{}\n", sep));
    out.push_str("Weight Impact (contribution to final score):\n");

    let total_weighted: f64 = analysis.tiers.iter().map(|t| t.weighted_score).sum();
    for ts in &analysis.tiers {
        if ts.total == 0 {
            continue;
        }
        let contribution = if total_weighted > 0.0 {
            (ts.weighted_score / total_weighted) * 100.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "  {:<16} {:>6.1}%  ({:.1}x weight \u{00d7} {} entries)\n",
            tier_label(ts.tier),
            contribution,
            ts.weight,
            ts.total,
        ));
    }

    out
}

/// Format tier target comparison
pub fn format_tier_targets(analysis: &TierWeightedScore) -> String {
    let mut out = String::new();
    let sep = "\u{2500}".repeat(70);

    out.push_str("Tier Target Rate Comparison (\u{00a7}2.3)\n");
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!(
        "{:<16} {:>10} {:>10} {:>10} {:>8}\n",
        "Tier", "Actual", "Target", "Delta", "Status"
    ));
    out.push_str(&sep);
    out.push('\n');

    for ts in &analysis.tiers {
        if ts.total == 0 {
            out.push_str(&format!(
                "{:<16} {:>10} {:>10} {:>10} {:>8}\n",
                tier_label(ts.tier),
                "-",
                format!("{:.0}%", ts.target_rate * 100.0),
                "-",
                "EMPTY",
            ));
            continue;
        }
        let actual = ts.pass_rate * 100.0;
        let target = ts.target_rate * 100.0;
        let delta = actual - target;
        let delta_str = format!("{:+.1}%", delta);
        let status = if ts.meets_target { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "{:<16} {:>10} {:>10} {:>10} {:>8}\n",
            tier_label(ts.tier),
            format!("{:.1}%", actual),
            format!("{:.0}%", target),
            delta_str,
            status,
        ));
    }

    out.push_str(&sep);
    out.push('\n');
    let overall_status = if analysis.all_targets_met {
        "ALL TARGETS MET"
    } else {
        "TARGETS NOT MET"
    };
    out.push_str(&format!("Overall: {}\n", overall_status));

    // Show which tiers have the smallest margin (most at risk)
    let mut margins: Vec<(&TierStats, f64)> = analysis
        .tiers
        .iter()
        .filter(|t| t.total > 0)
        .map(|t| (t, t.pass_rate - t.target_rate))
        .collect();
    margins.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if !margins.is_empty() {
        out.push_str("\nRisk ranking (smallest margin first):\n");
        for (i, (ts, margin)) in margins.iter().enumerate() {
            let risk = if *margin < 0.0 {
                "BELOW TARGET"
            } else if *margin < 0.02 {
                "AT RISK"
            } else if *margin < 0.05 {
                "MARGINAL"
            } else {
                "COMFORTABLE"
            };
            out.push_str(&format!(
                "  {}. {:<16} margin: {:+.1}%  ({})\n",
                i + 1,
                tier_label(ts.tier),
                margin * 100.0,
                risk,
            ));
        }
    }

    out
}

/// Human-readable tier label
fn tier_label(tier: CorpusTier) -> &'static str {
    match tier {
        CorpusTier::Trivial => "T1: Trivial",
        CorpusTier::Standard => "T2: Standard",
        CorpusTier::Complex => "T3: Complex",
        CorpusTier::Adversarial => "T4: Adversarial",
        CorpusTier::Production => "T5: Production",
    }
}

#[cfg(test)]
mod tests {
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
        registry.entries.push(make_entry("B-001", CorpusTier::Trivial));
        registry.entries.push(make_entry("B-002", CorpusTier::Trivial));

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
        registry.entries.push(make_entry("B-001", CorpusTier::Trivial));
        registry.entries.push(make_entry("B-002", CorpusTier::Production));

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
        registry.entries.push(make_entry("B-001", CorpusTier::Trivial));
        registry.entries.push(make_entry("B-002", CorpusTier::Production));

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
        registry.entries.push(make_entry("B-001", CorpusTier::Trivial));
        let results = vec![make_result("B-001", true)];
        let score = make_score(results);
        let analysis = analyze_tiers(&registry, &score);
        assert!(analysis.all_targets_met);
    }

    #[test]
    fn test_targets_not_met() {
        let mut registry = CorpusRegistry::new();
        registry.entries.push(make_entry("B-001", CorpusTier::Trivial));
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
        reg2.entries.push(make_entry("B-002", CorpusTier::Production));

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
        registry.entries.push(make_entry("B-001", CorpusTier::Complex));
        registry.entries.push(make_entry("B-002", CorpusTier::Complex));
        registry.entries.push(make_entry("B-003", CorpusTier::Complex));

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
}
