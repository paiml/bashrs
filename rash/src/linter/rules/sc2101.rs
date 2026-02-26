// SC2101: Named class needs outer [], e.g. [[:digit:]]
//
// POSIX character classes like [:digit:] need to be inside brackets.
// [[:digit:]] is correct, [:digit:] alone is not valid.
//
// Examples:
// Bad:
//   [[ $var =~ [:digit:] ]]      // Missing outer []
//   case $x in [:alpha:]*) ;;    // Incorrect syntax
//
// Good:
//   [[ $var =~ [[:digit:]] ]]    // Correct nesting
//   case $x in [[:alpha:]]*) ;;  // Proper syntax
//
// Impact: Pattern doesn't work as intended, matches wrong characters

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static POSIX_CLASS_NO_OUTER_BRACKET: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [:class:] without outer [] in patterns
    Regex::new(r"\[:[a-z]+:\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in POSIX_CLASS_NO_OUTER_BRACKET.find_iter(line) {
            let matched = mat.as_str();

            // Check if it's actually inside outer brackets already
            let pos = mat.start();
            if pos > 0 && line.chars().nth(pos - 1) == Some('[') {
                // Check if followed by ]
                let end_pos = mat.end();
                if end_pos < line.len() && line.chars().nth(end_pos) == Some(']') {
                    continue; // It's [[:class:]], correct syntax
                }
            }

            let start_col = pos + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2101",
                Severity::Warning,
                format!("Named class needs outer [], e.g. [{}]", matched),
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
    fn test_sc2101_digit_no_outer() {
        let code = "[[ $var =~ [:digit:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2101_alpha_no_outer() {
        let code = "case $x in [:alpha:]*) ;;";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2101_correct_nesting_ok() {
        let code = "[[ $var =~ [[:digit:]] ]]";
        let result = check(code);
        // Correct syntax
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2101_case_correct_ok() {
        let code = "case $x in [[:alpha:]]*) ;;";
        let result = check(code);
        // Correct syntax
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2101_comment_ok() {
        let code = "# [[ $var =~ [:digit:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2101_lower() {
        let code = "[[ $var =~ [:lower:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2101_upper() {
        let code = "[[ $var =~ [:upper:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2101_alnum() {
        let code = "[[ $var =~ [:alnum:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2101_space() {
        let code = "[[ $var =~ [:space:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2101_multiple() {
        let code = "[[ $a =~ [:digit:] ]] && [[ $b =~ [:alpha:] ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
