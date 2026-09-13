//! SC2081: Expressions don't expand in single quotes, use double quotes for that
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo 'Value: $var'  # Prints literal $var
//! echo 'Result: $(cmd)'  # Prints literal $(cmd)
//! ```
//!
//! Good:
//! ```bash
//! echo "Value: $var"  # Expands $var
//! echo "Result: $(cmd)"  # Expands $(cmd)
//! echo 'Literal $var'  # OK if literal is intended
//! ```
//!
//! # Rationale
//!
//! Single quotes preserve everything literally:
//! - $var is not expanded
//! - $(cmd) is not executed
//! - \n is literal backslash-n
//!
//! This is often not what the user intended.
//!
//! # Auto-fix
//!
//! Offers double quotes as SAFE-WITH-ASSUMPTIONS, never SAFE (bashrs#335).
//! Turning `'…$x…'` into `"…$x…"` makes `$x` expand, which is the rule's whole
//! premise and is never semantics-preserving: text that another evaluator reads
//! later -- a trap action, `eval`, `sh -c`, `ssh host '…'`, `awk`, `sed` -- is
//! single-quoted on purpose. The content's own `"`, `\` and `` ` `` are escaped so
//! the rewrite is still ONE word.

use crate::linter::quoted_segments::single_quoted_segments;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// A variable or command expansion that single quotes are keeping literal.
static SC2081_EXPANSION: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{?[A-Za-z_][A-Za-z0-9_]*\}?|\$\([^)]+\)").unwrap());

/// What must hold for the double-quoted rewrite to mean what was meant.
const EXPANSION_WAS_INTENDED: &str = "the expansion is meant to happen where the string is \
     written, not later: a trap action, eval, sh -c, ssh, awk or sed argument is \
     single-quoted on purpose";

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // bashrs#335: quotes are paired inside the words a quote-aware lexer
        // delimited, never across the raw line.
        for seg in single_quoted_segments(line) {
            if !SC2081_EXPANSION.is_match(&seg.content) {
                continue;
            }

            let diagnostic = Diagnostic::new(
                "SC2081",
                Severity::Info,
                "Expressions don't expand in single quotes, use double quotes for that",
                Span::new(line_num, seg.start_col, line_num, seg.end_col),
            )
            .with_fix(Fix::new_with_assumptions(
                double_quoted(&seg.content),
                vec![EXPANSION_WAS_INTENDED.to_string()],
            ));

            result.add(diagnostic);
        }
    }

    result
}

/// `content` inside double quotes, still one word.
///
/// `"`, `\` and `` ` `` are special inside double quotes and are escaped -- which
/// is what 7.4.0 did not do (`trap "rm -rf -- "${TD:?}"" EXIT`). `$` is left
/// alone on purpose: making it expand is the point of the rewrite.
fn double_quoted(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 2);
    out.push('"');
    for ch in content.chars() {
        if matches!(ch, '"' | '\\' | '`') {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2081_basic_detection() {
        let script = r#"echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2081");
    }

    #[test]
    fn test_sc2081_autofix() {
        let script = r#"echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"Value: $var\""
        );
    }

    #[test]
    fn test_sc2081_command_substitution() {
        let script = r#"echo 'Result: $(date)'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_braced_variable() {
        let script = r#"echo 'Name: ${username}'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_false_positive_double_quotes() {
        let script = r#"echo "Value: $var""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2081_false_positive_literal_only() {
        let script = r#"echo 'Hello World'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2081_false_positive_in_comment() {
        let script = r#"# echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2081_multiple_variables() {
        let script = r#"echo 'Name: $name, Age: $age'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_in_assignment() {
        let script = r#"msg='Error: $error_code'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_mixed_content() {
        let script = r#"echo 'Path is $HOME/bin'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"Path is $HOME/bin\""
        );
    }

    // bashrs#335. `bashrs fix` rewrote a single-quoted trap into a double-quoted
    // one and collapsed four arguments into one, on 7.3.0 and on published
    // 7.4.0. Two defects: the detector matched `'…$x…'` against the raw line with
    // no quoting state, so it paired the CLOSING quote of one string with the
    // OPENING quote of the next; and the fix wrapped content in `"` without
    // escaping the content's own `"`, and marked that SAFE.

    #[test]
    fn test_sc2081_335_no_match_across_string_boundaries() {
        // The only `$` is inside a double-quoted word that sits BETWEEN two
        // single-quoted strings. Nothing single-quoted contains an expansion.
        let script = r#"assert_row 'append one entry' PASS "$TD/append.yaml" 'added=1'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_sc2081_335_unquoted_expansion_between_strings_is_not_single_quoted() {
        let script = r#"f 'a' $x 'b'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_sc2081_335_fix_changes_meaning_so_it_is_not_safe() {
        // Single -> double quotes turns a literal into an expansion. That is the
        // rule's premise, and it is never semantics-preserving, so it must not be
        // applied by a default `bashrs fix`.
        let script = r#"echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        let fix = result.diagnostics[0]
            .fix
            .as_ref()
            .expect("a fix is still offered");
        assert!(
            !fix.is_safe(),
            "a change of quoting meaning must not be SAFE"
        );
    }

    #[test]
    fn test_sc2081_335_fix_escapes_embedded_double_quotes() {
        // 7.4.0 produced `trap "rm -rf -- "${TD:?}"" EXIT`: the content's own `"`
        // closed the new string early and left ${TD:?} unquoted.
        let script = r#"trap 'rm -rf -- "${TD:?}"' EXIT"#;
        let result = check(script);
        for d in &result.diagnostics {
            if let Some(fix) = &d.fix {
                assert_eq!(fix.replacement, r#""rm -rf -- \"${TD:?}\"""#);
            }
        }
    }
}
