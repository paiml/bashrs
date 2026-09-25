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

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Words after which a new command starts (`command eval`, `builtin eval` and
/// `time eval` still run the builtin).
const COMMAND_PREFIX_KEYWORDS: &[&str] = &[
    "if", "then", "do", "else", "elif", "while", "until", "command", "builtin", "time",
];

/// Does `code` invoke the `eval` builtin: the word `eval` where a command can
/// start (line start, after `;` `|` `&` `(` `` ` `` `{` `!`, or after a keyword
/// like `if`), followed by a blank or the end of the line? A substring test
/// read `.eval_count` in a jq program as the builtin (bashrs#375).
fn has_eval_command(code: &str) -> bool {
    code.match_indices("eval").any(|(i, _)| {
        let ends_word = code[i + 4..].chars().next().is_none_or(char::is_whitespace);
        let before = code[..i].trim_end();
        let starts_command = before.is_empty()
            || before.ends_with([';', '|', '&', '(', '`', '{', '!'])
            || (before.len() < i
                && COMMAND_PREFIX_KEYWORDS
                    .iter()
                    .any(|k| before.rsplit(|c: char| !c.is_alphanumeric()).next() == Some(*k)));
        ends_word && starts_command
    })
}

const EVAL_JQ: &str = "Unsafe deserialization: eval with jq can execute arbitrary code from JSON - validate data before eval or use safer parsing";
const SOURCE_REMOTE: &str = "Unsafe deserialization: sourcing remote content without verification - download, verify checksum, then source";
const EVAL_REMOTE: &str = "Unsafe deserialization: eval of remote content without verification - download, verify, validate, then execute";
const EVAL_YQ: &str = "Unsafe deserialization: eval with yq can execute arbitrary code from YAML - validate data before eval or use safer parsing";

fn is_source(code: &str) -> bool {
    code.contains("source") || code.starts_with('.')
}

/// Each pattern: (is the line an eval, else a source; the payload it needs; the message).
const PATTERNS: &[(bool, &str, &str)] = &[
    (true, "jq", EVAL_JQ),                // eval $(... jq ...) - JSON deserialization
    (false, "<(curl", SOURCE_REMOTE),     // source <(curl ...) - remote code execution
    (false, "<(wget", SOURCE_REMOTE),     // source <(wget ...)
    (true, "$(curl", EVAL_REMOTE),        // eval $(curl ...) - direct eval of remote content
    (true, "$(wget", EVAL_REMOTE),        // eval $(wget ...)
    (true, "yq", EVAL_YQ),                // eval with yq (YAML deserialization)
];

/// Check for unsafe deserialization patterns
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Strip comments
        let trimmed = line.trim();
        let code_only = trimmed.find('#').map_or(trimmed, |pos| &trimmed[..pos]).trim();
        let (eval, source) = (has_eval_command(code_only), is_source(code_only));

        for &(needs_eval, payload, message) in PATTERNS {
            let command = if needs_eval { eval } else { source };
            if command && code_only.contains(payload) {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                result.add(Diagnostic::new("SEC012", Severity::Error, message, span));
            }
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

    /// bashrs#375: `eval` inside a jq field name is not the eval builtin.
    #[test]
    fn test_SEC012_eval_in_a_jq_field_name_is_not_eval() {
        let script = r#"ct=$(jq -r '.eval_count // 0' <<<"$line"); pt=$(jq -r '.prompt_eval_count // 0' <<<"$line")"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    /// bashrs#375: the command word still fires wherever a command can start.
    #[test]
    fn test_SEC012_eval_as_a_command_word_still_fires() {
        for script in [
            r#"eval "$(jq -r '.a' "$f")""#,
            r#"x=1; eval $(jq -r '.a' f)"#,
            r#"true && eval "$(jq -r .a f)""#,
            r#"if eval "$(jq -r .a f)"; then :; fi"#,
            r#"out=$(eval "$(jq -r .a f)")"#,
            r#"command eval "$(jq -r .a f)""#,
            r#"builtin eval "$(jq -r .a f)""#,
            r#"time eval "$(jq -r .a f)""#,
        ] {
            let result = check(script);
            assert_eq!(result.diagnostics.len(), 1, "{script}");
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
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
