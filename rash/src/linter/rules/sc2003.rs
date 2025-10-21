// SC2003: expr is antiquated. Consider rewriting this using $((..))
//
// The expr command is a deprecated external command for arithmetic. Modern
// shells support $((...)) syntax which is faster, safer, and more portable.
//
// Examples:
// Bad:
//   result=$(expr 1 + 2)           // External command, slow
//   count=$(expr $count + 1)       // External command
//   value=`expr 5 \* 3`            // Old backticks, needs escaping
//
// Good:
//   result=$((1 + 2))              // Built-in, fast
//   count=$((count + 1))           // No $ needed inside
//   value=$((5 * 3))               // No escaping needed
//
// Note: expr is an external command that forks a process, making it much
// slower than built-in arithmetic. It also requires careful quoting and escaping.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXPR_USAGE: Lazy<Regex> = Lazy::new(|| {
    // Match: $(expr ...) or `expr ...`
    Regex::new(r"(\$\(expr\s+|`expr\s+)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for expr command usage
        for m in EXPR_USAGE.find_iter(line) {
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2003",
                Severity::Info,
                "expr is antiquated. Consider rewriting this using $((..)), ${} or [[ ]]".to_string(),
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
    fn test_sc2003_expr_in_command_subst() {
        let code = r#"result=$(expr 1 + 2)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2003");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("expr"));
    }

    #[test]
    fn test_sc2003_expr_with_variable() {
        let code = r#"count=$(expr $count + 1)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2003_expr_in_backticks() {
        let code = r#"value=`expr 5 \* 3`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2003_expr_multiplication() {
        let code = r#"result=$(expr $a \* $b)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2003_arithmetic_expansion_ok() {
        let code = r#"result=$((1 + 2))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2003_let_command_ok() {
        let code = r#"let count=count+1"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2003_double_bracket_ok() {
        let code = r#"[[ $count -gt 5 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2003_multiple_expr() {
        let code = r#"
a=$(expr 1 + 1)
b=$(expr 2 + 2)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2003_expr_in_comment_ok() {
        let code = r#"# Use $(expr 1 + 1) for old shells"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2003_expr_string_match() {
        let code = r#"len=$(expr "$str" : '.*')"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
