//! SC1009: Comment detected where command was expected
//!
//! Detects cases where a comment appears immediately after a control
//! structure keyword (`then`, `do`, `else`, `{`) with no command between.
//! The comment is the only thing where a command body is expected.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if true; then
//!     # TODO: implement
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if true; then
//!     : # TODO: implement
//! fi
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Keywords after which a command is expected on the next line.
const CONTROL_KEYWORDS: &[&str] = &["then", "do", "else", "{"];

/// The first word of a trimmed line, with a single trailing `;` stripped.
///
/// `"fi;"`, `"fi"`, `"fi; exit 0"` all yield `"fi"`.
fn first_word(trimmed: &str) -> &str {
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches(';')
}

/// Does this line close the current block (or hand off to the next branch of
/// it), rather than being a command inside it?
///
/// `elif`/`else` count: if nothing but comments preceded them, THIS branch's
/// body genuinely had no command, which is exactly what SC1009 warns about.
fn is_block_closer(trimmed: &str) -> bool {
    trimmed.starts_with('}')
        || matches!(
            first_word(trimmed),
            "fi" | "done" | "esac" | "elif" | "else"
        )
}

/// Does the block body starting at `lines[from..]` contain a real command
/// before it closes (or before the source ends)?
///
/// Comments and blank lines are skipped; a bare `#238`: only when EVERY line
/// up to the close is a comment (or the body is empty) is there no command.
fn body_has_a_command(lines: &[&str], from: usize) -> bool {
    let mut idx = from;
    while idx < lines.len() {
        let t = lines[idx].trim();
        if t.is_empty() || t.starts_with('#') {
            idx += 1;
            continue;
        }
        return !is_block_closer(t);
    }
    false
}

/// Does `trimmed` end a control-flow keyword, i.e. does a command body
/// follow it on the next line?
fn ends_with_control_keyword(trimmed: &str) -> bool {
    CONTROL_KEYWORDS
        .iter()
        .any(|kw| trimmed == *kw || trimmed.ends_with(kw))
}

/// Index of the first non-blank line at or after `from`, if any.
fn next_non_blank(lines: &[&str], from: usize) -> Option<usize> {
    (from..lines.len()).find(|&idx| !lines[idx].trim().is_empty())
}

/// If the body opening at `lines[i + 1..]` is the GH-238 defect - it opens
/// with a comment AND contains no command at all before the block closes -
/// return the index of that leading comment line.
fn leading_comment_defect(lines: &[&str], i: usize) -> Option<usize> {
    let comment_idx = next_non_blank(lines, i + 1)?;
    if !lines[comment_idx].trim().starts_with('#') {
        return None;
    }
    if body_has_a_command(lines, comment_idx + 1) {
        return None;
    }
    Some(comment_idx)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() || !ends_with_control_keyword(trimmed) {
            continue;
        }

        // GH-238: a body that OPENS with a comment used to be reported
        // unconditionally - even when a real command followed it. Only a
        // body that contains NO command at all (just comments, or nothing)
        // before the block closes is what this diagnostic actually claims.
        let Some(comment_idx) = leading_comment_defect(&lines, i) else {
            continue;
        };

        let comment_trimmed = lines[comment_idx].trim();
        let line_num = comment_idx + 1;
        let col = lines[comment_idx].find('#').unwrap_or(0) + 1;
        result.add(Diagnostic::new(
            "SC1009",
            Severity::Warning,
            "Comment here is not a command. Use a no-op `:` if the body is empty",
            Span::new(line_num, col, line_num, col + comment_trimmed.len()),
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1009_comment_after_then() {
        let script = "if true; then\n    # todo\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1009");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1009_comment_after_do() {
        let script = "for i in 1 2 3; do\n    # process\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1009_comment_after_else() {
        let script = "if true; then\n    echo ok\nelse\n    # fallback\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1009_command_after_then_ok() {
        let script = "if true; then\n    echo hello\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1009_colon_after_then_ok() {
        let script = "if true; then\n    : # placeholder\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1009_comment_not_after_keyword() {
        let script = "echo hello\n# just a comment\necho world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1009_blank_lines_between() {
        let script = "if true; then\n\n    # todo\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

/// PMAT-251: end-to-end reproducers for GH-238, run through the public
/// `lint_shell` entry point (the diagnostic is renamed SC1009 -> BRS0001 by
/// `code_namespace`, which only `lint_shell`/`lint_shell_filtered` apply).
#[cfg(test)]
#[path = "sc1009_tests_gh238.rs"]
mod tests_gh238;
