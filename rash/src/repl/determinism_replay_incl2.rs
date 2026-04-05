#[cfg(test)]
mod diff_tests {
    use super::*;

    #[test]
    fn test_REPL_011_003_format_single_difference() {
        // ARRANGE: One difference
        let differences = vec![OutputDifference {
            line: 1,
            run1: "Random: 12345".to_string(),
            run2: "Random: 67890".to_string(),
        }];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show line number and both outputs
        assert!(formatted.contains("Line 1:"));
        assert!(formatted.contains("Run 1: Random: 12345"));
        assert!(formatted.contains("Run 2: Random: 67890"));
    }

    #[test]
    fn test_REPL_011_003_format_multiple_differences() {
        // ARRANGE: Multiple differences
        let differences = vec![
            OutputDifference {
                line: 1,
                run1: "First: abc".to_string(),
                run2: "First: xyz".to_string(),
            },
            OutputDifference {
                line: 3,
                run1: "Third: 123".to_string(),
                run2: "Third: 456".to_string(),
            },
        ];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show all differences
        assert!(formatted.contains("Line 1:"));
        assert!(formatted.contains("Line 3:"));
        assert!(formatted.contains("2 difference"));
    }

    #[test]
    fn test_REPL_011_003_format_no_differences() {
        // ARRANGE: Empty differences
        let differences = vec![];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show success message
        assert!(formatted.contains("deterministic"));
    }

    #[test]
    fn test_REPL_011_003_format_empty_lines() {
        // ARRANGE: Differences with empty output
        let differences = vec![OutputDifference {
            line: 5,
            run1: "".to_string(),
            run2: "Something appeared".to_string(),
        }];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should handle empty strings
        assert!(formatted.contains("Line 5:"));
        assert!(formatted.contains("Something appeared"));
    }

    #[test]
    fn test_REPL_011_003_replay_result_format() {
        // ARRANGE: Non-deterministic replay result
        let result = ReplayResult {
            is_deterministic: false,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "Random: 12345\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "Random: 67890\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![OutputDifference {
                line: 1,
                run1: "Random: 12345".to_string(),
                run2: "Random: 67890".to_string(),
            }],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show status, runs, and differences
        assert!(formatted.contains("Runs: 2"));
        assert!(formatted.contains("Exit codes:"));
        assert!(formatted.contains("Line 1:"));
    }

    #[test]
    fn test_REPL_011_003_deterministic_result_format() {
        // ARRANGE: Deterministic replay result
        let result = ReplayResult {
            is_deterministic: true,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "Constant output\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "Constant output\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show success status
        assert!(formatted.contains("deterministic"));
    }
}

#[cfg(test)]
mod diff_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_011_003_format_never_panics(
            num_diffs in 0usize..10,
            line in 1usize..100,
        ) {
            // Generate arbitrary differences
            let differences: Vec<OutputDifference> = (0..num_diffs)
                .map(|i| OutputDifference {
                    line: line + i,
                    run1: format!("run1_{}", i),
                    run2: format!("run2_{}", i),
                })
                .collect();

            // Format should never panic
            let _ = format_replay_diff(&differences);
        }

        #[test]
        fn prop_REPL_011_003_format_preserves_line_numbers(
            line_num in 1usize..1000,
        ) {
            let differences = vec![OutputDifference {
                line: line_num,
                run1: "a".to_string(),
                run2: "b".to_string(),
            }];

            let formatted = format_replay_diff(&differences);

            // Line number should appear in output
            prop_assert!(
                formatted.contains(&format!("Line {}:", line_num)),
                "Should contain line number {}: {}",
                line_num,
                formatted
            );
        }

        #[test]
        fn prop_REPL_011_003_empty_always_deterministic(
            _any in 0..100,
        ) {
            let formatted = format_replay_diff(&[]);

            // Empty differences should always show deterministic
            prop_assert!(
                formatted.contains("deterministic"),
                "Empty diff should show deterministic: {}",
                formatted
            );
        }
    }
}
