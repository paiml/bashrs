//! SC1035: Missing space after certain keywords
//!
//! Detects missing spaces after shell keywords like `then`, `do`, `else`,
//! `elif`, `fi`, `done`, `while`, `until`, `for`, `case`, `esac` when they
//! are immediately followed by a non-space character.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if true; then(echo hi)
//! for i in 1 2; do{echo $i;}
//! ```
//!
//! Good:
//! ```bash
//! if true; then echo hi
//! for i in 1 2; do echo $i; done
//! ```
//!
//! # A keyword is only a keyword when it is a whole token (GH-272)
//!
//! The check used to be "the keyword appears, the character after it is not a
//! blank, a `;` or alphanumeric". That is not shell tokenisation. Shell splits
//! words on blanks and on the operators `; | & ( ) < >`; every other character
//! — `-`, `.`, `/`, `=`, `:`, `,` … — CONTINUES a word. So
//! `git for-each-ref` is one command name and `--do=1` is one option, and both
//! were reported as `Severity::Error`. Four of those (all `for-each-ref`) were
//! left in the rmedia corpus after the quoting work, and `shellcheck` calls
//! every one of them clean.
//!
//! Two conditions now have to hold, and both are about tokenisation:
//!
//! - the character BEFORE the keyword must end a token (line start, blank, or
//!   an operator) — not merely "non-alphanumeric", which let `--do=1` through;
//! - the character AFTER must be one bash cannot make a word out of. That is
//!   `(`, `{`, `}`, and `&` where a command was due. `for-each-ref` keeps its
//!   hyphen.
//!
//! `((` is excluded: `for((i=0;i<3;i++))`, `while((n<3))` and `then((n++))`
//! are arithmetic commands that bash accepts with no space, so the "missing
//! space" advice would be wrong.
//!
//! ## Which trailing metacharacters, and why not the rest (GH-268 + GH-272)
//!
//! #276 used the full metacharacter set — `( ) { } | & < >` — for the
//! right-hand side. Three of those have MEASURED valid counterexamples, so
//! they are excluded here; each row below is `bash -n`'s own verdict:
//!
//! | after a keyword | valid example                                | verdict |
//! |-----------------|----------------------------------------------|---------|
//! | `{` `}`         | `do{echo`, `then{`, `fi{`, `done}`           | always broken — REPORT |
//! | `&`             | `then&echo`, `do&echo`, `else&`              | broken after a command-position keyword — REPORT |
//! | `(` (not `((`)  | `then(echo hi)` is accepted by bash          | pre-existing divergence, kept — see below |
//! | `)`             | `x=$(echo done)`, `case $x in do) …`         | valid — NOT reported |
//! | `\|`            | `while …; done\|cat`, `case $x in do\|done) …` | valid — NOT reported |
//! | `<` `>`         | `then>out.txt`, `do>x`, `fi>out.txt`         | valid — a bare redirection IS a command |
//!
//! `then|echo hi` IS a syntax error, and this rule deliberately does not claim
//! it: `case $x in do|done) …` is the same two bytes and is valid, and the
//! line alone cannot tell them apart. ShellCheck does not report SC1035 there
//! either — it reports the parse errors SC1073/SC1072, which is the honest home
//! for it.
//!
//! `(` after a keyword is a divergence in the other direction: `then(echo hi)`
//! and `while(true)` are accepted by bash, so reporting them is a false
//! positive. It predates both #276 and this change and is left alone here
//! rather than folded into a conflict resolution — it is its own ticket.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

const KEYWORDS: &[&str] = &[
    "then", "do", "else", "elif", "fi", "done", "while", "until", "for", "case", "esac", "in",
];

/// Does `b` end the previous shell token?
///
/// Blanks and the operator characters do; a `-`, `=` or `/` does not, which is
/// why `--do=1` is one word and not the `do` keyword.
fn ends_token(b: u8) -> bool {
    b.is_ascii_whitespace() || matches!(b, b';' | b'|' | b'&' | b'(' | b')' | b'<' | b'>')
}

/// Keywords that CLOSE a compound command rather than introduce one.
///
/// The distinction is what makes `&` diagnosable at all. After `fi`, `done` or
/// `esac` the compound command is complete, so `&` backgrounds it —
/// `if true; then :; fi&` and `for i in 1; do :; done&` are both `bash -n`
/// clean. After a command-position keyword there is no command yet, so the same
/// byte is a syntax error: `if true; then&echo hi; fi` gives "syntax error near
/// unexpected token `&'".
const TERMINATOR_KEYWORDS: &[&str] = &["fi", "done", "esac"];

/// Is the keyword at `[start, after)` a complete token in command position?
fn is_token(bytes: &[u8], start: usize, after: usize, kw: &str) -> bool {
    let left_ok = start == 0 || ends_token(bytes[start - 1]);
    let introduces_a_command = !TERMINATOR_KEYWORDS.contains(&kw);
    // A keyword glued to the next word is that word, not a keyword: `esac.txt`,
    // `for-each-ref`, `in/dir`. Only the bytes bash cannot make a word out of
    // are candidates, and of those only the ones with no measured valid
    // counterexample are reported — see the module docs for the table.
    let right_ok = match bytes.get(after) {
        Some(b'{') | Some(b'}') => true,
        // `(` opens a subshell — but `((` is an arithmetic command, which is
        // valid here with no space at all.
        Some(b'(') => bytes.get(after + 1) != Some(&b'('),
        // `&` where a command was due. `fi&` / `done&` background the compound
        // command and are valid, which is why the class matters.
        Some(b'&') => introduces_a_command,
        _ => false,
    };
    left_ok && right_ok
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let bytes = trimmed.as_bytes();

        for kw in KEYWORDS {
            let kw_len = kw.len();
            let mut search_start = 0;

            while let Some(pos) = trimmed[search_start..].find(kw) {
                let abs_pos = search_start + pos;
                let after = abs_pos + kw_len;

                if is_token(bytes, abs_pos, after, kw) {
                    let line_offset = line.find(trimmed).unwrap_or(0);
                    let col = line_offset + abs_pos + 1;
                    result.add(Diagnostic::new(
                        "SC1035",
                        Severity::Error,
                        format!("Missing space after '{}' keyword", kw),
                        Span::new(line_num, col, line_num, col + kw_len),
                    ));
                }

                search_start = abs_pos + kw_len;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1035_then_no_space() {
        let result = check("if true; then(echo hi)");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1035");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1035_do_no_space() {
        let result = check("for i in 1 2; do{echo $i;}");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1035_proper_spacing_ok() {
        let result = check("if true; then echo hi; fi");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_keyword_as_part_of_word_ok() {
        // "done_flag" should not be flagged (done is part of a larger word)
        let result = check("done_flag=1");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_semicolon_after_keyword_ok() {
        let result = check("then;");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_comment_not_flagged() {
        let result = check("# then(echo)");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_else_no_space() {
        let result = check("else(echo fallback)");
        assert_eq!(result.diagnostics.len(), 1);
    }

    // ── GH-272: a keyword is only a keyword when it is a whole TOKEN ───────
    //
    // Shell splits words on blanks and the operators `; | & ( ) < >`. `-`, `.`,
    // `/`, `=`, `:` and friends CONTINUE a word, so `for-each-ref` is one
    // command name, not the `for` keyword. Each of these is `shellcheck`-clean
    // and `bash -n`-clean; every one was an error-severity finding before.

    #[test]
    fn test_sc1035_hyphenated_command_name_is_not_a_keyword() {
        // rmedia/scripts/local-state-snapshot.sh:68,120,139,158 — four of the
        // ten false positives left on that corpus.
        let result = check("heads=$(git for-each-ref refs/heads)");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_sc1035_keyword_followed_by_word_characters_is_not_a_keyword() {
        for line in [
            "echo done-flag",
            "x=case-insensitive",
            "ls in/dir",
            "cat esac.txt",
            "while-true",
            "foo --do=1",
            "run --until=now",
            "helm template --for.each x",
        ] {
            let result = check(line);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "{line:?} -> {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn test_sc1035_arithmetic_for_and_while_are_valid_bash() {
        // `for((i=0;i<3;i++))` and `while((n<3))` are accepted by bash with no
        // space. `((` after a keyword is an arithmetic command, never a typo.
        for line in [
            "for((i=0;i<3;i++)); do echo $i; done",
            "while((n<3)); do n=$((n+1)); done",
            "until((n>3)); do n=$((n+1)); done",
            "if true; then((n++)); fi",
        ] {
            let result = check(line);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "{line:?} -> {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn test_sc1035_keyword_glued_to_the_left_is_not_in_command_position() {
        // The old boundary test accepted any non-alphanumeric char before the
        // keyword, so the `do` in `--do=1` counted as a token start.
        let result = check("cmd --then{x}");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    // ── must still fire ────────────────────────────────────────────────────

    #[test]
    fn must_still_fire_do_glued_to_brace() {
        // `bash -n` rejects this: "syntax error near unexpected token `do{'".
        let result = check("for i in 1 2; do{ echo $i;}; done");
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1035");
        assert!(result.diagnostics[0].message.contains("'do'"));
    }

    #[test]
    fn must_still_fire_then_glued_to_brace() {
        // `bash -n` rejects this too.
        let result = check("if true; then{ echo hi;}; fi");
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert!(result.diagnostics[0].message.contains("'then'"));
    }

    #[test]
    fn must_still_fire_at_line_start() {
        let result = check("else(echo fallback)");
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn must_still_fire_span_points_at_the_keyword() {
        // A finding whose span drifts is a finding nobody can act on.
        let result = check("if true; then(echo hi)");
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        let span = &result.diagnostics[0].span;
        assert_eq!((span.start_line, span.start_col, span.end_col), (1, 10, 14));
    }
}

/// GH-275 (#276), rebased onto GH-272's tokeniser.
///
/// #276 fixed the same defect from the other direction and shipped its own
/// corpus of word-boundary cases. They are kept verbatim where they agree, and
/// where they disagree the disagreement is settled by `bash -n` rather than by
/// whichever side merged last — the verdict is recorded next to each case.
#[cfg(test)]
mod tests_gh275_word_boundaries {
    use super::*;

    #[test]
    fn test_sc1035_git_for_each_ref_not_flagged() {
        // rmedia scripts/local-state-snapshot.sh:68 — 4 findings in that file,
        // all this shape. `bash -n` and `shellcheck` are both clean on it.
        let result =
            check("heads=$(git for-each-ref --format='%(objectname) %(refname:short)' refs/heads)");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    #[test]
    fn test_sc1035_hyphen_continues_the_word() {
        for line in [
            "git for-each-ref refs/heads",
            "do-release-upgrade --help",
            "case-insensitive-sort < in.txt",
            "esac-like-name --flag",
            "in-place-edit file",
        ] {
            let result = check(line);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "{line} -> {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn test_sc1035_dot_and_slash_continue_the_word() {
        for line in [
            "ls /var/log/done.txt",
            "rm -f while.tmp",
            "source ./then.sh",
            "cat /etc/case.conf",
        ] {
            let result = check(line);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "{line} -> {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn test_sc1035_dollar_continues_the_word() {
        // `echo done$count` prints "done5". Nothing is missing here.
        let result = check("echo done$count");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    // ---- MUST STILL FIRE ----

    #[test]
    fn test_sc1035_must_still_fire_metacharacters() {
        // Each line here is one `bash -n` REJECTS, so the keyword really is a
        // keyword and really is missing its space. `then(`/`else(`/`while(` are
        // the pre-existing divergence documented in the module docs: bash
        // accepts them, and they are reported anyway, unchanged by this PR.
        for line in [
            "if true; then(echo hi)",
            "else(echo fallback)",
            "for i in 1 2; do{echo $i;}",
            "if true; then&echo hi",
            "while(true); do break; done",
        ] {
            let result = check(line);
            assert!(
                result.diagnostics.iter().any(|d| d.code == "SC1035"),
                "SC1035 stopped firing on a genuine defect: {line}"
            );
        }
    }

    /// #276 asserted `for((i=0;i<3;i++))` MUST fire. MEASURED, and it must not:
    ///
    /// ```text
    /// $ bash -n t.sh   # for((i=0;i<3;i++)); do echo $i; done
    /// $ echo $?
    /// 0
    /// $ bash t.sh
    /// 0
    /// 1
    /// 2
    /// $ shellcheck t.sh
    /// (SC2148 shebang, SC2086 quoting — no SC1035)
    /// ```
    ///
    /// `for ((` WITH the space is rejected by dash exactly as `for((` is, which
    /// settles it: the arithmetic for-loop is a bash extension, and the missing
    /// space is not the defect. A must-fire test that pins a false positive is
    /// worse than no test — it makes the fix look like the regression.
    #[test]
    fn test_sc1035_c_style_for_is_valid_bash_and_must_not_fire() {
        let result = check("for((i=0;i<3;i++)); do echo $i; done");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
    }

    /// The metacharacters #276 reported that `bash -n` accepts. Each one was an
    /// error-severity false positive; none may come back.
    #[test]
    fn test_sc1035_metacharacters_with_a_valid_counterexample_do_not_fire() {
        for line in [
            // `)` — the keyword closes a substitution, or is a case pattern.
            "x=$(echo done)",
            "case $x in do) echo;; esac",
            // `|` — a compound command being piped, or a case pattern list.
            "while true; do echo; done|cat",
            "case $x in do|done) echo;; esac",
            // `<` `>` — a bare redirection IS a command, and a terminator takes
            // the loop's own redirect.
            "if true; then>out.txt; fi",
            "for i in 1; do echo hi; done<in.txt",
            "for i in 1; do echo hi; done>out.txt",
            // `&` after a TERMINATOR backgrounds the compound command.
            "if true; then :; fi&",
            "for i in 1; do :; done&",
        ] {
            let result = check(line);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "false positive on valid shell: {line} -> {:?}",
                result.diagnostics
            );
        }
    }
}
