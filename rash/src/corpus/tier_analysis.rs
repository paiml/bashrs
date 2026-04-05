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
#[path = "tier_analysis_tests_extracted.rs"]
mod tests_extracted;
