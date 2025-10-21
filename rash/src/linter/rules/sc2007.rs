// SC2007: Use $((..)) instead of deprecated $[..]
//
// The $[...] syntax for arithmetic is deprecated and no longer recommended.
// Use the standard $((...)) syntax instead.
//
// Examples:
// Bad:
//   result=$[1 + 2]                // Deprecated syntax
//   count=$[$count + 1]            // Old style
//   sum=$[$a * $b]                 // Use $((...))
//
// Good:
//   result=$((1 + 2))              // Standard syntax
//   count=$((count + 1))           // Modern
//   sum=$((a * b))                 // Recommended
//
// Note: $[...] was deprecated long ago and removed from POSIX.
// Some shells may not support it at all.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DEPRECATED_ARITHMETIC: Lazy<Regex> = Lazy::new(|| {
    // Match: $[...] arithmetic syntax
    Regex::new(r"\$\[[^\]]+\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for $[...] syntax
        for m in DEPRECATED_ARITHMETIC.find_iter(line) {
            let matched = m.as_str();
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            // Extract the expression inside $[...]
            let expr = matched.trim_start_matches("$[").trim_end_matches(']');

            let diagnostic = Diagnostic::new(
                "SC2007",
                Severity::Warning,
                format!(
                    "Use $((..)) instead of deprecated $[..]. Replace '$[{}]' with '$(({}))'",
                    expr, expr
                ),
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
    fn test_sc2007_deprecated_syntax() {
        let code = r#"result=$[1 + 2]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2007");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("deprecated"));
    }

    #[test]
    fn test_sc2007_with_variable() {
        let code = r#"count=$[$count + 1]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2007_multiplication() {
        let code = r#"sum=$[$a * $b]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2007_complex_expression() {
        let code = r#"value=$[($a + $b) * ($c - $d)]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2007_modern_syntax_ok() {
        let code = r#"result=$((1 + 2))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2007_array_subscript_ok() {
        let code = r#"item=${array[0]}"#;
        let result = check(code);
        // ${array[0]} is not arithmetic, it's array subscript
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2007_test_bracket_ok() {
        let code = r#"[ $x -eq 5 ]"#;
        let result = check(code);
        // [ ] is test command, not arithmetic
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2007_multiple_deprecated() {
        let code = r#"
a=$[1 + 1]
b=$[2 * 2]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2007_in_command_subst() {
        let code = r#"echo "Result: $[5 + 3]""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2007_nested_parens() {
        let code = r#"result=$[(x + y) * 2]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
