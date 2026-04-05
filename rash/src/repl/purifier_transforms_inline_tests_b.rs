//! Tests extracted from purifier_transforms.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::repl::purifier_transforms::*;

// ===== REPL-013-001: TRANSFORMATION EXPLANATION TESTS (RED PHASE) =====

#[test]
fn test_safety_rationale_with_impact() {
    let rationale = SafetyRationale::new().with_impact("Data loss");
    assert_eq!(rationale.impact_without_fix, "Data loss");
}

#[test]
fn test_safety_rationale_with_severity() {
    let rationale = SafetyRationale::new().with_severity(SafetySeverity::Critical);
    assert_eq!(rationale.severity, SafetySeverity::Critical);
}

#[test]
fn test_safety_rationale_builder_chain() {
    let rationale = SafetyRationale::new()
        .add_vulnerability("Injection")
        .add_failure("Crash")
        .add_attack_vector("RCE")
        .with_impact("System compromise")
        .with_severity(SafetySeverity::High);

    assert_eq!(rationale.vulnerabilities_prevented.len(), 1);
    assert_eq!(rationale.failures_eliminated.len(), 1);
    assert_eq!(rationale.attack_vectors_closed.len(), 1);
    assert_eq!(rationale.impact_without_fix, "System compromise");
    assert_eq!(rationale.severity, SafetySeverity::High);
}

// ===== SafetySeverity tests =====

#[test]
fn test_safety_severity_eq() {
    assert_eq!(SafetySeverity::Critical, SafetySeverity::Critical);
    assert_ne!(SafetySeverity::Critical, SafetySeverity::High);
}

#[test]
fn test_safety_severity_clone() {
    let severities = [
        SafetySeverity::Critical,
        SafetySeverity::High,
        SafetySeverity::Medium,
        SafetySeverity::Low,
    ];
    for severity in severities {
        let _ = severity.clone();
    }
}

// ===== Alternative tests =====

#[test]
fn test_alternative_new() {
    let alt = Alternative::new(
        "Use set -e",
        "set -e; rm file",
        "When you want script to fail on error",
    );
    assert_eq!(alt.approach, "Use set -e");
    assert_eq!(alt.example, "set -e; rm file");
    assert_eq!(alt.when_to_use, "When you want script to fail on error");
    assert!(alt.pros.is_empty());
    assert!(alt.cons.is_empty());
}

#[test]
fn test_alternative_add_pro() {
    let alt = Alternative::new("Approach", "Example", "When").add_pro("Fast");
    assert!(alt.pros.contains(&"Fast".to_string()));
}

#[test]
fn test_alternative_add_con() {
    let alt = Alternative::new("Approach", "Example", "When").add_con("Complex");
    assert!(alt.cons.contains(&"Complex".to_string()));
}

#[test]
fn test_alternative_builder_chain() {
    let alt = Alternative::new("Approach", "Example", "When")
        .add_pro("Simple")
        .add_pro("Fast")
        .add_con("Verbose");

    assert_eq!(alt.pros.len(), 2);
    assert_eq!(alt.cons.len(), 1);
}

#[test]
fn test_alternative_clone() {
    let alt = Alternative::new("Approach", "Example", "When")
        .add_pro("Fast")
        .add_con("Complex");
    let cloned = alt.clone();
    assert_eq!(cloned.approach, "Approach");
    assert_eq!(cloned.pros.len(), 1);
}

// ===== TransformationExplanation tests =====

#[test]
fn test_transformation_explanation_new() {
    let exp = TransformationExplanation::new(
        TransformationCategory::Idempotency,
        "Use mkdir -p",
        "mkdir /dir",
        "mkdir -p /dir",
        "Added -p flag",
        "Prevents errors on rerun",
    );
    assert_eq!(exp.category, TransformationCategory::Idempotency);
    assert_eq!(exp.title, "Use mkdir -p");
    assert!(exp.line_number.is_none());
}

#[test]
fn test_transformation_explanation_with_line_number() {
    let exp = TransformationExplanation::new(
        TransformationCategory::Safety,
        "Title",
        "Original",
        "Transformed",
        "What",
        "Why",
    )
    .with_line_number(42);
    assert_eq!(exp.line_number, Some(42));
}

#[test]
fn test_transformation_explanation_with_safety_rationale() {
    let rationale = SafetyRationale::new().add_vulnerability("Injection");
    let exp = TransformationExplanation::new(
        TransformationCategory::Safety,
        "Title",
        "Original",
        "Transformed",
        "What",
        "Why",
    )
    .with_safety_rationale(rationale);
    assert!(!exp.safety_rationale.vulnerabilities_prevented.is_empty());
}

#[test]
fn test_transformation_explanation_with_alternatives() {
    let alternatives = vec![Alternative::new("Alt", "Example", "When")];
    let exp = TransformationExplanation::new(
        TransformationCategory::Determinism,
        "Title",
        "Original",
        "Transformed",
        "What",
        "Why",
    )
    .with_alternatives(alternatives);
    assert_eq!(exp.alternatives.len(), 1);
}

// ===== TransformationCategory tests =====

#[test]
fn test_transformation_category_eq() {
    assert_eq!(
        TransformationCategory::Idempotency,
        TransformationCategory::Idempotency
    );
    assert_ne!(
        TransformationCategory::Idempotency,
        TransformationCategory::Safety
    );
}

#[test]
fn test_transformation_category_clone() {
    let categories = [
        TransformationCategory::Idempotency,
        TransformationCategory::Determinism,
        TransformationCategory::Safety,
    ];
    for cat in categories {
        let _ = cat.clone();
    }
}

// ===== PurifiedLintResult tests =====

#[test]
fn test_purified_lint_result_new_clean() {
    let lint_result = LintResult::new();
    let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    assert!(plr.is_clean);
    assert_eq!(plr.critical_violations(), 0);
}

#[test]
fn test_purified_lint_result_det_violations_empty() {
    let lint_result = LintResult::new();
    let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    assert!(plr.det_violations().is_empty());
}

#[test]
fn test_purified_lint_result_idem_violations_empty() {
    let lint_result = LintResult::new();
    let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    assert!(plr.idem_violations().is_empty());
}

#[test]
fn test_purified_lint_result_sec_violations_empty() {
    let lint_result = LintResult::new();
    let plr = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    assert!(plr.sec_violations().is_empty());
}
}

// ===== PURIFIER_COV: Coverage tests for explain_purification_changes_detailed and format_transformation_report =====

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(non_snake_case)]
mod purifier_cov_tests {
use crate::repl::purifier_transforms::*;

// --- explain_purification_changes_detailed tests ---

#[test]
fn test_PURIFIER_COV_001_explain_changes_empty_no_transformation() {
    // ARRANGE: Code that needs no purification (already clean)
    let original = "echo hello";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: No changes detected, returns empty vec
    assert!(result.is_ok());
    let explanations = result.unwrap();
    assert!(
        explanations.is_empty(),
        "Already-pure code should return empty explanations"
    );
}

#[test]
fn test_PURIFIER_COV_002_explain_changes_with_determinism_random() {
    // ARRANGE: Code with $RANDOM (non-deterministic)
    let original = "x=$RANDOM";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should detect $RANDOM removal
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let random_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.category == TransformationCategory::Determinism)
        .filter(|e| e.title.contains("$RANDOM"))
        .collect();
    assert!(
        !random_explanations.is_empty(),
        "Should detect $RANDOM removal as Determinism transformation"
    );
    assert!(random_explanations[0].what_changed.contains("$RANDOM"));
    assert!(random_explanations[0]
        .why_it_matters
        .contains("unpredictable"));
}

#[test]
fn test_PURIFIER_COV_003_explain_changes_with_determinism_seconds() {
    // ARRANGE: Code with $SECONDS (non-deterministic timestamp)
    let original = "elapsed=$SECONDS";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should detect $SECONDS as timestamp removal
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let timestamp_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.category == TransformationCategory::Determinism)
        .filter(|e| e.title.contains("timestamp"))
        .collect();
    assert!(
        !timestamp_explanations.is_empty(),
        "Should detect $SECONDS removal as Determinism/timestamp transformation"
    );
    assert!(timestamp_explanations[0]
        .what_changed
        .contains("time-based"));
    assert!(timestamp_explanations[0]
        .why_it_matters
        .contains("non-reproducible"));
}

#[test]
fn test_PURIFIER_COV_004_explain_changes_with_idempotency_rm() {
    // ARRANGE: Code with rm that needs -f flag
    let original = "rm /tmp/file";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should detect rm → rm -f transformation
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let rm_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.category == TransformationCategory::Idempotency)
        .filter(|e| e.title.contains("rm"))
        .collect();
    assert!(
        !rm_explanations.is_empty(),
        "Should detect rm → rm -f as Idempotency transformation"
    );
    assert_eq!(rm_explanations[0].title, "rm → rm -f");
    assert!(rm_explanations[0].what_changed.contains("-f flag"));
    assert!(rm_explanations[0].why_it_matters.contains("safe to re-run"));
}

#[test]
fn test_PURIFIER_COV_005_explain_changes_with_idempotency_mkdir() {
    // ARRANGE: Code with mkdir that needs -p flag
    let original = "mkdir /tmp/test";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should detect mkdir → mkdir -p transformation
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let mkdir_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.category == TransformationCategory::Idempotency)
        .filter(|e| e.title.contains("mkdir"))
        .collect();
    assert!(
        !mkdir_explanations.is_empty(),
        "Should detect mkdir → mkdir -p as Idempotency transformation"
    );
    assert_eq!(mkdir_explanations[0].title, "mkdir → mkdir -p");
    assert!(mkdir_explanations[0].what_changed.contains("-p flag"));
    assert!(mkdir_explanations[0]
        .why_it_matters
        .contains("safe to re-run"));
}

#[test]
fn test_PURIFIER_COV_006_explain_changes_with_quoting_safety() {
    // ARRANGE: Code with unquoted variable
    let original = "echo $var";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should detect variable quoting as Safety transformation
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let safety_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.category == TransformationCategory::Safety)
        .collect();
    assert!(
        !safety_explanations.is_empty(),
        "Should detect unquoted variable as Safety transformation"
    );
    assert!(safety_explanations[0].title.contains("Quote"));
    assert!(safety_explanations[0].what_changed.contains("quotes"));
    assert!(safety_explanations[0].why_it_matters.contains("injection"));
}

#[test]
fn test_PURIFIER_COV_007_explain_changes_mixed_multiple() {
    // ARRANGE: Code with multiple issues
    let original = "mkdir /tmp/test\nrm /tmp/file\necho $var\nx=$RANDOM";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should detect multiple transformations
    assert!(result.is_ok());
    let explanations = result.unwrap();
    assert!(
        explanations.len() >= 3,
        "Mixed input should produce at least 3 explanations, got {}",
        explanations.len()
    );

    // Check categories present
    let has_idempotency = explanations
        .iter()
        .any(|e| e.category == TransformationCategory::Idempotency);
    let has_determinism = explanations
        .iter()
        .any(|e| e.category == TransformationCategory::Determinism);
    let has_safety = explanations
        .iter()
        .any(|e| e.category == TransformationCategory::Safety);

    assert!(
        has_idempotency,
        "Should have at least one Idempotency transformation"
    );
    assert!(
        has_determinism,
        "Should have at least one Determinism transformation"
    );
    assert!(has_safety, "Should have at least one Safety transformation");
}

#[test]
fn test_PURIFIER_COV_008_explain_changes_already_has_mkdir_p() {
    // ARRANGE: Code that already has mkdir -p (no transformation needed)
    let original = "mkdir -p /tmp/test";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should NOT detect mkdir transformation (already has -p)
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let mkdir_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.title.contains("mkdir"))
        .collect();
    assert!(
        mkdir_explanations.is_empty(),
        "Already-correct mkdir -p should not trigger a transformation"
    );
}

#[test]
fn test_PURIFIER_COV_009_explain_changes_already_has_rm_f() {
    // ARRANGE: Code that already has rm -f (no transformation needed)
    let original = "rm -f /tmp/file";

    // ACT: Get detailed explanations
    let result = explain_purification_changes_detailed(original);

    // ASSERT: Should NOT detect rm transformation (already has -f)
    assert!(result.is_ok());
    let explanations = result.unwrap();
    let rm_explanations: Vec<_> = explanations
        .iter()
        .filter(|e| e.title.contains("rm"))
        .collect();
    assert!(
        rm_explanations.is_empty(),
        "Already-correct rm -f should not trigger a transformation"
    );
}

// --- format_transformation_report tests ---

#[test]
fn test_PURIFIER_COV_010_format_report_single_idempotency() {
    // ARRANGE: Single idempotency transformation
    let transformations = vec![TransformationExplanation::new(
        TransformationCategory::Idempotency,
        "mkdir → mkdir -p",
        "mkdir /tmp/test",
        "mkdir -p /tmp/test",
        "Added -p flag",
        "Makes directory creation safe to re-run.",
    )];

    // ACT: Format the report
    let report = format_transformation_report(&transformations);

    // ASSERT: Report contains expected content
    assert!(report.contains("Transformation Report"));
    assert!(report.contains("IDEMPOTENCY"));
    assert!(report.contains("mkdir → mkdir -p"));
    assert!(report.contains("Added -p flag"));
    assert!(report.contains("safe to re-run"));
    assert!(report.contains("Original:"));
    assert!(report.contains("Transformed:"));
    assert!(report.contains("mkdir /tmp/test"));
    assert!(report.contains("mkdir -p /tmp/test"));
}
