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

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// One `[ … ]` test on a line, as a byte range STRICTLY BETWEEN the bracket
/// and its closing `]`.
///
/// Issue: SC1028 used to ask two independent questions — "does this line
/// contain a `[ ` anywhere?" and "does this line contain a bare paren
/// anywhere?" — and report the second whenever the first was true. The parens
/// never had to be inside the test. On a 1200-line script of ordinary shell,
/// 148 of 148 SC1028 findings were parens that live somewhere a test cannot
/// reach:
///
/// ```sh
/// [ "$n" = 3 ] && say "  (collapses to the standing line tonight)"
/// [ "${#argv[@]}" -gt 0 ] && try_i+=(fail)
/// [ -f "$t" ] || { warn "no table $t (see peers/README)"; return 0; }
/// ```
///
/// Every one is `\(`-unfixable: escaping those parens changes the string, or
/// breaks the array append, and `[ ]` has nothing to do with any of them.
/// shellcheck reports none of the three. At Severity::Error that made the
/// commonest shape in POSIX shell — a short-circuit after a test — unlintable,
/// which is the third time this rule has been widened past its own subject
/// (see the arithmetic-expansion note below).
///
/// So the scan is bounded by the test itself. A `[` opens one only outside
/// quotes, at command-substitution depth 0, when the next byte is a space and
/// the previous byte is not `[`; the matching `]` closes it.
/// Shell quoting state, fed one byte at a time.
///
/// Factored out because BOTH scanners below need it and neither had it: a
/// paren inside `'…'` or `"…"` is a character in a string, and telling the
/// author to escape it would change the string. `[ "$x" = "(" ]` is legal,
/// means what it says, and shellcheck is silent on it.
#[derive(Default)]
struct Quotes {
    squote: bool,
    dquote: bool,
}

impl Quotes {
    /// Consume `b` if it is an unescaped quote delimiter, toggling state.
    /// Returns whether it was consumed, so callers just advance.
    fn feed(&mut self, b: u8) -> bool {
        match b {
            b'\'' if !self.dquote => {
                self.squote = !self.squote;
                true
            }
            b'"' if !self.squote => {
                self.dquote = !self.dquote;
                true
            }
            _ => false,
        }
    }

    fn inside(&self) -> bool {
        self.squote || self.dquote
    }
}

/// True when byte `i` opens a POSIX single-bracket test.
///
/// A `[` opens one only when the next byte is a space; `[[` is bash's own
/// construct and is excluded by checking the preceding byte.
fn opens_test(bytes: &[u8], i: usize) -> bool {
    bytes[i] == b'[' && bytes.get(i + 1) == Some(&b' ') && (i == 0 || bytes[i - 1] != b'[')
}

/// Each `[ … ]` on the line, as a byte range STRICTLY BETWEEN the bracket and
/// its closing `]`.
///
/// SC1028 used to ask two independent questions — "does this line contain a
/// `[ ` anywhere?" and "does this line contain a bare paren anywhere?" — and
/// report the second whenever the first was true. The parens never had to be
/// inside the test. On a 1200-line script of ordinary shell, 148 of 148
/// SC1028 findings were parens somewhere a test cannot reach:
///
/// ```sh
/// [ "$n" = 3 ] && say "  (collapses to the standing line tonight)"
/// [ "${#argv[@]}" -gt 0 ] && try_i+=(fail)
/// [ -f "$t" ] || { warn "no table $t (see peers/README)"; return 0; }
/// ```
///
/// Every one is `\(`-unfixable: escaping those parens changes the string or
/// breaks the array append, and `[ ]` has nothing to do with any of them.
/// shellcheck reports none of the three. At Severity::Error that made the
/// commonest shape in POSIX shell — a short-circuit after a test —
/// unlintable, which is the third time this rule has been widened past its own
/// subject (see the arithmetic-expansion note below). The common cause each
/// time was handing the WHOLE LINE to a scanner and asking it to reason about
/// which parens belong to a construct it could not see, so the scan is now
/// bounded by the construct.
fn test_spans(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    let mut q = Quotes::default();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if !q.feed(bytes[i]) && !q.inside() {
            if start.is_none() && opens_test(bytes, i) {
                start = Some(i + 1);
            } else if bytes[i] == b']' {
                if let Some(s) = start.take() {
                    spans.push((s, i));
                }
            }
        }
        i += 1;
    }
    // An unterminated `[` (a continuation line) is still scanned to the end of
    // the line: dropping it would silently stop reporting real bare parens in
    // a multi-line test.
    if let Some(s) = start {
        spans.push((s, bytes.len()));
    }
    spans
}

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
///
/// `offset` is where `slice` begins in the original line, so the returned
/// positions are line-relative and callers do not have to add it back.
///
/// Quote tracking is new alongside the span bound above: a paren inside `'…'`
/// or `"…"` is a character in a string, and telling the author to escape it
/// would change the string's contents. `[ "$x" = "(" ]` is legal, means what
/// it says, and shellcheck is silent on it.
/// Decide what a paren byte means. `Some(true)` = report it.
///
/// Split out of the scan loop so neither function carries the whole grammar:
/// the two `)` arms differ only by command-substitution depth, which is easy
/// to lose track of inline and was lost track of twice already.
fn classify_paren(b: u8, cmd_sub: &mut u32, dquote: bool) -> Option<bool> {
    match b {
        b'(' if *cmd_sub == 0 && !dquote => Some(true),
        b')' if *cmd_sub > 0 => {
            *cmd_sub -= 1;
            Some(false)
        }
        b')' if !dquote => Some(true),
        _ => None,
    }
}

/// Positions of parens that are bare test-grouping syntax, not expansion, not
/// a function definition, and not text inside a quoted string.
///
/// `offset` is where `slice` begins in the original line, so the returned
/// positions are line-relative and callers do not have to add it back.
fn find_bare_parens(slice: &str, offset: usize) -> Vec<usize> {
    let bytes = slice.as_bytes();
    let mut results = Vec::new();
    let mut cmd_sub_depth: u32 = 0;
    let mut arith_depth: u32 = 0;
    let mut q = Quotes::default();
    let mut i = 0;

    while i < bytes.len() {
        if q.feed(bytes[i]) || q.squote {
            i += 1;
            continue;
        }
        if let Some(next) = consume_non_bare_token(bytes, i, &mut arith_depth, &mut cmd_sub_depth) {
            i = next;
            continue;
        }
        if classify_paren(bytes[i], &mut cmd_sub_depth, q.dquote) == Some(true) {
            results.push(offset + i);
        }
        i += 1;
    }
    results
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

        // Bare parens INSIDE each `[ … ]`, and nowhere else on the line. The
        // span bound is the fix: `has_single_bracket_test(line)` answered "is
        // there a test somewhere?" and then handed the WHOLE line to the paren
        // scanner, so `[ -n "$x" ] && f "(note)"` was reported at the parens of
        // a string a test cannot see.
        for span in test_spans(line) {
            let (s, e) = span;
            for col in find_bare_parens(&line[s..e], s) {
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

    // ── The parens are not in the test ────────────────────────────────────
    //
    // Every case below was a Severity::Error finding whose suggested fix —
    // "use `\(` and `\)`" — would have CHANGED THE PROGRAM: escaping a paren
    // inside a string alters the string, and escaping an array append breaks
    // it. shellcheck reports none of them. Measured on one 1200-line script:
    // 148 of 148 SC1028 findings were of these shapes.

    #[test]
    fn parens_in_a_string_after_the_test_are_not_test_parens() {
        let code = r#"[ "$n" = 3 ] && say "  (collapses to the standing line tonight)""#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired on parens inside a string that follows the test: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn array_append_after_the_test_is_not_a_test_paren() {
        let code = r#"[ "${#argv[@]}" -gt 0 ] && try_i+=(fail)"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired on an array append after the test: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn parens_in_a_brace_group_after_the_test_are_not_test_parens() {
        let code = r#"[ -f "$t" ] || { warn "no table $t (see peers/README)"; return 0; }"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired inside a `||` brace group: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn a_paren_inside_a_quoted_operand_is_a_character_not_syntax() {
        // `[ "$x" = "(" ]` is legal and means what it says. Escaping it would
        // compare against `\(` instead.
        let code = r#"[ "$x" = "(" ]"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired on a paren that is a quoted operand: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn a_paren_in_a_single_quoted_operand_is_a_character_not_syntax() {
        let code = r#"[ "$k" = '(' ] && echo yes"#;
        let result = check(code);
        assert!(
            result.diagnostics.is_empty(),
            "SC1028 fired on a single-quoted paren operand: {:?}",
            result.diagnostics
        );
    }

    // ── Guard the guard ───────────────────────────────────────────────────
    //
    // Narrowing a rule is one edit away from disabling it, so the span bound
    // has to be shown NOT to swallow the thing the rule exists for. Each of
    // these must still fire.

    #[test]
    fn bare_paren_still_flagged_when_the_line_continues_after_the_test() {
        let code = r#"[ (a = b) ] && echo yes"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            2,
            "the test's own bare parens must still be reported: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn bare_paren_flagged_in_the_second_of_two_tests_on_a_line() {
        let code = r#"[ -f a ] && [ (x) ]"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            2,
            "a bare paren in the SECOND test on the line must be reported: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn reported_columns_point_at_the_offending_parens() {
        // Columns are 1-based. In `[ (a) ]` the parens are at byte 2 and 4.
        let code = r#"[ (a) ]"#;
        let result = check(code);
        let cols: Vec<usize> = result
            .diagnostics
            .iter()
            .map(|d| d.span.start_col)
            .collect();
        assert_eq!(
            cols,
            vec![3, 5],
            "the span must point at the paren, not at an offset into a slice"
        );
    }
}
