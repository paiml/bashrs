// SC2188: Redirection without command
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static LONE_REDIRECT: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^\s*[<>]").unwrap());

/// Does this physical line end in a backslash that continues the logical line?
///
/// Only an ODD number of trailing backslashes continues: `foo \` continues,
/// while `foo \\` ends in an escaped literal backslash and does not.
fn ends_with_continuation(line: &str) -> bool {
    line.chars().rev().take_while(|&c| c == '\\').count() % 2 == 1
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // #211: `source.lines()` yields PHYSICAL lines, but "does this redirection
    // have a command?" is a question about the LOGICAL line. In
    //
    //     sha256sum a \
    //               b \
    //       > out
    //
    // the redirect belongs to `sha256sum`; reporting it as a lone redirect is a
    // false positive. Since this rule is Severity::Error, that false positive is
    // not cosmetic - it aborts `forjar apply`, which treats a bashrs error as a
    // fatal I8 violation.
    //
    // So: carry whether the previous physical line left the logical line open,
    // and skip continuation lines. Diagnostics still report the physical line of
    // a genuine lone redirect, which is what a reader needs to find it.
    let mut prev_continues = false;

    for (idx, raw) in source.lines().enumerate() {
        let line_num = idx + 1;
        // `str::lines()` already strips a trailing \r for CRLF input; this is
        // belt-and-braces so a lone \r cannot defeat the backslash test.
        let line = raw.strip_suffix('\r').unwrap_or(raw);

        let is_comment = line.trim_start().starts_with('#');
        // A comment runs to end of line: a trailing backslash inside one does
        // NOT continue it, so it must not swallow the following line.
        let continues = !is_comment && ends_with_continuation(line);

        let was_continuation = prev_continues;
        prev_continues = continues;

        if was_continuation || is_comment {
            continue;
        }

        if LONE_REDIRECT.is_match(line) && !line.contains("<<") {
            let diagnostic = Diagnostic::new(
                "SC2188",
                Severity::Error,
                "Redirection without command".to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
            );
            result.add(diagnostic);
        }
    }
    result
}

/// Kani proof harnesses for GH-211 continuation folding.
///
/// Contract: contracts/linter-logical-lines-v1.yaml
/// These verify the parity equation over a bounded run of trailing backslashes,
/// which is the part of the fix that is easy to get subtly wrong by hand.
#[cfg(kani)]
mod kani_proofs {
    use super::{check, ends_with_continuation};

    /// KANI-LINTER_LOGICAL_LINES-001
    /// An ODD run of trailing backslashes continues the logical line; an EVEN
    /// run is an escaped literal backslash and terminates it.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_continuation_parity() {
        let n: usize = kani::any();
        kani::assume(n <= 5);
        let line = format!("cat a {}", "\\".repeat(n));
        assert_eq!(ends_with_continuation(&line), n % 2 == 1);
    }

    /// KANI-LINTER_LOGICAL_LINES-002 (precision)
    /// When the previous line genuinely continues, a following redirect is
    /// never reported as a lone redirect.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_continuation_suppresses_redirect() {
        let n: usize = kani::any();
        kani::assume(n >= 1 && n <= 5);
        kani::assume(n % 2 == 1);
        let src = format!("cat a {}\n  > out.txt\n", "\\".repeat(n));
        assert_eq!(check(&src).diagnostics.len(), 0);
    }

    /// KANI-LINTER_LOGICAL_LINES-003 (soundness)
    /// When the previous line does NOT continue, the redirect is still caught.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_lone_redirect_still_detected() {
        let n: usize = kani::any();
        kani::assume(n <= 4);
        kani::assume(n % 2 == 0);
        let src = format!("cat a {}\n> out.txt\n", "\\".repeat(n));
        assert_eq!(check(&src).diagnostics.len(), 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2188_lone_redirect() {
        let code = "> output.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2188_normal_ok() {
        let code = "echo test > output.txt";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // --- #211: backslash line-continuation ------------------------------------
    //
    // `source.lines()` yields PHYSICAL lines. A line whose predecessor ends in an
    // unescaped `\` is a continuation of the same LOGICAL line, so the redirect
    // there does have a command - it is just on an earlier physical line.
    // Severity is Error, so a false positive here does not merely add noise: it
    // blocks `forjar apply`, which treats a bashrs error as a fatal I8 violation.

    #[test]
    fn continuation_into_redirect_is_not_a_lone_redirect() {
        // Valid POSIX sh; `bash -n` accepts it.
        let code = "sha256sum /etc/hostname \\\n          /etc/hosts \\\n  > /tmp/out.sha256\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "line 3 continues the sha256sum command begun on line 1; \
             flagging it as a lone redirect is a false positive (#211)"
        );
    }

    #[test]
    fn continuation_single_break_before_redirect() {
        let code = "cat a.txt \\\n  > out.txt\n";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn genuine_lone_redirect_after_a_continued_command_still_flagged() {
        // The continued command ends on line 2; line 3 really is a lone redirect.
        let code = "cat a.txt \\\n  b.txt\n> truncate.txt\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "a real lone redirect must still be caught after a continuation ends"
        );
        assert_eq!(
            result.diagnostics[0].span.start_line, 3,
            "diagnostic must point at the offending physical line"
        );
    }

    #[test]
    fn escaped_backslash_at_eol_is_not_a_continuation() {
        // `printf 'x\\'` ends with an ESCAPED backslash, so the logical line ends
        // there and the following `> f.txt` is a genuine lone redirect.
        let code = "printf 'x' \\\\\n> f.txt\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "an escaped backslash is not a line continuation"
        );
    }

    #[test]
    fn lone_redirect_on_first_line_still_flagged_at_line_1() {
        let result = check("> output.txt\n");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 1);
    }

    #[test]
    fn commented_continuation_does_not_suppress_a_later_lone_redirect() {
        let code = "# a comment \\\n> real.txt\n";
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "a trailing backslash inside a comment must not swallow the next line"
        );
    }

    #[test]
    fn crlf_continuation_is_handled() {
        let code = "cat a.txt \\\r\n  > out.txt\r\n";
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "CRLF input must fold continuations the same way as LF"
        );
    }

    #[test]
    fn multiple_independent_lone_redirects_all_flagged() {
        let code = "> a.txt\necho ok\n< b.txt\n";
        assert_eq!(check(code).diagnostics.len(), 2);
    }

    #[test]
    fn heredoc_still_exempt_under_continuation_folding() {
        let code = "cat <<EOF\nhello\nEOF\n";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    // The Kani harnesses in `kani_proofs` above cannot currently be executed in
    // this repo: the crate does not compile under `cfg(kani)` because of 19
    // pre-existing errors in verifier/kani_harnesses.rs and
    // formal/kani_harnesses.rs (hence the `|| true` on `make verify-kani`).
    // These two tests verify the SAME bounded properties exhaustively over the
    // same domain in plain Rust, so the contract's obligations are actually
    // checked on every `cargo test` rather than only aspirationally.

    #[test]
    fn parity_holds_exhaustively_over_bounded_backslash_runs() {
        for n in 0..=8usize {
            let line = format!("cat a {}", "\\".repeat(n));
            assert_eq!(
                ends_with_continuation(&line),
                n % 2 == 1,
                "trailing backslash run of {n} misclassified"
            );
        }
    }

    #[test]
    fn folding_soundness_and_precision_exhaustively_over_bounded_runs() {
        for n in 0..=8usize {
            let src = format!("cat a {}\n> out.txt\n", "\\".repeat(n));
            let got = check(&src).diagnostics.len();
            if n % 2 == 1 {
                assert_eq!(got, 0, "odd run ({n}) continues: redirect must be exempt");
            } else {
                assert_eq!(got, 1, "even run ({n}) does not continue: redirect is lone");
            }
        }
    }
}
