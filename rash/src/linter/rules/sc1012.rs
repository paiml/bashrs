//! SC1012: `\t`/`\n`/`\r` is a literal escape the shell drops, not a real tab,
//! newline, or carriage return.
//!
//! GH-261: this used to mean "backslash-letter inside single quotes is
//! literal" — a bashrs-only meaning on a shellcheck code. shellcheck's SC1012
//! fires only where the *shell itself* strips the backslash: an **unquoted**
//! `\t`, `\n`, or `\r`. Inside single quotes the escape reaches the command
//! intact (exactly what `printf '...\n'`, `awk`, and `sed` programs rely on),
//! so it must never fire there. Inside double quotes the pair is also kept
//! literally (bash does not expand `\n` there either), so this rule stays
//! silent there too — that case is not this rule's subject.
//!
//! `echo 'a\nb'` is a different defect entirely (`echo`'s own escape
//! handling is shell-dependent) and is owned by SC2028/SC2271.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Which quoting context the scanner is currently inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteCtx {
    /// Not inside any quotes — backslash escapes are live shell syntax.
    None,
    /// Inside `'...'` — no escapes; only a `'` ends the string.
    Single,
    /// Inside `"..."` — `\` escapes the next byte (not this rule's subject).
    Double,
}

/// Check for escape-like sequences the shell drops (unquoted `\t`, `\n`, `\r`)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        check_line(line, line_num, &mut result);
    }

    result
}

/// Human-readable name of the real character an escape stands in for.
fn escape_name(c: u8) -> &'static str {
    match c {
        b't' => "tab",
        b'n' => "newline",
        _ => "carriage return",
    }
}

fn report_dropped_escape(line_num: usize, col: usize, c: u8, result: &mut LintResult) {
    let ch = c as char;
    let diagnostic = Diagnostic::new(
        "SC1012",
        Severity::Info,
        format!(
            "\\{ch} is just literal '{ch}' here. For a real {}, use printf",
            escape_name(c)
        ),
        Span::new(line_num, col + 1, line_num, col + 3),
    );
    result.add(diagnostic);
}

fn check_line(line: &str, line_num: usize, result: &mut LintResult) {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut ctx = QuoteCtx::None;
    let mut i = 0;

    while i < len {
        i = step(bytes, len, i, line_num, &mut ctx, result);
    }
}

/// Advance the scanner by one token, updating `ctx` and reporting a
/// diagnostic when an unquoted dropped-escape is found. Returns the next
/// byte index to resume scanning from.
///
/// A short dispatcher: all the per-context decisions live in
/// `step_single`/`step_double`/`step_none` below, so this function itself
/// has nothing to reason about beyond "which context am I in".
fn step(
    bytes: &[u8],
    len: usize,
    i: usize,
    line_num: usize,
    ctx: &mut QuoteCtx,
    result: &mut LintResult,
) -> usize {
    match *ctx {
        QuoteCtx::Single => step_single(bytes, i, ctx),
        QuoteCtx::Double => step_double(bytes, len, i, ctx),
        QuoteCtx::None => step_none(bytes, len, i, line_num, ctx, result),
    }
}

/// Which quoting context, if any, byte `b` opens.
fn quote_opened_by(b: u8) -> Option<QuoteCtx> {
    match b {
        b'\'' => Some(QuoteCtx::Single),
        b'"' => Some(QuoteCtx::Double),
        _ => None,
    }
}

/// If bytes `i..i+2` are an unquoted dropped-escape (`\t`, `\n`, `\r`),
/// the escaped character. Caller has already checked `bytes[i] == b'\\'`
/// and `i + 1 < len`.
fn dropped_escape_char(bytes: &[u8], i: usize) -> Option<u8> {
    let next = bytes[i + 1];
    matches!(next, b't' | b'n' | b'r').then_some(next)
}

/// Inside `'...'`: no escapes; only a `'` ends the string.
fn step_single(bytes: &[u8], i: usize, ctx: &mut QuoteCtx) -> usize {
    if bytes[i] == b'\'' {
        *ctx = QuoteCtx::None;
    }
    i + 1
}

/// Inside `"..."`: `\` escapes the next byte and never toggles quote state
/// (so `\"` does not close the string). Never reported here: an escape kept
/// literal in double quotes is not this rule's subject.
fn step_double(bytes: &[u8], len: usize, i: usize, ctx: &mut QuoteCtx) -> usize {
    if bytes[i] == b'\\' && i + 1 < len {
        return i + 2;
    }
    if bytes[i] == b'"' {
        *ctx = QuoteCtx::None;
    }
    i + 1
}

/// Outside any quotes: a `'`/`"` opens a quoting context, and an unquoted
/// `\t`/`\n`/`\r` is exactly the escape the shell drops, so it is reported.
fn step_none(
    bytes: &[u8],
    len: usize,
    i: usize,
    line_num: usize,
    ctx: &mut QuoteCtx,
    result: &mut LintResult,
) -> usize {
    let b = bytes[i];
    if let Some(opened) = quote_opened_by(b) {
        *ctx = opened;
        return i + 1;
    }
    if b == b'\\' && i + 1 < len {
        if let Some(esc) = dropped_escape_char(bytes, i) {
            report_dropped_escape(line_num, i, esc, result);
        }
        return i + 2;
    }
    i + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    // Single-quoted escapes reach the command intact; the shell never drops
    // the backslash there, so SC1012 does not fire. These three were
    // positives before GH-261; rewritten as negatives under the shellcheck
    // meaning.
    #[test]
    fn test_sc1012_tab_in_single_quotes_is_not_reported() {
        let script = r"echo 'hello\tworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1012_newline_in_single_quotes_is_not_reported() {
        let script = r"echo 'hello\nworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1012_carriage_return_in_single_quotes_is_not_reported() {
        let script = r"echo 'hello\rworld'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1012_no_false_positive_double_quotes() {
        let script = r#"echo "hello\tworld""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1012_no_false_positive_no_escape() {
        let script = "echo 'hello world'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Multiple escapes inside single quotes: zero diagnostics under the new
    // meaning — rewritten from a positive (count == 2) test.
    #[test]
    fn test_sc1012_multiple_escapes_in_single_quotes_is_not_reported() {
        let script = r"echo 'col1\tcol2\tcol3'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // New: unquoted `\t` — the shell drops the backslash, leaving a literal
    // 't'. This is exactly shellcheck's SC1012.
    #[test]
    fn test_sc1012_unquoted_tab_is_reported() {
        let script = r"echo a\tb";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1012");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("\\t"));
    }

    // New: unquoted `\n` at end of an `echo` argument.
    #[test]
    fn test_sc1012_unquoted_newline_is_reported() {
        let script = r"echo done\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("\\n"));
    }

    // New: a mixed line — the single-quoted `\n` reaches printf intact; the
    // unquoted `\t` is dropped by the shell. Only the unquoted one fires.
    #[test]
    fn test_sc1012_mixed_quoted_and_unquoted_reports_only_unquoted() {
        let script = r"printf 'x\n' a\tb";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("\\t"));
    }

    // New: a comment line is never scanned, regardless of its content.
    #[test]
    fn test_sc1012_comment_line_no_false_positive() {
        let script = "# echo a\\tb";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
