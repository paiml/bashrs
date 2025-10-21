// SC2127: To compare constant values, use [[ ... ]] or (( ... ))
//
// Comparing two literal values in [ ] test is always constant.
// This is likely a mistake - either use variables or remove the test.
//
// Examples:
// Bad:
//   [ 1 -eq 1 ]            // Always true
//   [ "foo" = "foo" ]      // Always true
//   if [ 0 -lt 5 ]; then   // Constant comparison
//
// Good:
//   [ $var -eq 1 ]         // Variable comparison
//   [[ 1 -eq 1 ]]          // Intentional constant (syntax check)
//   (( 1 == 1 ))           // Arithmetic context
//
// Impact: Logic error, dead code

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static CONSTANT_COMPARISON: Lazy<Regex> = Lazy::new(|| {
    // Match: [ literal op literal ] where both sides are constants
    // Numbers or quoted strings without $
    Regex::new(
        r#"\[\s+([0-9]+|"[^$"]*")\s+(-eq|-ne|-lt|-le|-gt|-ge|=|!=)\s+([0-9]+|"[^$"]*")\s+\]"#,
    )
    .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] (double brackets are OK for constant checks)
        if line.contains("[[") {
            continue;
        }

        for mat in CONSTANT_COMPARISON.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2127",
                Severity::Warning,
                "Constant comparison in [ ] - this is always true or false. Use a variable or [[ ]]".to_string(),
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
    fn test_sc2127_constant_numbers() {
        let code = "[ 1 -eq 1 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2127_constant_strings() {
        let code = r#"[ "foo" = "foo" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2127_variable_ok() {
        let code = "[ $var -eq 1 ]";
        let result = check(code);
        // Variable comparison is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2127_double_bracket_ok() {
        let code = "[[ 1 -eq 1 ]]";
        let result = check(code);
        // [[ ]] allows constant checks for syntax
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2127_comment_ok() {
        let code = "# [ 1 -eq 1 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2127_in_if() {
        let code = "if [ 0 -lt 5 ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2127_not_equal() {
        let code = "[ 1 -ne 2 ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2127_string_inequality() {
        let code = r#"[ "a" != "b" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2127_with_variable_ok() {
        let code = r#"[ "$var" = "test" ]"#;
        let result = check(code);
        // Has $ so it's a variable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2127_multiline() {
        let code = r#"
if [ 5 -gt 3 ]; then
    echo "Always true"
fi
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
