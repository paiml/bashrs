//! Tests for corpus decision, analysis coverage, and ranking dimension stats modules.
//! These tests target lightweight pure functions that do not invoke runner.run().
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// ---------------------------------------------------------------------------
// corpus_decision_commands::score_impact_color
// ---------------------------------------------------------------------------

#[cfg(test)]
mod decision_score_impact_color {
    use super::super::corpus_decision_commands::score_impact_color;

    #[test]
    fn test_score_08_is_high() {
        let (impact, _color) = score_impact_color(0.8);
        assert!(impact.contains("HIGH"), "Score 0.8 should be HIGH: {impact}");
    }

    #[test]
    fn test_score_1_0_is_high() {
        let (impact, _color) = score_impact_color(1.0);
        assert!(impact.contains("HIGH"), "Score 1.0 should be HIGH: {impact}");
    }

    #[test]
    fn test_score_0_5_is_medium() {
        let (impact, _color) = score_impact_color(0.5);
        assert!(impact.contains("MEDIUM"), "Score 0.5 should be MEDIUM: {impact}");
    }

    #[test]
    fn test_score_0_7_is_medium() {
        let (impact, _color) = score_impact_color(0.7);
        assert!(impact.contains("MEDIUM"), "Score 0.7 should be MEDIUM: {impact}");
    }

    #[test]
    fn test_score_0_0_is_low() {
        let (impact, _color) = score_impact_color(0.0);
        assert!(impact.contains("LOW"), "Score 0.0 should be LOW: {impact}");
    }

    #[test]
    fn test_score_0_49_is_low() {
        let (impact, _color) = score_impact_color(0.49);
        assert!(impact.contains("LOW"), "Score 0.49 should be LOW: {impact}");
    }

    #[test]
    fn test_returns_color_str() {
        let (_impact, color) = score_impact_color(0.9);
        // Color should be a non-empty ANSI escape or similar string reference
        assert!(!color.is_empty(), "Color should not be empty");
    }
}

// ---------------------------------------------------------------------------
// corpus_decision_commands::accumulate_decision_stats
// ---------------------------------------------------------------------------

#[cfg(test)]
mod decision_accumulate_stats {
    use super::super::corpus_decision_commands::accumulate_decision_stats;
    use crate::corpus::runner::CorpusResult;
    use crate::emitter::trace::TranspilerDecision;
    use std::collections::HashMap;

    fn make_decision(decision_type: &str, choice: &str) -> TranspilerDecision {
        TranspilerDecision {
            decision_type: decision_type.to_string(),
            choice: choice.to_string(),
            ir_node: "TestNode".to_string(),
        }
    }

    #[test]
    fn test_no_trace_returns_false() {
        let r = CorpusResult {
            transpiled: true,
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            decision_trace: None,
            ..Default::default()
        };
        let mut stats = HashMap::new();
        let had_trace = accumulate_decision_stats(&r, &mut stats);
        assert!(!had_trace, "No trace should return false");
        assert!(stats.is_empty(), "No trace should not populate stats");
    }

    #[test]
    fn test_empty_trace_returns_false() {
        let r = CorpusResult {
            transpiled: true,
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            decision_trace: Some(vec![]),
            ..Default::default()
        };
        let mut stats = HashMap::new();
        let had_trace = accumulate_decision_stats(&r, &mut stats);
        assert!(!had_trace, "Empty trace should return false");
    }

    #[test]
    fn test_single_decision_passing_increments_pass() {
        let r = CorpusResult {
            transpiled: true,
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            decision_trace: Some(vec![make_decision("FunctionCall", "println")]),
            ..Default::default()
        };
        let mut stats = HashMap::new();
        let had_trace = accumulate_decision_stats(&r, &mut stats);
        assert!(had_trace);
        let (count, pass, fail) = stats["FunctionCall:println"];
        assert_eq!(count, 1);
        assert_eq!(pass, 1);
        assert_eq!(fail, 0);
    }

    #[test]
    fn test_single_decision_failing_increments_fail() {
        let r = CorpusResult {
            transpiled: false, // failing
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            decision_trace: Some(vec![make_decision("FunctionCall", "println")]),
            ..Default::default()
        };
        let mut stats = HashMap::new();
        accumulate_decision_stats(&r, &mut stats);
        let (count, pass, fail) = stats["FunctionCall:println"];
        assert_eq!(count, 1);
        assert_eq!(pass, 0);
        assert_eq!(fail, 1);
    }

    #[test]
    fn test_multiple_decisions_all_accumulated() {
        let r = CorpusResult {
            transpiled: true,
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            decision_trace: Some(vec![
                make_decision("FunctionCall", "println"),
                make_decision("BinaryOp", "add"),
                make_decision("FunctionCall", "println"), // duplicate key
            ]),
            ..Default::default()
        };
        let mut stats = HashMap::new();
        accumulate_decision_stats(&r, &mut stats);
        // "FunctionCall:println" should have count=2
        assert_eq!(stats["FunctionCall:println"].0, 2);
        // "BinaryOp:add" should have count=1
        assert_eq!(stats["BinaryOp:add"].0, 1);
    }

    #[test]
    fn test_pass_requires_all_conditions() {
        // If deterministic=false, result is "failing" even if transpiled=true
        let r = CorpusResult {
            transpiled: true,
            output_contains: true,
            schema_valid: true,
            lint_clean: true,
            deterministic: false, // This makes it fail
            decision_trace: Some(vec![make_decision("Assign", "x")]),
            ..Default::default()
        };
        let mut stats = HashMap::new();
        accumulate_decision_stats(&r, &mut stats);
        let (_, pass, fail) = stats["Assign:x"];
        assert_eq!(pass, 0, "Failing entry should not increment pass");
        assert_eq!(fail, 1, "Failing entry should increment fail");
    }
}

// ---------------------------------------------------------------------------
// corpus_ranking_commands::compute_dimension_stats
// (DimStat fields are private; test via length and via corpus_dimensions)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod ranking_compute_dimension_stats {
    use super::super::corpus_ranking_commands::compute_dimension_stats;
    use crate::corpus::runner::CorpusResult;

    #[test]
    fn test_returns_9_dimensions() {
        let dims = compute_dimension_stats(&[], 0);
        assert_eq!(dims.len(), 9, "Should have 9 V2 dimensions (A, B1, B2, B3, C, D, E, F, G)");
    }

    #[test]
    fn test_empty_results_does_not_panic() {
        // Just verify it doesn't panic with empty input
        let dims = compute_dimension_stats(&[], 0);
        assert!(!dims.is_empty());
    }

    #[test]
    fn test_single_all_pass_does_not_panic() {
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
        let dims = compute_dimension_stats(&[r], 1);
        assert_eq!(dims.len(), 9);
    }
}

// ---------------------------------------------------------------------------
// corpus_analysis_commands::count_format (helper)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod analysis_count_format {
    use super::super::corpus_analysis_commands::count_format;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier, CorpusRegistry};

    fn make_registry_with_entries(entries: Vec<CorpusEntry>) -> CorpusRegistry {
        CorpusRegistry { entries }
    }

    #[test]
    fn test_count_bash_entries() {
        let entries = vec![
            CorpusEntry::new("B-001", "t", "d", CorpusFormat::Bash, CorpusTier::Trivial,
                "fn main() { println!(\"x\"); }", "x"),
            CorpusEntry::new("B-002", "t", "d", CorpusFormat::Bash, CorpusTier::Trivial,
                "fn main() { println!(\"y\"); }", "y"),
            CorpusEntry::new("M-001", "t", "d", CorpusFormat::Makefile, CorpusTier::Standard,
                "all:", "all"),
        ];
        let registry = make_registry_with_entries(entries);
        assert_eq!(count_format(&registry, &CorpusFormat::Bash), 2);
    }

    #[test]
    fn test_count_makefile_entries() {
        let entries = vec![
            CorpusEntry::new("B-001", "t", "d", CorpusFormat::Bash, CorpusTier::Trivial,
                "fn main() { println!(\"x\"); }", "x"),
            CorpusEntry::new("M-001", "t", "d", CorpusFormat::Makefile, CorpusTier::Standard,
                "all:", "all"),
        ];
        let registry = make_registry_with_entries(entries);
        assert_eq!(count_format(&registry, &CorpusFormat::Makefile), 1);
    }

    #[test]
    fn test_count_dockerfile_entries_zero() {
        let entries = vec![
            CorpusEntry::new("B-001", "t", "d", CorpusFormat::Bash, CorpusTier::Trivial,
                "fn main() { println!(\"x\"); }", "x"),
        ];
        let registry = make_registry_with_entries(entries);
        assert_eq!(count_format(&registry, &CorpusFormat::Dockerfile), 0);
    }

    #[test]
    fn test_count_empty_registry() {
        let registry = make_registry_with_entries(vec![]);
        assert_eq!(count_format(&registry, &CorpusFormat::Bash), 0);
        assert_eq!(count_format(&registry, &CorpusFormat::Makefile), 0);
        assert_eq!(count_format(&registry, &CorpusFormat::Dockerfile), 0);
    }
}

// ---------------------------------------------------------------------------
// corpus_entry_commands::collect_risk_failures
// ---------------------------------------------------------------------------

#[cfg(test)]
mod entry_collect_risk_failures {
    use super::super::corpus_entry_commands::collect_risk_failures;
    use crate::corpus::runner::CorpusResult;

    fn make_result_with_id(id: &str, transpiled: bool, lint_clean: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_no_failures_returns_empty() {
        let results = vec![
            make_result_with_id("B-001", true, true),
            make_result_with_id("B-002", true, true),
        ];
        let failures = collect_risk_failures(&results, None);
        assert!(failures.is_empty(), "No failures should return empty vec");
    }

    #[test]
    fn test_transpile_fail_is_high_risk() {
        let results = vec![make_result_with_id("B-001", false, true)];
        let failures = collect_risk_failures(&results, None);
        let high_count = failures.iter().filter(|(_, _, r)| *r == "HIGH").count();
        assert!(high_count > 0, "Transpile failure should be HIGH risk");
    }

    #[test]
    fn test_lint_fail_is_medium_risk() {
        let results = vec![make_result_with_id("B-001", true, false)];
        let failures = collect_risk_failures(&results, None);
        assert_eq!(failures.len(), 1, "Should have one failure for lint");
        let (_, dim, risk) = failures[0];
        assert_eq!(dim, "D");
        assert_eq!(risk, "MEDIUM");
    }

    #[test]
    fn test_filter_by_high_only() {
        let results = vec![
            make_result_with_id("B-001", false, false), // A=HIGH, D=MEDIUM
        ];
        let high_only = collect_risk_failures(&results, Some("HIGH"));
        for (_, _, risk) in &high_only {
            assert_eq!(*risk, "HIGH", "Filtered results should all be HIGH");
        }
    }

    #[test]
    fn test_filter_by_medium_only() {
        let results = vec![
            make_result_with_id("B-001", false, false), // A=HIGH, D=MEDIUM
        ];
        let medium_only = collect_risk_failures(&results, Some("MEDIUM"));
        for (_, _, risk) in &medium_only {
            assert_eq!(*risk, "MEDIUM", "Filtered results should all be MEDIUM");
        }
    }

    #[test]
    fn test_no_filter_returns_all() {
        let results = vec![
            make_result_with_id("B-001", false, false), // A=HIGH, D=MEDIUM
        ];
        let all_failures = collect_risk_failures(&results, None);
        assert!(all_failures.len() >= 2, "Should return both HIGH and MEDIUM failures");
    }

    #[test]
    fn test_id_is_in_result() {
        let results = vec![make_result_with_id("B-042", false, true)];
        let failures = collect_risk_failures(&results, None);
        assert!(!failures.is_empty());
        let (id, _, _) = failures[0];
        assert_eq!(id, "B-042");
    }
}

// ---------------------------------------------------------------------------
// corpus_failure_commands::count_dimension_failures
// ---------------------------------------------------------------------------

#[cfg(test)]
mod failure_count_dimension_failures {
    use super::super::corpus_failure_commands::count_dimension_failures;
    use crate::corpus::runner::CorpusResult;

    #[test]
    fn test_all_pass_returns_empty_sorted_vec() {
        let results = vec![CorpusResult {
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            schema_valid: true,
            ..Default::default()
        }];
        let failures = count_dimension_failures(&results);
        assert!(failures.is_empty(), "All-pass results should have no dimension failures");
    }

    #[test]
    fn test_single_transpile_fail_counted() {
        let results = vec![CorpusResult {
            transpiled: false,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            schema_valid: true,
            ..Default::default()
        }];
        let failures = count_dimension_failures(&results);
        assert!(!failures.is_empty(), "Should report A dimension failure");
        let a_entry = failures.iter().find(|(d, _)| d.contains("Transpilation"));
        assert!(a_entry.is_some(), "Should have Transpilation dimension in failures");
        let (_, count) = a_entry.unwrap();
        assert_eq!(*count, 1);
    }

    #[test]
    fn test_sorted_descending_by_count() {
        let results = vec![
            // 2 transpile failures, 1 lint failure
            CorpusResult {
                transpiled: false, lint_clean: false,
                output_contains: true, output_exact: true, output_behavioral: true,
                deterministic: true, metamorphic_consistent: true, cross_shell_agree: true,
                schema_valid: true, ..Default::default()
            },
            CorpusResult {
                transpiled: false, lint_clean: true,
                output_contains: true, output_exact: true, output_behavioral: true,
                deterministic: true, metamorphic_consistent: true, cross_shell_agree: true,
                schema_valid: true, ..Default::default()
            },
        ];
        let failures = count_dimension_failures(&results);
        // Should be sorted descending: transpilation (2) before lint (1)
        if failures.len() >= 2 {
            assert!(failures[0].1 >= failures[1].1, "Should be sorted descending");
        }
    }

    #[test]
    fn test_zero_count_dims_excluded() {
        let results = vec![CorpusResult {
            transpiled: false,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            schema_valid: true,
            ..Default::default()
        }];
        let failures = count_dimension_failures(&results);
        // Only non-zero counts should appear
        for (_, count) in &failures {
            assert!(*count > 0, "All entries should have count > 0");
        }
    }
}
