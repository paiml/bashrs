//! SC2086 heredoc-body detection (issue #236): a heredoc body is data.
//!
//! Openers come from [`crate::linter::quoting::QuotedRegions`], which only
//! recognises `<<` outside quotes, comments and arithmetic, and handles `<<-`,
//! quoted delimiters, here-strings and several heredocs per line.
//! Semantics:
//! - Quoted delimiter (`<<'EOF'`, `<<"EOF"`, `<<\EOF`): the body is literal,
//!   nothing expands, so there is nothing to flag. Quiet.
//! - Unquoted delimiter: expansions happen but word splitting does not apply
//!   inside the body, so SC2086 would be a false positive. Quiet, except on a
//!   line holding a command substitution (`$(...)` or a backtick), whose
//!   contents are shell code (not `$(( ))` arithmetic). A substitution that
//!   spans lines keeps the following lines code until it closes.
//! - Unterminated heredoc: if the terminator never appears before EOF, the
//!   "body" was a guess and `QuotedRegions` reports no body at all, so nothing
//!   after the opener is suppressed.

use crate::linter::quoting;

/// Advance `stack` (`(` subst/subshell, `A` arithmetic, backtick) over `line`;
/// true when the line is inside or opens a command substitution outside
/// arithmetic. \$( and \` are escapes; quoted parens are ignored. Limit: quotes
/// are per line, and a case-pattern `a)` closes a multi-line `$(...)` early.
fn scan_line(line: &str, stack: &mut Vec<u8>) -> bool {
    let mut code = !stack.is_empty() && !stack.contains(&b'A');
    let (b, mut q, mut i) = (line.as_bytes(), 0u8, 0);
    while i < b.len() {
        let c = b[i];
        let n = |k: usize| b.get(i + k).copied().unwrap_or(0);
        if c == b'\\' {
            i += 1;
        } else if matches!(c, b'\'' | b'"') && !stack.is_empty() && (q == 0 || q == c) {
            q ^= c;
        } else if q == 0 {
            i += step(c, [n(1), n(2)], stack, &mut code);
        }
        i += 1;
    }
    code
}

fn push(st: &mut Vec<u8>, code: &mut bool, k: u8, skip: usize) -> usize {
    *code |= k != b'A' && !st.contains(&b'A');
    st.push(k);
    skip
}

fn pop(st: &mut Vec<u8>, skip: usize) -> usize {
    st.pop();
    skip
}

fn step(c: u8, n: [u8; 2], st: &mut Vec<u8>, code: &mut bool) -> usize {
    match c {
        b'`' if st.last() == Some(&c) => pop(st, 0),
        b'`' => push(st, code, c, 0),
        b'$' if n == [b'(', b'('] => push(st, code, b'A', 2),
        b'$' if n[0] == b'(' => push(st, code, b'(', 1),
        b'(' if !st.is_empty() => push(st, code, c, 0),
        b')' if st.last() == Some(&b'A') && n[0] == b')' => pop(st, 1),
        b')' => pop(st, 0),
        _ => 0,
    }
}

/// For each line of `source` (0-indexed), true when SC2086 must not fire there.
pub fn heredoc_body_lines(source: &str) -> Vec<bool> {
    let body = quoting::heredoc_body_lines(source);
    let quoted = quoting::quoted_heredoc_lines(source);
    let mut stack: Vec<u8> = Vec::new();
    source
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let n = i + 1;
            if !body.contains(&n) || quoted.contains(&n) {
                stack.clear();
                return body.contains(&n);
            }
            !scan_line(line, &mut stack)
        })
        .collect()
}
