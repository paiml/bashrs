#[cfg(test)]
mod idempotency_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_001_mkdir_without_p_always_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir without -p should always be detected
            prop_assert_eq!(issues.len(), 1);
            prop_assert_eq!(issues[0].operation_type, NonIdempotentOperation::MkdirWithoutP);
        }

        #[test]
        fn prop_REPL_012_001_mkdir_with_p_never_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir -p {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir -p should never be detected as non-idempotent
            prop_assert_eq!(issues.len(), 0);
            prop_assert!(checker.is_idempotent());
        }

        #[test]
        fn prop_REPL_012_001_scan_never_panics(
            script in ".*{0,1000}"
        ) {
            let mut checker = IdempotencyChecker::new();
            // Should never panic on any input
            let _ = checker.scan(&script);
        }

        #[test]
        fn prop_REPL_012_001_rescan_always_clears(
            script1 in "mkdir [a-z]{1,20}",
            script2 in "mkdir -p [a-z]{1,20}"
        ) {
            let mut checker = IdempotencyChecker::new();

            // First scan should find issue
            let issues1 = checker.scan(&script1);
            prop_assert_eq!(issues1.len(), 1);

            // Second scan should clear and find no issues
            let issues2 = checker.scan(&script2);
            prop_assert_eq!(issues2.len(), 0);
            prop_assert!(checker.is_idempotent());
        }
    }
}

#[cfg(test)]
mod idempotency_report_tests {
    use super::*;

    // ===== REPL-012-002: REPORT FORMATTING TESTS =====

    #[test]
    fn test_REPL_012_002_format_single_mkdir_issue() {
        // ARRANGE: One mkdir issue
        let issues = vec![IdempotencyIssue {
            line: 10,
            operation_type: NonIdempotentOperation::MkdirWithoutP,
            code: "mkdir /tmp/test".to_string(),
            explanation: "mkdir without -p fails if directory already exists".to_string(),
            suggestion: "Add -p flag: mkdir -p".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show line, code, problem, and fix
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("mkdir /tmp/test"));
        assert!(formatted.contains("mkdir without -p"));
        assert!(formatted.contains("Add -p flag"));
        assert!(formatted.contains("⚠️") || formatted.contains("warning"));
    }

    #[test]
    fn test_REPL_012_002_format_single_rm_issue() {
        // ARRANGE: One rm issue
        let issues = vec![IdempotencyIssue {
            line: 15,
            operation_type: NonIdempotentOperation::RmWithoutF,
            code: "rm /tmp/file.txt".to_string(),
            explanation: "rm without -f fails if file doesn't exist".to_string(),
            suggestion: "Add -f flag: rm -f".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show rm-specific details
        assert!(formatted.contains("Line 15:"));
        assert!(formatted.contains("rm /tmp/file.txt"));
        assert!(formatted.contains("rm without -f"));
        assert!(formatted.contains("Add -f flag"));
    }

    #[test]
    fn test_REPL_012_002_format_multiple_issues() {
        // ARRANGE: Multiple issues
        let issues = vec![
            IdempotencyIssue {
                line: 5,
                operation_type: NonIdempotentOperation::MkdirWithoutP,
                code: "mkdir foo".to_string(),
                explanation: "mkdir without -p fails if directory already exists".to_string(),
                suggestion: "Add -p flag: mkdir -p".to_string(),
            },
            IdempotencyIssue {
                line: 10,
                operation_type: NonIdempotentOperation::RmWithoutF,
                code: "rm bar".to_string(),
                explanation: "rm without -f fails if file doesn't exist".to_string(),
                suggestion: "Add -f flag: rm -f".to_string(),
            },
        ];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show all issues
        assert!(formatted.contains("Line 5:"));
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("Found 2 issue(s)") || formatted.contains("2 issue"));
    }

    #[test]
    fn test_REPL_012_002_format_no_issues() {
        // ARRANGE: Empty issues
        let issues = vec![];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show success message
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
        assert!(formatted.contains("safe") || formatted.contains("re-run"));
    }

    #[test]
    fn test_REPL_012_002_checker_format_report() {
        // ARRANGE: Checker with detected issues
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir /tmp/foo
rm /tmp/bar
ln -s /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show status and breakdown
        assert!(formatted.contains("non-idempotent") || formatted.contains("issue"));
        assert!(
            formatted.contains("mkdir") || formatted.contains("rm") || formatted.contains("ln")
        );
    }

    #[test]
    fn test_REPL_012_002_checker_format_idempotent() {
        // ARRANGE: Checker with idempotent script
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir -p /tmp/foo
rm -f /tmp/bar
ln -sf /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show success status
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
    }

    #[test]
    fn test_REPL_012_002_format_preserves_line_numbers() {
        // ARRANGE: Issues with specific line numbers
        let issues = vec![IdempotencyIssue {
            line: 42,
            operation_type: NonIdempotentOperation::MkdirWithoutP,
            code: "mkdir test".to_string(),
            explanation: "explanation".to_string(),
            suggestion: "suggestion".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Line number should be preserved
        assert!(formatted.contains("Line 42:") || formatted.contains("42"));
    }
}

#[cfg(test)]
mod idempotency_report_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_002_format_never_panics(
            num_issues in 0usize..10,
            line in 1usize..100,
        ) {
            // Generate arbitrary issues
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: line + i,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: format!("mkdir /tmp/test{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            // Format should never panic
            let _ = format_idempotency_report(&issues);
        }

        #[test]
        fn prop_REPL_012_002_empty_always_idempotent(
            _any in 0..100,
        ) {
            let formatted = format_idempotency_report(&[]);

            // Empty issues should always show idempotent
            prop_assert!(
                (formatted.contains("✓") || formatted.contains("success")) && formatted.contains("idempotent"),
                "Empty report should show idempotent: {}",
                formatted
            );
        }

        #[test]
        fn prop_REPL_012_002_count_matches_issues(
            num_issues in 1usize..20,
        ) {
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: i + 1,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: format!("rm file{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            let formatted = format_idempotency_report(&issues);

            // Count should match number of issues (flexible matching for different formats)
            let count_str = format!("{}", num_issues);
            prop_assert!(
                formatted.contains(&count_str),
                "Report should show count {}: {}",
                num_issues,
                formatted
            );
        }
    }
}

#[cfg(test)]
mod idempotency_verification_tests {
    use super::*;

    // ===== REPL-012-003: IDEMPOTENCY VERIFICATION TESTS =====

    #[test]
    fn test_REPL_012_003_verifier_new_default() {
        // ARRANGE & ACT: Create default verifier
        let verifier = IdempotencyVerifier::new();

        // ASSERT: Should have 3 runs by default
        assert_eq!(verifier.run_count, 3);
    }

    #[test]
    fn test_REPL_012_003_verifier_custom_count() {
        // ARRANGE & ACT: Create verifier with custom count
        let verifier = IdempotencyVerifier::with_run_count(5);

        // ASSERT: Should have 5 runs
        assert_eq!(verifier.run_count, 5);
    }

    #[test]
    fn test_REPL_012_003_verifier_minimum_two_runs() {
        // ARRANGE & ACT: Try to create verifier with 1 run
        let verifier = IdempotencyVerifier::with_run_count(1);

        // ASSERT: Should enforce minimum of 2 runs
        assert!(verifier.run_count >= 2);
    }

    #[test]
    fn test_REPL_012_003_idempotent_script_passes() {
        // ARRANGE: Idempotent script (echo with constant)
        let verifier = IdempotencyVerifier::new();
        let script = "echo 'hello world'";

        // ACT: Verify idempotency
        let result = verifier.verify(script);

        // ASSERT: Should pass as idempotent
        assert!(result.is_idempotent);
        assert_eq!(result.run_count, 3);
        assert!(result.differences.is_empty());
    }

    #[test]
    fn test_REPL_012_003_nonidempotent_script_fails() {
        // ARRANGE: Non-idempotent script (random)
        let verifier = IdempotencyVerifier::new();
        let script = "echo $RANDOM";

        // ACT: Verify idempotency
        let result = verifier.verify(script);

        // ASSERT: Should fail as non-idempotent
        assert!(!result.is_idempotent);
        assert_eq!(result.run_count, 3);
        assert!(!result.differences.is_empty());
    }

    #[test]
    fn test_REPL_012_003_result_format_idempotent() {
        // ARRANGE: Idempotent result
        let result = IdempotencyResult {
            is_idempotent: true,
            run_count: 3,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 3,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show success
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
    }

    #[test]
    fn test_REPL_012_003_result_format_nonidempotent() {
        // ARRANGE: Non-idempotent result
        let result = IdempotencyResult {
            is_idempotent: false,
            run_count: 3,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "123\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "456\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 3,
                    stdout: "789\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![OutputDifference {
                line: 1,
                run1: "123".to_string(),
                run2: "456".to_string(),
            }],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show failure
        assert!(
            formatted.contains("❌")
                || formatted.contains("fail")
                || formatted.contains("non-idempotent")
        );
    }
}

#[cfg(test)]
mod idempotency_verification_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_003_verifier_enforces_minimum(
            count in 0usize..10,
        ) {
            let verifier = IdempotencyVerifier::with_run_count(count);

            // Should always have at least 2 runs
            prop_assert!(verifier.run_count >= 2);
        }

        #[test]
        fn prop_REPL_012_003_constant_script_idempotent(
            constant in "[a-z]{1,20}",
        ) {
            let verifier = IdempotencyVerifier::new();
            let script = format!("echo '{}'", constant);

            let result = verifier.verify(&script);

            // Constant output should always be idempotent
            prop_assert!(
                result.is_idempotent,
                "Constant script should be idempotent: {}",
                script
            );
            prop_assert_eq!(result.differences.len(), 0);
        }

        #[test]
        fn prop_REPL_012_003_result_runs_match_count(
            run_count in 2usize..10,
        ) {
            let verifier = IdempotencyVerifier::with_run_count(run_count);
            let result = verifier.verify("echo 'test'");

            // Result should have exactly run_count runs
            prop_assert_eq!(result.runs.len(), run_count);
            prop_assert_eq!(result.run_count, run_count);
        }
    }
}
