// SC2133: Unexpected tokens in arithmetic expansion
//
// Arithmetic expansions $(( )) should contain valid arithmetic expressions.
// Unexpected tokens like unbalanced parentheses or quotes cause errors.
//
// Examples:
// Bad:
//   echo $((foo))           // Variables should use $ inside $(())
//   result=$((5 +))          // Incomplete expression
//   value=$((x y))           // Missing operator
//
// Good:
//   echo $(($foo))           // $ prefix for variables
//   result=$((5 + 3))        // Complete expression
//   value=$((x + y))         // Proper operator
//
// Impact: Syntax error, script will fail

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static ARITH_EXPR: Lazy<Regex> = Lazy::new(|| {
    // Match: $(( ... )) arithmetic expressions
    Regex::new(r"\$\(\(([^)]+)\)\)").unwrap()
});

static VAR_NAME: Lazy<Regex> = Lazy::new(|| {
    // Match: word boundaries (variable names)
    Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap()
});

static INCOMPLETE_ARITH: Lazy<Regex> = Lazy::new(|| {
    // Match: $(( expr operator )) with no second operand
    Regex::new(r"\$\(\([^)]*[+\-*/]\s*\)\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for variables without $ in arithmetic expressions
        for arith_mat in ARITH_EXPR.find_iter(line) {
            let arith_content = &line[arith_mat.start()..arith_mat.end()];

            // Find all potential variable names in the arithmetic expression
            for var_cap in VAR_NAME.captures_iter(arith_content) {
                if let Some(var_match) = var_cap.get(1) {
                    let var_name = var_match.as_str();
                    let var_pos = var_match.start();

                    // Check if preceded by $ (i.e., check character before if exists)
                    if var_pos > 0 {
                        let prev_char = arith_content.chars().nth(var_pos - 1);
                        if prev_char == Some('$') {
                            continue; // Has $, skip
                        }
                    }

                    // Skip if it's a pure number
                    if var_name.chars().all(|c| c.is_ascii_digit()) {
                        continue;
                    }

                    let abs_start = arith_mat.start() + var_pos;
                    let abs_end = abs_start + var_name.len();

                    let diagnostic = Diagnostic::new(
                        "SC2133",
                        Severity::Error,
                        format!(
                            "Use ${} instead of {} in arithmetic. Variables need $ prefix",
                            var_name, var_name
                        ),
                        Span::new(line_num, abs_start + 1, line_num, abs_end + 1),
                    );

                    result.add(diagnostic);
                }
            }
        }

        // Check for incomplete arithmetic expressions
        for mat in INCOMPLETE_ARITH.find_iter(line) {
            let expr = mat.as_str();

            // Check if operator is at the end (incomplete)
            if expr
                .trim_end_matches(')')
                .trim()
                .ends_with(&['+', '-', '*', '/'])
            {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2133",
                    Severity::Error,
                    "Incomplete arithmetic expression - missing operand after operator".to_string(),
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
    fn test_sc2133_variable_without_dollar() {
        let code = "echo $((foo))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Use $foo"));
    }

    #[test]
    fn test_sc2133_variable_with_dollar_ok() {
        let code = "echo $(($foo))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_incomplete_addition() {
        let code = "result=$((5 +))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Incomplete"));
    }

    #[test]
    fn test_sc2133_complete_expression_ok() {
        let code = "result=$((5 + 3))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_number_literal_ok() {
        let code = "echo $((42))";
        let result = check(code);
        // Numbers without $ are OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_comment_ok() {
        let code = "# echo $((foo))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_multiple_variables() {
        let code = "value=$((x + y))";
        let result = check(code);
        // Both x and y need $
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2133_incomplete_subtraction() {
        let code = "val=$((10 -))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2133_mixed_ok_and_bad() {
        let code = "result=$(($a + b))";
        let result = check(code);
        // $a is OK, b needs $
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2133_multiline() {
        let code = r#"
x=$((foo))
y=$((5 +))
"#;
        let result = check(code);
        // One for 'foo', one for incomplete '5 +'
        assert_eq!(result.diagnostics.len(), 2);
    }
}
