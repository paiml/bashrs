//! SC1078: Did you forget to close this double-quoted string?
//!
//! Reports a double-quoted string that is opened and never closed **in the
//! whole file**.
//!
//! ## Why this is a whole-source scan and not a per-line quote count
//!
//! This rule used to flag any line with an odd number of unescaped double
//! quotes. That is not what bash means by an unterminated string. Two real
//! constructs are odd-on-a-line and perfectly valid:
//!
//! ```sh
//! DIRS="one two             # opens here...
//!       three four"         # ...and closes here. Both lines flagged before.
//!
//! echo "intel's timer runs daily."   # the apostrophe is LITERAL inside "..."
//! ```
//!
//! The second case was a state bug: the counter tracked single-quote state but
//! not double-quote state, so an apostrophe inside a double-quoted string
//! opened a phantom single-quoted region that swallowed the closing quote.
//!
//! The tell for the multi-line case was that the old rule flagged BOTH the
//! opening and the closing line — a genuinely unterminated string can only be
//! opened once. Scanning the source as one stream reports it once, at the line
//! where the string actually opens, which is the line the author must edit.
//!
//! Heredoc bodies are skipped between their start marker and terminator:
//! their contents are data, and an apostrophe in prose would otherwise poison
//! the quote state for the rest of the file.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Where an unterminated double-quoted string was opened.
struct OpenQuote {
    line: usize,
    col: usize,
}

/// Check for unclosed double-quoted strings.
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut open: Option<OpenQuote> = None;
    let mut heredoc_end: Option<String> = None;
    // Single-quote state spans lines exactly as double-quote state does. It
    // used to be a local reset on every line, so the second line of a
    // multi-line `'...'` was scanned as code and a `"` in an embedded awk or
    // sed program opened a phantom double-quoted string that ran to EOF.
    let mut in_single = false;

    for (idx, line) in source.lines().enumerate() {
        let line_num = idx + 1;

        // Inside a heredoc: the body is data until the terminator.
        if let Some(term) = &heredoc_end {
            if line.trim() == term.as_str() {
                heredoc_end = None;
            }
            continue;
        }

        // A whole-line comment cannot open a string — but only when we are not
        // already inside one, where `#` is an ordinary character.
        if open.is_none() && !in_single && line.trim_start().starts_with('#') {
            continue;
        }

        // `<<` inside a string literal is text, not a redirection.
        if open.is_none() && !in_single {
            heredoc_end = heredoc_terminator(line);
        }
        scan_line(line, line_num, &mut open, &mut in_single);
    }

    if let Some(q) = open {
        result.add(Diagnostic::new(
            "SC1078",
            Severity::Error,
            "Did you forget to close this double-quoted string?",
            Span::new(q.line, q.col + 1, q.line, q.col + 2),
        ));
    }

    result
}

/// Advance the double-quote state across one line.
///
/// Three states, one step function each: inside `'...'` nothing is special but
/// the closing quote; inside `"..."` a backslash escapes and only `"` closes;
/// otherwise quotes open and an unquoted `#` ends the line.
fn scan_line(line: &str, line_num: usize, open: &mut Option<OpenQuote>, in_single: &mut bool) {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if *in_single {
            *in_single = bytes[i] != b'\'';
            i += 1;
        } else if open.is_some() {
            i = step_in_double(bytes, i, open);
        } else {
            match step_neutral(bytes, i, line_num, open, in_single) {
                Some(next) => i = next,
                None => return, // a comment began; the rest is prose
            }
        }
    }
}

/// One byte inside a double-quoted string. Returns the next index.
fn step_in_double(bytes: &[u8], i: usize, open: &mut Option<OpenQuote>) -> usize {
    match bytes[i] {
        b'\\' => i + 2, // escapes the next byte, whatever it is
        b'"' => {
            *open = None;
            i + 1
        }
        _ => i + 1,
    }
}

/// One byte outside any string. Returns the next index, or `None` if a comment
/// started and the remainder of the line must not be scanned.
fn step_neutral(
    bytes: &[u8],
    i: usize,
    line_num: usize,
    open: &mut Option<OpenQuote>,
    in_single: &mut bool,
) -> Option<usize> {
    match bytes[i] {
        b'\\' => return Some(i + 2),
        b'\'' => *in_single = true,
        b'"' => {
            *open = Some(OpenQuote {
                line: line_num,
                col: i,
            })
        }
        // An unquoted `#` at the start of a word begins a comment.
        b'#' if i == 0 || bytes[i - 1].is_ascii_whitespace() => return None,
        _ => {}
    }
    Some(i + 1)
}

/// What follows a `<<` on the line.
enum Redirect {
    /// A heredoc whose body ends at this terminator word.
    Heredoc(String),
    /// `<<<` is a herestring: no body, no terminator. Resume scanning here.
    Herestring(usize),
    /// `<<` not followed by a word (e.g. a left-shift in arithmetic).
    Neither,
}

/// If `line` starts a heredoc, return the terminator word to watch for.
fn heredoc_terminator(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'<' {
            match classify_redirect(line, i + 2) {
                Redirect::Heredoc(word) => return Some(word),
                Redirect::Herestring(next) => {
                    i = next;
                    continue;
                }
                Redirect::Neither => {}
            }
        }
        i += 1;
    }
    None
}

/// Classify what follows a `<<` that starts at `start`.
///
/// Accepts the optional `-` of `<<-`, surrounding spaces, and a quoted
/// terminator (`<<'EOF'`), since all four spellings delimit the same body.
fn classify_redirect(line: &str, start: usize) -> Redirect {
    let bytes = line.as_bytes();
    let mut j = start;

    if bytes.get(j) == Some(&b'<') {
        return Redirect::Herestring(j + 1);
    }
    if bytes.get(j) == Some(&b'-') {
        j += 1;
    }
    while bytes.get(j) == Some(&b' ') {
        j += 1;
    }
    if matches!(bytes.get(j), Some(b'\'') | Some(b'"')) {
        j += 1;
    }

    let word_start = j;
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
        j += 1;
    }
    if j > word_start {
        Redirect::Heredoc(line[word_start..j].to_string())
    } else {
        Redirect::Neither
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1078_unclosed_double_quote() {
        let script = "echo \"hello world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1078_closed_double_quote() {
        let script = "echo \"hello world\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_escaped_quote_not_flagged() {
        let script = r#"echo "hello \" world""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_comment_skipped() {
        let script = "# echo \"unclosed";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_single_quote_inside_not_counted() {
        // Single-quoted section containing " should not affect count
        let script = "echo 'he said \"hi\"'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ── regressions found on real infra scripts (paiml/bashrs) ──────────────

    #[test]
    fn test_sc1078_apostrophe_inside_double_quotes_is_literal() {
        // machines/lambda-labs/rag:85. The `'` in "intel's" is an ordinary
        // character inside "..."; the old counter treated it as opening a
        // single-quoted span, which swallowed the closing double quote.
        let script = r#"echo "corpus built; intel's timer runs daily." >&2"#;
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_multiline_string_is_valid() {
        // machines/unas/nas-sweep.sh:30-32. Valid bash, and the old rule
        // flagged BOTH lines — a string can only be opened once.
        let script = "DIRS=\"one two\n      three four\"\necho \"$DIRS\"";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_multiline_unterminated_reports_once_at_the_opening() {
        // Genuinely broken: report once, on the line the author must edit.
        let script = "die \"usage: tool <dir>\nthis line never closes it";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 1);
    }

    #[test]
    fn test_sc1078_heredoc_body_does_not_poison_quote_state() {
        // Prose in a heredoc is data. Without terminator tracking the
        // apostrophe here would open a string for the rest of the file.
        let script = "cat <<'EOF'\nit's fine to write \" here\nEOF\necho \"ok\"";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_trailing_comment_is_not_scanned() {
        let script = "ls -l   # don't count this apostrophe\necho \"ok\"";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_herestring_is_not_a_heredoc() {
        // `<<<` must not swallow the rest of the file looking for a terminator.
        let script = "grep x <<< \"$var\"\necho \"unclosed";
        assert_eq!(check(script).diagnostics.len(), 1);
    }
}

#[cfg(test)]
mod tests_multiline_single_quote {
    use super::*;

    /// A single-quoted string that spans lines keeps its literal state across
    /// the newline. A `"` on a continuation line is an ordinary character, so
    /// it must not open a phantom double-quoted string.
    ///
    /// Found in infra's `ci-blackbox.sh`: an embedded awk program whose regex
    /// `/"ts":"[^"]+"/` holds an odd number of `"`. `bash -n` and `shellcheck`
    /// both accept it.
    #[test]
    fn multiline_single_quote_carries_across_lines() {
        let script = "echo x | awk '\n    match($0, /\"ts\":\"[^\"]+\"/) {\n        ts = 1\n    }'\necho done\n";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "awk body inside '...' is literal; got {:?}",
            result.diagnostics
        );
    }

    /// The smallest form of the same defect.
    #[test]
    fn double_quote_inside_multiline_single_quote_is_literal() {
        let result = check("echo '\na\"b\n'\n");
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }

    /// MUST STILL FIRE: carrying single-quote state must not blind the rule to
    /// a genuinely unterminated double-quoted string.
    #[test]
    fn still_fires_on_genuine_unterminated_double_quote_after_single_quoted_block() {
        let script = "echo 'literal\ntext'\necho \"unterminated\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert_eq!(result.diagnostics[0].span.start_line, 3);
    }

    /// MUST STILL FIRE: an unterminated double quote opened *before* a
    /// single-quoted line is still reported at the line where it opened.
    #[test]
    fn still_fires_when_unterminated_quote_precedes_single_quoted_block() {
        let script = "echo \"oops\necho 'plain'\necho tail\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert_eq!(result.diagnostics[0].span.start_line, 1);
    }

    /// MUST STILL FIRE: a multi-line double-quoted string that never closes is
    /// still reported, even when a single-quoted region precedes it.
    #[test]
    fn still_fires_on_unterminated_double_quote_spanning_lines() {
        let script = "awk 'BEGIN{print 1}'\nDIRS=\"one two\nthree four\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].span.start_line, 2);
    }

    /// `echo "oops` followed by `echo 'a"b'` is NOT an unterminated *double*
    /// quote: the `"` in `a"b` closes it. bash reports the unmatched `'`
    /// instead ("unexpected EOF while looking for matching `''"), and
    /// shellcheck reports SC1073 on the single quote. SC1078's subject is the
    /// double quote, so it correctly stays silent here.
    #[test]
    fn unmatched_single_quote_is_not_sc1078s_finding() {
        let result = check("echo \"oops\necho 'a\"b'\n");
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }
}
