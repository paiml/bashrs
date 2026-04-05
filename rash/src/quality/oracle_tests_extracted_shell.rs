
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

include!("oracle_tests_extracted_shell_feature.rs");
