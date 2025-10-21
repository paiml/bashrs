// SC2100: Use $(...) instead of deprecated expr arithmetic
//
// The expr command is deprecated for arithmetic. Use $((...)) or let instead.
//
// Examples:
// Bad:
//   result=`expr $a + $b`        // Deprecated expr
//   count=`expr $count + 1`      // Old style
//
// Good:
//   result=$((a + b))            // Modern arithmetic
//   count=$((count + 1))         // Cleaner
//   ((count++))                  // Even simpler
//
// Impact: Deprecated command, slower execution

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPR_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match: expr in command substitution (backticks or $())
    Regex::new(r"(`|\$\()expr\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in EXPR_COMMAND.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2100",
                Severity::Info,
                "Use $((...)) instead of deprecated expr command".to_string(),
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
    fn test_sc2100_expr_backticks() {
        let code = "result=`expr $a + $b`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2100_expr_dollar_paren() {
        let code = "result=$(expr $a + $b)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2100_arithmetic_ok() {
        let code = "result=$((a + b))";
        let result = check(code);
        // Modern arithmetic
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2100_expr_increment() {
        let code = "count=`expr $count + 1`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2100_let_ok() {
        let code = "let result=a+b";
        let result = check(code);
        // let is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2100_comment_ok() {
        let code = "# result=`expr $a + $b`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2100_expr_multiply() {
        let code = "result=$(expr $x \\* $y)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2100_expr_as_command_ok() {
        let code = "expr 1 + 1";
        let result = check(code);
        // Direct expr command (not in substitution)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2100_multiple() {
        let code = "a=`expr 1 + 1`; b=$(expr 2 + 2)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2100_in_test() {
        let code = "[ `expr $a + 1` -gt 10 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
