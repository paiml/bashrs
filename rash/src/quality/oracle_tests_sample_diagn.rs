
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

    // Add mostly rejected (below baseline)
    for _ in 0..3 {
        detector.record(true);
    }
    for _ in 0..7 {
        detector.record(false);
    }

    let status = detector.detect_drift();
    assert!(status.needs_retrain());
}

#[test]
fn test_ml_010_oracle_integration() {
    let oracle = Oracle::new();

    let diag = sample_diagnostic("SC2086", "Double quote to prevent globbing");
    let result = oracle.classify(&diag, "echo $x");

    assert_eq!(result.category, ShellErrorCategory::MissingQuotes);

    let patterns = oracle.get_patterns(ShellErrorCategory::MissingQuotes);
    assert!(!patterns.is_empty());
}

#[test]
fn test_ml_007_feature_vector_to_vec() {
    let features = FeatureVector {
        code_numeric: 2086,
        message_length: 30,
        has_variable_reference: true,
        ..Default::default()
    };

    let vec = features.to_vec();
    assert!(!vec.is_empty());
    assert_eq!(vec[0], 2086.0);
    assert_eq!(vec[1], 30.0);
}

#[test]
fn test_shell_error_category_names() {
    assert_eq!(
        ShellErrorCategory::CommandInjection.name(),
        "Command Injection"
    );
    assert_eq!(ShellErrorCategory::MissingQuotes.name(), "Missing Quotes");
    assert_eq!(
        ShellErrorCategory::NonDeterministicRandom.name(),
        "Non-Deterministic Random"
    );
}

include!("oracle_tests_extracted_shell.rs");
