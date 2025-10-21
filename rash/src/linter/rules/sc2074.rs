// SC2074: Can't use =~ in [ ]. Use [[ ]] instead
//
// The =~ regex matching operator only works in [[ ]], not in [ ].
// Using it in [ ] will result in a syntax error or literal string comparison.
//
// Examples:
// Bad:
//   [ "$var" =~ pattern ]      // Syntax error or wrong behavior
//   if [ "$x" =~ [0-9]+ ]; then  // Won't work
//
// Good:
//   [[ "$var" =~ pattern ]]    // Regex matching works
//   [[ "$x" =~ [0-9]+ ]]       // Correct regex test
//
// Impact: Syntax errors, incorrect pattern matching

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX_IN_SINGLE_BRACKET: Lazy<Regex> = Lazy::new(|| {
    // Match: [ ... =~ ... ] but not [[ ... =~ ... ]]
    // Use negative lookbehind would be ideal but Rust regex doesn't support it
    // So we'll check in the code instead
    Regex::new(r"\[\s+[^\]]*=~[^\]]*\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for =~ in single brackets
        for mat in REGEX_IN_SINGLE_BRACKET.find_iter(line) {
            // Check if this is inside [[ ]] by looking at the character before [
            let pos = mat.start();
            let is_double_bracket = if pos > 0 {
                line.chars().nth(pos - 1) == Some('[')
            } else {
                false
            };

            // Also check if followed by ]
            let end_pos = mat.end();
            let followed_by_bracket = if end_pos < line.len() {
                line.chars().nth(end_pos) == Some(']')
            } else {
                false
            };

            // Skip if this is part of [[ ]]
            if is_double_bracket || followed_by_bracket {
                continue;
            }

            let start_col = pos + 1;
            let end_col = end_pos + 1;

            let diagnostic = Diagnostic::new(
                "SC2074",
                Severity::Error,
                "Can't use =~ in [ ]. Use [[ ]] for regex matching".to_string(),
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
    fn test_sc2074_regex_in_single_bracket() {
        let code = r#"[ "$var" =~ pattern ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2074_regex_with_class() {
        let code = r#"if [ "$x" =~ [0-9]+ ]; then echo "match"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2074_double_bracket_ok() {
        let code = r#"[[ "$var" =~ pattern ]]"#;
        let result = check(code);
        // [[ ]] is correct for =~
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2074_double_bracket_regex_ok() {
        let code = r#"if [[ "$x" =~ [0-9]+ ]]; then echo "match"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2074_equality_ok() {
        let code = r#"[ "$var" = "value" ]"#;
        let result = check(code);
        // Regular equality in [ ] is fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2074_comment_ok() {
        let code = r#"# [ "$var" =~ pattern ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2074_with_variable() {
        let code = r#"[ "$name" =~ $pattern ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2074_negation() {
        let code = r#"[ ! "$var" =~ pattern ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2074_complex_pattern() {
        let code = r#"[ "$email" =~ ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$ ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2074_test_command() {
        let code = r#"test "$var" =~ pattern"#;
        let result = check(code);
        // Using 'test' command, not [ ]
        assert_eq!(result.diagnostics.len(), 0);
    }
}
