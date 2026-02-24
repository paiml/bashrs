// SC1083: This `{` or `}` is literal. Check expression (missing `;`?) or quote it
//
// Detects `{` or `}` used as literal arguments to commands like echo, printf,
// cat, etc. These braces are not performing brace expansion and may indicate
// a missing `;` in a command group or an unquoted literal.
//
// Examples:
// Bad:
//   echo {
//   echo }
//   echo hello }
//   printf "%s" {
//
// Good:
//   echo "{"
//   echo "{a,b}"          # brace expansion
//   { echo hello; }       # command group
//   echo ${var}           # parameter expansion
//
// Impact: Warning - likely a typo or missing syntax

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        check_literal_braces(trimmed, line_num, line, &mut result);
    }

    result
}

/// Commands where a lone brace argument is suspicious
const OUTPUT_COMMANDS: &[&str] = &["echo", "printf", "cat", "print"];

fn check_literal_braces(trimmed: &str, line_num: usize, line: &str, result: &mut LintResult) {
    for cmd in OUTPUT_COMMANDS {
        // Check for `echo {` or `echo }` patterns
        if let Some(rest) = trimmed.strip_prefix(cmd) {
            if !rest.starts_with(char::is_whitespace) {
                continue;
            }
            let args = rest.trim_start();
            check_args_for_literal_braces(args, cmd, line_num, line, result);
        }
    }
}

fn check_args_for_literal_braces(
    args: &str,
    cmd: &str,
    line_num: usize,
    line: &str,
    result: &mut LintResult,
) {
    // Split arguments on whitespace and check each
    for arg in args.split_whitespace() {
        // Skip if it's a brace expansion like {a,b} or {1..5}
        if arg.contains(',') || arg.contains("..") {
            continue;
        }
        // Skip if it's a parameter expansion like ${var}
        if arg.contains("${") {
            continue;
        }
        // Skip if the brace is quoted
        if arg.starts_with('"') || arg.starts_with('\'') {
            continue;
        }
        // Skip if it's a redirect
        if arg.starts_with('>') || arg.starts_with('<') {
            continue;
        }

        // Flag lone `{` or `}` as an argument
        if arg == "{" || arg == "}" {
            result.add(Diagnostic::new(
                "SC1083",
                Severity::Warning,
                format!(
                    "SC1083: This {} is literal in {}. Check expression (missing `;`?) or quote it",
                    arg, cmd
                ),
                Span::new(line_num, 1, line_num, line.len() + 1),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1083_echo_open_brace() {
        let code = "echo {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1083");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("{"));
    }

    #[test]
    fn test_sc1083_echo_close_brace() {
        let code = "echo }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("}"));
    }

    #[test]
    fn test_sc1083_printf_brace() {
        let code = "printf \"%s\" {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1083_brace_expansion_no_match() {
        // This is valid brace expansion
        let code = "echo {a,b,c}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_sequence_expansion_no_match() {
        let code = "echo {1..5}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_parameter_expansion_no_match() {
        let code = "echo ${var}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_quoted_brace_no_match() {
        let code = r#"echo "{""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_command_group_no_match() {
        // Command group syntax, not an argument to echo
        let code = "{ echo hello; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_comment_no_match() {
        let code = "# echo {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_empty_no_match() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1083_echo_with_brace_in_middle() {
        let code = "echo hello }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1083_cat_with_brace() {
        let code = "cat {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1083_non_output_command_no_match() {
        // grep is not in our output commands list
        let code = "grep {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
