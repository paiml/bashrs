//! SC1044: End token not found (unterminated heredoc)
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cat <<EOF
//! hello world
//! # missing closing EOF
//! ```
//!
//! Good:
//! ```bash
//! cat <<EOF
//! hello world
//! EOF
//! ```
//!
//! # Rationale
//!
//! Every heredoc must be terminated by its delimiter on its own line.
//! If the closing delimiter is never found, the shell will consume the rest
//! of the script as heredoc content, causing confusing errors.
//!
//! # `<<<` is a herestring, not a heredoc
//!
//! ```bash
//! grep -c . <<<'hello'      # one line of input. No body, no terminator.
//! ```
//!
//! The delimiter regex is unanchored, so on `<<<'hello'` it matched the `<<`
//! at offset 1 — the *second and third* angle brackets — read `'hello'` as a
//! quoted terminator, and reported an unterminated heredoc for a construct
//! that has no body to terminate. A gating ERROR, on correct shell.
//!
//! Only the quoted spellings were affected, because `<<<$var` and `<<<word`
//! leave a non-word character (`$`) or make the third `<` part of a match that
//! still starts at offset 0. That is why this survived: the two forms a script
//! is most likely to contain are the two that behaved.
//!
//! SC1078's scanner already models this correctly — it classifies a `<<` into
//! `Heredoc` / `Herestring` / `Neither` — so the rule here is the outlier, and
//! the fix is the same distinction: a `<<` preceded by another `<` is the tail
//! of a herestring and starts nothing.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Regex to match heredoc start and capture the delimiter (no backreferences)
#[allow(clippy::expect_used)]
static HEREDOC_START: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<<-?\s*\\?(?:'(\w+)'|"(\w+)"|(\w+))"#).expect("valid heredoc start regex")
});

/// Extract the delimiter from captures (whichever alternative matched)
fn extract_delimiter<'a>(caps: &'a regex::Captures<'a>) -> Option<&'a str> {
    caps.get(1)
        .or_else(|| caps.get(2))
        .or_else(|| caps.get(3))
        .map(|m| m.as_str())
}

/// Where a heredoc opens on a line, and the word that must close it.
struct HeredocOpen {
    /// Byte offset of the `<<` within the line.
    start: usize,
    /// Byte length of the whole `<<DELIM` match, for the span.
    len: usize,
    delimiter: String,
    /// `<<-` strips leading tabs from the terminator line too.
    strips_tabs: bool,
}

/// The first `<<` on the line that actually opens a heredoc.
///
/// The regex has no lookbehind, so a match that begins immediately after
/// another `<` is the tail of a `<<<` herestring and is skipped. Scanning ON
/// rather than returning `None` at the first herestring keeps a real heredoc
/// later on the same line visible: `grep . <<<"$x" && cat <<EOF` opens one.
fn first_heredoc_start(line: &str) -> Option<HeredocOpen> {
    HEREDOC_START.captures_iter(line).find_map(|caps| {
        let m = caps.get(0)?;
        // Group 0 begins at the `<<`; a `<` right before it means `<<<`.
        if m.start()
            .checked_sub(1)
            .is_some_and(|prev| line.as_bytes()[prev] == b'<')
        {
            return None;
        }
        Some(HeredocOpen {
            start: m.start(),
            len: m.len(),
            delimiter: extract_delimiter(&caps)?.to_string(),
            strips_tabs: line[m.start()..].starts_with("<<-"),
        })
    })
}

/// Index of the line that closes `open`, searching from `from` onwards.
fn terminator_line(lines: &[&str], from: usize, open: &HeredocOpen) -> Option<usize> {
    lines.iter().enumerate().skip(from).find_map(|(j, line)| {
        let candidate = if open.strips_tabs {
            line.trim_start_matches('\t')
        } else {
            line
        };
        (candidate.trim() == open.delimiter).then_some(j)
    })
}

/// Check for unterminated heredocs
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line.trim_start().starts_with('#') {
            i += 1;
            continue;
        }

        let Some(open) = first_heredoc_start(line) else {
            i += 1;
            continue;
        };

        match terminator_line(&lines, i + 1, &open) {
            Some(j) => i = j + 1,
            None => {
                let line_num = i + 1;
                let col = open.start + 1;
                result.add(Diagnostic::new(
                    "SC1044",
                    Severity::Error,
                    format!(
                        "Couldn't find end token '{}' for this heredoc",
                        open.delimiter
                    ),
                    Span::new(line_num, col, line_num, col + open.len),
                ));
                i += 1;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1044_detects_unterminated_heredoc() {
        let script = "cat <<EOF\nhello world\n# no closing delimiter";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1044");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("EOF"));
    }

    #[test]
    fn test_sc1044_no_flag_terminated_heredoc() {
        let script = "cat <<EOF\nhello world\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1044_no_flag_strip_heredoc_with_tabs() {
        let script = "cat <<-EOF\n\thello\n\tEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1044_detects_wrong_delimiter() {
        // Opening says EOF but closing says END
        let script = "cat <<EOF\nhello\nEND";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1044_multiple_heredocs_one_unterminated() {
        let script = "cat <<EOF\nhello\nEOF\ncat <<MARKER\nworld";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("MARKER"));
    }

    #[test]
    fn test_sc1044_no_false_positive_comment() {
        let script = "# cat <<EOF\nhello\n# not a heredoc";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    /// A herestring has no body and no terminator. Every spelling of one, and
    /// one row per QUOTING FORM rather than one for the form as a whole —
    /// `<<<$var` and `<<<word` were always fine and would have signed off on
    /// the bug, which is exactly how it shipped.
    #[test]
    fn test_sc1044_herestring_is_not_a_heredoc() {
        for script in [
            "grep -c . <<<'hello'",
            "grep -c . <<<\"hello\"",
            "grep -c . <<<\"$x\"",
            "grep -c . <<<word",
            "grep -qxF -- \"$j\" <<<'literal with spaces'",
            "read -r a b <<<\"$line\"",
        ] {
            let result = check(script);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "herestring reported as an unterminated heredoc: {script}"
            );
        }
    }

    /// Skipping the herestring must SCAN ON, not give up on the line: a real
    /// heredoc opened after one is still unterminated and must still be caught.
    #[test]
    fn test_sc1044_real_heredoc_after_a_herestring_still_flagged() {
        let script = "grep . <<<'x' && cat <<EOF\nbody\n# no terminator";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("EOF"));
    }

    /// ...and a terminated one after a herestring is still clean.
    #[test]
    fn test_sc1044_terminated_heredoc_after_a_herestring_is_clean() {
        let script = "grep . <<<'x' && cat <<EOF\nbody\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
