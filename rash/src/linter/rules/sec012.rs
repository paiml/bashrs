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

/// Does `code` hold `eval` as a whole shell word: preceded by the line start, a
/// blank or a command separator (`;` `|` `&` `(` `` ` `` `{` `!`), and followed by
/// a blank or the line end? That covers every wrapper (`if`, `command`,
/// `builtin`, `sudo`, `env`, `nice`, `timeout`, ...) without a list to fall out of
/// date. A substring test read `.eval_count` in a jq program as the builtin
/// (bashrs#375); a word test does not, because `eval` there is part of a word.
fn has_eval_command(code: &str) -> bool {
    code.match_indices("eval").any(|(i, _)| {
        let starts_word = code[..i]
            .chars()
            .next_back()
            .is_none_or(|c| c.is_whitespace() || ";|&(`{!".contains(c));
        let ends_word = code[i + 4..].chars().next().is_none_or(char::is_whitespace);
        starts_word && ends_word
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
    (true, "jq", EVAL_JQ),            // eval $(... jq ...) - JSON deserialization
    (false, "<(curl", SOURCE_REMOTE), // source <(curl ...) - remote code execution
    (false, "<(wget", SOURCE_REMOTE), // source <(wget ...)
    (true, "$(curl", EVAL_REMOTE),    // eval $(curl ...) - direct eval of remote content
    (true, "$(wget", EVAL_REMOTE),    // eval $(wget ...)
    (true, "yq", EVAL_YQ),            // eval with yq (YAML deserialization)
];

/// The eval message for a payload in `text`, if any: the eval patterns of
/// [`PATTERNS`], in order.
fn eval_payload(text: &str) -> Option<&'static str> {
    PATTERNS
        .iter()
        .find(|&&(needs_eval, payload, _)| needs_eval && text.contains(payload))
        .map(|&(_, _, message)| message)
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Every `NAME=value` in `code`, the value running to the end of the code.
/// `NAME` must start a word, so `local`/`export`/`readonly`/`declare` forms are
/// found and `[[ $a == b ]]` is not.
fn assignments(code: &str) -> impl Iterator<Item = (&str, &str)> {
    code.match_indices('=').filter_map(move |(eq, _)| {
        // Step past the boundary char by its width: it may be multi-byte.
        let start = code[..eq]
            .char_indices()
            .rev()
            .find(|&(_, c)| !is_name_char(c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let name = &code[start..eq];
        let starts_word = code[..start]
            .chars()
            .next_back()
            .is_none_or(|c| c.is_whitespace() || ";|&(".contains(c));
        let valid = name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_');
        (starts_word && valid).then(|| (name, &code[eq + 1..]))
    })
}

/// The command substitution `value` starts with (`$(..)`, `"$(..)"` or
/// backticks), opener to its own close: `A=$(date); echo "$(jq ..)"` holds
/// `date`, not the rest of the line. `None` for anything else.
fn substitution(value: &str) -> Option<&str> {
    let value = value.strip_prefix('"').unwrap_or(value);
    if let Some(body) = value.strip_prefix('`') {
        return Some(body.find('`').map_or(body, |end| &body[..end]));
    }
    let body = value.strip_prefix("$(")?;
    let mut depth = 1;
    for (i, c) in body.char_indices() {
        depth += match c {
            '(' => 1,
            ')' => -1,
            _ => 0,
        };
        if depth == 0 {
            return Some(&value[..i + 3]);
        }
    }
    Some(value)
}

/// Does `code` expand `$name` or `${name...}`, and not a longer name?
fn expands(code: &str, name: &str) -> bool {
    [format!("${{{name}"), format!("${name}")]
        .iter()
        .any(|form| {
            code.match_indices(form.as_str())
                .any(|(i, _)| !code[i + form.len()..].starts_with(is_name_char))
        })
}

/// Check for unsafe deserialization patterns
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    // bashrs#428: variables holding a payload substitution, with the line
    // (1-based) that assigned them and the message an eval of them earns.
    let mut tainted: Vec<(String, usize, &'static str)> = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Strip comments
        let trimmed = line.trim();
        let code_only = trimmed
            .find('#')
            .map_or(trimmed, |pos| &trimmed[..pos])
            .trim();
        let (eval, source) = (has_eval_command(code_only), is_source(code_only));
        let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
        let mut fired = false;

        for &(needs_eval, payload, message) in PATTERNS {
            let command = if needs_eval { eval } else { source };
            if command && code_only.contains(payload) {
                result.add(Diagnostic::new("SEC012", Severity::Error, message, span));
                fired = true;
            }
        }

        // bashrs#428: `X=$(yq ..)` then `eval "$X"` is `eval "$(yq ..)"`.
        if eval && !fired {
            if let Some((name, at, message)) = tainted.iter().find(|(n, ..)| expands(code_only, n))
            {
                let message = format!("{message} (${name} assigned on line {at})");
                result.add(Diagnostic::new("SEC012", Severity::Error, message, span));
            }
        }

        for (name, value) in assignments(code_only) {
            tainted.retain(|(n, ..)| n != name);
            if let Some(message) = substitution(value).and_then(eval_payload) {
                tainted.push((name.to_string(), line_num + 1, message));
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
            r#"sudo eval "$(jq -r .a f)""#,
            r#"env X=1 eval "$(jq -r .a f)""#,
            r#"nice -n19 eval "$(jq -r .a f)""#,
            r#"timeout 5 eval "$(jq -r .a f)""#,
            "eval\t\"$(jq -r .a f)\"",
        ] {
            let result = check(script);
            assert_eq!(result.diagnostics.len(), 1, "{script}");
        }
    }

    /// bashrs#428: the substitution reaches eval through a variable. Each of
    /// these is the same deserialization as `eval "$(yq ...)"`, one line apart.
    #[test]
    fn test_SEC012_eval_of_a_var_assigned_from_a_payload_fires() {
        for (script, word) in [
            ("YQ=$(yq -r .env f.yaml)\neval \"$YQ\"", "yq"),
            ("YQ=\"$(yq -r .env f.yaml)\"\neval \"$YQ\"", "yq"),
            ("YQ=`yq -r .env f.yaml`\neval $YQ", "yq"),
            ("J=$(jq -r .a f.json)\neval \"${J}\"", "jq"),
            ("local J=\"$(jq -r .a f.json)\"\neval \"$J\"", "jq"),
            ("export R=$(curl -s https://x/y)\neval \"$R\"", "remote"),
            (
                "R=$(wget -qO- https://x/y)\nif eval \"$R\"; then :; fi",
                "remote",
            ),
            ("N=$(echo \"$(yq -r .a f)\")\neval \"$N\"", "yq"),
        ] {
            let result = check(script);
            assert_eq!(result.diagnostics.len(), 1, "{script}");
            let diag = &result.diagnostics[0];
            assert_eq!(diag.code, "SEC012");
            assert_eq!(diag.span.start_line, 2, "{script}");
            assert!(diag.message.contains(word), "{script}: {}", diag.message);
            assert!(
                diag.message.contains("line 1"),
                "{script}: {}",
                diag.message
            );
        }
    }

    /// bashrs#428: what must stay silent — a literal string is not
    /// deserialized output, a reassignment drops the taint, a different
    /// variable or a longer name sharing the prefix is not the tainted one,
    /// and the direct form still fires exactly once.
    #[test]
    fn test_SEC012_eval_of_an_untainted_var_stays_silent() {
        for script in [
            "YQ=\"yq -r .a f.yaml\"\neval \"$YQ\"",
            "YQ=\"yq -r .a f.yaml\"\neval $YQ",
            "YQ=$(yq -r .a f.yaml)\nYQ=\"echo hi\"\neval \"$YQ\"",
            "YQ=$(yq -r .a f.yaml)\neval \"$OTHER\"",
            "YQ=$(yq -r .a f.yaml)\neval \"$YQX\"",
            "YQ=$(yq -r .a f.yaml)\necho \"$YQ\"",
            "N=$(date +%s)\neval \"$N\"",
            "A=$(date); echo \"$(jq -r .a f)\"\neval \"$A\"",
            "A=$(date) && B=$(curl -s https://x)\neval \"$A\"",
            "A=`date`; B=`yq .a f`\neval \"$A\"",
        ] {
            let result = check(script);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "{script}: {:?}",
                result.diagnostics
            );
        }
        // A multi-byte char just before `NAME=` (found by prop_sec012_never_panics).
        assert_eq!(
            check("¥YQ=$(yq .a f)\n€=1\neval \"$YQ\"").diagnostics.len(),
            0
        );
        let direct = check("YQ=$(yq -r .a f.yaml)\neval \"$(yq -r .b f.yaml) $YQ\"");
        assert_eq!(direct.diagnostics.len(), 1, "{:?}", direct.diagnostics);
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
