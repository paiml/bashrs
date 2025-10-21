// SC2004: $/${} is unnecessary on arithmetic variables
//
// Inside $((...)) arithmetic expressions, you don't need $ to reference variables.
// The shell automatically treats them as variable names.
//
// Examples:
// Bad:
//   result=$(($x + $y))            // Unnecessary $
//   count=$(($count + 1))          // Unnecessary $
//   sum=$((${a} + ${b}))           // Unnecessary ${}
//
// Good:
//   result=$((x + y))              // Clean, idiomatic
//   count=$((count + 1))           // Simple
//   sum=$((a + b))                 // No braces needed
//
// Note: Using $ inside $((...)) still works, but is considered poor style
// and can sometimes cause unexpected behavior with special variables.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITHMETIC_EXPR: Lazy<Regex> = Lazy::new(|| {
    // Match: $(( ... ))
    Regex::new(r"\$\(\(([^)]+)\)\)").unwrap()
});

static VAR_REF: Lazy<Regex> = Lazy::new(|| {
    // Match: $var or ${var} inside arithmetic
    Regex::new(r"\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find all $((...)) arithmetic expressions
        for arith_match in ARITHMETIC_EXPR.find_iter(line) {
            let arith_text = arith_match.as_str();
            let arith_start = arith_match.start();

            // Find all $var or ${var} within this arithmetic expression
            for var_match in VAR_REF.find_iter(arith_text) {
                let var_ref = var_match.as_str();
                let var_pos_in_arith = var_match.start();

                let start_col = arith_start + var_pos_in_arith + 1;
                let end_col = start_col + var_ref.len();

                let var_name = var_ref
                    .trim_start_matches('$')
                    .trim_start_matches('{')
                    .trim_end_matches('}');
                let diagnostic = Diagnostic::new(
                    "SC2004",
                    Severity::Info,
                    format!(
                        "$/${{}} is unnecessary on arithmetic variables. Use '{}' instead of '{}'",
                        var_name, var_ref
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2004_dollar_in_arithmetic() {
        let code = r#"result=$(($x + $y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2); // Both $x and $y
        assert_eq!(result.diagnostics[0].code, "SC2004");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_sc2004_braces_in_arithmetic() {
        let code = r#"sum=$((${a} + ${b}))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2004_increment() {
        let code = r#"count=$(($count + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2004_clean_arithmetic_ok() {
        let code = r#"result=$((x + y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2004_clean_increment_ok() {
        let code = r#"count=$((count + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2004_mixed() {
        let code = r#"result=$((x + $y))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1); // Only $y
    }

    #[test]
    fn test_sc2004_multiple_expressions() {
        let code = r#"
a=$(($x + 1))
b=$(($y - 1))
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2004_complex_expression() {
        let code = r#"result=$(($a * $b / $c))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc2004_outside_arithmetic_ok() {
        let code = r#"echo $x $y"#;
        let result = check(code);
        // Not in arithmetic context
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2004_let_command_ok() {
        let code = r#"let x=$x+1"#;
        let result = check(code);
        // let command is different context
        assert_eq!(result.diagnostics.len(), 0);
    }
}
