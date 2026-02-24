// SC1028: Parentheses in `[ ]` need escaping
//
// In single-bracket test expressions, parentheses must be escaped with `\`
// or they will be interpreted as subshell syntax.
//
// Examples:
// Bad:
//   [ (expr) ]
//   [ ( -f file ) ]
//
// Good:
//   [ \( expr \) ]
//   [ \( -f file \) ]
//   [[ (expr) ]]   # double brackets handle parens natively

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Find bare `(` or `)` characters that are NOT part of `$(...)` command
/// substitution or `\(` / `\)` escaped parens.
/// Returns byte offsets of each bare paren.
fn find_bare_parens(line: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    let mut results = Vec::new();
    let mut cmd_sub_depth: u32 = 0;

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                // Skip escaped character entirely
                i += 2;
                continue;
            }
            b'$' if i + 1 < bytes.len() && bytes[i + 1] == b'(' => {
                // Start of $(...) command substitution
                cmd_sub_depth += 1;
                i += 2;
                continue;
            }
            b'(' if cmd_sub_depth == 0 => {
                results.push(i);
            }
            b')' if cmd_sub_depth > 0 => {
                cmd_sub_depth -= 1;
            }
            b')' => {
                results.push(i);
            }
            _ => {}
        }
        i += 1;
    }
    results
}

/// Check if a line contains a single-bracket test `[ ... ]` (not `[[ ... ]]`).
fn has_single_bracket_test(line: &str) -> bool {
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'[' {
            // Check next char is space (single bracket test)
            if i + 1 < bytes.len() && bytes[i + 1] == b' ' {
                // But NOT `[[` (double bracket)
                if i > 0 && bytes[i - 1] == b'[' {
                    continue;
                }
                if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                    continue;
                }
                return true;
            }
        }
    }
    false
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines with [[ (double bracket handles parens fine)
        if line.contains("[[") {
            continue;
        }

        // Only check lines that contain a single-bracket test
        if !has_single_bracket_test(line) {
            continue;
        }

        // Find bare parentheses (not $( or \() within the line
        for col in find_bare_parens(line) {
            let start_col = col + 1;
            let end_col = col + 2;

            let diagnostic = Diagnostic::new(
                "SC1028",
                Severity::Error,
                "Parentheses inside `[ ]` need escaping: use `\\(` and `\\)`".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1028_unescaped_paren() {
        let code = "[ (expr) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2); // ( and )
        assert_eq!(result.diagnostics[0].code, "SC1028");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1028_unescaped_paren_with_file_test() {
        let code = "[ ( -f file ) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2); // ( and )
    }

    #[test]
    fn test_sc1028_escaped_paren_ok() {
        let code = r"[ \( -f file \) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_double_bracket_ok() {
        let code = "[[ ( -f file ) ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_comment_ok() {
        let code = "# [ (expr) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_command_substitution_ok() {
        // $( ) inside [ ] should NOT trigger — it's command substitution, not grouping
        let code = "[ -n \"$(echo hello)\" ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_no_bracket_test() {
        let code = "echo (hello)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
