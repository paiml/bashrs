//! Tests extracted from audit.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::installer::audit::*;

/// Simplified test spec for audit testing
    #[test]
    fn test_SEC_COV_012_clean_spec_no_findings() {
        // Fully clean spec should have no security findings
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

[[artifact]]
id = "myapp"
url = "https://example.com/myapp-1.0.0.tar.gz"
sha256 = "abc123def456"
signature = "myapp.sig"
signed_by = "key-001"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "tar xf myapp.tar.gz && ./install.sh"
"#,
        );
        let sec_findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id.starts_with("SEC"))
            .collect();
        assert!(
            sec_findings.is_empty(),
            "Clean spec should have no SEC findings, got: {sec_findings:?}"
        );
    }
}
