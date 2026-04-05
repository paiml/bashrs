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
    assert!(report
        .findings
        .iter()
        .any(|f| f.rule_id == "QUAL001" || f.rule_id == "QUAL003"));
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
    assert!(report
        .findings
        .iter()
        .all(|f| f.severity >= AuditSeverity::Error));
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
    assert!(report
        .findings
        .iter()
        .all(|f| f.severity >= AuditSeverity::Warning));
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
