// SC1037: Braces are required for positional parameters beyond $9
//
// In shell, $10 is interpreted as $1 followed by literal '0'. To access
// the 10th positional parameter and beyond, you must use ${10}, ${11}, etc.
//
// Examples:
// Bad:
//   echo $10          # Interpreted as $1 followed by '0'
//   echo $11          # Interpreted as $1 followed by '1'
//   echo $123         # Interpreted as $1 followed by '23'
//
// Good:
//   echo ${10}        # Correct: 10th positional parameter
//   echo ${11}        # Correct: 11th positional parameter
//   echo ${123}       # Correct: 123rd positional parameter

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches $10, $11, etc. that are NOT inside ${...}
/// We look for $ followed by a digit 1-9 then more digits, but NOT preceded by ${
static UNBRACED_POSITIONAL: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$([1-9][0-9]+)").expect("SC1037 regex must compile"));

/// Matches ${digits} to exclude braced forms
static BRACED_POSITIONAL: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"\$\{[1-9][0-9]+\}").expect("SC1037 braced regex must compile")
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        for mat in UNBRACED_POSITIONAL.find_iter(line) {
            // Check that this match is not inside ${...}
            // If the character before '$' is '{' preceded by '$', skip it
            let start = mat.start();
            if start >= 2
                && line.is_char_boundary(start - 2)
                && line.is_char_boundary(start)
                && &line[start - 2..start] == "${"
            {
                continue;
            }
            if start >= 1 && line.as_bytes()[start - 1] == b'{' {
                continue;
            }

            // Also check it's not part of a ${...} by looking at context
            if !line.is_char_boundary(start) {
                continue;
            }
            let before = &line[..start];
            let after_end = mat.end();
            if !line.is_char_boundary(after_end) {
                continue;
            }
            // If there's a matching ${ before this position, skip
            if is_inside_braced_expansion(before, &line[after_end..]) {
                continue;
            }

            let param = &mat.as_str()[1..]; // strip the $
            let start_col = start + 1;
            let end_col = mat.end() + 1;

            result.add(Diagnostic::new(
                "SC1037",
                Severity::Error,
                format!(
                    "Braces are required for positional parameters beyond $9. Use ${{{}}} instead of ${}.",
                    param, param
                ),
                Span::new(line_num, start_col, line_num, end_col),
            ));
        }
    }

    result
}

fn is_inside_braced_expansion(before: &str, _after: &str) -> bool {
    // Check if there's an unclosed ${ before our position
    let mut depth = 0i32;
    let bytes = before.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            depth += 1;
            i += 2;
            continue;
        }
        if bytes[i] == b'}' && depth > 0 {
            depth -= 1;
        }
        i += 1;
    }
    depth > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1037_unbraced_positional_10() {
        let code = "echo $10";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1037");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("${10}"));
    }

    #[test]
    fn test_sc1037_unbraced_positional_11() {
        let code = "echo $11";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1037_unbraced_positional_123() {
        let code = "echo $123";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("${123}"));
    }

    #[test]
    fn test_sc1037_braced_positional_ok() {
        let code = "echo ${10}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1037_braced_positional_11_ok() {
        let code = "echo ${11} ${12}";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1037_single_digit_ok() {
        let code = "echo $1 $2 $9";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1037_comment_ok() {
        let code = "# echo $10";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1037_multiple_violations() {
        let code = "echo $10 $20 $30";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 3);
    }
}
