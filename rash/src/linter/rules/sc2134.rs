// SC2134: Use arithmetic context (( )) for numeric tests instead of [ ]
//
// When doing numeric comparisons, (( )) is clearer and more portable
// than [ ] test syntax. It also supports C-style operators.
//
// Examples:
// Bad:
//   [ $x -gt 0 ]            // Old-style numeric test
//   [ "$count" -eq 1 ]      // Test syntax for numbers
//
// Good:
//   (( x > 0 ))             // Arithmetic context (clearer)
//   (( count == 1 ))        // C-style operators
//   [[ $x -gt 0 ]]          // If test syntax preferred
//
// Note: This is a style recommendation. [ ] works but (( )) is clearer.
// Impact: Style/readability improvement

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static NUMERIC_TEST_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    // Match: [ $var -gt/-lt/-ge/-le/-eq/-ne number ]
    Regex::new(r"\[\s+\$?\w+\s+(-gt|-lt|-ge|-le|-eq|-ne)\s+\d+\s+\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if using [[ ]] (double brackets are OK for portability)
        if line.contains("[[") {
            continue;
        }

        for mat in NUMERIC_TEST_BRACKETS.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2134",
                Severity::Info,
                "Consider using (( )) for numeric tests. Example: (( x > 0 )) instead of [ $x -gt 0 ]".to_string(),
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
    fn test_sc2134_numeric_test_gt() {
        let code = "[ $x -gt 0 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("(( ))"));
    }

    #[test]
    fn test_sc2134_numeric_test_eq() {
        let code = "[ $count -eq 1 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2134_double_bracket_ok() {
        let code = "[[ $x -gt 0 ]]";
        let result = check(code);
        // [[ ]] is acceptable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2134_arithmetic_context_ok() {
        let code = "(( x > 0 ))";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2134_comment_ok() {
        let code = "# [ $x -gt 0 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2134_test_lt() {
        let code = "[ $n -lt 10 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2134_test_ge() {
        let code = "[ $val -ge 5 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2134_test_le() {
        let code = "[ $x -le 100 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2134_test_ne() {
        let code = "[ $status -ne 0 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2134_multiline() {
        let code = r#"
if [ $x -gt 5 ]; then
    echo "greater"
fi
if [ $y -lt 10 ]; then
    echo "less"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
