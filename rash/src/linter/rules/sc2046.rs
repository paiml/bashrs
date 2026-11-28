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
//! PMAT-250 update: `shell_words::Expansion` now covers command-substitution
//! markers too (`$( … )` / `` ` … ` ``, including nested and inside double
//! quotes), exposed on `ShellWord::substitutions` with the same `quoted` flag
//! variable expansions already carried. This module used to carry its own
//! small, local, quote-aware scanner over each reportable word's `raw` text
//! (mirroring `shell_words`'s private `find_close` / `read_backtick`
//! byte-for-byte) to find those markers itself; that scanner is gone and this
//! module now simply reads `word.substitutions`. This is a pure refactor: no
//! verdict changes, since `shell_words` computed the identical positions and
//! `quoted` status the private scanner used to compute for itself.
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

use crate::linter::shell_words::{self, ExpansionKind, ShellWord, WordRole};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

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

/// One reportable command substitution found on a reportable word.
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

/// True for `$(seq …)` / `` `seq …` ``: shellcheck never reports SC2046 on a
/// `seq` substitution (its output is numeric words, safe to split; GH-236).
fn is_seq_substitution(text: &str) -> bool {
    let inner = text
        .strip_prefix("$(")
        .and_then(|s| s.strip_suffix(')'))
        .or_else(|| text.strip_prefix('`').and_then(|s| s.strip_suffix('`')))
        .unwrap_or("");
    inner
        .trim_start()
        .strip_prefix("seq")
        .is_some_and(|rest| rest.starts_with(char::is_whitespace))
}

/// Command-substitution markers on a reportable word, filtered to the
/// unquoted ones. `shell_words` (PMAT-250) already excludes `$((...))`
/// arithmetic expansion from ever producing a marker, and already resets
/// quoting at each `$( … )` nesting level (POSIX 2.6.3), so no local
/// re-scan is needed here any more - not recursing into a substitution's
/// body is likewise inherited for free: `shell_words::simple_commands`
/// recurses into every `$( … )` / `` ` … ` `` body as its own command, so a
/// nested substitution is visited again, separately, as a word of that
/// recursed command.
fn word_offences(word: &ShellWord, out: &mut Vec<Offence>) {
    for sub in &word.substitutions {
        if sub.quoted {
            continue;
        }
        if is_seq_substitution(&sub.text) {
            continue;
        }
        // bashrs#679: `$(cmd)` in the word of an unquoted `${x:-…}` is split together with
        // the whole expansion. ShellCheck 0.8.0 reports that expansion as SC2086, not SC2046.
        if word
            .expansions
            .iter()
            .any(|e| e.braced && e.col < sub.col && sub.end_col <= e.end_col)
        {
            continue;
        }
        let backtick = matches!(
            sub.kind,
            ExpansionKind::CommandSubstitution { backtick: true }
        );
        out.push(Offence {
            col: sub.col,
            end_col: sub.end_col,
            text: sub.text.clone(),
            backtick,
        });
    }
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

/// Builtins whose `NAME=value` arguments are assignments, not split words.
const DECLARATION_BUILTINS: &[&str] = &["export", "readonly", "declare", "local", "typeset"];

/// True for a `NAME=…` / `NAME+=…` word (identifier before the `=`).
fn is_assignment_arg(raw: &str) -> bool {
    let Some(eq) = raw.find('=') else {
        return false;
    };
    let name = raw[..eq].strip_suffix('+').unwrap_or(&raw[..eq]);
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
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
/// Blank out every `(( … ))` arithmetic command on a line, keeping byte
/// columns intact. Its operands are never word-split, so a `$(…)` inside it
/// is not SC2046 (shellcheck: silent).
fn arith_command_end(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (j, &c) in chars.iter().enumerate().skip(start) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

fn mask_arith_commands(line: &str) -> String {
    let mut chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i + 1 < chars.len() {
        let is_open = chars[i] == '(' && chars[i + 1] == '(';
        let is_expansion = i > 0 && (chars[i - 1] == '$' || chars[i - 1] == '(');
        match arith_command_end(&chars, i) {
            Some(e) if is_open && !is_expansion => {
                chars[i..=e].iter_mut().for_each(|c| *c = ' ');
                i = e + 1;
            }
            _ => i += 1,
        }
    }
    chars.into_iter().collect()
}

fn scan_line(line: &str) -> Vec<Offence> {
    let mut offences: Vec<Offence> = Vec::new();
    let masked = mask_arith_commands(line);
    for cmd in shell_words::simple_commands(&masked) {
        let head = cmd.words.first().map(|w| w.literal.as_str()).unwrap_or("");
        // `[[ … ]]` never word-splits its operands.
        if head == "[[" {
            continue;
        }
        let decl = DECLARATION_BUILTINS.contains(&head);
        for (idx, word) in cmd.words.iter().enumerate() {
            // `export A=$(cmd)`: declaration builtins take assignment
            // arguments that are not word-split (shellcheck: no SC2046).
            if decl && idx > 0 && is_assignment_arg(&word.raw) {
                continue;
            }
            if is_reportable(&cmd.words, idx) {
                word_offences(word, &mut offences);
            }
        }
    }
    offences.retain(|o| !is_bare_array_value(line, o));
    offences.sort_by_key(|o| o.col);
    offences
}

/// `NAME=(` at the end of the text before an offence.
static ARRAY_OPEN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\w+\s*=\s*\(\s*$").unwrap());

/// bashrs#677: an offence that is the whole value of an array assignment,
/// `NAME=( $(cmd) )`. The `(` ends the simple command in `shell_words`, so the
/// substitution reads as a command name. ShellCheck 0.8.0 reports only SC2207
/// for this shape, so it is not SC2046. Any other element next to it still is.
fn is_bare_array_value(line: &str, o: &Offence) -> bool {
    let before = line.get(..o.col.saturating_sub(1)).unwrap_or("");
    let after = line.get(o.end_col.saturating_sub(1)..).unwrap_or("");
    ARRAY_OPEN.is_match(before) && after.trim_start().starts_with(')')
}

/// Check for unquoted command substitutions (SC2046).
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    // GH-236: a heredoc body (quoted or not) is never word-split; an
    // unquoted body expands `$(cmd)` but keeps the result as text.
    let heredoc = crate::linter::quoting::heredoc_body_lines(source);

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        if heredoc.contains(&line_num) {
            continue;
        }

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
    fn test_sc2046_gh236_declaration_and_dbl_bracket_not_reported() {
        for src in [
            "export A=$(date)",
            "local d=$(date)",
            "readonly B+=$(date)",
            "declare C=$(date)",
            "[[ $(uname) == Linux ]]",
            "[[ -n $(ls) ]] && :",
        ] {
            assert_eq!(check(src).diagnostics.len(), 0, "{src}");
        }
        assert_eq!(check("export $(date)").diagnostics.len(), 1);
        assert_eq!(check("echo A=$(date)").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2046_gh236_heredoc_body_not_reported() {
        for src in [
            "cat <<EOT\ntoday is $(date)\nEOT\n",
            "cat <<-EOT\n\t$(date)\n\tEOT\n",
            "cat <<'EOT'\n$(date)\nEOT\n",
        ] {
            assert_eq!(check(src).diagnostics.len(), 0, "{src}");
        }
        // The opener line and code after the body are still checked.
        let src = "cat <<EOT $(date)\nx\nEOT\necho $(date)\n";
        let d = check(src).diagnostics;
        assert_eq!(d.len(), 2);
        assert_eq!(d[1].span.start_line, 4);
    }

    #[test]
    fn test_sc2046_gh236_seq_not_reported() {
        for src in [
            "for i in $(seq 1 10); do echo \"$i\"; done",
            "echo $(seq 1 3)",
            "for i in `seq 1 $n`; do :; done",
        ] {
            assert_eq!(check(src).diagnostics.len(), 0, "{src}");
        }
        assert_eq!(
            check("for i in $(sequence 1); do :; done")
                .diagnostics
                .len(),
            1
        );
        assert_eq!(check("for i in $(ls); do :; done").diagnostics.len(), 1);
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
    fn test_sc236_arith_command_cmdsub_not_reported() {
        let result = check(
            r#"if (( $(echo "$X >= 85" | bc -l 2>/dev/null || echo "1") )); then
  echo ok
fi"#,
        );
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
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

/// PMAT-250 (contracts/linter-lexer-context-v1.yaml F-SW-PMAT250-*): SC2046
/// now consumes `shell_words::ShellWord::substitutions` instead of its own
/// private scanner. Every case below is a case the deleted scanner used to
/// decide on its own; the verdict here must be byte-for-byte identical to
/// before the refactor.
#[cfg(test)]
mod tests_pmat250 {
    use super::*;

    #[test]
    fn test_PMAT255_pmat250_no_verdict_change() {
        // GH-237: arithmetic expansion is never a command substitution.
        assert_eq!(check("echo $((x+1))").diagnostics.len(), 0);

        // GH-252: an escaped backtick inside "..." is text, not a marker.
        assert_eq!(check(r#"echo "a \` b" $(date)"#).diagnostics.len(), 1);

        // GH-262: SC2046 only where the shell field-splits.
        assert_eq!(check("x=$(date)").diagnostics.len(), 0);
        assert_eq!(check(r#"echo "$(date)""#).diagnostics.len(), 0);
        assert_eq!(check("case $(uname) in").diagnostics.len(), 0);
        assert_eq!(check("echo $(date)").diagnostics.len(), 1);
        assert_eq!(check("$(get_command)").diagnostics.len(), 1);

        // Nesting and backticks still balance and still report every level.
        assert_eq!(check("echo $(echo $(date))").diagnostics.len(), 2);
        assert_eq!(check("echo `date`").diagnostics.len(), 1);
    }

    #[test]
    fn test_PMAT255_pmat250_uses_shell_words_substitutions_field() {
        // Confirms the new field is actually what feeds SC2046's offences,
        // not a coincidence of some other path.
        let cmds = shell_words::simple_commands("echo $(date)");
        let word = cmds[0]
            .words
            .iter()
            .find(|w| !w.substitutions.is_empty())
            .expect("shell_words must record the $(date) marker");
        assert_eq!(word.substitutions[0].text, "$(date)");
        assert_eq!(
            check("echo $(date)").diagnostics[0].span.start_col,
            word.substitutions[0].col
        );
    }
}

#[cfg(test)]
mod default_cmdsub_tests_679 {
    use super::check;

    // bashrs#679: for `${x:-$(date)}` shellcheck 0.8.0 reports one SC2086 on the
    // expansion and no SC2046 on the inner `$(date)`.
    #[test]
    fn default_value_cmdsub_is_not_sc2046() {
        let r = check("x=\"\"; echo ${x:-$(date)}");
        assert!(
            r.diagnostics.iter().all(|d| d.code != "SC2046"),
            "SC2046 fired on $(date) inside ${{x:-...}}: {:?}",
            r.diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn plain_cmdsub_still_sc2046() {
        let r = check("echo $(date)");
        assert_eq!(
            r.diagnostics.iter().filter(|d| d.code == "SC2046").count(),
            1
        );
    }
}
