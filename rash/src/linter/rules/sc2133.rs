// SC2133: Incomplete arithmetic expressions
//
// Arithmetic expansions $(( )) should contain complete expressions.
// Operators must have both operands.
//
// NOTE: In bash arithmetic context, variables can be used with or without $:
// Both $((foo)) and $(($foo)) are valid and equivalent.
//
// Examples:
// Bad:
//   result=$((5 +))          // Incomplete expression - missing second operand
//   value=$((x * ))          // Incomplete expression - missing second operand
//
// Good:
//   echo $((foo))            // Variable without $ - VALID in arithmetic
//   echo $(($foo))           // Variable with $ - also VALID
//   result=$((5 + 3))        // Complete expression
//   value=$((x + y))         // Proper operator (both forms valid)
//
// Impact: Incomplete expressions cause syntax errors

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARITH_EXPR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: $(( ... )) arithmetic expressions
    Regex::new(r"\$\(\(([^)]+)\)\)").unwrap()
});

static VAR_NAME: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: word boundaries (variable names)
    Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap()
});

static INCOMPLETE_ARITH: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
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

        // NOTE: Variables in arithmetic context can be used with or without $
        // Both $((foo)) and $(($foo)) are valid bash syntax
        // We ONLY check for incomplete expressions (operators without operands)

        // Check for incomplete arithmetic expressions
        for mat in INCOMPLETE_ARITH.find_iter(line) {
            let expr = mat.as_str();

            // Check if operator is at the end (incomplete)
            if expr
                .trim_end_matches(')')
                .trim()
                .ends_with(['+', '-', '*', '/'])
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
    fn test_sc2133_variable_without_dollar_ok() {
        // Variables without $ are VALID in arithmetic context
        let code = "echo $((foo))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_variable_with_dollar_ok() {
        // Variables with $ are also VALID in arithmetic context
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
    fn test_sc2133_multiple_variables_ok() {
        // Variables in arithmetic don't need $
        let code = "value=$((x + y))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_incomplete_subtraction() {
        let code = "val=$((10 -))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2133_mixed_ok() {
        // Both forms are OK in arithmetic
        let code = "result=$(($a + b))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2133_multiline() {
        let code = r#"
x=$((foo))
y=$((5 +))
"#;
        let result = check(code);
        // Only the incomplete expression should be flagged
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Incomplete"));
    }
}
