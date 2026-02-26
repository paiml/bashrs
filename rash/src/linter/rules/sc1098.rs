//! SC1098: Quote/escape special characters when using eval
//!
//! Detects `eval` with unquoted `$` variables (e.g., `eval $cmd`),
//! which can lead to word splitting and unexpected behavior.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for unquoted variables in eval statements
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Look for eval followed by unquoted variable
        // Pattern: eval $var or eval $var args
        if let Some(eval_pos) = find_eval_command(trimmed) {
            let after_eval = &trimmed[eval_pos..];
            check_eval_args(after_eval, line, line_num, &mut result);
        }
    }

    result
}

/// Find the position after `eval ` in a command
fn find_eval_command(line: &str) -> Option<usize> {
    // Match standalone eval command (not part of another word)
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if i == 0 || !bytes[i - 1].is_ascii_alphanumeric() {
            // Only index into the string at ASCII char boundaries
            if !line.is_char_boundary(i) {
                continue;
            }
            if line[i..].starts_with("eval ") || line[i..].starts_with("eval\t") {
                return Some(i + 5);
            }
        }
    }
    None
}

fn check_eval_args(args: &str, full_line: &str, line_num: usize, result: &mut LintResult) {
    let trimmed_args = args.trim_start();

    // Check for unquoted $variable (not inside quotes)
    if trimmed_args.starts_with('$') && !trimmed_args.starts_with("$(") {
        // It's an unquoted variable reference in eval
        let dollar_pos = full_line.find(trimmed_args).unwrap_or(0);
        let diagnostic = Diagnostic::new(
            "SC1098",
            Severity::Warning,
            "Quote/escape special characters when using eval, e.g., eval \"$cmd\"",
            Span::new(
                line_num,
                dollar_pos + 1,
                line_num,
                dollar_pos
                    + trimmed_args
                        .split_whitespace()
                        .next()
                        .map_or(1, |w| w.len())
                    + 1,
            ),
        );
        result.add(diagnostic);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1098_unquoted_eval_var() {
        let script = "eval $cmd";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1098");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1098_quoted_eval_var() {
        let script = r#"eval "$cmd""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1098_eval_literal_string() {
        let script = "eval echo hello";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1098_comment_skipped() {
        let script = "# eval $cmd";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1098_eval_with_braces() {
        let script = "eval ${command}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
