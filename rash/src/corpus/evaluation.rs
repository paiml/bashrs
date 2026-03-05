//! Binary classification evaluation metrics (SSC v11 Section 5.5).
//!
//! Computes MCC, accuracy, precision, recall, F1, and confusion matrix
//! for evaluating classifier performance against baselines.

use serde::Serialize;

/// Confusion matrix for binary classification.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConfusionMatrix {
    pub tp: usize, // true positive (predicted unsafe, actually unsafe)
    pub fp: usize, // false positive (predicted unsafe, actually safe)
    pub tn: usize, // true negative (predicted safe, actually safe)
    pub fn_: usize, // false negative (predicted safe, actually unsafe)
}

impl ConfusionMatrix {
    pub fn total(&self) -> usize {
        self.tp + self.fp + self.tn + self.fn_
    }
}

/// Full evaluation report for a binary classifier.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationReport {
    pub name: String,
    pub confusion: ConfusionMatrix,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub mcc: f64,
    pub total: usize,
    pub beats_majority: bool,
}

/// Compute evaluation metrics from predictions and ground truth.
///
/// - `predictions`: slice of (predicted_label, true_label) pairs
///   where 0=safe, 1=unsafe
/// - `name`: identifier for this classifier/baseline
pub fn evaluate(predictions: &[(u8, u8)], name: &str) -> EvaluationReport {
    let mut cm = ConfusionMatrix::default();

    for &(pred, truth) in predictions {
        match (pred, truth) {
            (1, 1) => cm.tp += 1,
            (1, 0) => cm.fp += 1,
            (0, 0) => cm.tn += 1,
            (0, 1) => cm.fn_ += 1,
            _ => {}
        }
    }

    let total = cm.total();
    let accuracy = if total > 0 {
        (cm.tp + cm.tn) as f64 / total as f64
    } else {
        0.0
    };

    let precision = if cm.tp + cm.fp > 0 {
        cm.tp as f64 / (cm.tp + cm.fp) as f64
    } else {
        0.0
    };

    let recall = if cm.tp + cm.fn_ > 0 {
        cm.tp as f64 / (cm.tp + cm.fn_) as f64
    } else {
        0.0
    };

    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    let mcc = compute_mcc(&cm);

    // Majority baseline: predict all safe (MCC=0, accuracy=safe_pct)
    let majority_accuracy = if total > 0 {
        (cm.tn + cm.fn_) as f64 / total as f64 // all safe
    } else {
        0.0
    };
    // Spec Section 5.5: accuracy must beat 93.5% majority baseline
    let beats_majority = accuracy > majority_accuracy && accuracy > 0.935;

    EvaluationReport {
        name: name.to_string(),
        confusion: cm,
        accuracy,
        precision,
        recall,
        f1,
        mcc,
        total,
        beats_majority,
    }
}

/// Matthew's Correlation Coefficient.
///
/// MCC ranges from -1 (total disagreement) to +1 (perfect prediction).
/// MCC = 0 is equivalent to random guessing.
/// SSC v11 Section 5.5: CI lower bound > 0.2.
fn compute_mcc(cm: &ConfusionMatrix) -> f64 {
    let tp = cm.tp as f64;
    let fp = cm.fp as f64;
    let tn = cm.tn as f64;
    let fn_ = cm.fn_ as f64;

    let numerator = tp * tn - fp * fn_;
    let denominator = ((tp + fp) * (tp + fn_) * (tn + fp) * (tn + fn_)).sqrt();

    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Format evaluation report as a human-readable string.
pub fn format_report(report: &EvaluationReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "  Classifier: {}", report.name);
    let _ = writeln!(out, "  Total:      {}", report.total);
    let _ = writeln!(
        out,
        "  Confusion:  TP={} FP={} TN={} FN={}",
        report.confusion.tp, report.confusion.fp, report.confusion.tn, report.confusion.fn_
    );
    let _ = writeln!(out, "  Accuracy:   {:.3} (target: >0.935)", report.accuracy);
    let _ = writeln!(out, "  Precision:  {:.3}", report.precision);
    let _ = writeln!(
        out,
        "  Recall:     {:.3} (target: >=0.60)",
        report.recall
    );
    let _ = writeln!(out, "  F1:         {:.3}", report.f1);
    let _ = writeln!(
        out,
        "  MCC:        {:.3} (target: CI lower >0.2)",
        report.mcc
    );
    let _ = writeln!(
        out,
        "  Beats majority: {}",
        if report.beats_majority { "yes" } else { "no" }
    );

    out
}

/// Compare multiple evaluation reports side by side.
pub fn format_comparison(reports: &[EvaluationReport]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(
        out,
        "  {:<25} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Classifier", "Acc", "Prec", "Recall", "F1", "MCC"
    );
    let _ = writeln!(out, "  {}", "-".repeat(73));

    for r in reports {
        let _ = writeln!(
            out,
            "  {:<25} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3}",
            r.name, r.accuracy, r.precision, r.recall, r.f1, r.mcc
        );
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_classifier() {
        let preds = vec![(1, 1), (0, 0), (1, 1), (0, 0)];
        let report = evaluate(&preds, "perfect");
        assert!((report.accuracy - 1.0).abs() < 1e-9);
        assert!((report.mcc - 1.0).abs() < 1e-9);
        assert!((report.precision - 1.0).abs() < 1e-9);
        assert!((report.recall - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_random_classifier() {
        // All same prediction → MCC = 0
        let preds = vec![(0, 1), (0, 0), (0, 1), (0, 0)];
        let report = evaluate(&preds, "all-safe");
        assert!((report.mcc - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_accuracy_calculation() {
        let preds = vec![(1, 1), (0, 0), (0, 1), (1, 0)];
        let report = evaluate(&preds, "mixed");
        assert!((report.accuracy - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_mcc_range() {
        let preds = vec![(1, 1), (0, 1), (1, 0), (0, 0), (1, 1)];
        let report = evaluate(&preds, "test");
        assert!(report.mcc >= -1.0 && report.mcc <= 1.0);
    }

    #[test]
    fn test_empty_predictions() {
        let preds: Vec<(u8, u8)> = vec![];
        let report = evaluate(&preds, "empty");
        assert_eq!(report.total, 0);
        assert!((report.accuracy - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_majority_baseline_check() {
        // 95% safe, 5% unsafe — majority classifier gets 95% accuracy
        let mut preds = Vec::new();
        for _ in 0..95 {
            preds.push((1, 0)); // FP: predicts unsafe, actually safe
        }
        for _ in 0..5 {
            preds.push((1, 1)); // TP: predicts unsafe, actually unsafe
        }
        let report = evaluate(&preds, "all-unsafe");
        // Accuracy = 5/100 = 5% → does NOT beat majority
        assert!(!report.beats_majority);
    }

    #[test]
    fn test_format_report() {
        let preds = vec![(1, 1), (0, 0), (1, 1), (0, 0)];
        let report = evaluate(&preds, "test");
        let formatted = format_report(&report);
        assert!(formatted.contains("Accuracy"));
        assert!(formatted.contains("MCC"));
    }

    #[test]
    fn test_format_comparison() {
        let r1 = evaluate(&[(1, 1), (0, 0)], "baseline");
        let r2 = evaluate(&[(1, 1), (1, 0)], "model");
        let table = format_comparison(&[r1, r2]);
        assert!(table.contains("baseline"));
        assert!(table.contains("model"));
    }

    #[test]
    fn test_confusion_matrix_total() {
        let cm = ConfusionMatrix {
            tp: 10,
            fp: 5,
            tn: 80,
            fn_: 5,
        };
        assert_eq!(cm.total(), 100);
    }
}
