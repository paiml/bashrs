// SC1028: Parentheses in `[ ]` need escaping
//
// In single-bracket test expressions, parentheses must be escaped with `\`
// or they will be interpreted as subshell syntax.
//
// Examples:
// Bad:
//   [ (expr) ]
//   [ ( -f file ) ]
//
// Good:
//   [ \( expr \) ]
//   [ \( -f file \) ]
//   [[ (expr) ]]   # double brackets handle parens natively

use crate::linter::rules::quoting::is_inside_quoted_string;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Find bare `(` or `)` characters that are NOT part of `$(...)` command
/// substitution or `\(` / `\)` escaped parens.
/// Returns byte offsets of each bare paren.
/// Consume a multi-byte token at `i` that is NOT a bare paren, updating depths.
///
/// Returns the index just past it, or `None` when `i` is not one of them.
///
/// Issue #243: arithmetic expansion is tracked SEPARATELY from command
/// substitution because it is not symmetric with it. `$((` opens with two
/// parens and closes with two, but the old code matched it as a plain `$(`:
/// depth went up by ONE, both parens of `))` were then seen with depth 1 and 0
/// respectively, and the second fell through to the bare-paren arm. So
///
/// ```sh
/// [ -n "$(find /tmp -mmin "+$((H * 60))" 2>/dev/null)" ]
/// ```
///
/// produced three SC1028 findings telling the author to write `\(` — which
/// would break the script, since these parens are required syntax. shellcheck
/// accepts it. At Severity::Error this made the ordinary "did this command
/// produce output" idiom unlintable.
///
/// The empty `()` arm is a POSIX function definition — `name() { ... }`. An
/// empty pair is never test-grouping syntax, so it cannot be what this rule
/// looks for. Before this, a function defined on a line that also contained a
/// test was flagged twice, at the parens of the definition itself.
fn consume_non_bare_token(
    bytes: &[u8],
    i: usize,
    arith: &mut u32,
    cmd_sub: &mut u32,
) -> Option<usize> {
    let next = bytes.get(i + 1).copied();
    match bytes[i] {
        // An escaped character is already escaped; skip it entirely.
        b'\\' => Some(i + 2),
        b'$' if next == Some(b'(') && bytes.get(i + 2) == Some(&b'(') => {
            *arith += 1;
            Some(i + 3)
        }
        b')' if *arith > 0 && next == Some(b')') => {
            *arith -= 1;
            Some(i + 2)
        }
        b'$' if next == Some(b'(') => {
            *cmd_sub += 1;
            Some(i + 2)
        }
        b'(' if *cmd_sub == 0 && next == Some(b')') => Some(i + 2),
        _ => None,
    }
}

/// Positions of parens that are bare test-grouping syntax, not expansion,
/// not a function definition, and not text inside a quoted string.
fn find_bare_parens(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    let mut results = Vec::new();
    let mut cmd_sub_depth: u32 = 0;
    let mut arith_depth: u32 = 0;
    let mut i = 0;

    while i < bytes.len() {
        if let Some(next) = consume_non_bare_token(bytes, i, &mut arith_depth, &mut cmd_sub_depth) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'(' if cmd_sub_depth == 0 && !is_inside_quoted_string(line, i) => results.push(i),
            b')' if cmd_sub_depth > 0 => cmd_sub_depth -= 1,
            // Issue #243: parens inside a quoted string are text, not test
            // grouping. `log "waiting (${n}s elapsed)"` is not a syntax error.
            b')' if !is_inside_quoted_string(line, i) => results.push(i),
            _ => {}
        }
        i += 1;
    }
    results
}

/// Check if a line contains a single-bracket test `[ ... ]` (not `[[ ... ]]`).
/// True if the line contains a POSIX single-bracket test (`[ ... ]`).
///
/// A `[` opens one only when the next byte is a space. `[[` is bash's own
/// construct and is excluded by checking the preceding byte.
///
/// (The original also skipped when the byte after `[` was another `[`, which
/// could never fire: that byte had already been required to be a space.)
fn has_single_bracket_test(line: &str) -> bool {
    let bytes = line.as_bytes();
    bytes.iter().enumerate().any(|(i, &b)| {
        b == b'[' && bytes.get(i + 1) == Some(&b' ') && (i == 0 || bytes[i - 1] != b'[')
    })
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines with [[ (double bracket handles parens fine)
        if line.contains("[[") {
            continue;
        }

        // Only check lines that contain a single-bracket test
        if !has_single_bracket_test(line) {
            continue;
        }

        // Find bare parentheses (not $( or \() within the line
        for col in find_bare_parens(line) {
            let start_col = col + 1;
            let end_col = col + 2;

            let diagnostic = Diagnostic::new(
                "SC1028",
                Severity::Error,
                "Parentheses inside `[ ]` need escaping: use `\\(` and `\\)`".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1028_unescaped_paren() {
        let code = "[ (expr) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2); // ( and )
        assert_eq!(result.diagnostics[0].code, "SC1028");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1028_unescaped_paren_with_file_test() {
        let code = "[ ( -f file ) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2); // ( and )
    }

    #[test]
    fn test_sc1028_escaped_paren_ok() {
        let code = r"[ \( -f file \) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_double_bracket_ok() {
        let code = "[[ ( -f file ) ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_comment_ok() {
        let code = "# [ (expr) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_command_substitution_ok() {
        // $( ) inside [ ] should NOT trigger — it's command substitution, not grouping
        let code = "[ -n \"$(echo hello)\" ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_no_bracket_test() {
        let code = "echo (hello)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    /// Issue #243: parens belonging to arithmetic expansion are not test parens.
    #[test]
    fn test_arithmetic_expansion_parens_are_not_bare() {
        let code = r#"is_stale() { [ -n "$(find /tmp -mmin "+$((HOURS * 60))" 2>/dev/null)" ]; }"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired on $(( )) / $( ) parens, which are required syntax: {:?}",
            result.diagnostics
        );
    }

    /// Bare arithmetic expansion inside a test, without command substitution.
    #[test]
    fn test_arithmetic_expansion_alone_inside_test() {
        let code = r#"[ "$((a + b))" -gt 0 ] && echo yes"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired on a bare $(( )) inside a test: {:?}",
            result.diagnostics
        );
    }

    /// Guard the guard: a genuinely bare paren inside a test must STILL be
    /// reported, so the expansion-tracking cannot be widened into "never fire".
    #[test]
    fn test_genuinely_bare_paren_still_detected() {
        let code = r#"[ (a = b) ]"#;
        let result = check(code);
        assert!(
            !result.diagnostics.is_empty(),
            "a real bare paren in a test must still be flagged"
        );
    }
}
