#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for installer audit module.
//!
//! Focuses on uncovered edge cases in:
//! - AuditSeverity display methods
//! - AuditCategory display methods
//! - AuditFinding formatting with all optional fields
//! - AuditReport scoring edge cases (score floor, grade boundaries)
//! - AuditReport JSON export with special characters
//! - AuditReport format output with multiple categories
//! - AuditContext filtering (min_severity, ignored_rules)
//! - Hermetic audit rules (HERM001, HERM002, HERM003)
//! - Best practices audit rules (BP001-BP005)
//! - Quality audit rules (QUAL001-QUAL005)

use std::path::PathBuf;

use crate::installer::audit::{
    AuditCategory, AuditContext, AuditFinding, AuditMetadata, AuditReport, AuditSeverity,
};

// =============================================================================
// AuditSeverity edge cases
// =============================================================================

#[test]
fn test_AUDIT_COV_severity_suggestion_symbol() {
    assert_eq!(AuditSeverity::Suggestion.symbol(), "\u{1f4a1}");
}

#[test]
fn test_AUDIT_COV_severity_warning_symbol() {
    assert_eq!(AuditSeverity::Warning.symbol(), "\u{26a0}");
}

#[test]
fn test_AUDIT_COV_severity_error_symbol() {
    assert_eq!(AuditSeverity::Error.symbol(), "\u{274c}");
}

#[test]
fn test_AUDIT_COV_severity_all_names() {
    assert_eq!(AuditSeverity::Info.name(), "INFO");
    assert_eq!(AuditSeverity::Suggestion.name(), "SUGGESTION");
    assert_eq!(AuditSeverity::Warning.name(), "WARNING");
    assert_eq!(AuditSeverity::Error.name(), "ERROR");
    assert_eq!(AuditSeverity::Critical.name(), "CRITICAL");
}

#[test]
fn test_AUDIT_COV_severity_ordering_total() {
    let severities = [
        AuditSeverity::Info,
        AuditSeverity::Suggestion,
        AuditSeverity::Warning,
        AuditSeverity::Error,
        AuditSeverity::Critical,
    ];
    for i in 0..severities.len() {
        for j in (i + 1)..severities.len() {
            assert!(severities[i] < severities[j]);
        }
    }
}

#[test]
fn test_AUDIT_COV_severity_eq() {
    assert_eq!(AuditSeverity::Warning, AuditSeverity::Warning);
    assert_ne!(AuditSeverity::Warning, AuditSeverity::Error);
}

// =============================================================================
// AuditCategory edge cases
// =============================================================================

#[test]
fn test_AUDIT_COV_category_all_names() {
    assert_eq!(AuditCategory::Security.name(), "Security");
    assert_eq!(AuditCategory::Quality.name(), "Quality");
    assert_eq!(AuditCategory::Hermetic.name(), "Hermetic");
    assert_eq!(AuditCategory::BestPractices.name(), "Best Practices");
    assert_eq!(AuditCategory::Configuration.name(), "Configuration");
}

#[test]
fn test_AUDIT_COV_category_eq_and_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(AuditCategory::Security);
    set.insert(AuditCategory::Security);
    assert_eq!(set.len(), 1);
    set.insert(AuditCategory::Quality);
    assert_eq!(set.len(), 2);
}

// =============================================================================
// AuditFinding formatting edge cases
// =============================================================================

#[test]
fn test_AUDIT_COV_finding_format_with_all_optional_fields() {
    let finding = AuditFinding::new(
        "TEST001",
        AuditSeverity::Error,
        AuditCategory::Security,
        "Title here",
        "Description here",
    )
    .with_location("step-42")
    .with_suggestion("Fix the thing")
    .with_doc_url("https://docs.example.com/rule");

    let formatted = finding.format();
    assert!(formatted.contains("TEST001"));
    assert!(formatted.contains("ERROR"));
    assert!(formatted.contains("Title here"));
    assert!(formatted.contains("Description here"));
    assert!(formatted.contains("Location: step-42"));
    assert!(formatted.contains("Suggestion: Fix the thing"));
    // doc_url is stored but not printed in format()
    assert_eq!(
        finding.doc_url,
        Some("https://docs.example.com/rule".to_string())
    );
}

#[test]
fn test_AUDIT_COV_finding_format_without_optional_fields() {
    let finding = AuditFinding::new(
        "TEST002",
        AuditSeverity::Info,
        AuditCategory::Configuration,
        "Bare finding",
        "Just a description",
    );

    let formatted = finding.format();
    assert!(formatted.contains("TEST002"));
    assert!(formatted.contains("INFO"));
    assert!(!formatted.contains("Location:"));
    assert!(!formatted.contains("Suggestion:"));
}

#[test]
fn test_AUDIT_COV_finding_with_doc_url() {
    let finding = AuditFinding::new(
        "DOC001",
        AuditSeverity::Warning,
        AuditCategory::BestPractices,
        "Has docs",
        "Desc",
    )
    .with_doc_url("https://example.com");

    assert_eq!(finding.doc_url, Some("https://example.com".to_string()));
}

// =============================================================================
// AuditReport scoring edge cases
// =============================================================================

#[test]
fn test_AUDIT_COV_score_floor_at_zero() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    // Add 5 critical findings: 5 * 25 = 125 deductions > 100
    for i in 0..5 {
        report.add_finding(AuditFinding::new(
            format!("CRIT{}", i),
            AuditSeverity::Critical,
            AuditCategory::Security,
            "Critical",
            "D",
        ));
    }
    assert_eq!(report.score(), 0);
    assert_eq!(report.grade(), "F");
}

#[test]
fn test_AUDIT_COV_score_grade_boundary_90() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    // 1 error = -10, score = 90 => grade A
    report.add_finding(AuditFinding::new(
        "E1",
        AuditSeverity::Error,
        AuditCategory::Quality,
        "E",
        "D",
    ));
    assert_eq!(report.score(), 90);
    assert_eq!(report.grade(), "A");
}

#[test]
fn test_AUDIT_COV_score_grade_boundary_89() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    // 1 error (-10) + 1 suggestion (-1) = 89 => grade B
    report.add_finding(AuditFinding::new(
        "E1",
        AuditSeverity::Error,
        AuditCategory::Quality,
        "E",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "S1",
        AuditSeverity::Suggestion,
        AuditCategory::Quality,
        "S",
        "D",
    ));
    assert_eq!(report.score(), 89);
    assert_eq!(report.grade(), "B");
}

#[test]
fn test_AUDIT_COV_score_grade_c() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    // 3 errors = -30, score = 70 => grade C
    for i in 0..3 {
        report.add_finding(AuditFinding::new(
            format!("E{}", i),
            AuditSeverity::Error,
            AuditCategory::Quality,
            "E",
            "D",
        ));
    }
    assert_eq!(report.score(), 70);
    assert_eq!(report.grade(), "C");
}

#[test]
fn test_AUDIT_COV_score_grade_d() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    // 4 errors = -40, score = 60 => grade D
    for i in 0..4 {
        report.add_finding(AuditFinding::new(
            format!("E{}", i),
            AuditSeverity::Error,
            AuditCategory::Quality,
            "E",
            "D",
        ));
    }
    assert_eq!(report.score(), 60);
    assert_eq!(report.grade(), "D");
}

#[test]
fn test_AUDIT_COV_score_info_no_deduction() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    for i in 0..10 {
        report.add_finding(AuditFinding::new(
            format!("I{}", i),
            AuditSeverity::Info,
            AuditCategory::Configuration,
            "I",
            "D",
        ));
    }
    assert_eq!(report.score(), 100);
}

// =============================================================================
// AuditReport findings_by_severity / findings_by_category
// =============================================================================

#[test]
fn test_AUDIT_COV_findings_by_severity() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    report.add_finding(AuditFinding::new(
        "W1",
        AuditSeverity::Warning,
        AuditCategory::Security,
        "W",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "W2",
        AuditSeverity::Warning,
        AuditCategory::Quality,
        "W",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "E1",
        AuditSeverity::Error,
        AuditCategory::Security,
        "E",
        "D",
    ));

    let warnings = report.findings_by_severity(AuditSeverity::Warning);
    assert_eq!(warnings.len(), 2);

    let errors = report.findings_by_severity(AuditSeverity::Error);
    assert_eq!(errors.len(), 1);

    let criticals = report.findings_by_severity(AuditSeverity::Critical);
    assert_eq!(criticals.len(), 0);
}

#[test]
fn test_AUDIT_COV_findings_by_category() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    report.add_finding(AuditFinding::new(
        "S1",
        AuditSeverity::Warning,
        AuditCategory::Security,
        "S",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "Q1",
        AuditSeverity::Error,
        AuditCategory::Quality,
        "Q",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "Q2",
        AuditSeverity::Warning,
        AuditCategory::Quality,
        "Q",
        "D",
    ));

    let security = report.findings_by_category(AuditCategory::Security);
    assert_eq!(security.len(), 1);

    let quality = report.findings_by_category(AuditCategory::Quality);
    assert_eq!(quality.len(), 2);

    let hermetic = report.findings_by_category(AuditCategory::Hermetic);
    assert_eq!(hermetic.len(), 0);
}

// =============================================================================
// AuditReport format output with multiple categories
// =============================================================================

#[test]
fn test_AUDIT_COV_report_format_multiple_categories() {
    let mut report = AuditReport::new("multi-cat", "2.0.0", PathBuf::from("/multi"));
    report.metadata.audited_at = "2026-01-15T08:00:00Z".to_string();
    report.metadata.steps_audited = 3;
    report.metadata.artifacts_audited = 2;

    report.add_finding(
        AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Sec warn",
            "D",
        )
        .with_location("artifact-1"),
    );
    report.add_finding(AuditFinding::new(
        "QUAL001",
        AuditSeverity::Warning,
        AuditCategory::Quality,
        "Qual warn",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "HERM001",
        AuditSeverity::Info,
        AuditCategory::Hermetic,
        "Herm info",
        "D",
    ));
    report.add_finding(AuditFinding::new(
        "BP001",
        AuditSeverity::Suggestion,
        AuditCategory::BestPractices,
        "BP sug",
        "D",
    ));

    let formatted = report.format();
    assert!(formatted.contains("Installer Audit Report"));
    assert!(formatted.contains("multi-cat v2.0.0"));
    assert!(formatted.contains("3 steps, 2 artifacts"));
    assert!(formatted.contains("Score:"));
    assert!(formatted.contains("Grade:"));
    assert!(formatted.contains("Security\n"));
    assert!(formatted.contains("Quality\n"));
    assert!(formatted.contains("Hermetic\n"));
    assert!(formatted.contains("Best Practices\n"));
}

#[test]

include!("audit_tests_tests_AUDIT_2.rs");
