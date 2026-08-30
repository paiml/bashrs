//! SC1078: Did you forget to close this quoted string?
//!
//! Reports a quoted string that is opened and never closed **in the whole
//! file**.
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
//! The tell for the multi-line case was that the old rule flagged BOTH the
//! opening and the closing line — a genuinely unterminated string can only be
//! opened once. Scanning the source as one stream reports it once, at the line
//! where the string actually opens, which is the line the author must edit.
//!
//! ## One scanner, not two (GH-272)
//!
//! The whole-source scan was a second, weaker lexer living inside this rule:
//! three states — neutral, in `'`, in `"` — with `in_single` reset at every
//! newline and no notion of `$( )`, `${ }`, backticks, `$'…'` or `(( ))`. It
//! produced 73 of the 150 errors bashrs 6.67.0 reported on the rmedia script
//! corpus, and those 73 are what buried the corpus's real SEC011 findings.
//! Two shapes did it:
//!
//! ```sh
//! awk '                                   # a program spanning lines...
//!     while (match(line, /"[^"]+"/)) { }  # ...with an ODD number of " in it
//! ' "$file"
//!
//! echo "legs [$(printf '%s' "$f" | tr '\n' ' ')] over $scope"   # $( ) nesting
//! ```
//!
//! `linter::quoting` already carries all of that state — it is what the
//! literal mask every other syntax rule reads is built from. The rule now asks
//! it instead of guessing, so a file cannot be masked as well-quoted and
//! reported as unterminated at the same time.

use crate::linter::quoting::{unterminated_quote, QuoteKind};
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for unclosed quoted strings.
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    if let Some(q) = unterminated_quote(source) {
        // A `'` that never closes runs to EOF — that is what bash does, and
        // `bash -n` fails with "unexpected EOF". shellcheck reports it as the
        // parse errors SC1072/SC1073, neither of which bashrs implements, so
        // it is reported here rather than dropped. The message names the kind
        // so the diagnosis is not the wrong one the old scanner gave, which
        // blamed whichever `"` it happened to meet first inside the `'...'`.
        let message = match q.kind {
            QuoteKind::Double => "Did you forget to close this double-quoted string?",
            QuoteKind::Single => "Did you forget to close this single-quoted string?",
        };
        result.add(Diagnostic::new(
            "SC1078",
            Severity::Error,
            message,
            Span::new(q.line, q.col, q.line, q.col + 1),
        ));
    }

    result
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

    // ── GH-272: single-quote state must survive the newline ────────────────

    #[test]
    fn test_sc1078_multiline_single_quoted_program_is_not_shell() {
        // rmedia/scripts/lint-verb-coverage.sh:58-69. An awk program passed as
        // one single-quoted argument. The regex `/"[^"]+"/` holds an ODD number
        // of double quotes, which is fine — inside '...' they are text.
        //
        // `in_single` used to be a local of `scan_line`, reset at every
        // newline, so the second line of the program was scanned as code and
        // the first `"` opened a string that nothing ever closed.
        let script = "awk '\n    while (match(line, /\"[^\"]+\"/)) {\n        print \"x\"\n    }\n' file\necho \"tail\"\n";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_multiline_single_quoted_heredocish_text_is_not_a_heredoc() {
        // A `<<EOF` inside a single-quoted program is text, not a redirection.
        let script = "awk '\n  print \"a <<EOF b\"\n' file\necho \"tail\"\n";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1078_hash_inside_multiline_single_quotes_is_not_a_comment() {
        // An awk comment line inside '...' must not be skipped as a shell
        // comment, and must not end the single-quoted region.
        let script = "awk '\n  # counts \" marks\n  { n++ }\n' file\necho \"tail\"\n";
        assert_eq!(check(script).diagnostics.len(), 0);
    }

    // ── must still fire ────────────────────────────────────────────────────

    #[test]
    fn must_still_fire_unterminated_double_quote_after_a_closed_awk_program() {
        // The state fix must not blind the rule: once the '...' closes, a
        // genuinely unterminated " is still SC1078.
        let script = "awk '\n  print \"x\"\n' file\necho \"unclosed\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert_eq!(result.diagnostics[0].span.start_line, 4);
    }

    #[test]
    fn must_still_fire_unterminated_double_quote_opened_inside_a_multiline_command() {
        // Carrying state across lines must not swallow a real defect that
        // opens on a later line.
        let script = "x=1\ny=2\necho \"still open at EOF\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].span.start_line, 3);
    }

    #[test]
    fn must_still_fire_unterminated_single_quote_is_reported_not_dropped() {
        // Carrying single-quote state means a stray ' now runs to EOF. That is
        // what bash does (`bash -n` fails with "unexpected EOF"), and
        // shellcheck calls it out as SC1072 "Expected end of single quoted
        // string". bashrs implements neither SC1072 nor SC1073, so the finding
        // is reported here rather than dropped — with a message that names the
        // quote kind, so the diagnosis is not the wrong one it used to give.
        let script = "echo don't\necho \"ok\"\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert_eq!(result.diagnostics[0].span.start_line, 1);
        assert!(
            result.diagnostics[0].message.contains("single"),
            "message must name the quote kind, got: {}",
            result.diagnostics[0].message
        );
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

    /// `echo "oops` followed by `echo 'a"b'`: the `"` in `a"b` closes the
    /// double quote opened on line 1, and the trailing `'` then opens a single
    /// quote that runs to EOF.
    ///
    /// This test arrived from #278 asserting SILENCE, on the reasoning that
    /// "SC1078's subject is the double quote". MEASURED, and that reasoning
    /// does not survive the measurement:
    ///
    /// ```text
    /// $ bash -n t.sh
    /// t.sh: line 2: unexpected EOF while looking for matching `''
    /// $ shellcheck t.sh
    /// line 1: SC1078 (warning): Did you forget to close this double quoted string?
    /// line 2: SC1073 (error): Couldn't parse this single quoted string.
    /// line 3: SC1072 (error): Expected end of single quoted string.
    /// ```
    ///
    /// The file is genuinely broken shell, and shellcheck itself emits SC1078
    /// on it. Staying silent would be a FALSE NEGATIVE on a file `bash -n`
    /// rejects — exactly the trade this PR's gate exists to refuse. bashrs
    /// implements neither SC1072 nor SC1073, so the unterminated quote is
    /// reported here, naming the kind that is actually open at EOF.
    #[test]
    fn unmatched_single_quote_is_still_reported_not_silently_accepted() {
        let result = check("echo \"oops\necho 'a\"b'\n");
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC1078");
        assert!(
            result.diagnostics[0].message.contains("single"),
            "the quote open at EOF is the single one, got: {}",
            result.diagnostics[0].message
        );
    }
}
