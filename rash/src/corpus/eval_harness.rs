//! ShellSafetyBench evaluation harness (SSC v12 S14.5).
//!
//! Computes 6 weighted metrics for evaluating shell safety model quality:
//!
//! | Metric              | Weight | Description                                |
//! |---------------------|--------|--------------------------------------------|
//! | Detection F1        | 25%    | Binary classification F1 score             |
//! | Rule Citation       | 20%    | Correct rule ID (SEC001, DET001, etc.)     |
//! | CWE Mapping         | 10%    | Correct CWE ID in response                 |
//! | Fix Validity        | 15%    | Proposed fix removes the finding            |
//! | Explanation Quality | 15%    | Coherence of natural-language explanation   |
//! | OOD Generalization  | 15%    | Performance on unseen CWE patterns          |

use crate::corpus::evaluation::{evaluate, EvaluationReport};
use crate::linter;
use serde::Serialize;

/// Weights for the 6 ShellSafetyBench metrics (sum = 1.0).
pub const DETECTION_F1_WEIGHT: f64 = 0.25;
pub const RULE_CITATION_WEIGHT: f64 = 0.20;
pub const CWE_MAPPING_WEIGHT: f64 = 0.10;
pub const FIX_VALIDITY_WEIGHT: f64 = 0.15;
pub const EXPLANATION_WEIGHT: f64 = 0.15;
pub const OOD_WEIGHT: f64 = 0.15;

/// A single model prediction for evaluation.
#[derive(Debug, Clone)]
pub struct Prediction {
    /// Entry ID (SSB-NNNNN)
    pub id: String,
    /// Model's classification: "safe" or "unsafe"
    pub classification: String,
    /// Rule IDs cited in model response
    pub cited_rules: Vec<String>,
    /// CWE IDs cited in model response
    pub cited_cwes: Vec<String>,
    /// Proposed fix (if any)
    pub proposed_fix: Option<String>,
    /// Full explanation text
    pub explanation: String,
}

/// Ground truth for a single evaluation entry.
#[derive(Debug, Clone)]
pub struct GroundTruth {
    /// Entry ID (SSB-NNNNN)
    pub id: String,
    /// True label: 0=safe, 1=unsafe
    pub label: u8,
    /// Ground truth rule IDs
    pub rules: Vec<String>,
    /// Ground truth CWE IDs
    pub cwes: Vec<String>,
    /// Original script (for fix validation)
    pub script: String,
}

/// Full 6-metric evaluation result.
#[derive(Debug, Clone, Serialize)]
pub struct EvalResult {
    /// Binary classification report
    pub detection: EvaluationReport,
    /// Detection F1 score (0.0-1.0)
    pub detection_f1: f64,
    /// Rule citation accuracy (0.0-1.0)
    pub rule_citation: f64,
    /// CWE mapping accuracy (0.0-1.0)
    pub cwe_mapping: f64,
    /// Fix validity rate (0.0-1.0)
    pub fix_validity: f64,
    /// Explanation quality score (0.0-1.0)
    pub explanation_quality: f64,
    /// OOD generalization score (0.0-1.0)
    pub ood_generalization: f64,
    /// Weighted composite score (0.0-1.0)
    pub composite_score: f64,
    /// Per-metric weighted contributions
    pub weighted_breakdown: WeightedBreakdown,
    /// Total entries evaluated
    pub total: usize,
    /// Gap between static and dynamic eval (anti-overfitting)
    pub static_dynamic_gap: Option<f64>,
    /// Whether model beats keyword baseline (MCC)
    pub model_mcc_vs_keyword: Option<f64>,
}

/// Weighted contribution of each metric to composite score.
#[derive(Debug, Clone, Serialize)]
pub struct WeightedBreakdown {
    pub detection_f1: f64,
    pub rule_citation: f64,
    pub cwe_mapping: f64,
    pub fix_validity: f64,
    pub explanation: f64,
    pub ood: f64,
}

/// Run the full 6-metric evaluation harness.
pub fn run_eval(predictions: &[Prediction], ground_truth: &[GroundTruth]) -> EvalResult {
    // Build lookup for ground truth
    let gt_map: std::collections::HashMap<&str, &GroundTruth> =
        ground_truth.iter().map(|gt| (gt.id.as_str(), gt)).collect();

    // 1. Detection F1
    let detection_pairs: Vec<(u8, u8)> = predictions
        .iter()
        .filter_map(|p| {
            gt_map.get(p.id.as_str()).map(|gt| {
                let pred_label = if p.classification == "unsafe" {
                    1u8
                } else {
                    0u8
                };
                (pred_label, gt.label)
            })
        })
        .collect();
    let detection = evaluate(&detection_pairs, "model");
    let detection_f1 = detection.f1;

    // 2. Rule Citation
    let rule_citation = compute_rule_citation(predictions, &gt_map);

    // 3. CWE Mapping
    let cwe_mapping = compute_cwe_mapping(predictions, &gt_map);

    // 4. Fix Validity
    let fix_validity = compute_fix_validity(predictions, &gt_map);

    // 5. Explanation Quality (automated proxy: length + structure heuristics)
    let explanation_quality = compute_explanation_quality(predictions);

    // 6. OOD Generalization (placeholder: must be computed with separate OOD dataset)
    let ood_generalization = 0.0;

    // Composite weighted score
    let weighted = WeightedBreakdown {
        detection_f1: detection_f1 * DETECTION_F1_WEIGHT,
        rule_citation: rule_citation * RULE_CITATION_WEIGHT,
        cwe_mapping: cwe_mapping * CWE_MAPPING_WEIGHT,
        fix_validity: fix_validity * FIX_VALIDITY_WEIGHT,
        explanation: explanation_quality * EXPLANATION_WEIGHT,
        ood: ood_generalization * OOD_WEIGHT,
    };

    let composite = weighted.detection_f1
        + weighted.rule_citation
        + weighted.cwe_mapping
        + weighted.fix_validity
        + weighted.explanation
        + weighted.ood;

    EvalResult {
        detection,
        detection_f1,
        rule_citation,
        cwe_mapping,
        fix_validity,
        explanation_quality,
        ood_generalization,
        composite_score: composite,
        weighted_breakdown: weighted,
        total: predictions.len(),
        static_dynamic_gap: None,
        model_mcc_vs_keyword: None,
    }
}

/// Compute rule citation accuracy.
///
/// For each unsafe prediction with ground truth rules, check if model cites
/// at least one correct rule ID.
fn compute_rule_citation(
    predictions: &[Prediction],
    gt_map: &std::collections::HashMap<&str, &GroundTruth>,
) -> f64 {
    let mut correct = 0usize;
    let mut total = 0usize;

    for pred in predictions {
        if let Some(gt) = gt_map.get(pred.id.as_str()) {
            if gt.label == 1 && !gt.rules.is_empty() {
                total += 1;
                // Check if any cited rule matches ground truth
                if pred.cited_rules.iter().any(|r| gt.rules.contains(r)) {
                    correct += 1;
                }
            }
        }
    }

    if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    }
}

/// Compute CWE mapping accuracy.
///
/// For each unsafe prediction with ground truth CWEs, check if model cites
/// at least one correct CWE ID.
fn compute_cwe_mapping(
    predictions: &[Prediction],
    gt_map: &std::collections::HashMap<&str, &GroundTruth>,
) -> f64 {
    let mut correct = 0usize;
    let mut total = 0usize;

    for pred in predictions {
        if let Some(gt) = gt_map.get(pred.id.as_str()) {
            if gt.label == 1 && !gt.cwes.is_empty() {
                total += 1;
                if pred.cited_cwes.iter().any(|c| gt.cwes.contains(c)) {
                    correct += 1;
                }
            }
        }
    }

    if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    }
}

/// Compute fix validity rate.
///
/// For each prediction with a proposed fix, lint the fix and check that
/// the original finding is no longer present.
fn compute_fix_validity(
    predictions: &[Prediction],
    gt_map: &std::collections::HashMap<&str, &GroundTruth>,
) -> f64 {
    let mut valid = 0usize;
    let mut total = 0usize;

    for pred in predictions {
        if let Some(fix) = &pred.proposed_fix {
            if let Some(gt) = gt_map.get(pred.id.as_str()) {
                if gt.label == 1 && !fix.is_empty() {
                    total += 1;
                    // Lint the proposed fix
                    let result = linter::lint_shell(fix);
                    // Check that original rules are no longer firing
                    let remaining_rules: Vec<&str> =
                        result.diagnostics.iter().map(|d| d.code.as_str()).collect();
                    let original_fixed = gt
                        .rules
                        .iter()
                        .all(|r| !remaining_rules.contains(&r.as_str()));
                    if original_fixed {
                        valid += 1;
                    }
                }
            }
        }
    }

    if total > 0 {
        valid as f64 / total as f64
    } else {
        // No fixes to validate — return neutral score
        0.5
    }
}

/// Compute explanation quality (automated proxy).
///
/// Heuristic scoring:
/// - Has structured response (contains rule ID, "unsafe"/"safe")
/// - Minimum length (>50 chars for unsafe explanations)
/// - Contains actionable advice (e.g., "use", "instead", "replace")
#[allow(clippy::if_same_then_else)] // Intentional: safe and unsafe-with-keywords both get 0.25 for different reasons
fn compute_explanation_quality(predictions: &[Prediction]) -> f64 {
    if predictions.is_empty() {
        return 0.0;
    }

    let mut total_score = 0.0;

    for pred in predictions {
        let mut score = 0.0;

        // Has classification keyword
        if pred.explanation.contains("safe") || pred.explanation.contains("unsafe") {
            score += 0.25;
        }

        // Has rule citation
        if !pred.cited_rules.is_empty() {
            score += 0.25;
        }

        // Minimum length for unsafe explanations
        if pred.classification == "unsafe" && pred.explanation.len() > 50 {
            score += 0.25;
        } else if pred.classification == "safe" {
            score += 0.25; // safe explanations can be shorter
        }

        // Contains actionable advice
        let actionable_keywords = ["use", "instead", "replace", "remove", "avoid", "fix"];
        if pred.classification == "unsafe"
            && actionable_keywords
                .iter()
                .any(|kw| pred.explanation.to_lowercase().contains(kw))
        {
            score += 0.25;
        } else if pred.classification == "safe" {
            score += 0.25;
        }

        total_score += score;
    }

    total_score / predictions.len() as f64
}

/// Format eval result as human-readable report.
pub fn format_eval_report(result: &EvalResult) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "ShellSafetyBench Evaluation Report");
    let _ = writeln!(out, "==================================");
    let _ = writeln!(out, "Total entries: {}", result.total);
    let _ = writeln!(out);
    let _ = writeln!(out, "Metrics (weighted):");
    let _ = writeln!(
        out,
        "  Detection F1:      {:.3} (x{:.0}% = {:.3})",
        result.detection_f1,
        DETECTION_F1_WEIGHT * 100.0,
        result.weighted_breakdown.detection_f1
    );
    let _ = writeln!(
        out,
        "  Rule Citation:     {:.3} (x{:.0}% = {:.3})",
        result.rule_citation,
        RULE_CITATION_WEIGHT * 100.0,
        result.weighted_breakdown.rule_citation
    );
    let _ = writeln!(
        out,
        "  CWE Mapping:       {:.3} (x{:.0}% = {:.3})",
        result.cwe_mapping,
        CWE_MAPPING_WEIGHT * 100.0,
        result.weighted_breakdown.cwe_mapping
    );
    let _ = writeln!(
        out,
        "  Fix Validity:      {:.3} (x{:.0}% = {:.3})",
        result.fix_validity,
        FIX_VALIDITY_WEIGHT * 100.0,
        result.weighted_breakdown.fix_validity
    );
    let _ = writeln!(
        out,
        "  Explanation:       {:.3} (x{:.0}% = {:.3})",
        result.explanation_quality,
        EXPLANATION_WEIGHT * 100.0,
        result.weighted_breakdown.explanation
    );
    let _ = writeln!(
        out,
        "  OOD Generalize:    {:.3} (x{:.0}% = {:.3})",
        result.ood_generalization,
        OOD_WEIGHT * 100.0,
        result.weighted_breakdown.ood
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  COMPOSITE SCORE:   {:.3} / 1.000",
        result.composite_score
    );

    if let Some(gap) = result.static_dynamic_gap {
        let _ = writeln!(
            out,
            "  Static-Dynamic Gap: {:.1}% (target: <15%)",
            gap * 100.0
        );
    }
    if let Some(mcc_diff) = result.model_mcc_vs_keyword {
        let _ = writeln!(out, "  MCC vs Keyword:    {:.3} (target: >0)", mcc_diff);
    }

    out
}

/// Deserialization struct for JSONL predictions from external models.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EvalPrediction {
    /// Entry ID
    #[serde(default)]
    pub id: String,
    /// Model classification ("safe" or "unsafe")
    pub classification: String,
    /// Ground truth label (0=safe, 1=unsafe)
    #[serde(default)]
    pub label: u8,
    /// Rule IDs cited
    #[serde(default)]
    pub cited_rules: Vec<String>,
    /// CWE IDs cited
    #[serde(default)]
    pub cited_cwes: Vec<String>,
    /// Proposed fix
    #[serde(default)]
    pub proposed_fix: Option<String>,
    /// Explanation text
    #[serde(default)]
    pub explanation: String,
    /// Original script
    #[serde(default)]
    pub script: String,
    /// Ground truth rules
    #[serde(default)]
    pub ground_truth_rules: Vec<String>,
    /// Ground truth CWEs
    #[serde(default)]
    pub ground_truth_cwes: Vec<String>,
}

/// Simple eval result for CLI output (subset of EvalResult).
#[derive(Debug, Clone, Serialize)]
pub struct SimpleEvalResult {
    pub detection_f1: f64,
    pub rule_citation: f64,
    pub cwe_mapping: f64,
    pub fix_validity: f64,
    pub explanation_quality: f64,
    pub ood_generalization: f64,
    pub weighted_score: f64,
    pub total: usize,
}

/// Evaluate predictions from JSONL file format.
///
/// Each line contains both prediction and ground truth fields.
pub fn evaluate_predictions(preds: &[EvalPrediction]) -> SimpleEvalResult {
    let predictions: Vec<Prediction> = preds
        .iter()
        .enumerate()
        .map(|(i, p)| Prediction {
            id: if p.id.is_empty() {
                format!("SSB-{:05}", i)
            } else {
                p.id.clone()
            },
            classification: p.classification.clone(),
            cited_rules: p.cited_rules.clone(),
            cited_cwes: p.cited_cwes.clone(),
            proposed_fix: p.proposed_fix.clone(),
            explanation: p.explanation.clone(),
        })
        .collect();

    let ground_truth: Vec<GroundTruth> = preds
        .iter()
        .enumerate()
        .map(|(i, p)| GroundTruth {
            id: if p.id.is_empty() {
                format!("SSB-{:05}", i)
            } else {
                p.id.clone()
            },
            label: p.label,
            rules: p.ground_truth_rules.clone(),
            cwes: p.ground_truth_cwes.clone(),
            script: p.script.clone(),
        })
        .collect();

    let result = run_eval(&predictions, &ground_truth);

    SimpleEvalResult {
        detection_f1: result.detection_f1,
        rule_citation: result.rule_citation,
        cwe_mapping: result.cwe_mapping,
        fix_validity: result.fix_validity,
        explanation_quality: result.explanation_quality,
        ood_generalization: result.ood_generalization,
        weighted_score: result.composite_score,
        total: result.total,
    }
}

#[cfg(test)]
#[path = "eval_harness_tests_extracted.rs"]
mod tests_extracted;
