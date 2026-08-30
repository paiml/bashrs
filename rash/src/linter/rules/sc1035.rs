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

use crate::linter::{Diagnostic, LintResult, Severity, Span};

const KEYWORDS: &[&str] = &[
    "then", "do", "else", "elif", "fi", "done", "while", "until", "for", "case", "esac", "in",
];

/// Does `b` end a shell WORD?
///
/// GH-275. A word is "a sequence of characters treated as a unit by the shell",
/// terminated only by an unquoted **metacharacter**: `| & ; ( ) < >` plus space,
/// tab and newline. Every other byte — `-`, `.`, `/`, `$`, `=`, `:`, `*`, quotes
/// — *continues* the word.
///
/// This rule used to treat any non-alphanumeric byte as a terminator, so it read
/// `git for-each-ref` as the `for` keyword followed by `-each-ref` and reported a
/// missing space in a line that `bash -n` and `shellcheck` both accept. Four of
/// the 150 errors on the rmedia corpus were that one line shape.
fn ends_shell_word(b: u8) -> bool {
    matches!(
        b,
        b' ' | b'\t' | b'\n' | b'\r' | b'|' | b'&' | b';' | b'(' | b')' | b'<' | b'>'
    )
}

/// Does a keyword butted directly against `b` mean a space is missing?
///
/// The correct spellings — a space, a tab, a newline, `;`, or a trailing `#`
/// comment — are excluded, leaving the metacharacters that prove the shell ended
/// the word right there. `{` and `}` are not POSIX metacharacters, but bash's
/// parser rejects a keyword fused to one (`do{echo` is `syntax error near
/// unexpected token 'do{'`), so they belong here too.
///
/// Deliberately NOT included: `$` and the quote characters. `echo done$count`
/// and `echo done"$x"` are ordinary, correct shell — the keyword text is just
/// the literal prefix of a word. That leaves `case$x in` (a genuine bash syntax
/// error) unreported; shellcheck does not report SC1035 there either, and a
/// false positive on every `done$n` in the fleet is the worse trade.
fn indicates_missing_space(b: u8) -> bool {
    matches!(b, b'(' | b')' | b'{' | b'}' | b'|' | b'&' | b'<' | b'>')
}

/// Is the `kw`-length token at `at` in `trimmed` a keyword fused to the next
/// word — that is, does the shell end a word on both sides of it, with the
/// right-hand side proving a space is missing?
fn is_fused_keyword(trimmed: &str, at: usize, kw_len: usize) -> bool {
    let bytes = trimmed.as_bytes();
    let after = at + kw_len;

    // The keyword must START a word: the shell has to have ended the previous
    // word right before it. `--for(x)` is one word, so the `for` in it is not a
    // keyword either.
    let starts_word = at == 0 || ends_shell_word(bytes[at - 1]);

    // ...and the shell must END the word right after it. Anything that continues
    // the word (`-`, `.`, `/`, `_`, alphanumerics) means this is a longer word
    // that merely begins with the keyword's letters — `for-each-ref`,
    // `done.txt`, `done_flag`.
    starts_word && after < bytes.len() && indicates_missing_space(bytes[after])
}

/// Every position in `trimmed` where `kw` appears fused to the next word.
fn fused_positions(trimmed: &str, kw: &str) -> Vec<usize> {
    let mut hits = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = trimmed[search_start..].find(kw) {
        let at = search_start + pos;
        if is_fused_keyword(trimmed, at, kw.len()) {
            hits.push(at);
        }
        search_start = at + kw.len();
    }
    hits
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        let line_offset = line.find(trimmed).unwrap_or(0);

        for kw in KEYWORDS {
            for at in fused_positions(trimmed, kw) {
                let col = line_offset + at + 1;
                result.add(Diagnostic::new(
                    "SC1035",
                    Severity::Error,
                    format!("Missing space after '{}' keyword", kw),
                    Span::new(line_num, col, line_num, col + kw.len()),
                ));
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

    // ---- GH-275: keyword matching must respect shell word boundaries ----
    //
    // A shell WORD is terminated by an unquoted metacharacter. `for-each-ref`
    // is ONE word, so the `for` in it is not the `for` keyword. The rule used
    // to treat every non-alphanumeric byte as a word terminator, so `-`, `.`,
    // `/` and `$` all split a word that the shell does not split.

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
        // Each of these is a metacharacter: the shell ends the word there, so
        // the keyword really is a keyword and really is missing its space.
        for line in [
            "if true; then(echo hi)",
            "else(echo fallback)",
            "for i in 1 2; do{echo $i;}",
            "if true; then|echo hi",
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

    #[test]
    fn test_sc1035_must_still_fire_c_style_for() {
        let result = check("for((i=0;i<3;i++)); do echo $i; done");
        assert!(
            result.diagnostics.iter().any(|d| d.code == "SC1035"),
            "{:?}",
            result.diagnostics
        );
    }
}
