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

/// Is `tok` a redirection that already carries its target?
///
/// `>&2`, `2>&1`, `<&-`, `>>out`, `2>/dev/null` — the operand is attached, so
/// no following word is consumed.
fn redirect_has_attached_target(tok: &str) -> bool {
    let b = tok.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i >= b.len() || (b[i] != b'<' && b[i] != b'>') {
        return false;
    }
    i += 1;
    // `>>`, `<<`, `<>`, `>|`, `>&`, `<&`
    if i < b.len() && matches!(b[i], b'>' | b'<' | b'|' | b'&') {
        i += 1;
    }
    i < b.len() // something follows the operator, so the target is attached
}

/// Is `tok` a bare redirection operator, whose target is the NEXT word?
fn is_bare_redirect_operator(tok: &str) -> bool {
    let b = tok.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i >= b.len() || (b[i] != b'<' && b[i] != b'>') {
        return false;
    }
    i += 1;
    if i < b.len() && matches!(b[i], b'>' | b'<' | b'|' | b'&') {
        i += 1;
    }
    i == b.len()
}

/// Drop a trailing comment, so it is not counted as the command (GH-272).
///
/// `#211`/`#239` taught this rule to consume leading redirections and report
/// only if no word was left. `split_whitespace` makes `#` a word, so
/// `> deploy.log  # note` read as "redirect, then a command" and the rule went
/// silent on a genuine lone redirect — a FALSE NEGATIVE, which is the worse
/// half of the trade this rule exists to make.
///
/// A `#` opens a comment only at the start of a word, which is what keeps the
/// `#` of `${#arr}` and `$#` out of it. This rule is now fed the source with
/// string literals masked (`quoting::QUOTE_SENSITIVE_RULES`), so a `#` inside
/// `"…"` or `'…'` has already become filler and cannot reach here at all.
fn strip_trailing_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
            return &line[..i];
        }
    }
    line
}

/// Does a COMMAND follow the leading redirections on this line?
///
/// Issue #239: POSIX permits redirections anywhere in a simple command,
/// including BEFORE the command name (`>&2 echo "..."`, `> out.txt cat in.txt`).
/// Matching `^\s*[<>]` alone reports those as "redirection without command",
/// and at Severity::Error that aborts `forjar apply`.
///
/// Consume leading redirections — with attached targets, or bare operators that
/// take the next word — and report only if nothing is left.
fn command_follows_redirections(line: &str) -> bool {
    let mut toks = strip_trailing_comment(line).split_whitespace().peekable();
    while let Some(tok) = toks.peek().copied() {
        if redirect_has_attached_target(tok) {
            toks.next();
        } else if is_bare_redirect_operator(tok) {
            toks.next();
            toks.next(); // its target
        } else {
            return true; // a word that is not a redirection: the command
        }
    }
    false
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

        if LONE_REDIRECT.is_match(line)
            && !line.contains("<<")
            && !command_follows_redirections(line)
        {
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

    // ── Issue #239: a redirection may PRECEDE the command ────────────────────

    /// The reported case. `>&2 echo "..."` is valid POSIX and extremely common
    /// for writing to stderr; at Severity::Error this aborted `forjar apply`.
    #[test]
    fn test_sc2188_redirection_before_the_command_is_valid() {
        for src in [
            ">&2 echo 'message to stderr'\n",
            "2>&1 echo hi\n",
            "> out.txt cat in.txt\n",
            "  >>log.txt printf '%s\\n' done\n",
            "2>/dev/null command -v rsync\n",
            "< input.txt sort\n",
        ] {
            let r = check(src);
            assert!(
                r.diagnostics.is_empty(),
                "SC2188 fired on valid POSIX `{}`: {:?}",
                src.trim(),
                r.diagnostics
            );
        }
    }

    /// Guard the guard: a genuinely commandless redirection must still fire, or
    /// the fix has simply switched the rule off.
    #[test]
    fn test_sc2188_a_truly_lone_redirection_still_fires() {
        // `2>&1` alone is NOT here: `LONE_REDIRECT` is `^\s*[<>]`, so a line
        // starting with an fd digit was never in this rule's scope. That is a
        // pre-existing gap (paiml/bashrs#249), not a regression from this fix —
        // and widening a Severity::Error rule to report MORE does not belong in
        // a release whose purpose is removing false positives.
        for src in ["> file\n", ">> file\n", "< input\n", ">&2\n"] {
            let r = check(src);
            assert_eq!(
                r.diagnostics.len(),
                1,
                "SC2188 missed a real lone redirect `{}`",
                src.trim()
            );
        }
    }

    // ── GH-272: a trailing comment is not a command ────────────────────────

    #[test]
    fn must_still_fire_with_a_trailing_comment() {
        // Regression, pre-existing on main: #250 taught this rule that a
        // redirection may PRECEDE its command, by consuming leading redirects
        // and reporting only if no word is left. A trailing comment is a word
        // to `split_whitespace`, so `> deploy.log  # note` looked like a
        // redirect followed by a command and the rule went quiet on a genuine
        // lone redirect. A false negative is the worse half of the trade.
        let result = check("> deploy.log  # ERROR (SC2188: Redirection without command)\n");
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC2188");
    }

    #[test]
    fn known_gap_a_numbered_fd_never_reaches_this_rule_at_all() {
        // NOT a claim that silence is right here — `2> err.log` IS a
        // redirection without a command. `LONE_REDIRECT` is `^\s*[<>]`, so a
        // leading file descriptor number is never matched, while the rest of
        // the rule (`redirect_has_attached_target`, `is_bare_redirect_operator`)
        // handles numbered fds perfectly well. The entry test and the body
        // disagree.
        //
        // Left alone deliberately: widening the reach of a Severity::Error rule
        // belongs in a change that has corpus evidence for it, not in one whose
        // subject is removing false positives. Recorded so it is not lost.
        assert_eq!(check("2> err.log # no command\n").diagnostics.len(), 0);
        assert_eq!(check("> err.log # no command\n").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2188_a_command_after_the_redirect_still_silences_it() {
        // The #239 behaviour must survive: a real command after a leading
        // redirection means there is nothing to report.
        assert_eq!(check("> out.txt cat in.txt\n").diagnostics.len(), 0);
        assert_eq!(check("> out.txt cat in.txt  # note\n").diagnostics.len(), 0);
        assert_eq!(check(">&2 echo oops\n").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2188_a_hash_that_is_not_a_comment_is_not_a_comment() {
        // `${#arr}` and `$#` contain a `#` that starts no comment: it does not
        // begin a word. Whatever the verdict, it must not come from mistaking
        // these for a comment marker.
        assert_eq!(check("> out.txt echo ${#arr}\n").diagnostics.len(), 0);
        assert_eq!(check("> out.txt echo $#\n").diagnostics.len(), 0);
    }
}

#[cfg(test)]
mod tests_literal_content {
    use super::*;

    /// XML, HTML or a diff inside a multi-line single-quoted string is data.
    /// Its `<` and `>` are not redirections. Reaches this rule already masked:
    /// SC2188 is in `quoting::QUOTE_SENSITIVE_RULES`.
    ///
    /// Found in llama.cpp's `build-xcframework.sh`, where a plist fragment is
    /// assigned with `local device_family='…'`. `bash -n` accepts and
    /// shellcheck reports nothing at any severity.
    #[test]
    fn angle_brackets_in_a_single_quoted_literal_are_not_redirections() {
        let masked = crate::linter::quoting::mask_literals(
            "f(){\n    local xml='    <key>K</key>\n    <array>\n        <integer>1</integer>\n    </array>'\n}\n",
        );
        let result = check(&masked);
        assert_eq!(result.diagnostics.len(), 0, "got {:?}", result.diagnostics);
    }

    /// MUST STILL FIRE: a genuine redirection with no command is still a bug.
    #[test]
    fn still_fires_on_a_real_bare_redirection() {
        let masked = crate::linter::quoting::mask_literals("echo start\n> out.txt\necho done\n");
        let result = check(&masked);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
        assert_eq!(result.diagnostics[0].code, "SC2188");
    }

    /// MUST STILL FIRE: the redirection operator itself is outside the quotes,
    /// so a quoted *target* does not hide it.
    #[test]
    fn still_fires_when_only_the_target_is_quoted() {
        let masked = crate::linter::quoting::mask_literals("echo start\n> \"$out\"\n");
        let result = check(&masked);
        assert_eq!(result.diagnostics.len(), 1, "got {:?}", result.diagnostics);
    }
}
