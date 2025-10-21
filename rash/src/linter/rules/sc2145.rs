// SC2145: Argument mixin in arrays - unquoted $@ or $* in quotes.
//
// When using $@ or $* inside double quotes without proper quoting, array elements
// concatenate incorrectly. Use "$@" for separate arguments or "$*" for concatenation.
//
// Examples:
// Bad:
//   echo "Args: $@"              // Elements concatenate with spaces
//   echo "All: $*"                // Same issue, unclear intent
//   msg="Files: $@"               // Array elements join incorrectly
//
// Good:
//   echo "Args: $*"               // Explicit concatenation with IFS
//   printf '%s\n' "$@"            // Each arg separate
//   for arg in "$@"; do           // Proper iteration
//
// Impact: Incorrect argument handling and word splitting

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNQUOTED_AT_IN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: "...$@..." (unquoted $@ inside double quotes)
    // Look for $@ that's NOT immediately preceded by opening quote or space-quote
    Regex::new(r#""[^"]*\$@[^"]*""#).unwrap()
});

static UNQUOTED_STAR_IN_QUOTES: Lazy<Regex> = Lazy::new(|| {
    // Match: "...$*..." (unquoted $* inside double quotes)
    Regex::new(r#""[^"]*\$\*[^"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for unquoted $@ in double quotes
        for mat in UNQUOTED_AT_IN_QUOTES.find_iter(line) {
            let matched = mat.as_str();

            // Skip if it's properly quoted: "$@" (the entire quoted string is just "$@")
            if matched == r#""$@""# {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2145",
                Severity::Warning,
                "Argument mixin: Use \"$*\" for concatenation or \"$@\" as separate arguments"
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for unquoted $* in double quotes
        for mat in UNQUOTED_STAR_IN_QUOTES.find_iter(line) {
            let matched = mat.as_str();

            // Skip if it's properly quoted: "$*" (the entire quoted string is just "$*")
            if matched == r#""$*""# {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2145",
                Severity::Warning,
                "Argument mixin: Use \"$*\" for concatenation or \"$@\" as separate arguments"
                    .to_string(),
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
    fn test_sc2145_unquoted_at_in_quotes() {
        let code = r#"echo "Args: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Argument mixin"));
    }

    #[test]
    fn test_sc2145_unquoted_star_in_quotes() {
        let code = r#"echo "All: $*""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("Argument mixin"));
    }

    #[test]
    fn test_sc2145_quoted_at_ok() {
        let code = r#"printf '%s\n' "$@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_quoted_star_ok() {
        let code = r#"echo "$*""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_unquoted_at_ok() {
        let code = r#"for arg in $@; do"#;
        let result = check(code);
        // Unquoted outside of quotes is a different issue (SC2068)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_in_assignment() {
        let code = r#"msg="Files: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_comment_ok() {
        let code = r#"# echo "Args: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2145_multiple() {
        let code = r#"
echo "Args: $@"
msg="All: $*"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2145_at_beginning() {
        let code = r#"echo "$@ are the args""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2145_at_end() {
        let code = r#"echo "Arguments: $@""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
