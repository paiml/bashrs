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
    assert_eq!(finding.doc_url, Some("https://docs.example.com/rule".to_string()));
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
        "E1", AuditSeverity::Error, AuditCategory::Quality, "E", "D",
    ));
    assert_eq!(report.score(), 90);
    assert_eq!(report.grade(), "A");
}

#[test]
fn test_AUDIT_COV_score_grade_boundary_89() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    // 1 error (-10) + 1 suggestion (-1) = 89 => grade B
    report.add_finding(AuditFinding::new(
        "E1", AuditSeverity::Error, AuditCategory::Quality, "E", "D",
    ));
    report.add_finding(AuditFinding::new(
        "S1", AuditSeverity::Suggestion, AuditCategory::Quality, "S", "D",
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
            format!("E{}", i), AuditSeverity::Error, AuditCategory::Quality, "E", "D",
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
            format!("E{}", i), AuditSeverity::Error, AuditCategory::Quality, "E", "D",
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
            format!("I{}", i), AuditSeverity::Info, AuditCategory::Configuration, "I", "D",
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
        "W1", AuditSeverity::Warning, AuditCategory::Security, "W", "D",
    ));
    report.add_finding(AuditFinding::new(
        "W2", AuditSeverity::Warning, AuditCategory::Quality, "W", "D",
    ));
    report.add_finding(AuditFinding::new(
        "E1", AuditSeverity::Error, AuditCategory::Security, "E", "D",
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
        "S1", AuditSeverity::Warning, AuditCategory::Security, "S", "D",
    ));
    report.add_finding(AuditFinding::new(
        "Q1", AuditSeverity::Error, AuditCategory::Quality, "Q", "D",
    ));
    report.add_finding(AuditFinding::new(
        "Q2", AuditSeverity::Warning, AuditCategory::Quality, "Q", "D",
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
        AuditFinding::new("SEC001", AuditSeverity::Warning, AuditCategory::Security, "Sec warn", "D")
            .with_location("artifact-1"),
    );
    report.add_finding(
        AuditFinding::new("QUAL001", AuditSeverity::Warning, AuditCategory::Quality, "Qual warn", "D"),
    );
    report.add_finding(
        AuditFinding::new("HERM001", AuditSeverity::Info, AuditCategory::Hermetic, "Herm info", "D"),
    );
    report.add_finding(
        AuditFinding::new("BP001", AuditSeverity::Suggestion, AuditCategory::BestPractices, "BP sug", "D"),
    );

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
fn test_AUDIT_COV_report_format_empty_findings() {
    let report = AuditReport::new("clean", "1.0.0", PathBuf::from("/clean"));
    let formatted = report.format();
    assert!(formatted.contains("Score: 100/100"));
    assert!(formatted.contains("Grade: A"));
    // No category sections printed
    assert!(!formatted.contains("Security\n-"));
}

#[test]
fn test_AUDIT_COV_report_format_summary_only_nonzero() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    report.add_finding(AuditFinding::new(
        "E1", AuditSeverity::Error, AuditCategory::Quality, "E", "D",
    ));
    let formatted = report.format();
    // Summary should show ERROR: 1, but not other severity lines with 0 count
    assert!(formatted.contains("ERROR"));
    // Info line should not appear since count is 0
    assert!(!formatted.contains("INFO: 0"));
}

// =============================================================================
// AuditReport JSON export edge cases
// =============================================================================

#[test]
fn test_AUDIT_COV_json_with_special_chars_in_title() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    report.metadata.audited_at = "2026-01-01T00:00:00Z".to_string();
    report.add_finding(
        AuditFinding::new(
            "SP001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Title with \"quotes\"",
            "Description with \"quotes\"",
        )
        .with_suggestion("Use 'single quotes' or escape \"doubles\""),
    );

    let json = report.to_json();
    assert!(json.contains("\"installer_name\": \"test\""));
    assert!(json.contains("\"rule_id\": \"SP001\""));
    // Quotes in title/description should be escaped
    assert!(json.contains("\\\"quotes\\\""));
}

#[test]
fn test_AUDIT_COV_json_with_null_location_and_suggestion() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    report.metadata.audited_at = "2026-02-01T00:00:00Z".to_string();
    report.add_finding(AuditFinding::new(
        "NUL001",
        AuditSeverity::Info,
        AuditCategory::Configuration,
        "No location",
        "No suggestion either",
    ));

    let json = report.to_json();
    assert!(json.contains("\"location\": null"));
    assert!(json.contains("\"suggestion\": null"));
}

#[test]
fn test_AUDIT_COV_json_with_location_and_suggestion() {
    let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    report.metadata.audited_at = "2026-02-01T00:00:00Z".to_string();
    report.add_finding(
        AuditFinding::new(
            "LOC001",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "Has location",
            "Has suggestion",
        )
        .with_location("step-7")
        .with_suggestion("Do the fix"),
    );

    let json = report.to_json();
    assert!(json.contains("\"location\": \"step-7\""));
    assert!(json.contains("\"suggestion\": \"Do the fix\""));
}

#[test]
fn test_AUDIT_COV_json_empty_findings() {
    let report = AuditReport::new("empty", "0.1.0", PathBuf::from("/empty"));
    let json = report.to_json();
    assert!(json.contains("\"findings\": ["));
    assert!(json.contains("\"score\": 100"));
    assert!(json.contains("\"grade\": \"A\""));
}

#[test]
fn test_AUDIT_COV_json_multiple_findings() {
    let mut report = AuditReport::new("multi", "1.0.0", PathBuf::from("/m"));
    report.metadata.audited_at = "2026-01-01T00:00:00Z".to_string();
    report.add_finding(AuditFinding::new(
        "A1", AuditSeverity::Warning, AuditCategory::Security, "F1", "D1",
    ));
    report.add_finding(AuditFinding::new(
        "A2", AuditSeverity::Error, AuditCategory::Quality, "F2", "D2",
    ));

    let json = report.to_json();
    // Multiple findings separated by comma
    assert!(json.contains("\"rule_id\": \"A1\""));
    assert!(json.contains("\"rule_id\": \"A2\""));
}

// =============================================================================
// AuditContext filtering (min_severity, ignored_rules)
// =============================================================================

#[test]
fn test_AUDIT_COV_context_with_min_severity() {
    let ctx = AuditContext::new().with_min_severity(AuditSeverity::Error);
    assert_eq!(ctx.min_severity, AuditSeverity::Error);
    assert!(ctx.check_security);
    assert!(ctx.check_quality);
}

#[test]
fn test_AUDIT_COV_context_with_ignored_rule() {
    let ctx = AuditContext::new()
        .with_ignored_rule("SEC001")
        .with_ignored_rule("qual002");

    assert!(ctx.ignored_rules.contains("SEC001"));
    // Case normalization
    assert!(ctx.ignored_rules.contains("QUAL002"));
}

#[test]
fn test_AUDIT_COV_context_ignored_rule_case_insensitive() {
    let ctx = AuditContext::new()
        .with_ignored_rule("sec001");
    assert!(ctx.ignored_rules.contains("SEC001"));
}

// =============================================================================
// AuditContext with parsed spec (integration-style, using real TOML)
// =============================================================================

#[test]
fn test_AUDIT_COV_parsed_spec_hermetic_no_lockfile_with_artifacts() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp-1.0.tar.gz"
sha256 = "deadbeef"
signature = "myapp.sig"
signed_by = "key-1"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::new();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // HERM001: no lockfile config with artifacts present
    assert!(report.findings.iter().any(|f| f.rule_id == "HERM001"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_hermetic_unpinned_version_variable() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp-${VERSION}.tar.gz"
sha256 = "deadbeef"
signature = "myapp.sig"
signed_by = "key-1"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::new();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // HERM002: URL contains ${VERSION}
    assert!(report.findings.iter().any(|f| f.rule_id == "HERM002"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_hermetic_network_steps() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "download"
name = "Download stuff"
action = "script"

[step.script]
content = "curl https://example.com/file.tar.gz -o file.tar.gz"

[[step]]
id = "update"
name = "Update packages"
action = "script"

[step.script]
content = "apt-get update && apt-get install -y pkg"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::new();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // HERM003: network-dependent steps (curl and apt-get update)
    let herm003 = report.findings.iter().find(|f| f.rule_id == "HERM003");
    assert!(herm003.is_some());
    assert!(herm003.unwrap().description.contains("2 steps"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_bp003_missing_step_name() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "nameless"
action = "script"

[step.script]
content = "echo hello"
"#;
    // InstallerSpec::parse requires `name` field — this TOML is intentionally
    // invalid to test BP003 (missing step name). If parsing fails, the audit
    // can't run, so we verify the parse error mentions "name".
    match InstallerSpec::parse(toml) {
        Ok(spec) => {
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));
            assert!(report.findings.iter().any(|f| f.rule_id == "BP003"));
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("name"),
                "Parse error should mention missing 'name': {msg}"
            );
        }
    }
}

#[test]
fn test_AUDIT_COV_parsed_spec_bp004_orphan_step() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "step-1"
name = "First"
action = "script"

[step.script]
content = "echo first"

[[step]]
id = "step-2"
name = "Second"
action = "script"

[step.script]
content = "echo second"

[[step]]
id = "step-3"
name = "Third"
action = "script"
depends_on = ["step-1"]

[step.script]
content = "echo third"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::new();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // BP004: step-2 is orphan (not first, no deps, nothing depends on it)
    let bp004 = report.findings.iter().find(|f| f.rule_id == "BP004");
    assert!(bp004.is_some());
    assert!(bp004.unwrap().description.contains("step-2"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_bp005_long_script() {
    use crate::installer::spec::InstallerSpec;

    let long_script = (0..60).map(|i| format!("echo line{}", i)).collect::<Vec<_>>().join("\n");
    let toml = format!(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "long-step"
name = "Very long step"
action = "script"

[step.script]
content = """
{}
"""
"#,
        long_script
    );
    let spec = InstallerSpec::parse(&toml).unwrap();
    let ctx = AuditContext::new();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // BP005: script with > 50 lines
    let bp005 = report.findings.iter().find(|f| f.rule_id == "BP005");
    assert!(bp005.is_some());
    assert!(bp005.unwrap().description.contains("60 lines"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_qual003_no_timeout() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "no-timeout"
name = "No timeout"
action = "script"

[step.script]
content = "echo hello"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::new();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // QUAL003: no timeout specified
    assert!(report.findings.iter().any(|f| f.rule_id == "QUAL003"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_ignored_rules_filter() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = false
trust_model = "tofu"

[[step]]
id = "step-1"
name = "Step 1"
action = "script"

[step.script]
content = "echo hello"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();

    // Ignore SEC001 and SEC002
    let ctx = AuditContext::new()
        .with_ignored_rule("SEC001")
        .with_ignored_rule("SEC002");
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // SEC001 and SEC002 should be filtered out
    assert!(!report.findings.iter().any(|f| f.rule_id == "SEC001"));
    assert!(!report.findings.iter().any(|f| f.rule_id == "SEC002"));
    // But other findings should remain
    assert!(report.findings.iter().any(|f| f.rule_id == "QUAL001" || f.rule_id == "QUAL003"));
}

#[test]
fn test_AUDIT_COV_parsed_spec_min_severity_filters_low() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = false
trust_model = "tofu"

[[step]]
id = "step-1"
name = "Step 1"
action = "script"

[step.script]
content = "echo hello"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::new().with_min_severity(AuditSeverity::Error);
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // Only Error+ findings should remain
    assert!(report.findings.iter().all(|f| f.severity >= AuditSeverity::Error));
}

#[test]
fn test_AUDIT_COV_parsed_spec_security_only_context() {
    use crate::installer::spec::InstallerSpec;

    let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = false

[[step]]
id = "step-1"
name = "Step 1"
action = "script"

[step.script]
content = "echo hello"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ctx = AuditContext::security_only();
    let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

    // Should have SEC001 (signatures not required)
    assert!(report.findings.iter().any(|f| f.rule_id == "SEC001"));
    // Should NOT have quality findings since check_quality is false
    // But min_severity is Warning for security_only, so suggestions are also filtered
    assert!(report.findings.iter().all(|f| f.severity >= AuditSeverity::Warning));
}

// =============================================================================
// AuditMetadata direct testing
// =============================================================================

#[test]
fn test_AUDIT_COV_metadata_default_values() {
    let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
    assert_eq!(report.metadata.audited_at, "");
    assert_eq!(report.metadata.steps_audited, 0);
    assert_eq!(report.metadata.artifacts_audited, 0);
    assert_eq!(report.metadata.duration_ms, 0);
}
