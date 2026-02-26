// SC2078: This expression is constant; did you forget the $ on a variable?
//
// In test commands, using a bare word instead of a variable reference
// results in a constant expression that always evaluates the same way.
//
// Examples:
// Bad:
//   if [ count -gt 5 ]; then   // "count" is literal string, not variable
//   if [ x -eq 10 ]; then      // "x" is literal, always false
//
// Good:
//   if [ $count -gt 5 ]; then  // $count expands to value
//   if [ "$x" -eq 10 ]; then   // Proper variable reference
//
// Impact: Logic always takes same branch, dead code

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static CONSTANT_IN_TEST: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ word -op number ] or [ ! word -op number ] where word has no $ (likely forgot $)
    // !?\s* handles optional negation (! with optional space)
    Regex::new(r"\[\s+!?\s*([a-zA-Z_][a-zA-Z0-9_]*)\s+(-eq|-ne|-lt|-le|-gt|-ge)\s+[0-9]+\s+\]")
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in CONSTANT_IN_TEST.captures_iter(line) {
            let word = cap.get(1).unwrap().as_str();

            // Skip common constants/commands
            if matches!(
                word,
                "true" | "false" | "yes" | "no" | "on" | "off" | "RANDOM"
            ) {
                continue;
            }

            let full_match = cap.get(0).unwrap();
            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2078",
                Severity::Warning,
                format!(
                    "This expression is constant. Did you forget the $ on '{}'?",
                    word
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
    fn test_sc2078_forgot_dollar() {
        let code = r#"if [ count -gt 5 ]; then echo "yes"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2078_variable_x() {
        let code = r#"[ x -eq 10 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2078_with_dollar_ok() {
        let code = r#"if [ $count -gt 5 ]; then echo "yes"; fi"#;
        let result = check(code);
        // Correct - has $ on variable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2078_quoted_ok() {
        let code = r#"[ "$count" -gt 5 ]"#;
        let result = check(code);
        // Quoted variable is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2078_random_ok() {
        let code = r#"[ RANDOM -gt 100 ]"#;
        let result = check(code);
        // RANDOM is special variable
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2078_comment_ok() {
        let code = r#"# [ count -gt 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2078_all_operators() {
        let code = r#"
[ x -eq 1 ]
[ y -ne 2 ]
[ z -lt 3 ]
[ a -le 4 ]
[ b -gt 5 ]
[ c -ge 6 ]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 6);
    }

    #[test]
    fn test_sc2078_double_bracket() {
        let code = r#"[[ count -gt 5 ]]"#;
        let result = check(code);
        // Regex pattern handles [[ ]] correctly
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2078_string_comparison() {
        let code = r#"[ name = "test" ]"#;
        let result = check(code);
        // String comparison, not numeric - different rule
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2078_negation() {
        let code = r#"[ ! count -gt 5 ]"#;
        let result = check(code);
        // Regex now handles negation (!) operator
        assert_eq!(result.diagnostics.len(), 1);
    }
}
