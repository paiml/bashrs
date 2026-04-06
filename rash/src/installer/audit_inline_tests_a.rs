//! Tests extracted from audit.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::installer::audit::*;

/// Simplified test spec for audit testing
        #[test]
        fn prop_json_is_valid_structure(name in "[a-z]{5,10}", version in "[0-9]\\.[0-9]\\.[0-9]") {
            let report = AuditReport::new(&name, &version, PathBuf::from("/test"));
            let json = report.to_json();

            // Basic JSON structure validation
            let starts_correct = json.starts_with("{");
            let ends_correct = json.ends_with("}");
            let expected_name = format!("\"installer_name\": \"{}\"", name);
            let contains_name = json.contains(&expected_name);
            prop_assert!(starts_correct, "JSON should start with opening brace");
            prop_assert!(ends_correct, "JSON should end with closing brace");
            prop_assert!(contains_name, "JSON should contain installer name");
        }

/// Tests for Issue #112: audit postconditions not recognized with commands format
mod issue_112_tests {
    use crate::installer::audit::*;
    use crate::installer::spec::InstallerSpec;

    #[test]
    fn test_112_postconditions_verification_commands_recognized() {
        // Issue #112: Step with verification.commands should NOT trigger QUAL001
        let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"
description = "Test installer"

[[step]]
id = "install-app"
name = "Install Application"
action = "script"

[step.script]
content = "apt-get install app"

[step.verification]
commands = [
{ cmd = "which app", expect = "/usr/bin/app" }
]
"#;
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let ctx = AuditContext::new();
        let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

        // Should NOT have QUAL001 finding - verification.commands counts as postconditions
        let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
        assert!(
            qual001.is_none(),
            "QUAL001 should not be raised when verification.commands is present"
        );
    }

    #[test]
    fn test_112_postconditions_file_mode_recognized() {
        // Issue #112: Step with file_mode postcondition should NOT trigger QUAL001
        let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "set-perms"
name = "Set Permissions"
action = "script"

[step.script]
content = "chmod 755 /app"

[step.postconditions]
file_mode = "/app:755"
"#;
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let ctx = AuditContext::new();
        let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

        let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
        assert!(
            qual001.is_none(),
            "QUAL001 should not be raised when file_mode postcondition is present"
        );
    }

    #[test]
    fn test_112_postconditions_service_active_recognized() {
        // Issue #112: Step with service_active postcondition should NOT trigger QUAL001
        let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "start-service"
name = "Start Service"
action = "script"

[step.script]
content = "systemctl start myapp"

[step.postconditions]
service_active = "myapp"
"#;
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let ctx = AuditContext::new();
        let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

        let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
        assert!(
            qual001.is_none(),
            "QUAL001 should not be raised when service_active postcondition is present"
        );
    }

    #[test]
    fn test_112_postconditions_env_matches_recognized() {
        // Issue #112: Step with env_matches postcondition should NOT trigger QUAL001
        let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "setup-env"
name = "Setup Environment"
action = "script"

[step.script]
content = "export PATH=/app/bin:$PATH"

[step.postconditions.env_matches]
PATH = "/app/bin"
"#;
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let ctx = AuditContext::new();
        let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

        let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
        assert!(
            qual001.is_none(),
            "QUAL001 should not be raised when env_matches postcondition is present"
        );
    }

    #[test]
    fn test_112_postconditions_user_in_group_recognized() {
        // Issue #112: Step with user_in_group postcondition should NOT trigger QUAL001
        let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "add-group"
name = "Add User to Group"
action = "script"

[step.script]
content = "usermod -aG docker $USER"

[step.postconditions.user_in_group]
user = "deploy"
group = "docker"
"#;
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let ctx = AuditContext::new();
        let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

        let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
        assert!(
            qual001.is_none(),
            "QUAL001 should not be raised when user_in_group postcondition is present"
        );
    }

    #[test]
    fn test_112_no_postconditions_triggers_qual001() {
        // Sanity check: Step with NO postconditions SHOULD trigger QUAL001
        let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "no-post"
name = "Step Without Postconditions"
action = "script"

[step.script]
content = "echo hello"
"#;
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let ctx = AuditContext::new();
        let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

        let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
        assert!(
            qual001.is_some(),
            "QUAL001 should be raised when no postconditions are present"
        );
    }
}

// ============================================================================
// Coverage Tests - audit_security_parsed (SEC_COV_001-012)
// ============================================================================
mod security_parsed_tests {
    use crate::installer::audit::*;
    use crate::installer::spec::InstallerSpec;

    fn sec_audit(toml: &str) -> AuditReport {
        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        // Use new() not security_only() — security_only has min_severity=Warning
        // which filters out Info-level findings like SEC002 and SEC006
        let ctx = AuditContext::new();
        ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"))
    }

    #[test]
    fn test_SEC_COV_001_signatures_not_required() {
        // SEC001: require_signatures = false triggers warning
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = false
"#,
        );
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC001"));
    }

    #[test]
    fn test_SEC_COV_002_signatures_required_no_sec001() {
        // SEC001: require_signatures = true should NOT trigger
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = true
"#,
        );
        assert!(!report.findings.iter().any(|f| f.rule_id == "SEC001"));
    }

    #[test]
    fn test_SEC_COV_003_tofu_trust_model() {
        // SEC002: trust_model = "tofu" triggers info
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "tofu"
require_signatures = true
"#,
        );
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC002"));
    }

    #[test]
    fn test_SEC_COV_004_keyring_trust_model_no_sec002() {
        // SEC002: trust_model = "keyring" should NOT trigger
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = true
"#,
        );
        assert!(!report.findings.iter().any(|f| f.rule_id == "SEC002"));
    }

    #[test]
    fn test_SEC_COV_005_unsigned_artifact() {
        // SEC004: artifact without signature or signed_by
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
sha256 = "abc123"
"#,
        );
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC004"));
    }

    #[test]
    fn test_SEC_COV_006_signed_artifact_no_sec004() {
        // SEC004: signed artifact should NOT trigger
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
sha256 = "abc123"
signature = "myapp.sig"
signed_by = "key-001"
"#,
        );
        assert!(!report.findings.iter().any(|f| f.rule_id == "SEC004"));
    }

    #[test]
    fn test_SEC_COV_007_missing_sha256() {
        // SEC005: artifact without sha256
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
signature = "myapp.sig"
signed_by = "key-001"
"#,
        );
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC005"));
    }

    #[test]
    fn test_SEC_COV_008_root_privileges() {
        // SEC006: privileges = "root" triggers info
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "root"
"#,
        );
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC006"));
    }

    #[test]
    fn test_SEC_COV_009_user_privileges_no_sec006() {
        // SEC006: privileges = "user" should NOT trigger
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "user"
"#,
        );
        assert!(!report.findings.iter().any(|f| f.rule_id == "SEC006"));
    }

    #[test]
    fn test_SEC_COV_010_curl_pipe_bash() {
        // SEC007: curl ... | bash pattern
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "curl https://example.com/setup.sh | bash"
"#,
        );
        let sec007 = report.findings.iter().find(|f| f.rule_id == "SEC007");
        assert!(sec007.is_some());
        assert_eq!(
            sec007.expect("has sec007").severity,
            AuditSeverity::Critical
        );
    }

    #[test]
    fn test_SEC_COV_011_eval_in_script() {
        // SEC008: eval in script
        let report = sec_audit(
            r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "eval $DYNAMIC_CMD"
"#,
        );
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC008"));
    }
