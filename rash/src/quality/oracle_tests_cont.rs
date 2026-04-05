//! Oracle ML-Powered Error Classifier (ML-007, ML-008, ML-009, ML-010)
//!
//! Implements ML-based error classification using k-NN and pattern matching
//! following Toyota Way principles of continuous improvement (Kaizen).
//!
//! # Toyota Way Principles
//!
//! - **Kaizen** (Continuous improvement): Learn from user fix acceptance
//! - **Jidoka** (Automation with human touch): ML classifies, human approves
//! - **Poka-yoke** (Error-proofing): Confidence scores prevent bad auto-fixes
//!
//! # References
//!
//! - BASHRS-SPEC-ML-007: Feature Extraction
//! - BASHRS-SPEC-ML-008: k-NN Classifier
//! - BASHRS-SPEC-ML-009: Pattern Library
//! - Kim et al. (2013): Bug localization with learning-to-rank
//! - Le et al. (2016): Learning-to-rank fault localization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::linter::{Diagnostic, Severity};

/// ML-classified error categories for shell scripts

/// k-NN classifier for error categorization (ML-008)
pub struct KnnClassifier {
    k: usize,
    training_data: Vec<(FeatureVector, ShellErrorCategory)>,
}

impl KnnClassifier {
    /// Create a new k-NN classifier
    pub fn new(k: usize) -> Self {
        Self {
            k,
            training_data: Vec::new(),
        }
    }

    /// Add training example
    pub fn add_example(&mut self, features: FeatureVector, category: ShellErrorCategory) {
        self.training_data.push((features, category));
    }

    /// Classify a diagnostic
    pub fn classify(&self, features: &FeatureVector) -> ClassificationResult {
        if self.training_data.is_empty() {
            // Fall back to rule-based classification
            return self.rule_based_classify(features);
        }

        let target = features.to_vec();
        let mut distances: Vec<(f64, ShellErrorCategory)> = self
            .training_data
            .iter()
            .map(|(f, cat)| (self.euclidean_distance(&target, &f.to_vec()), *cat))
            .collect();

        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Count votes from k nearest neighbors
        let mut votes: HashMap<ShellErrorCategory, usize> = HashMap::new();
        for (_, cat) in distances.iter().take(self.k) {
            *votes.entry(*cat).or_default() += 1;
        }

        // Find majority
        let (category, vote_count) = votes
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .unwrap_or((ShellErrorCategory::Unknown, 0));

        let confidence = if self.k > 0 {
            vote_count as f64 / self.k as f64
        } else {
            0.0
        };

        ClassificationResult {
            category,
            confidence,
            method: "k-NN".to_string(),
        }
    }

    /// Rule-based fallback classification
    fn rule_based_classify(&self, features: &FeatureVector) -> ClassificationResult {
        let category = classify_by_prefix(features);

        ClassificationResult {
            category,
            confidence: 0.85, // Rule-based has fixed confidence
            method: "rule-based".to_string(),
        }
    }

    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Classify an error category based on the code prefix
fn classify_by_prefix(features: &FeatureVector) -> ShellErrorCategory {
    match features.code_prefix.to_uppercase().as_str() {
        "SEC" => classify_sec(features),
        "DET" => classify_det(features),
        "IDEM" => classify_idem(features),
        "SC" => classify_sc(features),
        _ => ShellErrorCategory::Unknown,
    }
}

/// Classify security (SEC) errors by code number
fn classify_sec(features: &FeatureVector) -> ShellErrorCategory {
    if features.code_numeric == 1 || features.code_numeric == 2 {
        ShellErrorCategory::CommandInjection
    } else if features.code_numeric == 10 {
        ShellErrorCategory::PathTraversal
    } else {
        ShellErrorCategory::UnsafeExpansion
    }
}

/// Classify determinism (DET) errors by operation type
fn classify_det(features: &FeatureVector) -> ShellErrorCategory {
    if features.random_operation {
        ShellErrorCategory::NonDeterministicRandom
    } else if features.date_time_operation {
        ShellErrorCategory::TimestampUsage
    } else if features.process_operation {
        ShellErrorCategory::ProcessIdDependency
    } else {
        ShellErrorCategory::NonDeterministicRandom
    }
}

/// Classify idempotency (IDEM) errors by write status
fn classify_idem(features: &FeatureVector) -> ShellErrorCategory {
    if features.is_write_operation {
        ShellErrorCategory::UnsafeOverwrite
    } else {
        ShellErrorCategory::NonIdempotentOperation
    }
}

/// Classify shellcheck (SC) errors by code number and features
fn classify_sc(features: &FeatureVector) -> ShellErrorCategory {
    if features.code_numeric == 2086 {
        ShellErrorCategory::MissingQuotes
    } else if features.has_glob {
        ShellErrorCategory::GlobbingRisk
    } else {
        ShellErrorCategory::WordSplitting
    }
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: ShellErrorCategory,
    pub confidence: f64,
    pub method: String,
}

/// Drift detector for monitoring fix acceptance (ML-010)
pub struct DriftDetector {
    window_size: usize,
    acceptance_history: Vec<bool>,
    baseline_rate: f64,
    drift_threshold: f64,
}

impl DriftDetector {
    /// Create a new drift detector
    pub fn new(window_size: usize, baseline_rate: f64, drift_threshold: f64) -> Self {
        Self {
            window_size,
            acceptance_history: Vec::new(),
            baseline_rate,
            drift_threshold,
        }
    }

    /// Record a fix acceptance/rejection
    pub fn record(&mut self, accepted: bool) {
        self.acceptance_history.push(accepted);
        if self.acceptance_history.len() > self.window_size {
            self.acceptance_history.remove(0);
        }
    }

    /// Check if drift has been detected
    pub fn detect_drift(&self) -> DriftStatus {
        if self.acceptance_history.len() < self.window_size / 2 {
            return DriftStatus::InsufficientData;
        }

        let current_rate = self.current_acceptance_rate();
        let drift = (current_rate - self.baseline_rate).abs();

        if drift > self.drift_threshold {
            if current_rate < self.baseline_rate {
                DriftStatus::NegativeDrift {
                    baseline: self.baseline_rate,
                    current: current_rate,
                }
            } else {
                DriftStatus::PositiveDrift {
                    baseline: self.baseline_rate,
                    current: current_rate,
                }
            }
        } else {
            DriftStatus::Stable { rate: current_rate }
        }
    }

    /// Get current acceptance rate
    pub fn current_acceptance_rate(&self) -> f64 {
        if self.acceptance_history.is_empty() {
            return self.baseline_rate;
        }

        let accepted = self.acceptance_history.iter().filter(|&&x| x).count();
        accepted as f64 / self.acceptance_history.len() as f64
    }

    /// Update baseline rate
    pub fn update_baseline(&mut self, new_baseline: f64) {
        self.baseline_rate = new_baseline;
    }
}

/// Drift detection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftStatus {
    InsufficientData,
    Stable { rate: f64 },
    PositiveDrift { baseline: f64, current: f64 },
    NegativeDrift { baseline: f64, current: f64 },
}

impl DriftStatus {
    /// Check if retraining is recommended
    pub fn needs_retrain(&self) -> bool {
        matches!(self, DriftStatus::NegativeDrift { .. })
    }
}

/// Oracle system combining all ML components
pub struct Oracle {
    classifier: KnnClassifier,
    patterns: Vec<FixPattern>,
    drift_detector: DriftDetector,
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oracle {
    /// Create a new Oracle with default settings
    pub fn new() -> Self {
        Self {
            classifier: KnnClassifier::new(5),
            patterns: bootstrap_patterns(),
            drift_detector: DriftDetector::new(100, 0.85, 0.15),
        }
    }

    /// Classify a diagnostic
    pub fn classify(&self, diagnostic: &Diagnostic, source: &str) -> ClassificationResult {
        let features = FeatureVector::extract(diagnostic, source);
        self.classifier.classify(&features)
    }

    /// Get suggested fix patterns for a category
    pub fn get_patterns(&self, category: ShellErrorCategory) -> Vec<&FixPattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get the best pattern for a category
    pub fn best_pattern(&self, category: ShellErrorCategory) -> Option<&FixPattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Record fix acceptance
    pub fn record_fix_result(&mut self, pattern_id: &str, accepted: bool) {
        if let Some(pattern) = self.patterns.iter_mut().find(|p| p.id == pattern_id) {
            if accepted {
                pattern.record_accepted();
            } else {
                pattern.record_rejected();
            }
        }
        self.drift_detector.record(accepted);
    }

    /// Check drift status
    pub fn drift_status(&self) -> DriftStatus {
        self.drift_detector.detect_drift()
    }

    /// Get all patterns
    pub fn all_patterns(&self) -> &[FixPattern] {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Span;

    fn sample_diagnostic(code: &str, message: &str) -> Diagnostic {
        Diagnostic {
            code: code.to_string(),
            severity: Severity::Warning,
            message: message.to_string(),
            span: Span::new(10, 5, 10, 20),
            fix: None,
        }
    }

    #[test]
    fn test_ml_007_feature_extraction_basic() {
        let diag = sample_diagnostic("SC2086", "Double quote to prevent globbing");
        let source = "echo $var";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "SC");
        assert_eq!(features.code_numeric, 2086);
        assert!(features.message_length > 0);
        assert_eq!(features.operation_type, "shellcheck");
    }

    #[test]
    fn test_ml_007_feature_extraction_determinism() {
        let diag = sample_diagnostic("DET001", "Non-deterministic use of $RANDOM");
        let source = "x=$RANDOM";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "DET");
        assert_eq!(features.code_numeric, 1);
        assert!(!features.is_deterministic);
        assert!(features.random_operation);
    }

    #[test]
    fn test_ml_007_feature_extraction_security() {
        let diag = sample_diagnostic("SEC010", "Hardcoded path /tmp detected");
        let source = "cd /tmp";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "SEC");
        assert!(features.has_path_reference);
        assert!(features.has_side_effects);
    }

    #[test]
    fn test_ml_008_knn_rule_based() {
        let classifier = KnnClassifier::new(5);

        // Without training data, falls back to rule-based
        let diag = sample_diagnostic("SC2086", "Quote this");
        let features = FeatureVector::extract(&diag, "echo $x");

        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::MissingQuotes);
        assert_eq!(result.method, "rule-based");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_ml_008_knn_with_training() {
        let mut classifier = KnnClassifier::new(3);

        // Add training examples
        for _ in 0..5 {
            let diag = sample_diagnostic("SC2086", "Quote variable");
            let features = FeatureVector::extract(&diag, "echo $x");
            classifier.add_example(features, ShellErrorCategory::MissingQuotes);
        }

        let diag = sample_diagnostic("SC2086", "Quote variable expansion");
        let features = FeatureVector::extract(&diag, "echo $var");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::MissingQuotes);
        assert_eq!(result.method, "k-NN");
    }

    #[test]
    fn test_ml_009_bootstrap_patterns() {
        let patterns = bootstrap_patterns();

        assert_eq!(patterns.len(), 15);

        // Check categories are distributed
        let quoting = patterns
            .iter()
            .filter(|p| p.category == ShellErrorCategory::MissingQuotes)
            .count();
        let determinism = patterns
            .iter()
            .filter(|p| p.category == ShellErrorCategory::NonDeterministicRandom)
            .count();

        assert!(quoting > 0);
        assert!(determinism > 0);
    }

    #[test]
    fn test_ml_009_pattern_tracking() {
        let mut pattern = FixPattern::new(
            "TEST-001",
            ShellErrorCategory::MissingQuotes,
            "test",
            r"\$x",
            "\"$x\"",
            "Test pattern",
        );

        assert_eq!(pattern.total_applications, 0);

        pattern.record_accepted();
        pattern.record_accepted();
        pattern.record_rejected();

        assert_eq!(pattern.total_applications, 3);
        assert_eq!(pattern.accepted_count, 2);
        assert_eq!(pattern.rejected_count, 1);
        assert!((pattern.success_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_ml_010_drift_detection_stable() {
        let mut detector = DriftDetector::new(10, 0.8, 0.2);

        // Add mostly accepted (matching baseline)
        for _ in 0..8 {
            detector.record(true);
        }
        for _ in 0..2 {
            detector.record(false);
        }

        match detector.detect_drift() {
            DriftStatus::Stable { rate } => assert!((rate - 0.8).abs() < 0.1),
            _ => panic!("Expected stable status"),
        }
    }

    #[test]
    fn test_ml_010_drift_detection_negative() {
        let mut detector = DriftDetector::new(10, 0.9, 0.2);


        include!("oracle_part3_incl2.rs");
