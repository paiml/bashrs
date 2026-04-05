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

    #[test]
    fn test_shell_error_category_severity() {
        assert_eq!(
            ShellErrorCategory::CommandInjection.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::StyleViolation.default_severity(),
            Severity::Info
        );
        assert_eq!(
            ShellErrorCategory::MissingQuotes.default_severity(),
            Severity::Warning
        );
    }

    // ===== Additional tests for coverage =====

    #[test]
    fn test_shell_error_category_all_names() {
        assert_eq!(ShellErrorCategory::PathTraversal.name(), "Path Traversal");
        assert_eq!(
            ShellErrorCategory::UnsafeExpansion.name(),
            "Unsafe Expansion"
        );
        assert_eq!(ShellErrorCategory::TimestampUsage.name(), "Timestamp Usage");
        assert_eq!(
            ShellErrorCategory::ProcessIdDependency.name(),
            "Process ID Dependency"
        );
        assert_eq!(
            ShellErrorCategory::NonIdempotentOperation.name(),
            "Non-Idempotent Operation"
        );
        assert_eq!(ShellErrorCategory::MissingGuard.name(), "Missing Guard");
        assert_eq!(
            ShellErrorCategory::UnsafeOverwrite.name(),
            "Unsafe Overwrite"
        );
        assert_eq!(ShellErrorCategory::GlobbingRisk.name(), "Globbing Risk");
        assert_eq!(ShellErrorCategory::WordSplitting.name(), "Word Splitting");
        assert_eq!(ShellErrorCategory::SyntaxError.name(), "Syntax Error");
        assert_eq!(ShellErrorCategory::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_shell_error_category_all_severities() {
        assert_eq!(
            ShellErrorCategory::PathTraversal.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::UnsafeExpansion.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::TimestampUsage.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::ProcessIdDependency.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::NonIdempotentOperation.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::MissingGuard.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::UnsafeOverwrite.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::GlobbingRisk.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::WordSplitting.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::SyntaxError.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::Unknown.default_severity(),
            Severity::Info
        );
    }

    #[test]
    fn test_feature_extraction_idem_prefix() {
        let diag = sample_diagnostic("IDEM001", "Non-idempotent mkdir operation");
        let source = "mkdir /tmp/test";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "IDEM");
        assert_eq!(features.code_numeric, 1);
        assert!(!features.is_idempotent);
        assert!(features.has_side_effects);
        assert_eq!(features.operation_type, "idempotency");
    }

    #[test]
    fn test_feature_extraction_unknown_prefix() {
        let diag = sample_diagnostic("XYZ123", "Some unknown rule");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "XYZ");
        assert_eq!(features.code_numeric, 123);
        assert_eq!(features.operation_type, "unknown");
    }

    #[test]
    fn test_feature_extraction_det_date() {
        let diag = sample_diagnostic("DET002", "Non-deterministic use of date command");
        let source = "date=$(date +%Y%m%d)";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "DET");
        assert!(!features.is_deterministic);
        assert!(features.date_time_operation);
    }

    #[test]
    fn test_feature_extraction_det_pid() {
        let diag = sample_diagnostic("DET003", "Non-deterministic use of pid $$");
        let source = "echo $$";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "DET");
        assert!(features.process_operation);
    }

    #[test]
    fn test_feature_extraction_lexical_features() {
        let diag = sample_diagnostic(
            "SC2086",
            "Quote $var expansion in /path/* to prevent globbing",
        );
        let source = "echo $HOME/*.txt | cat > output.log";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_variable_reference); // $var
        assert!(features.has_path_reference); // / in message
        assert!(features.has_glob); // * in message
        assert!(features.word_count > 0);
        assert!(features.special_char_count > 0);
    }

    #[test]
    fn test_feature_extraction_command_sub() {
        let diag = sample_diagnostic("SC2086", "Quote $(command) substitution");
        let source = "x=$(echo test)";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_command_reference);
        assert!(features.has_subshell);
    }

    #[test]
    fn test_feature_extraction_backtick_command() {
        let diag = sample_diagnostic("SC2086", "Quote `command` backtick");
        let source = "x=`echo test`";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_command_reference);
        assert!(features.has_subshell);
    }

    #[test]
    fn test_feature_extraction_array() {
        let diag = sample_diagnostic("SC2086", "Array element [0] needs quoting");
        let source = "echo ${arr[0]}";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_array_reference);
    }

    #[test]
    fn test_feature_extraction_arithmetic() {
        let diag = sample_diagnostic("SC2086", "Arithmetic $((x+1)) expansion");
        let source = "x=$((a + b))";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_arithmetic);
    }

    #[test]
    fn test_feature_extraction_pipe_redirect() {
        let diag = sample_diagnostic("SC2086", "Pipe | and redirect > operations");
        let source = "cat file | grep test > output";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_pipe);
        assert!(features.has_redirect);
    }

    #[test]
    fn test_feature_extraction_quotes() {
        let diag = sample_diagnostic("SC2086", "Missing quotes around \"var\" and 'literal'");
        let source = "echo \"$var\"";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_quote_chars);
    }

    #[test]
    fn test_feature_extraction_file_extension() {
        let diag = sample_diagnostic("SC2086", "Script file.sh needs quoting");
        let source = "bash script.sh";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_file_extension);
    }

    #[test]
    fn test_feature_extraction_url() {
        let diag = sample_diagnostic("SC2086", "Download from https://example.com");
        let source = "curl https://example.com";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_url);
    }

    #[test]
    fn test_feature_extraction_multiline() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Multiline span".to_string(),
            span: Span::new(1, 1, 5, 10),
            fix: None,
        };
        let source = "line1\nline2\nline3\nline4\nline5";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.is_multiline);
    }

    #[test]
    fn test_feature_extraction_continuation() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Line continuation".to_string(),
            span: Span::new(1, 1, 1, 20),
            fix: None,
        };
        let source = "echo hello world \\";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_continuation);
    }

    #[test]
    fn test_feature_extraction_uppercase_ratio() {
        let diag = sample_diagnostic("SC2086", "ALL CAPS MESSAGE");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        // "ALL CAPS MESSAGE" has high uppercase ratio
        assert!(features.uppercase_ratio > 0.5);
    }

    #[test]
    fn test_feature_extraction_digit_ratio() {
        let diag = sample_diagnostic("SC2086", "Error 12345 on line 67890");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        // Message has many digits
        assert!(features.digit_ratio > 0.0);
    }

    #[test]
    fn test_feature_extraction_empty_message() {
        let diag = sample_diagnostic("SC2086", "");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.message_length, 0);
        assert_eq!(features.uppercase_ratio, 0.0);
        assert_eq!(features.digit_ratio, 0.0);
    }

    #[test]
    fn test_feature_extraction_sc_string_operation() {
        let diag = sample_diagnostic("SC2086", "Double quote to prevent word splitting");
        let source = "echo $x";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.string_operation);
    }

    #[test]
    fn test_knn_rule_based_sec_injection() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SEC001", "Command injection via eval");
        let features = FeatureVector::extract(&diag, "eval $cmd");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::CommandInjection);
    }

    #[test]
    fn test_knn_rule_based_sec_traversal() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SEC010", "Path traversal detected");
        let features = FeatureVector::extract(&diag, "cat ../../../etc/passwd");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::PathTraversal);
    }

    #[test]
    fn test_knn_rule_based_sec_other() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SEC099", "Other security issue");
        let features = FeatureVector::extract(&diag, "chmod 777 file");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::UnsafeExpansion);
    }

    #[test]
    fn test_knn_rule_based_det_default() {
        let classifier = KnnClassifier::new(5);

        // DET without specific flags falls back to random
        let diag = sample_diagnostic("DET099", "Unknown determinism issue");
        let mut features = FeatureVector::extract(&diag, "echo test");
        features.random_operation = false;
        features.date_time_operation = false;
        features.process_operation = false;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::NonDeterministicRandom);
    }

    #[test]
    fn test_knn_rule_based_idem_write() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("IDEM001", "Non-idempotent write");
        let mut features = FeatureVector::extract(&diag, "echo > file");
        features.is_write_operation = true;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::UnsafeOverwrite);
    }

    #[test]
    fn test_knn_rule_based_idem_non_write() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("IDEM002", "Non-idempotent operation");
        let mut features = FeatureVector::extract(&diag, "mkdir /tmp/test");
        features.is_write_operation = false;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::NonIdempotentOperation);
    }

    #[test]
    fn test_knn_rule_based_sc_glob() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SC2035", "Glob * may expand incorrectly");
        let mut features = FeatureVector::extract(&diag, "ls *.txt");
        features.has_glob = true;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::GlobbingRisk);
    }

    #[test]
    fn test_knn_rule_based_sc_word_split() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SC2087", "Word splitting issue");
        let mut features = FeatureVector::extract(&diag, "echo $x");
        features.has_glob = false;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::WordSplitting);
    }

    #[test]
    fn test_knn_rule_based_unknown() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("UNKNOWN123", "Unknown rule");
        let features = FeatureVector::extract(&diag, "echo test");

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::Unknown);
    }

    #[test]
    fn test_knn_euclidean_distance() {
        let classifier = KnnClassifier::new(5);

        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];

        let distance = classifier.euclidean_distance(&a, &b);
        assert!((distance - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_drift_detector_insufficient_data() {
        let detector = DriftDetector::new(10, 0.8, 0.2);

        // No data recorded yet
        match detector.detect_drift() {
            DriftStatus::InsufficientData => (),
            _ => panic!("Expected InsufficientData"),
        }
    }

    #[test]
    fn test_drift_detector_positive_drift() {
        let mut detector = DriftDetector::new(10, 0.5, 0.2);

        // All accepted (above baseline of 0.5)
        for _ in 0..10 {
            detector.record(true);
        }

        match detector.detect_drift() {
            DriftStatus::PositiveDrift { baseline, current } => {
                assert!((baseline - 0.5).abs() < 0.01);
                assert!((current - 1.0).abs() < 0.01);
            }
            other => panic!("Expected PositiveDrift, got {:?}", other),
        }
    }

    #[test]
    fn test_drift_detector_update_baseline() {
        let mut detector = DriftDetector::new(10, 0.8, 0.2);

        detector.update_baseline(0.9);
        assert!((detector.current_acceptance_rate() - 0.9).abs() < 0.01); // Returns baseline when empty
    }

    #[test]
    fn test_drift_detector_window_overflow() {
        let mut detector = DriftDetector::new(5, 0.8, 0.2);

        // Add more than window size
        for _ in 0..10 {
            detector.record(true);
        }

        // Should only keep last 5
        assert_eq!(detector.acceptance_history.len(), 5);
    }

    #[test]
    fn test_drift_status_needs_retrain() {
        assert!(!DriftStatus::InsufficientData.needs_retrain());
        assert!(!DriftStatus::Stable { rate: 0.8 }.needs_retrain());
        assert!(!DriftStatus::PositiveDrift {
            baseline: 0.8,
            current: 0.9
        }
        .needs_retrain());
        assert!(DriftStatus::NegativeDrift {
            baseline: 0.8,
            current: 0.5
        }
        .needs_retrain());
    }

    #[test]
    fn test_fix_pattern_new() {
        let pattern = FixPattern::new(
            "TEST-001",
            ShellErrorCategory::MissingQuotes,
            "test_pattern",
            r"\$x",
            "\"$x\"",
            "Test description",
        );

        assert_eq!(pattern.id, "TEST-001");
        assert_eq!(pattern.category, ShellErrorCategory::MissingQuotes);
        assert_eq!(pattern.pattern_name, "test_pattern");
        assert_eq!(pattern.regex_match, r"\$x");
        assert_eq!(pattern.replacement_template, "\"$x\"");
        assert_eq!(pattern.description, "Test description");
        assert_eq!(pattern.total_applications, 0);
        assert!((pattern.confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_fix_pattern_confidence_calculation() {
        let mut pattern = FixPattern::new(
            "TEST-002",
            ShellErrorCategory::MissingQuotes,
            "test",
            r"\$x",
            "\"$x\"",
            "Test",
        );

        // Record 10 applications: 9 accepted, 1 rejected
        for _ in 0..9 {
            pattern.record_accepted();
        }
        pattern.record_rejected();

        assert_eq!(pattern.total_applications, 10);
        assert_eq!(pattern.accepted_count, 9);
        assert_eq!(pattern.rejected_count, 1);
        assert!((pattern.success_rate - 0.9).abs() < 0.01);
        // Confidence should be less than success_rate due to Bayesian update
        assert!(pattern.confidence < pattern.success_rate);
        assert!(pattern.confidence > 0.0);
    }

    #[test]
    fn test_oracle_default() {
        let oracle = Oracle::default();

        assert!(!oracle.all_patterns().is_empty());
    }

    #[test]
    fn test_oracle_best_pattern() {
        let oracle = Oracle::new();

        let best = oracle.best_pattern(ShellErrorCategory::MissingQuotes);
        assert!(best.is_some());
        let pattern = best.unwrap();
        assert_eq!(pattern.category, ShellErrorCategory::MissingQuotes);
    }

    #[test]
    fn test_oracle_best_pattern_none() {
        let oracle = Oracle::new();

        // SyntaxError has no patterns in bootstrap
        let best = oracle.best_pattern(ShellErrorCategory::SyntaxError);
        assert!(best.is_none());
    }

    #[test]
    fn test_oracle_record_fix_result_accepted() {
        let mut oracle = Oracle::new();

        // Get initial state of a pattern
        let initial_accepted = oracle.all_patterns()[0].accepted_count;

        // Record accepted fix for first pattern
        let pattern_id = oracle.all_patterns()[0].id.clone();
        oracle.record_fix_result(&pattern_id, true);

        // Find pattern and verify it was updated
        let pattern = oracle
            .all_patterns()
            .iter()
            .find(|p| p.id == pattern_id)
            .unwrap();
        assert_eq!(pattern.accepted_count, initial_accepted + 1);
    }

    #[test]
    fn test_oracle_record_fix_result_rejected() {
        let mut oracle = Oracle::new();

        let initial_rejected = oracle.all_patterns()[0].rejected_count;
        let pattern_id = oracle.all_patterns()[0].id.clone();
        oracle.record_fix_result(&pattern_id, false);

        let pattern = oracle
            .all_patterns()
            .iter()
            .find(|p| p.id == pattern_id)
            .unwrap();
        assert_eq!(pattern.rejected_count, initial_rejected + 1);
    }

    #[test]
    fn test_oracle_record_fix_result_unknown_pattern() {
        let mut oracle = Oracle::new();

        // Recording for unknown pattern should not panic
        oracle.record_fix_result("NONEXISTENT-999", true);
    }

    #[test]
    fn test_oracle_drift_status() {
        let oracle = Oracle::new();

        // Should return InsufficientData initially
        match oracle.drift_status() {
            DriftStatus::InsufficientData => (),
            _ => panic!("Expected InsufficientData for new Oracle"),
        }
    }

    #[test]
    fn test_oracle_classify_det() {
        let oracle = Oracle::new();

        let diag = sample_diagnostic("DET001", "Non-deterministic $RANDOM usage");
        let result = oracle.classify(&diag, "x=$RANDOM");

        assert_eq!(result.category, ShellErrorCategory::NonDeterministicRandom);
    }

    #[test]
    fn test_oracle_classify_sec() {
        let oracle = Oracle::new();

        let diag = sample_diagnostic("SEC001", "Command injection risk");
        let result = oracle.classify(&diag, "eval $cmd");

        assert_eq!(result.category, ShellErrorCategory::CommandInjection);
    }

    #[test]
    fn test_oracle_get_patterns_multiple() {
        let oracle = Oracle::new();

        let patterns = oracle.get_patterns(ShellErrorCategory::MissingQuotes);

        // Should have multiple quote-related patterns
        assert!(patterns.len() >= 3);
    }

    #[test]
    fn test_oracle_get_patterns_empty() {
        let oracle = Oracle::new();

        let patterns = oracle.get_patterns(ShellErrorCategory::SyntaxError);

        assert!(patterns.is_empty());
    }

    #[test]
    fn test_classification_result_clone() {
        let result = ClassificationResult {
            category: ShellErrorCategory::MissingQuotes,
            confidence: 0.95,
            method: "k-NN".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(cloned.category, ShellErrorCategory::MissingQuotes);
        assert!((cloned.confidence - 0.95).abs() < 0.001);
        assert_eq!(cloned.method, "k-NN");
    }

    #[test]
    fn test_fix_pattern_clone() {
        let pattern = FixPattern::new(
            "TEST-003",
            ShellErrorCategory::MissingQuotes,
            "test",
            r"\$x",
            "\"$x\"",
            "Test",
        );

        let cloned = pattern.clone();
        assert_eq!(cloned.id, pattern.id);
        assert_eq!(cloned.category, pattern.category);
    }

    #[test]
    fn test_feature_vector_default() {
        let features = FeatureVector::default();

        assert!(features.code_prefix.is_empty());
        assert_eq!(features.code_numeric, 0);
        assert_eq!(features.message_length, 0);
        assert!(!features.has_variable_reference);
    }

    #[test]
    fn test_feature_vector_clone() {
        let features = FeatureVector {
            code_prefix: "SC".to_string(),
            code_numeric: 2086,
            message_length: 100,
            ..Default::default()
        };

        let cloned = features.clone();
        assert_eq!(cloned.code_prefix, "SC");
        assert_eq!(cloned.code_numeric, 2086);
    }

    #[test]
    fn test_knn_k_zero() {
        let classifier = KnnClassifier::new(0);

        let diag = sample_diagnostic("SC2086", "Test");
        let features = FeatureVector::extract(&diag, "echo $x");
        let result = classifier.classify(&features);

        // With k=0 and no training data, should fall back to rule-based
        assert_eq!(result.method, "rule-based");
    }

    #[test]
    fn test_bootstrap_patterns_categories() {
        let patterns = bootstrap_patterns();

        // Check all expected categories are present
        let categories: std::collections::HashSet<_> =
            patterns.iter().map(|p| p.category).collect();

        assert!(categories.contains(&ShellErrorCategory::MissingQuotes));
        assert!(categories.contains(&ShellErrorCategory::NonDeterministicRandom));
        assert!(categories.contains(&ShellErrorCategory::NonIdempotentOperation));
        assert!(categories.contains(&ShellErrorCategory::CommandInjection));
        assert!(categories.contains(&ShellErrorCategory::WordSplitting));
        assert!(categories.contains(&ShellErrorCategory::GlobbingRisk));
        assert!(categories.contains(&ShellErrorCategory::TimestampUsage));
        assert!(categories.contains(&ShellErrorCategory::ProcessIdDependency));
        assert!(categories.contains(&ShellErrorCategory::UnsafeOverwrite));
        assert!(categories.contains(&ShellErrorCategory::MissingGuard));
        assert!(categories.contains(&ShellErrorCategory::PathTraversal));
        assert!(categories.contains(&ShellErrorCategory::UnsafeExpansion));
    }

    #[test]
    fn test_shell_error_category_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ShellErrorCategory::CommandInjection, "injection");
        map.insert(ShellErrorCategory::MissingQuotes, "quotes");

        assert_eq!(
            map.get(&ShellErrorCategory::CommandInjection),
            Some(&"injection")
        );
        assert_eq!(map.get(&ShellErrorCategory::MissingQuotes), Some(&"quotes"));
    }

    #[test]
    fn test_shell_error_category_debug() {
        let category = ShellErrorCategory::CommandInjection;
        let debug_str = format!("{:?}", category);
        assert!(debug_str.contains("CommandInjection"));
    }

    #[test]
    fn test_feature_extraction_no_source_line() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Test".to_string(),
            span: Span::new(999, 1, 999, 10), // Line 999 doesn't exist
            fix: None,
        };
        let source = "echo test"; // Only one line

        let features = FeatureVector::extract(&diag, source);

        // Should handle missing line gracefully
        assert_eq!(features.line_length, 0);
        assert_eq!(features.indentation_level, 0);
    }

    #[test]
    fn test_feature_extraction_semicolon_statements() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Test".to_string(),
            span: Span::new(1, 1, 1, 50),
            fix: None,
        };
        let source = "echo a; echo b; echo c";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.statement_count_in_line, 3);
    }

    #[test]
    fn test_drift_status_debug() {
        let status = DriftStatus::Stable { rate: 0.85 };
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Stable"));
        assert!(debug_str.contains("0.85"));
    }

    #[test]
    fn test_knn_with_mixed_training() {
        let mut classifier = KnnClassifier::new(3);

        // Add training examples for different categories
        for _ in 0..3 {
            let diag = sample_diagnostic("SC2086", "Quote variable");
            let features = FeatureVector::extract(&diag, "echo $x");
            classifier.add_example(features, ShellErrorCategory::MissingQuotes);
        }

        for _ in 0..2 {
            let diag = sample_diagnostic("DET001", "Random usage");
            let features = FeatureVector::extract(&diag, "echo $RANDOM");
            classifier.add_example(features, ShellErrorCategory::NonDeterministicRandom);
        }

        // Test classification for a quote-related issue
        let diag = sample_diagnostic("SC2086", "Quote this variable");
        let features = FeatureVector::extract(&diag, "echo $y");
        let result = classifier.classify(&features);

        // Should classify as MissingQuotes since we have more training examples for it
        assert_eq!(result.method, "k-NN");
    }
}
