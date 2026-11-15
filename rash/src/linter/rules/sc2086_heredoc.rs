//! SC2086 heredoc-body detection (issue #236): a heredoc body is data.
//!
//! Openers come from [`crate::linter::quoting::QuotedRegions`], which only
//! recognises `<<` outside quotes, comments and arithmetic. Rule (deliberately
//! simple, no shell parsing; every undecidable case resolves to "flag", which
//! is what main did for every heredoc line, so no new false negative):
//! - Quoted delimiter (`<<'EOF'`, `<<"EOF"`, `<<\EOF`): body is literal. Quiet.
//! - Unquoted delimiter: body lines are quiet until the first line that
//!   contains the text `$(` or a backtick (escaped or quoted ones included,
//!   `$((` included). That line and EVERY later line of the same body are
//!   flagged as on main. Nothing is tracked across lines: no quote, paren,
//!   arithmetic or case-pattern state, so none of those can cause a miss.
//! - Unterminated heredoc: `QuotedRegions` reports no body; nothing suppressed.
//!
//! Cost: lines after a substitution in the same body keep main's behaviour
//! (a possible false positive, never a false negative versus main).

use crate::linter::quoting;

/// For each line of `source` (0-indexed), true when SC2086 must not fire there.
pub fn heredoc_body_lines(source: &str) -> Vec<bool> {
    let body = quoting::heredoc_body_lines(source);
    let quoted = quoting::quoted_heredoc_lines(source);
    let mut seen_subst = false;
    source
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let n = i + 1;
            if !body.contains(&n) || quoted.contains(&n) {
                seen_subst = false;
                return body.contains(&n);
            }
            seen_subst |= line.contains("$(") || line.contains('`');
            !seen_subst
        })
        .collect()
}
