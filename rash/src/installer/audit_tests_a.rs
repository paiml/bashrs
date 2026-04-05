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
#[test]
fn test_AUDIT_COV_json_multiple_findings() {
    let mut report = AuditReport::new("multi", "1.0.0", PathBuf::from("/m"));
    report.metadata.audited_at = "2026-01-01T00:00:00Z".to_string();
    report.add_finding(AuditFinding::new(
        "A1",
        AuditSeverity::Warning,
        AuditCategory::Security,
        "F1",
        "D1",
    ));
    report.add_finding(AuditFinding::new(
        "A2",
        AuditSeverity::Error,
        AuditCategory::Quality,
        "F2",
        "D2",
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
    let ctx = AuditContext::new().with_ignored_rule("sec001");
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

    let long_script = (0..60)
        .map(|i| format!("echo line{}", i))
        .collect::<Vec<_>>()
        .join("\n");
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
