//! Tests for corpus helper functions in report, failure, compare, diag, and viz modules.
//! These tests target lightweight pure functions that do not invoke runner.run().
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// ---------------------------------------------------------------------------
// corpus_report_commands::trend_arrow
// ---------------------------------------------------------------------------

#[cfg(test)]
mod report_trend_arrow {
    use super::super::corpus_report_commands::trend_arrow;

    #[test]
    fn test_increasing_returns_up_arrow() {
        assert_eq!(trend_arrow(10, 5), "↑");
    }

    #[test]
    fn test_decreasing_returns_down_arrow() {
        assert_eq!(trend_arrow(3, 8), "↓");
    }

    #[test]
    fn test_equal_returns_right_arrow() {
        assert_eq!(trend_arrow(5, 5), "→");
    }

    #[test]
    fn test_zero_to_zero_is_right_arrow() {
        assert_eq!(trend_arrow(0, 0), "→");
    }

    #[test]
    fn test_from_zero_to_positive_is_up() {
        assert_eq!(trend_arrow(1, 0), "↑");
    }

    #[test]
    fn test_from_positive_to_zero_is_down() {
        assert_eq!(trend_arrow(0, 1), "↓");
    }

    #[test]
    fn test_large_values_increasing() {
        assert_eq!(trend_arrow(17000, 16000), "↑");
    }

    #[test]
    fn test_large_values_equal() {
        assert_eq!(trend_arrow(17942, 17942), "→");
    }
}

// ---------------------------------------------------------------------------
// corpus_report_commands::fmt_pass_total
// ---------------------------------------------------------------------------

#[cfg(test)]
mod report_fmt_pass_total {
    use super::super::corpus_report_commands::fmt_pass_total;

    #[test]
    fn test_nonzero_total_formats_as_fraction() {
        let result = fmt_pass_total(5, 10);
        assert_eq!(result, "5/10");
    }

    #[test]
    fn test_zero_total_returns_dash() {
        let result = fmt_pass_total(0, 0);
        assert_eq!(result, "-");
    }

    #[test]
    fn test_all_passed() {
        let result = fmt_pass_total(100, 100);
        assert_eq!(result, "100/100");
    }

    #[test]
    fn test_none_passed() {
        let result = fmt_pass_total(0, 50);
        assert_eq!(result, "0/50");
    }

    #[test]
    fn test_single_entry() {
        let result = fmt_pass_total(1, 1);
        assert_eq!(result, "1/1");
    }
}

// ---------------------------------------------------------------------------
// corpus_failure_commands::result_fail_dims
// ---------------------------------------------------------------------------

#[cfg(test)]
mod failure_result_fail_dims {
    use super::super::corpus_failure_commands::result_fail_dims;
    use crate::corpus::runner::CorpusResult;

    #[test]
    fn test_all_pass_returns_empty_vec() {
        let r = CorpusResult {
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            ..Default::default()
        };
        assert!(result_fail_dims(&r).is_empty());
    }

    #[test]
    fn test_transpile_fail_returns_a() {
        let r = CorpusResult {
            transpiled: false,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            ..Default::default()
        };
        let dims = result_fail_dims(&r);
        assert_eq!(dims, vec!["A"]);
    }

    #[test]
    fn test_output_contains_fail_returns_b1() {
        let r = CorpusResult {
            transpiled: true,
            output_contains: false,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            ..Default::default()
        };
        let dims = result_fail_dims(&r);
        assert_eq!(dims, vec!["B1"]);
    }

    #[test]
    fn test_multiple_failures_returns_multiple() {
        let r = CorpusResult {
            transpiled: false,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: false,
            deterministic: false,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            ..Default::default()
        };
        let dims = result_fail_dims(&r);
        assert!(dims.contains(&"A"));
        assert!(dims.contains(&"D"));
        assert!(dims.contains(&"E"));
        assert_eq!(dims.len(), 3);
    }

    #[test]
    fn test_all_fail_returns_eight_dims() {
        let r = CorpusResult {
            transpiled: false,
            output_contains: false,
            output_exact: false,
            output_behavioral: false,
            lint_clean: false,
            deterministic: false,
            metamorphic_consistent: false,
            cross_shell_agree: false,
            ..Default::default()
        };
        let dims = result_fail_dims(&r);
        assert_eq!(dims.len(), 8, "All-fail result should have 8 failing dims");
    }
}

// ---------------------------------------------------------------------------
// corpus_compare_commands::percentile
// ---------------------------------------------------------------------------

#[cfg(test)]
mod compare_percentile {
    use super::super::corpus_compare_commands::percentile;

    #[test]
    fn test_empty_slice_returns_zero() {
        assert_eq!(percentile(&[], 50.0), 0.0);
    }

    #[test]
    fn test_single_element_p50() {
        assert_eq!(percentile(&[42.0], 50.0), 42.0);
    }

    #[test]
    fn test_single_element_p0() {
        assert_eq!(percentile(&[42.0], 0.0), 42.0);
    }

    #[test]
    fn test_single_element_p100() {
        assert_eq!(percentile(&[42.0], 100.0), 42.0);
    }

    #[test]
    fn test_p0_returns_first() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 0.0), 1.0);
    }

    #[test]
    fn test_p100_returns_last() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 100.0), 5.0);
    }

    #[test]
    fn test_p50_median_of_five() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let result = percentile(&data, 50.0);
        // idx = round(0.5 * 4) = round(2.0) = 2 → data[2] = 30.0
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_result_in_data_range() {
        let data: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        let result = percentile(&data, 90.0);
        assert!(
            result >= 1.0 && result <= 10.0,
            "P90 should be in range [1,10], got {result}"
        );
    }
}

// ---------------------------------------------------------------------------
// corpus_diag_commands::result_dim_pass
// ---------------------------------------------------------------------------

#[cfg(test)]
mod diag_result_dim_pass {
    use super::super::corpus_diag_commands::result_dim_pass;
    use crate::corpus::runner::CorpusResult;

    #[test]
    fn test_dim_0_is_transpiled() {
        let r_pass = CorpusResult {
            transpiled: true,
            ..Default::default()
        };
        assert!(result_dim_pass(&r_pass, 0));
        let r_fail = CorpusResult {
            transpiled: false,
            ..Default::default()
        };
        assert!(!result_dim_pass(&r_fail, 0));
    }

    #[test]
    fn test_dim_1_is_output_contains() {
        let r = CorpusResult {
            output_contains: true,
            ..Default::default()
        };
        assert!(result_dim_pass(&r, 1));
        let r2 = CorpusResult {
            output_contains: false,
            ..Default::default()
        };
        assert!(!result_dim_pass(&r2, 1));
    }

    #[test]
    fn test_dim_4_is_lint_clean() {
        let r = CorpusResult {
            lint_clean: true,
            ..Default::default()
        };
        assert!(result_dim_pass(&r, 4));
    }

    #[test]
    fn test_dim_5_is_deterministic() {
        let r = CorpusResult {
            deterministic: true,
            ..Default::default()
        };
        assert!(result_dim_pass(&r, 5));
    }

    #[test]
    fn test_dim_6_is_metamorphic() {
        let r = CorpusResult {
            metamorphic_consistent: true,
            ..Default::default()
        };
        assert!(result_dim_pass(&r, 6));
    }

    #[test]
    fn test_dim_7_and_above_is_cross_shell() {
        let r_pass = CorpusResult {
            cross_shell_agree: true,
            ..Default::default()
        };
        assert!(result_dim_pass(&r_pass, 7));
        assert!(result_dim_pass(&r_pass, 99));
        let r_fail = CorpusResult {
            cross_shell_agree: false,
            ..Default::default()
        };
        assert!(!result_dim_pass(&r_fail, 7));
    }
}

// ---------------------------------------------------------------------------
// corpus_viz_commands::history_chart_cell (smoke tests for coverage)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod viz_history_chart_cell {
    use super::super::corpus_viz_commands::history_chart_cell;

    #[test]
    fn test_zero_score_does_not_panic() {
        // score = 0.0 should print a space and not panic
        history_chart_cell(0.0, 5, 80.0, 20.0, 10);
    }

    #[test]
    fn test_score_below_row_threshold_does_not_panic() {
        history_chart_cell(85.0, 9, 80.0, 20.0, 10);
    }

    #[test]
    fn test_high_score_gte_99_does_not_panic() {
        history_chart_cell(99.5, 0, 80.0, 20.0, 10);
    }

    #[test]
    fn test_medium_score_95_to_99_does_not_panic() {
        history_chart_cell(97.0, 0, 80.0, 20.0, 10);
    }

    #[test]
    fn test_low_score_below_95_does_not_panic() {
        history_chart_cell(90.0, 0, 80.0, 20.0, 10);
    }

    #[test]
    fn test_exactly_99_does_not_panic() {
        history_chart_cell(99.0, 0, 80.0, 20.0, 10);
    }

    #[test]
    fn test_exactly_95_does_not_panic() {
        history_chart_cell(95.0, 0, 80.0, 20.0, 10);
    }
}

// ---------------------------------------------------------------------------
// corpus_analysis_commands::validate_corpus_entry
// ---------------------------------------------------------------------------

#[cfg(test)]
mod analysis_validate_corpus_entry {
    use super::super::corpus_analysis_commands::validate_corpus_entry;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    use std::collections::HashSet;

    #[test]
    fn test_valid_bash_entry_no_issues() {
        let entry = CorpusEntry::new(
            "B-001",
            "test-entry",
            "A test entry",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.is_empty(),
            "Valid entry should have no issues: {issues:?}"
        );
    }

    #[test]
    fn test_duplicate_id_is_reported() {
        let entry = CorpusEntry::new(
            "B-001",
            "test-entry",
            "A test entry",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        seen.insert("B-001".to_string()); // Pre-insert to simulate duplicate
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("Duplicate")),
            "Should report duplicate ID: {issues:?}"
        );
    }

    #[test]
    fn test_wrong_prefix_bash_reported() {
        let entry = CorpusEntry::new(
            "M-001", // Wrong prefix for Bash format
            "test",
            "description",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("prefix")),
            "Should report prefix mismatch: {issues:?}"
        );
    }

    #[test]
    fn test_wrong_prefix_makefile_reported() {
        let entry = CorpusEntry::new(
            "B-001", // Wrong prefix for Makefile format
            "make-test",
            "description",
            CorpusFormat::Makefile,
            CorpusTier::Standard,
            "all:\n\techo hello",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("prefix")),
            "Should report prefix mismatch: {issues:?}"
        );
    }

    #[test]
    fn test_seen_ids_updated_after_validation() {
        let entry = CorpusEntry::new(
            "B-042",
            "test",
            "description",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        let _ = validate_corpus_entry(&entry, &mut seen);
        assert!(seen.contains("B-042"), "Seen IDs should contain B-042");
    }

    #[test]
    fn test_makefile_no_fn_main_requirement() {
        let entry = CorpusEntry::new(
            "M-001",
            "make-test",
            "A Makefile entry",
            CorpusFormat::Makefile,
            CorpusTier::Standard,
            "all:\n\techo hello",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        let has_main_issue = issues.iter().any(|i| i.contains("fn main"));
        assert!(
            !has_main_issue,
            "Makefile should not require fn main(): {issues:?}"
        );
    }

    #[test]
    fn test_bash_missing_fn_main_reported() {
        let entry = CorpusEntry::new(
            "B-999",
            "no-main",
            "Entry without fn main",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "echo hello", // No fn main()
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("fn main")),
            "Bash entry missing fn main() should be reported: {issues:?}"
        );
    }

    #[test]
    fn test_dockerfile_prefix_d_is_valid() {
        let entry = CorpusEntry::new(
            "D-001",
            "docker-test",
            "A Dockerfile entry",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18",
            "FROM alpine",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        let prefix_issue = issues.iter().any(|i| i.contains("prefix"));
        assert!(
            !prefix_issue,
            "D- prefix for Dockerfile should be valid: {issues:?}"
        );
    }
}
