// SC2049: =~ is for regex matching. Use = for string matching or quote regex.
//
// The =~ operator in [[ ]] performs regex matching, not literal string comparison.
// If you want literal string matching, use = or ==.
// If you want regex, don't quote the right-hand side.
//
// Examples:
// Bad:
//   [[ $var =~ "pattern" ]]   // Quoted regex is treated as literal (defeats purpose)
//   [[ $var =~ \.txt$ ]]      // Looks like regex but may not work as expected
//
// Good (regex):
//   [[ $var =~ pattern ]]     // Unquoted for regex matching
//   [[ $file =~ \.txt$ ]]     // Regex: ends with .txt
//   pat="\.txt$"; [[ $file =~ $pat ]]  // Regex in variable
//
// Good (literal):
//   [[ $var = "pattern" ]]    // Use = for literal string matching
//   [[ $var == "pattern" ]]   // Or == for literal matching
//
// Note: =~ only works in [[ ]], not in [ ]. Quote the right side only if
// you want literal matching (but then use = instead).

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static REGEX_MATCH_DOUBLE_QUOTED: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [[ ... =~ "..." ]]
    Regex::new(r#"=~\s*"([^"]+)""#).unwrap()
});

static REGEX_MATCH_SINGLE_QUOTED: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [[ ... =~ '...' ]]
    Regex::new(r"=~\s*'([^']+)'").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Only check [[ ]] tests (=~ only works there)
        if !line.contains("[[") {
            continue;
        }

        // Check for double-quoted patterns after =~
        for cap in REGEX_MATCH_DOUBLE_QUOTED.captures_iter(line) {
            let pattern = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2049",
                Severity::Warning,
                format!(
                    "=~ is for regex. Use = for literal string match, or unquote '{}' for regex",
                    pattern
                ),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for single-quoted patterns after =~
        for cap in REGEX_MATCH_SINGLE_QUOTED.captures_iter(line) {
            let pattern = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2049",
                Severity::Warning,
                format!(
                    "=~ is for regex. Use = for literal string match, or unquote '{}' for regex",
                    pattern
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
    fn test_sc2049_quoted_regex_double() {
        let code = r#"[[ $var =~ "pattern" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2049");
        assert!(result.diagnostics[0].message.contains("regex"));
    }

    #[test]
    fn test_sc2049_quoted_regex_single() {
        let code = r#"[[ $file =~ '\.txt$' ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2049_unquoted_regex_ok() {
        let code = r#"[[ $var =~ pattern ]]"#;
        let result = check(code);
        // Unquoted is correct for regex
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2049_unquoted_regex_dot_ok() {
        let code = r#"[[ $file =~ \.txt$ ]]"#;
        let result = check(code);
        // Unquoted regex, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2049_literal_match_ok() {
        let code = r#"[[ $var = "pattern" ]]"#;
        let result = check(code);
        // Using = for literal, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2049_literal_match_double_eq_ok() {
        let code = r#"[[ $var == "pattern" ]]"#;
        let result = check(code);
        // Using == for literal, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2049_regex_in_variable_ok() {
        let code = r"pat='\.txt$'; [[ $file =~ $pat ]]";
        let result = check(code);
        // Variable holds regex, unquoted, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2049_comment_ok() {
        let code = "# [[ $var =~ \"pattern\" ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2049_complex_pattern() {
        let code = "[[ $email =~ \"^[a-z]+@[a-z]+\\.com$\" ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2049_single_bracket_not_detected() {
        let code = "[ \"$var\" = \"pattern\" ]";
        let result = check(code);
        // =~ only works in [[]], this is [ ], so not detected
        assert_eq!(result.diagnostics.len(), 0);
    }
}
