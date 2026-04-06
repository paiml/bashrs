//! Installer Audit Command (#120)
//!
//! Security and quality review for installer specifications.
//!
//! This module provides comprehensive auditing capabilities:
//!
//! - **Security Audit**: Check signature requirements, trust model, privilege escalation
//! - **Quality Audit**: Validate idempotency, preconditions, postconditions
//! - **Hermetic Audit**: Verify lockfile integrity, reproducibility settings
//! - **Best Practices**: Check for common anti-patterns and recommendations
//!
//! # Example
//!
//! ```ignore
//! use bashrs::installer::{AuditContext, AuditReport, AuditSeverity};
//!
//! let ctx = AuditContext::new();
//! let report = ctx.audit_installer(&spec)?;
//!
//! if report.has_critical_issues() {
//!     eprintln!("Critical issues found!");
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Severity level for audit findings

#[cfg(test)]
mod tests {
    use super::*;

    /// Simplified test spec for audit testing
    #[derive(Debug, Clone)]
    struct TestSpec {
        name: String,
        version: String,
        description: String,
        author: Option<String>,
        security: Option<TestSecurity>,
        artifacts: Vec<TestArtifact>,
        steps: Vec<TestStep>,
    }

    #[derive(Debug, Clone)]
    struct TestSecurity {
        trust_model: String,
        require_signatures: bool,
    }

    #[derive(Debug, Clone)]
    struct TestArtifact {
        id: String,
        url: String,
        sha256: Option<String>,
        signature: Option<String>,
        signed_by: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct TestStep {
        id: String,
        name: String,
        action: TestAction,
        depends_on: Vec<String>,
        has_postconditions: bool,
        has_checkpoint: bool,
        has_timing: bool,
    }

    #[derive(Debug, Clone)]
    enum TestAction {
        Script { content: String },
        Other,
    }

    fn create_test_spec() -> TestSpec {
        TestSpec {
            name: "test-installer".to_string(),
            version: "1.0.0".to_string(),
            description: "Test installer".to_string(),
            author: Some("Test Author".to_string()),
            security: Some(TestSecurity {
                trust_model: "keyring".to_string(),
                require_signatures: true,
            }),
            artifacts: vec![],
            steps: vec![TestStep {
                id: "step-1".to_string(),
                name: "First Step".to_string(),
                action: TestAction::Script {
                    content: "echo hello".to_string(),
                },
                depends_on: vec![],
                has_postconditions: true,
                has_checkpoint: true,
                has_timing: true,
            }],
        }
    }

    /// Simple audit for test specs (mimics real audit logic)
    fn audit_test_spec(spec: &TestSpec, path: &Path) -> AuditReport {
        let mut report = AuditReport::new(&spec.name, &spec.version, path.to_path_buf());
        audit_test_security(&spec.security, &mut report);
        audit_test_artifacts(&spec.artifacts, &mut report);
        audit_test_steps(&spec.steps, &mut report);
        if spec.description.is_empty() {
            report.add_finding(AuditFinding::new(
                "BP001",
                AuditSeverity::Suggestion,
                AuditCategory::BestPractices,
                "Missing installer description",
                "No description field.",
            ));
        }
        report.metadata.audited_at = chrono_timestamp();
        report.metadata.steps_audited = spec.steps.len();
        report.metadata.artifacts_audited = spec.artifacts.len();
        report
    }

    fn audit_test_security(security: &Option<TestSecurity>, report: &mut AuditReport) {
        let Some(ref security) = security else {
            report.add_finding(AuditFinding::new(
                "SEC003",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "No security configuration",
                "No security section defined.",
            ));
            return;
        };
        if !security.require_signatures {
            report.add_finding(AuditFinding::new(
                "SEC001",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "Signatures not required",
                "Artifact signature verification is disabled.",
            ));
        }
        if security.trust_model == "tofu" {
            report.add_finding(AuditFinding::new(
                "SEC002",
                AuditSeverity::Info,
                AuditCategory::Security,
                "Using TOFU model",
                "TOFU is suitable for development.",
            ));
        }
    }

    fn audit_test_artifacts(artifacts: &[TestArtifact], report: &mut AuditReport) {
        for artifact in artifacts {
            if artifact.signature.is_none() && artifact.signed_by.is_none() {
                report.add_finding(
                    AuditFinding::new(
                        "SEC004",
                        AuditSeverity::Warning,
                        AuditCategory::Security,
                        "Unsigned artifact",
                        format!("Artifact '{}' has no signature.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
            if artifact.sha256.is_none() {
                report.add_finding(
                    AuditFinding::new(
                        "SEC005",
                        AuditSeverity::Error,
                        AuditCategory::Security,
                        "Missing artifact hash",
                        format!("Artifact '{}' has no SHA256.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
            if artifact.url.contains("latest") {
                report.add_finding(
                    AuditFinding::new(
                        "HERM002",
                        AuditSeverity::Warning,
                        AuditCategory::Hermetic,
                        "Unpinned artifact version",
                        format!("Artifact '{}' uses unpinned version.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
        }
    }

    fn audit_test_steps(steps: &[TestStep], report: &mut AuditReport) {
        let step_ids: std::collections::HashSet<&str> =
            steps.iter().map(|s| s.id.as_str()).collect();
        let mut seen_ids: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for step in steps {
            audit_test_step_quality(step, &step_ids, &mut seen_ids, report);
        }
    }

    fn audit_test_step_quality<'a>(
        step: &'a TestStep,
        step_ids: &std::collections::HashSet<&str>,
        seen_ids: &mut std::collections::HashSet<&'a str>,
        report: &mut AuditReport,
    ) {
        if seen_ids.contains(step.id.as_str()) {
            report.add_finding(
                AuditFinding::new(
                    "QUAL004",
                    AuditSeverity::Error,
                    AuditCategory::Quality,
                    "Duplicate step ID",
                    format!("Step ID '{}' is duplicated.", step.id),
                )
                .with_location(&step.id),
            );
        }
        seen_ids.insert(&step.id);

        for dep in &step.depends_on {
            if !step_ids.contains(dep.as_str()) {
                report.add_finding(
                    AuditFinding::new(
                        "QUAL005",
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Invalid dependency reference",
                        format!("Step '{}' depends on non-existent '{}'.", step.id, dep),
                    )
                    .with_location(&step.id),
                );
            }
        }

        if !step.has_postconditions {
            report.add_finding(
                AuditFinding::new(
                    "QUAL001",
                    AuditSeverity::Warning,
                    AuditCategory::Quality,
                    "Missing postconditions",
                    format!("Step '{}' has no postconditions.", step.id),
                )
                .with_location(&step.id),
            );
        }

        if let TestAction::Script { ref content } = step.action {
            if content.contains("curl") && content.contains("| bash") {
                report.add_finding(
                    AuditFinding::new(
                        "SEC007",
                        AuditSeverity::Critical,
                        AuditCategory::Security,
                        "Unsafe curl pipe to bash",
                        "Step contains 'curl ... | bash' pattern.",
                    )
                    .with_location(&step.id),
                );
            }
        }
    }

    #[test]
    fn test_AUDIT_120_severity_ordering() {
        assert!(AuditSeverity::Info < AuditSeverity::Suggestion);
        assert!(AuditSeverity::Suggestion < AuditSeverity::Warning);
        assert!(AuditSeverity::Warning < AuditSeverity::Error);
        assert!(AuditSeverity::Error < AuditSeverity::Critical);
    }

    #[test]
    fn test_AUDIT_120_severity_symbols() {
        assert_eq!(AuditSeverity::Info.symbol(), "ℹ");
        assert_eq!(AuditSeverity::Critical.symbol(), "🚨");
    }

    #[test]
    fn test_AUDIT_120_category_names() {
        assert_eq!(AuditCategory::Security.name(), "Security");
        assert_eq!(AuditCategory::Quality.name(), "Quality");
        assert_eq!(AuditCategory::Hermetic.name(), "Hermetic");
    }

    #[test]
    fn test_AUDIT_120_finding_creation() {
        let finding = AuditFinding::new(
            "TEST001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test finding",
            "This is a test finding",
        );

        assert_eq!(finding.rule_id, "TEST001");
        assert_eq!(finding.severity, AuditSeverity::Warning);
        assert_eq!(finding.category, AuditCategory::Security);
    }

    #[test]
    fn test_AUDIT_120_finding_with_location() {
        let finding = AuditFinding::new(
            "TEST002",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Test",
            "Description",
        )
        .with_location("step-1")
        .with_suggestion("Fix it");

        assert_eq!(finding.location, Some("step-1".to_string()));
        assert_eq!(finding.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_AUDIT_120_finding_format() {
        let finding = AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test title",
            "Test description",
        );

        let formatted = finding.format();
        assert!(formatted.contains("SEC001"));
        assert!(formatted.contains("WARNING"));
        assert!(formatted.contains("Test title"));
    }

    #[test]
    fn test_AUDIT_120_report_creation() {
        let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert_eq!(report.installer_name, "test");
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_AUDIT_120_report_add_finding() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        report.add_finding(AuditFinding::new(
            "TEST001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test",
            "Description",
        ));

        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn test_AUDIT_120_report_has_critical() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert!(!report.has_critical_issues());

        report.add_finding(AuditFinding::new(
            "CRIT001",
            AuditSeverity::Critical,
            AuditCategory::Security,
            "Critical",
            "Description",
        ));

        assert!(report.has_critical_issues());
    }

    #[test]
    fn test_AUDIT_120_report_has_errors() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert!(!report.has_errors());

        report.add_finding(AuditFinding::new(
            "ERR001",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Error",
            "Description",
        ));

        assert!(report.has_errors());
    }

    #[test]
    fn test_AUDIT_120_report_score_perfect() {
        let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert_eq!(report.score(), 100);
        assert_eq!(report.grade(), "A");
    }

    #[test]
    fn test_AUDIT_120_report_score_with_findings() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        // Add warning (-3) and error (-10)
        report.add_finding(AuditFinding::new(
            "WARN001",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "Warning",
            "Description",
        ));

        report.add_finding(AuditFinding::new(
            "ERR001",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Error",
            "Description",
        ));

        assert_eq!(report.score(), 87); // 100 - 3 - 10
        assert_eq!(report.grade(), "B");
    }

    #[test]
    fn test_AUDIT_120_report_count_by_severity() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        report.add_finding(AuditFinding::new(
            "W1",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "W1",
            "D",
        ));
        report.add_finding(AuditFinding::new(
            "W2",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "W2",
            "D",
        ));
        report.add_finding(AuditFinding::new(
            "E1",
            AuditSeverity::Error,
            AuditCategory::Security,
            "E1",
            "D",
        ));

        let counts = report.count_by_severity();
        assert_eq!(counts.get(&AuditSeverity::Warning), Some(&2));
        assert_eq!(counts.get(&AuditSeverity::Error), Some(&1));
    }

    #[test]
    fn test_AUDIT_120_report_format() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        report.metadata.audited_at = "2025-12-28T10:00:00Z".to_string();

        report.add_finding(AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test Warning",
            "Description",
        ));

        let formatted = report.format();
        assert!(formatted.contains("Installer Audit Report"));
        assert!(formatted.contains("test v1.0.0"));
        assert!(formatted.contains("SEC001"));
    }

    #[test]
    fn test_AUDIT_120_report_to_json() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        report.metadata.audited_at = "2025-12-28T10:00:00Z".to_string();


}
}

        include!("audit_part4_incl2.rs");
