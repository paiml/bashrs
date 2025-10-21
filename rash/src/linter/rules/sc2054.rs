// SC2054: Use spaces, not commas, to separate array elements (or quote if intentional)
//
// In bash, commas in [[ ]] are treated as literal characters, not separators.
// If you want to test multiple variables, use separate conditions or spaces.
//
// Examples:
// Bad:
//   [[ $a,$b == "1,2" ]]     // Tests if "$a,$b" equals "1,2" (literal comma)
//   [[ $x,$y -eq 5 ]]        // Syntax error or unexpected behavior
//
// Good:
//   [[ "$a $b" == "1 2" ]]   // Space-separated values
//   [[ $a == 1 && $b == 2 ]] // Separate conditions
//   [[ "$a,$b" == "1,2" ]]   // Quote if comma is intentional
//
// Impact: Logic errors, tests don't work as expected

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static COMMA_IN_TEST: Lazy<Regex> = Lazy::new(|| {
    // Match [[ with comma-separated vars ]]
    Regex::new(r"\[\[\s*[^\]]*\$[a-zA-Z_][a-zA-Z0-9_]*,").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Only check [[ ]] constructs
        if !line.contains("[[") {
            continue;
        }

        for mat in COMMA_IN_TEST.find_iter(line) {
            // Find the position of the $ in the match
            let matched_text = mat.as_str();
            if let Some(dollar_offset) = matched_text.find('$') {
                let dollar_pos = mat.start() + dollar_offset;
                let before_dollar = &line[..dollar_pos];

                // Count quotes before the $ - if odd, we're inside quotes
                let double_quote_count = before_dollar.matches('"').count();
                if double_quote_count % 2 == 1 {
                    continue; // Inside double quotes, skip
                }
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2054",
                Severity::Warning,
                "Use spaces, not commas, to separate array elements".to_string(),
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
    fn test_sc2054_comma_separated_vars() {
        let code = r#"[[ $a,$b == "1,2" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2054_multiple_commas() {
        let code = r#"[[ $x,$y,$z == "1,2,3" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2054_space_separated_ok() {
        let code = r#"[[ "$a $b" == "1 2" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2054_separate_conditions_ok() {
        let code = r#"[[ $a == 1 && $b == 2 ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2054_quoted_comma_ok() {
        let code = r#"[[ "$a,$b" == "1,2" ]]"#;
        let result = check(code);
        // Quoted, intentional comma
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2054_literal_comma_ok() {
        let code = r#"[[ $str == "value,with,commas" ]]"#;
        let result = check(code);
        // Comma in string literal only
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2054_comment_ok() {
        let code = r#"# [[ $a,$b == "1,2" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2054_single_bracket_ok() {
        let code = r#"[ "$a,$b" == "1,2" ]"#;
        let result = check(code);
        // Single bracket, not checked
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2054_if_statement() {
        let code = r#"if [[ $name,$value == "key,val" ]]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2054_no_variables_ok() {
        let code = r#"[[ "a,b" == "a,b" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
