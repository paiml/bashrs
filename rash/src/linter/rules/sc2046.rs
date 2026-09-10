//! SC2046: Quote command substitutions to prevent word splitting
//!
//! Detects unquoted command substitutions like $(cmd) or `cmd` that could
//! cause word splitting on the output.
//!
//! ## Lexer-context history (PMAT-248, GH-237, GH-262)
//!
//! The original implementation matched `$(...)` / `` `...` `` with regexes
//! over raw text. That produced two classes of false positive:
//!
//! - **Unbalanced spans** (GH-237): `[^)]+` stops at the *first* `)`, so
//!   `$((n % i))` (arithmetic expansion, one word, never split) was reported
//!   with the mangled span `$((n % i)`.
//! - **Wrong contexts** (GH-262): the regex fired on an assignment RHS
//!   (`x=$(date)`), inside double quotes (`echo "$(date)"`), and on the word
//!   of a `case $(uname) in`, none of which the shell ever word-splits.
//!
//! This rewrite is built on [`crate::linter::shell_words`]'s word/role
//! analysis: [`shell_words::simple_commands`] already resolves each word's
//! [`WordRole`] (so assignment prefixes and `case` operands are identified
//! for free) and already recurses into every `$( … )` / `` ` … ` `` body as
//! an independent command (so nested substitutions are visited on their own,
//! with correct absolute columns, without this module recursing itself).
//!
//! What `shell_words` does *not* expose is the position of a command
//! substitution *marker* itself (`Expansion` only tracks `$NAME` / `${NAME}`
//! variable expansions and their `quoted` flag) - a `$( … )` never becomes an
//! `Expansion`, only an internal, unexported byte range used purely for that
//! recursion. So this module does its own small, local, quote-aware scan of
//! each reportable word's `raw` text (which still has its original quote
//! characters) to find `$( … )` / `` ` … ` `` markers and to decide, at each
//! marker's own nesting level, whether it sits inside `'…'` / `"…"`. The
//! paren/backtick matching mirrors `shell_words`'s private `find_close` /
//! `read_backtick` byte-for-byte, so spans stay byte-accurate.
//!
//! ## Phase 7 review finding (PMAT-248): `CommandName` is reportable too
//!
//! Phase 2 gated on `[Argument, RedirectTarget]` only, which under-reported:
//! a *bare* `$(get_command)` (or `$(get_command) arg`) sits in `CommandName`
//! position (`shell_words::CmdState::role_for`'s fallthrough - the word
//! resolves to no literal name, exactly like the `case` operand), and the
//! shell still word-splits that substitution's output before exec'ing it
//! (`get_command` returning `ls -la` runs `ls` with arg `-la`). 7.0.2 and
//! shellcheck both report this. Measured against `shell_words`:
//!
//! - `if $(cmd); then :; fi` - `$(cmd)` is `CommandName` (same fallthrough as
//!   plain command position) - reportable, and shellcheck agrees.
//! - `eval $(get_command)` - `eval` resolves through `WRAPPER_COMMANDS` to
//!   `Reserved`, so `$(get_command)` is `CommandName` - reportable, and
//!   shellcheck agrees.
//! - `case $(uname) in` - `$(uname)` is *also* `CommandName` (it immediately
//!   follows the `Reserved` word `case`), but the shell never runs or
//!   word-splits a `case` operand, so this one case must stay excluded: skip
//!   a `CommandName` word whose immediately preceding word in the same
//!   `SimpleCommand` is the reserved word `case`.
//! - `for i in $(ls)` is unaffected - that word is `Argument` already.
//!
//! References:
//! - <https://www.shellcheck.net/wiki/SC2046>

use crate::linter::shell_words::{self, ShellWord, WordRole};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Roles in which an unquoted command substitution is reportable: the shell
/// word-splits an unquoted expansion in argument, redirect-target, or
/// command-name position (see the module-level "Phase 7 review finding" doc
/// for the `CommandName` measurements). It never splits an `AssignPrefix`
/// (`x=$(date)` runs `date` and assigns its output verbatim - no splitting
/// occurs) or a `Reserved` word. The one `CommandName` exception - the
/// operand of a `case … in` - is excluded separately by [`is_case_operand`],
/// since it depends on the *previous* word, not the word's own role.
const REPORTABLE_ROLES: &[WordRole] = &[
    WordRole::Argument,
    WordRole::RedirectTarget,
    WordRole::CommandName,
];

/// True when `words[idx]` is the operand of a `case … in`: a `CommandName`
/// word immediately preceded, in the same `SimpleCommand`, by the reserved
/// word `case`. `shell_words` gives that operand `CommandName` because it
/// never resolves to a literal command name (`shell_words::CmdState::role_for`),
/// but the shell never runs or word-splits it, so it must stay unreported
/// even though `CommandName` is otherwise reportable.
fn is_case_operand(words: &[ShellWord], idx: usize) -> bool {
    idx > 0 && words[idx - 1].role == WordRole::Reserved && words[idx - 1].literal == "case"
}

/// One reportable command substitution found inside a word's raw text.
struct Offence {
    /// 1-indexed byte column of the opening `$`/backtick, in the physical line.
    col: usize,
    /// 1-indexed byte column one past the closing `)`/backtick.
    end_col: usize,
    /// The substitution exactly as written, delimiters included:
    /// `$(cmd)` or `` `cmd` ``.
    text: String,
    /// True for `` `cmd` ``, false for `$(cmd)`.
    backtick: bool,
}

/// Local quoting state used only to decide whether a `$(`/backtick marker
/// sits in an unquoted position. Mirrors `shell_words::WordLexer`'s `Quote`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Quote {
    Bare,
    Single,
    Double,
}

/// Scan a word's raw source text (quotes included) for command-substitution
/// markers and append every one found in an *unquoted* position, at its own
/// nesting level, to `out`. Does not recurse into a substitution's body:
/// `shell_words::simple_commands` already recurses into every `$( … )` /
/// `` ` … ` `` body as its own command, so a nested substitution is visited
/// again, separately, as a word of that recursed command - recursing here
/// too would double-report it.
fn scan_word(raw: &str, word_col: usize, out: &mut Vec<Offence>) {
    let bytes = raw.as_bytes();
    let mut quote = Quote::Bare;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Quote::Single => {
                i += 1;
                if b == b'\'' {
                    quote = Quote::Bare;
                }
            }
            Quote::Double => match b {
                b'"' => {
                    quote = Quote::Bare;
                    i += 1;
                }
                // Inside `"…"` a backslash only escapes `$`, backtick, `"` and
                // `\` - matches `shell_words::WordLexer::escape_double`.
                b'\\' => {
                    i += match bytes.get(i + 1) {
                        Some(b'$' | b'`' | b'"' | b'\\') => 2,
                        _ => 1,
                    };
                }
                b'$' => i = handle_dollar(bytes, i, word_col, false, out),
                b'`' => i = handle_backtick(bytes, i, word_col, false, out),
                _ => i += 1,
            },
            Quote::Bare => match b {
                b'\'' => {
                    quote = Quote::Single;
                    i += 1;
                }
                b'"' => {
                    quote = Quote::Double;
                    i += 1;
                }
                b'\\' => i += 2,
                b'$' => i = handle_dollar(bytes, i, word_col, true, out),
                b'`' => i = handle_backtick(bytes, i, word_col, true, out),
                _ => i += 1,
            },
        }
    }
}

/// Handle a `$` found at `i`. Returns the index to resume scanning from.
///
/// `$((...))` is arithmetic expansion (one word, never split - GH-237): its
/// span is skipped without reporting. `$(...)` is a command substitution:
/// reported when `reportable`, using the full balanced span (fixing GH-237's
/// unbalanced-regex span) regardless of reportability, because the caller
/// must always skip past it correctly to keep scanning the rest of the word.
fn handle_dollar(
    bytes: &[u8],
    i: usize,
    word_col: usize,
    reportable: bool,
    out: &mut Vec<Offence>,
) -> usize {
    if bytes.get(i + 1) != Some(&b'(') {
        return i + 1;
    }
    let is_arith = bytes.get(i + 2) == Some(&b'(');
    let close = find_paren_close(bytes, i + 1);
    let end_excl = if close < bytes.len() {
        close + 1
    } else {
        bytes.len()
    };
    if !is_arith && reportable {
        let text = String::from_utf8_lossy(&bytes[i..end_excl]).into_owned();
        out.push(Offence {
            col: word_col + i,
            end_col: word_col + end_excl,
            text,
            backtick: false,
        });
    }
    end_excl
}

/// Handle a backtick found at `i`. Returns the index to resume scanning from.
fn handle_backtick(
    bytes: &[u8],
    i: usize,
    word_col: usize,
    reportable: bool,
    out: &mut Vec<Offence>,
) -> usize {
    let mut j = i + 1;
    while let Some(&b) = bytes.get(j) {
        if b == b'\\' {
            j += 2;
            continue;
        }
        if b == b'`' {
            break;
        }
        j += 1;
    }
    let end_excl = if j < bytes.len() { j + 1 } else { bytes.len() };
    if reportable {
        let text = String::from_utf8_lossy(&bytes[i..end_excl]).into_owned();
        out.push(Offence {
            col: word_col + i,
            end_col: word_col + end_excl,
            text,
            backtick: true,
        });
    }
    end_excl
}

/// The 0-indexed byte position of the `)` matching the `(` at `open_idx`,
/// honouring nested quotes, backslash escapes, and nested parens, so nesting
/// such as `$(echo $(date))` balances correctly. Falls back to `bytes.len()`
/// on unterminated input, exactly like `shell_words`'s private `find_close`,
/// so malformed input never panics.
fn find_paren_close(bytes: &[u8], open_idx: usize) -> usize {
    let mut depth = 1usize;
    let mut i = open_idx + 1;
    while let Some(&b) = bytes.get(i) {
        match b {
            b'\'' | b'"' => {
                i = skip_quoted(bytes, i, b);
                continue;
            }
            b'\\' => {
                i += 2;
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// The byte index one past the closing `quote`, starting the scan at `start`
/// (the index of the opening quote byte). `"` honours `\` escapes; `'` does
/// not (POSIX). Falls back to `bytes.len()` when unterminated.
fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while let Some(&b) = bytes.get(i) {
        if quote == b'"' && b == b'\\' {
            i += 2;
            continue;
        }
        if b == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

/// Build the SC2046 diagnostic for one offence.
///
/// `$(...)` gets the balanced full-text message and a fix that wraps the
/// expansion, verbatim, in double quotes. `` `...` `` keeps its pre-existing
/// message and fix (convert to `$(...)` form, then quote) - only the FP
/// *context* gating changed for backticks, not this true-positive shape.
fn make_diagnostic(o: &Offence, line: usize) -> Diagnostic {
    let span = Span::new(line, o.col, line, o.end_col);
    if o.backtick {
        let inner = o
            .text
            .strip_prefix('`')
            .and_then(|s| s.strip_suffix('`'))
            .unwrap_or(&o.text);
        let fix = Fix::new(format!("\"$({inner})\""));
        Diagnostic::new(
            "SC2046",
            Severity::Warning,
            "Quote this and use $(...) instead of backticks".to_string(),
            span,
        )
        .with_fix(fix)
    } else {
        let fix = Fix::new(format!("\"{}\"", o.text));
        Diagnostic::new(
            "SC2046",
            Severity::Warning,
            format!("Quote this to prevent word splitting: {}", o.text),
            span,
        )
        .with_fix(fix)
    }
}

/// True when `words[idx]` should be scanned for unquoted command
/// substitutions: its role is reportable, unless it is the `case … in`
/// exception carved out of `CommandName` by [`is_case_operand`].
fn is_reportable(words: &[ShellWord], idx: usize) -> bool {
    let word = &words[idx];
    REPORTABLE_ROLES.contains(&word.role)
        && !(word.role == WordRole::CommandName && is_case_operand(words, idx))
}

/// Collect every reportable command-substitution offence on one physical
/// line, in ascending column order.
fn scan_line(line: &str) -> Vec<Offence> {
    let mut offences: Vec<Offence> = Vec::new();
    for cmd in shell_words::simple_commands(line) {
        for (idx, word) in cmd.words.iter().enumerate() {
            if is_reportable(&cmd.words, idx) {
                scan_word(&word.raw, word.col, &mut offences);
            }
        }
    }
    offences.sort_by_key(|o| o.col);
    offences
}

/// Check for unquoted command substitutions (SC2046).
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        for o in &scan_line(line) {
            result.add(make_diagnostic(o, line_num));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2046_basic_detection() {
        let bash_code = "files=$(find . -name '*.txt')";
        let result = check(bash_code);

        // PMAT-248/GH-262 update: `files=$(...)` is an assignment RHS - the
        // shell never word-splits it, so this is no longer reported. This
        // test used to assert the old (wrong) behaviour; it now asserts the
        // fixed one via a true positive with the same substitution in an
        // argument position instead.
        assert_eq!(result.diagnostics.len(), 0);

        let bash_code = "echo $(find . -name '*.txt')";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
        assert!(result.diagnostics[0].message.contains("Quote this"));
    }

    #[test]
    fn test_sc2046_autofix() {
        let bash_code = "echo $(ls)";
        let result = check(bash_code);

        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"$(ls)\""
        );
    }

    #[test]
    fn test_sc2046_backtick_detection() {
        let bash_code = "echo `ls *.txt`";
        let result = check(bash_code);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
        assert!(result.diagnostics[0].message.contains("backticks"));
    }

    #[test]
    fn test_sc2046_backtick_autofix() {
        let bash_code = "echo `ls`";
        let result = check(bash_code);

        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"$(ls)\""
        );
    }

    #[test]
    fn test_sc2046_skip_quoted() {
        let bash_code = r#"files="$(find . -name '*.txt')""#;
        let result = check(bash_code);

        // Should NOT trigger - already quoted
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2046_multiple_substitutions() {
        let bash_code = "echo $(echo $(cat file.txt))";
        let result = check(bash_code);

        // Should detect nested unquoted substitutions: the outer $(...) and
        // the inner $(...), reported separately (PMAT-248: shell_words
        // recurses into the substitution body as its own command).
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2046_severity() {
        let bash_code = "echo $(ls)";
        let result = check(bash_code);

        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    // -----------------------------------------------------------------
    // PMAT-248 / GH-237: $((...)) is arithmetic expansion, never reported,
    // and never mangled into an unbalanced span.
    // -----------------------------------------------------------------

    #[test]
    fn test_PMAT248_gh237_arithmetic_not_reported() {
        let result = check("echo $((x+1))");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT248_gh237_arithmetic_in_test_not_reported() {
        let result = check("if [ $((n % i)) -eq 0 ]; then\n  echo yes\nfi");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // -----------------------------------------------------------------
    // PMAT-248 / GH-262: SC2046 fires only where the shell would split.
    // -----------------------------------------------------------------

    #[test]
    fn test_PMAT248_gh262_assignment_rhs_not_reported() {
        let result = check("x=$(date)");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT248_gh262_double_quoted_not_reported() {
        let result = check(r#"echo "$(date)""#);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT248_gh262_case_word_not_reported() {
        let result = check("case $(uname) in\n  Linux) echo l ;;\n  *) echo o ;;\nesac");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT248_gh262_nested_command_substitution_reports_both() {
        let result = check("echo $(echo $(date))");
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_PMAT248_gh262_backtick_still_reported() {
        let result = check("echo `date`");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
    }

    #[test]
    fn test_PMAT248_gh262_unquoted_argument_span_is_exact() {
        let src = "echo $(date)";
        let result = check(src);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("$(date)"));

        let span = result.diagnostics[0].span;
        // `$(date)` starts right after "echo " (1-indexed byte column 6) and
        // is 7 bytes long, so the span covers exactly `$(date)`.
        assert_eq!(&src[span.start_col - 1..span.end_col - 1], "$(date)");
    }

    // -----------------------------------------------------------------
    // PMAT-248 Phase 7 review finding: a bare `$(get_command)` sits in
    // `CommandName` position and the shell still word-splits it before
    // exec'ing, so it must be reported too (7.0.2 and shellcheck agree).
    // -----------------------------------------------------------------

    #[test]
    fn test_PMAT248_review_sc2046_command_position_is_still_reported() {
        let src = "$(get_command)";
        let result = check(src);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");

        // Span covers exactly the substitution: the whole line here.
        let span = result.diagnostics[0].span;
        assert_eq!(&src[span.start_col - 1..span.end_col - 1], "$(get_command)");
    }

    #[test]
    fn test_PMAT248_review_sc2046_command_position_with_arg_reports_once() {
        let result = check("$(get_command) arg");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
    }

    #[test]
    fn test_PMAT248_review_sc2046_case_operand_still_not_reported() {
        // The `CommandName` exception must survive: the operand of a `case
        // … in` never resolves to a literal name (same shell_words
        // fallthrough as a bare command substitution) but is never run or
        // word-split, so it stays unreported even now that `CommandName` is
        // otherwise reportable.
        let result = check("case $(uname) in");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT248_review_sc2046_if_condition_command_is_reported() {
        // Measured: `$(cmd)` in `if $(cmd); then` is `CommandName` (the same
        // fallthrough as plain command position, not a `case`-style
        // exclusion), and the shell splits its output before exec'ing it.
        // shellcheck reports this too.
        let result = check("if $(cmd); then :; fi");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
    }

    #[test]
    fn test_PMAT248_review_sc2046_eval_command_is_reported() {
        // Measured: `eval` resolves through `WRAPPER_COMMANDS` to `Reserved`,
        // so `$(get_command)` is `CommandName`, not the `case` exception.
        // shellcheck reports this too.
        let result = check("eval $(get_command)");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
    }

    #[test]
    fn test_PMAT248_review_sc2046_for_in_argument_still_reported() {
        // Unaffected by this change: `$(ls)` here is `Argument`, not
        // `CommandName`, already reportable before Phase 7.
        let result = check("for i in $(ls); do echo $i; done");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2046");
    }
}
