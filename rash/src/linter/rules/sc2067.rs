// SC2067: Missing $ on loop variables or assignment
//
// Variables used as indices or values need $ prefix to expand properly.
// Without $, they're treated as literal strings.
//
// Examples:
// Bad:
//   for i in 1 2 3; do
//     echo $array[i]            // i is literal, not expanded
//   done
//
// Good:
//   for i in 1 2 3; do
//     echo ${array[$i]}         // $i expands to value
//   done
//
// Impact: Wrong values accessed, logic errors

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static ARRAY_INDEX_WITHOUT_DOLLAR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: $array[var] or ${array[var]} where var has no $
    Regex::new(r"\$\{?[a-zA-Z_][a-zA-Z0-9_]*\[([a-zA-Z_][a-zA-Z0-9_]*)\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in ARRAY_INDEX_WITHOUT_DOLLAR.captures_iter(line) {
            let index_var = cap.get(1).unwrap().as_str();

            // Check if it's a number (numbers don't need $)
            if index_var.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            let full_match = cap.get(0).unwrap();
            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2067",
                Severity::Warning,
                format!(
                    "Missing $ on variable '{}'. Use ${{array[${}]}}",
                    index_var, index_var
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
    fn test_sc2067_array_index_no_dollar() {
        let code = r#"echo ${array[i]}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2067_simple_array() {
        let code = r#"value=$array[index]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2067_with_dollar_ok() {
        let code = r#"echo ${array[$i]}"#;
        let result = check(code);
        // Correct - has $ on index
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2067_numeric_index_ok() {
        let code = r#"echo ${array[0]}"#;
        let result = check(code);
        // Numeric index doesn't need $
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2067_comment_ok() {
        let code = r#"# echo ${array[i]}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2067_in_loop() {
        let code = r#"
for i in 0 1 2; do
    echo ${array[i]}
done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2067_nested_arrays() {
        let code = r#"echo ${outer[i]} ${inner[j]}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2067_mixed() {
        let code = r#"echo ${array[$i]} ${array[j]}"#;
        let result = check(code);
        // First is OK, second is wrong
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2067_assignment() {
        let code = r#"value=${data[key]}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2067_arithmetic_context() {
        let code = r#"result=$((array[i] + 1))"#;
        let result = check(code);
        // Arithmetic contexts have different rules
        assert_eq!(result.diagnostics.len(), 0);
    }
}
