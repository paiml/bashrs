
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

include!("oracle_tests_extracted_shell_feature_knn.rs");
