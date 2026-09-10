// SC2047: Quote this to prevent word splitting, or use [[ ]] for regex.
//
// Unquoted variables in test conditions can cause unexpected word splitting.
// This leads to syntax errors or incorrect test results.
//
// Examples:
// Bad:
//   [ -z $var ]              // If var is empty, becomes [ -z ], syntax error
//   [ $count -gt 5 ]         // If count="1 2", becomes [ 1 2 -gt 5 ], error
//   test $status = "ok"      // If status="not ok", word splits
//
// Good:
//   [ -z "$var" ]            // Properly quoted
//   [ "$count" -gt 5 ]       // Safe from word splitting
//   test "$status" = "ok"    // Correct
//   [[ -z $var ]]            // [[ ]] doesn't word split (bash/ksh)
//
// Note: Always quote variables in [ ] tests. Or use [[ ]] which is safer.
//
// ## Lexer-context rewrite (PMAT-248 phase 3, GH-241)
//
// The original implementation decided "is this `$var` quoted?" with a
// line-local heuristic (`is_variable_quoted`: does a `"` immediately precede
// and follow the match?). That heuristic cannot see that a command
// substitution opens a *fresh* quoting context (POSIX 2.6.3, GH-228), so on
//
//   if [ "$(echo "$coverage >= 80" | bc -l)" -eq 1 ]; then echo ok; fi
//
// it reported `$coverage`, even though `$coverage` sits inside the `"…"`
// that immediately wraps it — quoted twice over.
//
// This rewrite delegates all quoting decisions to
// [`crate::linter::shell_words`], which already resolves GH-228 once, for
// every rule that needs it. `shell_words::simple_commands` recurses into
// `$( … )` (and `` ` … ` ``) as *independent* command contexts, so the
// resulting flat list of [`SimpleCommand`]s already contains, at whatever
// depth they occur:
//
// * a `[ … ]` / `test …` command nested inside a substitution — it matches
//   [`is_test_command`] directly, exactly like a top-level one; and
// * a substitution nested inside a `[ … ]` operand — its expansions are
//   found by the byte-column containment check in [`offences`], because a
//   nested command's byte range can only fall inside the enclosing test's
//   span if it is physically written between the test's own first and last
//   word.
//
// Neither case needs special-casing or a second pass over the raw text.

use crate::linter::shell_words::{self, Expansion, SimpleCommand};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// The byte-column span (1-indexed, end-exclusive) covered by a `[ … ]` /
/// `test …` command's own words on its physical line.
#[derive(Clone, Copy)]
struct TestSpan {
    start: usize,
    end: usize,
}

/// `true` for the command whose name shell_words resolved to `[` or `test`.
///
/// `[[ … ]]` never matches: `[[` is a reserved word (see
/// `shell_words::RESERVED_WORDS`), so shell_words never assigns it as a
/// command name — the first operand word becomes the (usually unresolvable)
/// command name instead, which cannot equal `"["` or `"test"`.
fn is_test_command(cmd: &SimpleCommand) -> bool {
    matches!(cmd.name.as_deref(), Some("[") | Some("test"))
}

/// The column span from a test command's first word to the end of its last.
fn command_span(cmd: &SimpleCommand) -> Option<TestSpan> {
    let start = cmd.words.first()?.col;
    let last = cmd.words.last()?;
    Some(TestSpan {
        start,
        end: last.col + last.raw.len(),
    })
}

/// True when byte column `col` falls inside at least one `[ … ]` / `test …` span.
fn inside_any(col: usize, spans: &[TestSpan]) -> bool {
    spans.iter().any(|s| col >= s.start && col < s.end)
}

/// Every unquoted `$name` / `${name}` expansion, anywhere in `line`, whose
/// column falls inside at least one `[ … ]` / `test …` span — deduplicated
/// and returned in ascending column order.
fn offences(line: &str) -> Vec<Expansion> {
    let cmds = shell_words::simple_commands(line);

    let spans: Vec<TestSpan> = cmds
        .iter()
        .filter(|c| is_test_command(c))
        .filter_map(command_span)
        .collect();

    if spans.is_empty() {
        return Vec::new();
    }

    let mut found: std::collections::BTreeMap<usize, Expansion> = std::collections::BTreeMap::new();
    let candidates = cmds
        .iter()
        .flat_map(|cmd| cmd.words.iter())
        .flat_map(|word| word.expansions.iter())
        .filter(|exp| !exp.quoted && inside_any(exp.col, &spans));
    for exp in candidates {
        found.entry(exp.col).or_insert_with(|| exp.clone());
    }

    found.into_values().collect()
}

/// Create diagnostic for an unquoted variable in a test condition.
///
/// Columns come straight from the [`Expansion`], so they are byte-accurate
/// and cover exactly the `$name` / `${name}` text — the same span the
/// autofix splicer needs to double-quote it in place.
fn create_word_split_diagnostic(e: &Expansion, line_num: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2047",
        Severity::Warning,
        format!(
            "Quote {} to prevent word splitting, or use [[..]] instead of [..]",
            e.text
        ),
        Span::new(line_num, e.col, line_num, e.end_col),
    )
    .with_fix(Fix::new(format!("\"{}\"", e.text)))
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;
        for e in offences(line) {
            result.add(create_word_split_diagnostic(&e, line_num));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2047_unquoted_var_in_test() {
        let code = r#"[ -z $var ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2047");
        assert!(result.diagnostics[0].message.contains("Quote"));
    }

    #[test]
    fn test_sc2047_unquoted_var_with_gt() {
        let code = r#"[ $count -gt 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2047_test_command() {
        let code = r#"test $status = "ok""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2047_quoted_var_ok() {
        let code = r#"[ -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_double_bracket_ok() {
        let code = r#"[[ -z $var ]]"#;
        let result = check(code);
        // [[ ]] doesn't word split, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_quoted_count_ok() {
        let code = r#"[ "$count" -gt 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_test_quoted_ok() {
        let code = r#"test "$status" = "ok""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_comment_ok() {
        let code = r#"# [ -z $var ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2047_multiple_unquoted() {
        let code = r#"[ $a = $b ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2047_braced_var() {
        let code = r#"[ -n ${var} ]"#;
        let result = check(code);
        // Braced but not quoted
        assert_eq!(result.diagnostics.len(), 1);
    }

    // -----------------------------------------------------------------
    // PMAT-248 phase 3 (GH-241): lexer-context rewrite regression tests.
    // -----------------------------------------------------------------

    /// GH-241 reproducer: `$coverage` sits inside the `"…"` that directly
    /// wraps it, inside a command substitution that is itself
    /// double-quoted. Quoted twice over — must NOT be reported. The old
    /// quote-counting heuristic saw an even number of `"` before it and
    /// called it unquoted; this was the false positive PMAT-248 exists to
    /// fix (updates the pre-rewrite behaviour this test module encoded).
    #[test]
    fn test_PMAT248_gh241_quoted_var_inside_nested_substitution_is_quoted() {
        let code = r#"if [ "$(echo "$coverage >= 80" | bc -l)" -eq 1 ]; then echo ok; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }

    /// A true positive must still fire, with a byte-accurate span over
    /// exactly `$x`.
    #[test]
    fn test_PMAT248_unquoted_var_span_is_exact() {
        let code = r#"[ $x -eq 1 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        let d = &result.diagnostics[0];
        assert_eq!(d.span.start_line, 1);
        assert_eq!(d.span.start_col, 3); // "[ " -> $ at byte col 3
        assert_eq!(d.span.end_col, 5); // one past "$x"
    }

    /// `"$x"` is quoted at its own nesting level: no report.
    #[test]
    fn test_PMAT248_quoted_var_in_test_ok() {
        let code = r#"[ "$x" -eq 1 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    /// A substitution nested inside a `[ … ]` operand: `$y` is unquoted in
    /// the substitution's own fresh quoting context, so it is still a real
    /// SC2047 offence even though it is not a direct operand of `[`.
    #[test]
    fn test_PMAT248_unquoted_var_inside_substitution_in_test() {
        let code = r#"[ "$(cmd $y)" = a ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert!(result.diagnostics[0].message.contains("$y"));
    }

    /// `[[ … ]]` is never scanned, even with an otherwise-reportable shape.
    #[test]
    fn test_PMAT248_double_bracket_var_ok() {
        let code = r#"[[ $x -eq 1 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    /// A `[ … ]` test nested inside `$( … )` resolves naturally: shell_words
    /// recurses into the substitution as an independent command, so the
    /// inner `[` matches `is_test_command` directly.
    #[test]
    fn test_PMAT248_test_inside_substitution_in_longer_line() {
        let code = r#"echo start; result=$(if [ $a -eq 1 ]; then echo yes; fi); echo "$result""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert!(result.diagnostics[0].message.contains("$a"));
    }
}
