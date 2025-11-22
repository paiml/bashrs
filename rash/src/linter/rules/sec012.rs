//! SEC012: Unsafe Deserialization
//!
//! **Rule**: Detect unsafe deserialization of data from untrusted sources
//!
//! **Why this matters**:
//! Deserializing untrusted data without validation can lead to:
//! - Remote code execution via eval
//! - Arbitrary command execution via source
//! - Variable injection and environment poisoning
//! - Configuration tampering
//!
//! **Examples**:
//!
//! ❌ **DANGEROUS** (eval of JSON):
//! ```bash
//! # Attacker controls JSON, could inject malicious code
//! eval $(echo "$USER_JSON" | jq -r '. | to_entries[] | "\\(.key)=\\(.value)"')
//! ```
//!
//! ❌ **DANGEROUS** (source from remote):
//! ```bash
//! # Downloads and executes arbitrary code
//! source <(curl https://example.com/config.sh)
//! . <(wget -qO- https://example.com/setup.sh)
//! ```
//!
//! ✅ **SAFE** (validate before use):
//! ```bash
//! # Download, verify checksum, then source
//! curl -o config.sh https://example.com/config.sh
//! echo "$EXPECTED_SHA256  config.sh" | sha256sum -c || exit 1
//! source config.sh
//! ```
//!
//! ## Detection Patterns
//!
//! This rule detects:
//! - `eval $(... jq ...)` - JSON deserialization via eval
//! - `source <(curl ...)` - Remote code execution via source
//! - `eval $(curl ...)` - Direct eval of remote content
//! - `. <(wget ...)` - Alternative source syntax with wget
//!
//! ## Auto-fix
//!
//! This rule provides **warnings** but not automatic fixes, because:
//! - Context-dependent validation requirements
//! - Different security requirements per use case
//! - Requires understanding of data source trust model

use crate::linter::{Diagnostic, Severity, Span};
use crate::linter::LintResult;

/// Check for unsafe deserialization patterns
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Strip comments
        let code_only = if let Some(pos) = trimmed.find('#') {
            &trimmed[..pos]
        } else {
            trimmed
        };
        let code_only = code_only.trim();

        // Pattern 1: eval $(... jq ...) - JSON deserialization
        if code_only.contains("eval") && code_only.contains("jq") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC012",
                Severity::Error,
                "Unsafe deserialization: eval with jq can execute arbitrary code from JSON - validate data before eval or use safer parsing",
                span,
            );
            result.add(diag);
        }

        // Pattern 2: source <(curl ...) - Remote code execution
        if (code_only.contains("source") || code_only.starts_with('.')) && code_only.contains("<(curl") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC012",
                Severity::Error,
                "Unsafe deserialization: sourcing remote content without verification - download, verify checksum, then source",
                span,
            );
            result.add(diag);
        }

        // Pattern 3: source <(wget ...) - Remote code execution
        if (code_only.contains("source") || code_only.starts_with('.')) && code_only.contains("<(wget") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC012",
                Severity::Error,
                "Unsafe deserialization: sourcing remote content without verification - download, verify checksum, then source",
                span,
            );
            result.add(diag);
        }

        // Pattern 4: eval $(curl ...) - Direct eval of remote content
        if code_only.contains("eval") && code_only.contains("$(curl") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC012",
                Severity::Error,
                "Unsafe deserialization: eval of remote content without verification - download, verify, validate, then execute",
                span,
            );
            result.add(diag);
        }

        // Pattern 5: eval $(wget ...) - Direct eval of remote content
        if code_only.contains("eval") && code_only.contains("$(wget") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC012",
                Severity::Error,
                "Unsafe deserialization: eval of remote content without verification - download, verify, validate, then execute",
                span,
            );
            result.add(diag);
        }

        // Pattern 6: eval with yq (YAML deserialization)
        if code_only.contains("eval") && code_only.contains("yq") {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let diag = Diagnostic::new(
                "SEC012",
                Severity::Error,
                "Unsafe deserialization: eval with yq can execute arbitrary code from YAML - validate data before eval or use safer parsing",
                span,
            );
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first (EXTREME TDD)

    /// RED TEST 1: Detect eval with jq (JSON deserialization)
    #[test]
    fn test_SEC012_detects_eval_jq() {
        let script = r#"#!/bin/bash
eval $(echo "$JSON" | jq -r '. | to_entries[] | "\(.key)=\(.value)"')
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC012");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("jq"));
        assert!(diag.message.contains("eval"));
    }

    /// RED TEST 2: Detect source <(curl ...) - Remote code execution
    #[test]
    fn test_SEC012_detects_source_curl() {
        let script = r#"#!/bin/bash
source <(curl https://example.com/config.sh)
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC012");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("sourcing remote"));
    }

    /// RED TEST 3: Detect source <(wget ...) - Remote code execution
    #[test]
    fn test_SEC012_detects_source_wget() {
        let script = r#"#!/bin/bash
. <(wget -qO- https://example.com/setup.sh)
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC012");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("sourcing remote"));
    }

    /// RED TEST 4: Detect eval $(curl ...) - Direct remote eval
    #[test]
    fn test_SEC012_detects_eval_curl() {
        let script = r#"#!/bin/bash
eval $(curl https://example.com/script.sh)
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC012");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("eval of remote"));
    }

    /// RED TEST 5: Detect eval $(wget ...) - Direct remote eval
    #[test]
    fn test_SEC012_detects_eval_wget() {
        let script = r#"#!/bin/bash
eval $(wget -qO- https://example.com/vars.sh)
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC012");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("eval of remote"));
    }

    /// RED TEST 6: Detect eval with yq (YAML deserialization)
    #[test]
    fn test_SEC012_detects_eval_yq() {
        let script = r#"#!/bin/bash
eval $(yq '.config' config.yaml)
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC012");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("yq"));
    }

    /// RED TEST 7: Pass safe jq usage (no eval)
    #[test]
    fn test_SEC012_passes_safe_jq() {
        let script = r#"#!/bin/bash
# Safe: parse JSON without eval
CONFIG=$(echo "$JSON" | jq -r '.config')
echo "$CONFIG"
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Safe jq usage should pass");
    }

    /// RED TEST 8: Pass safe source usage (local file)
    #[test]
    fn test_SEC012_passes_safe_source() {
        let script = r#"#!/bin/bash
# Safe: source local verified file
source ./config.sh
. /etc/bashrc
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Safe source usage should pass");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY TEST 1: Never panics on any input
        #[test]
        fn prop_sec012_never_panics(s in ".*") {
            let _ = check(&s);
        }

        /// PROPERTY TEST 2: Always detects eval with jq
        #[test]
        fn prop_sec012_detects_eval_jq(
            json_var in "[A-Z_]{1,20}",
        ) {
            let script = format!("eval $(echo \"${}\" | jq -r '.')", json_var);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC012");
        }

        /// PROPERTY TEST 3: Always detects source with curl
        #[test]
        fn prop_sec012_detects_source_curl(
            url in "https?://[a-z.]{5,20}/[a-z]{1,10}\\.sh",
        ) {
            let script = format!("source <(curl {})", url);
            let result = check(&script);

            prop_assert_eq!(result.diagnostics.len(), 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC012");
        }
    }
}
