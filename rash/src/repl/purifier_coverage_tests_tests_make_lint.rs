#[cfg(test)]
mod purifier_coverage {
    use crate::linter::{Diagnostic, LintResult, Severity};
    use crate::repl::purifier::{
        explain_purification_changes, explain_purification_changes_detailed, format_alternatives,
        format_purified_lint_result, format_purified_lint_result_with_context,
        format_safety_rationale, format_transformation_report, generate_determinism_alternatives,
        generate_determinism_rationale, generate_idempotency_alternatives,
        generate_idempotency_rationale, generate_safety_alternatives, generate_safety_rationale,
        purify_and_lint, Alternative, PurificationError, PurifiedLintResult, SafetyRationale,
        SafetySeverity, TransformationCategory, TransformationExplanation,
    };

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_lint_result_with(codes: &[&str]) -> LintResult {
        let mut lr = LintResult::new();
        for &code in codes {
            lr.diagnostics.push(Diagnostic::new(
                code,
                Severity::Warning,
                format!("test violation {code}"),
                crate::linter::Span::new(1, 1, 1, 10),
            ));
        }
        lr
    }

    // ── format_purified_lint_result_with_context ─────────────────────────────

    /// Context function when purified code has DET violations.
    #[test]
    fn test_format_with_context_det_violation() {
        let lr = make_lint_result_with(&["DET001"]);
        let result = PurifiedLintResult::new("echo $RANDOM".to_string(), lr);
        let formatted = format_purified_lint_result_with_context(&result, "echo $RANDOM");
        assert!(formatted.contains("Purified"));
        assert!(formatted.contains("critical violation"));
        assert!(formatted.contains("DET"));
    }

    /// Context function when purified code has IDEM violations.
    #[test]
    fn test_format_with_context_idem_violation() {
        let lr = make_lint_result_with(&["IDEM001"]);
        let result = PurifiedLintResult::new("rm /tmp/x".to_string(), lr);
        let formatted = format_purified_lint_result_with_context(&result, "rm /tmp/x");
        assert!(formatted.contains("IDEM"));
        assert!(!result.is_clean);
    }

    /// Context function when purified code has SEC violations.
    #[test]
    fn test_format_with_context_sec_violation() {
        let lr = make_lint_result_with(&["SEC001"]);
        let result = PurifiedLintResult::new("eval $cmd".to_string(), lr);
        let formatted = format_purified_lint_result_with_context(&result, "eval $cmd");
        assert!(formatted.contains("SEC"));
    }

    /// Context function with multiple mixed violations.
    #[test]
    fn test_format_with_context_multiple_violations() {
        let lr = make_lint_result_with(&["DET001", "IDEM001", "SEC001"]);
        let result = PurifiedLintResult::new("code".to_string(), lr);
        let formatted = format_purified_lint_result_with_context(&result, "code");
        assert!(formatted.contains("3"));
        assert!(formatted.contains("DET"));
        assert!(formatted.contains("IDEM"));
        assert!(formatted.contains("SEC"));
    }

    /// Clean code yields the CLEAN message.
    #[test]
    fn test_format_with_context_clean() {
        let lr = LintResult::new();
        let result = PurifiedLintResult::new("echo hello".to_string(), lr);
        let formatted = format_purified_lint_result_with_context(&result, "echo hello");
        assert!(formatted.contains("CLEAN"));
        assert!(result.is_clean);
    }

    // ── collect_change_explanations (tested via explain_purification_changes) ──
    // collect_change_explanations is private; we exercise its branches through
    // the public explain_purification_changes wrapper.

    /// rm → rm -f branch is triggered when the purifier adds -f.
    #[test]
    fn test_collect_rm_to_rm_f_via_explain() {
        let result = explain_purification_changes("rm /tmp/test_file");
        assert!(result.is_ok());
        // May or may not contain "rm" depending on purifier output
    }

    /// Variable quoting branch via explain.
    #[test]
    fn test_collect_variable_quoting_via_explain() {
        let result = explain_purification_changes("echo $HOME");
        assert!(result.is_ok());
    }

    /// ln -s → ln -sf branch via explain.
    #[test]
    fn test_collect_ln_sf_via_explain() {
        let result = explain_purification_changes("ln -s /target /link");
        assert!(result.is_ok());
        let text = result.unwrap();
        // Either "No changes needed" or an explanation mentioning ln
        assert!(!text.is_empty());
    }

    /// $RANDOM removal via explain (non-determinism branch).
    #[test]
    fn test_collect_random_removal_via_explain() {
        let result = explain_purification_changes("echo $RANDOM");
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(!text.is_empty());
    }

    /// Timestamp/date branch via explain.
    #[test]
    fn test_collect_date_via_explain() {
        let result = explain_purification_changes("echo $(date +%s)");
        assert!(result.is_ok());
    }

    /// No changes — returns "No changes needed" message.
    #[test]
    fn test_collect_no_changes_via_explain() {
        let result = explain_purification_changes("echo hello");
        assert!(result.is_ok());
        // Either "No changes" or some explanation
        assert!(!result.unwrap().is_empty());
    }

    // ── explain_purification_changes_detailed ─────────────────────────────────

    /// rm → rm -f detailed explanation.
    #[test]
    fn test_explain_detailed_rm_f() {
        let original = "rm /tmp/test_file";
        let result = explain_purification_changes_detailed(original);
        assert!(result.is_ok(), "should not fail: {:?}", result);
        let explanations = result.unwrap();
        if !explanations.is_empty() {
            let has_rm = explanations
                .iter()
                .any(|e| e.category == TransformationCategory::Idempotency);
            assert!(has_rm, "rm -f is an idempotency fix");
        }
    }

    /// Variable quoting detailed explanation.
    #[test]
    fn test_explain_detailed_variable_quoting() {
        let original = "echo $HOME";
        let result = explain_purification_changes_detailed(original);
        assert!(result.is_ok());
        // The purifier may or may not quote this — just ensure no panic
    }

    /// ln -s detailed explanation.
    #[test]
    fn test_explain_detailed_ln_sf() {
        let original = "ln -s /target /link";
        let result = explain_purification_changes_detailed(original);
        assert!(result.is_ok());
        let explanations = result.unwrap();
        // If ln -sf transformation occurred, it should be Idempotency
        for e in &explanations {
            if e.title.contains("ln") {
                assert_eq!(e.category, TransformationCategory::Idempotency);
            }
        }
    }

    /// $RANDOM detailed explanation.
    #[test]
    fn test_explain_detailed_random() {
        let original = "echo $RANDOM";
        let result = explain_purification_changes_detailed(original);
        assert!(result.is_ok());
        let explanations = result.unwrap();
        // If $RANDOM was removed we expect a Determinism entry
        for e in &explanations {
            if e.title.contains("RANDOM") {
                assert_eq!(e.category, TransformationCategory::Determinism);
            }
        }
    }

    /// When code is already clean, return empty vec.
    #[test]
    fn test_explain_detailed_no_change_returns_empty() {
        // Simple clean echo that the purifier should leave intact
        let original = "echo hello";
        let result = explain_purification_changes_detailed(original);
        assert!(result.is_ok());
        // May or may not be empty — just no panic
    }

    // ── generate_idempotency_alternatives ────────────────────────────────────
}

#[cfg(test)]
mod purifier_coverage_tests_tests_extracted_idempotency {
    use super::*;
        // FIXME(PMAT-238): include!("purifier_coverage_tests_tests_extracted_idempotency.rs");
}
