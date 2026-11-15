//! SC2086 heredoc-body detection (issue #236): a heredoc body is data.
//!
//! Heredoc openers are NOT scanned for here. They come from
//! [`crate::linter::quoting::QuotedRegions`], which only recognises `<<` outside
//! quotes, comments and arithmetic (`$(( 1 << 8 ))`), handles `<<-` with tab
//! terminators, quoted/escaped delimiters, here-strings and several heredocs
//! per line.
//!
//! Semantics:
//! - Quoted delimiter (`<<'EOF'`, `<<"EOF"`, `<<\EOF`): the body is literal,
//!   nothing expands, so there is nothing to flag. Quiet.
//! - Unquoted delimiter: expansions happen but word splitting does not apply
//!   inside the body, so SC2086 would be a false positive. Quiet, except on a
//!   line holding a command substitution (`$(...)` or a backtick), whose
//!   contents are shell code. `$(( ))` arithmetic is not a command
//!   substitution. A `$(` that stays open past its line keeps the following
//!   lines code until its `)` closes it.
//! - Unterminated heredoc: if the terminator never appears before EOF, the
//!   "body" was a guess and `QuotedRegions` reports no body at all, so nothing
//!   after the opener is suppressed.

use crate::linter::quoting;

fn has_command_substitution(line: &str) -> bool {
    let bytes = line.as_bytes();
    line.contains('`')
        || line
            .match_indices("$(")
            .any(|(i, _)| bytes.get(i + 2) != Some(&b'('))
}

/// Net `$(` nesting after `line`, starting from `depth`. `$((` is arithmetic
/// and a bare `(` inside a substitution is a subshell, so both are counted.
fn subst_depth(line: &str, mut depth: usize) -> usize {
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let dollar_paren = b == b'(' && i > 0 && bytes[i - 1] == b'$';
        if dollar_paren || (b == b'(' && depth > 0) {
            depth += 1;
        } else if b == b')' && depth > 0 {
            depth -= 1;
        }
    }
    depth
}

/// For each line of `source` (0-indexed), true when SC2086 must not fire there.
pub fn heredoc_body_lines(source: &str) -> Vec<bool> {
    let body = quoting::heredoc_body_lines(source);
    let quoted = quoting::quoted_heredoc_lines(source);
    let mut depth = 0;
    source
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let n = i + 1;
            if !body.contains(&n) || quoted.contains(&n) {
                depth = 0;
                return body.contains(&n);
            }
            let open = depth > 0;
            depth = subst_depth(line, depth);
            !(open || has_command_substitution(line))
        })
        .collect()
}
