// SC2018: Use '[:lower:]' to support accents and foreign alphabets
//
// The range [a-z] only matches ASCII lowercase letters. To match lowercase
// letters including accented characters and foreign alphabets, use [:lower:].
//
// Examples:
// Bad:
//   tr [a-z] [A-Z]                  // Only ASCII
//   [[ $var =~ [a-z] ]]             // Only ASCII
//   grep '[a-z]' file               // Only ASCII
//
// Good:
//   tr '[:lower:]' '[:upper:]'      // Supports accents
//   [[ $var =~ [[:lower:]] ]]       // Supports accents
//   grep '[[:lower:]]' file         // Supports accents
//
// Note: [:lower:] includes accented characters like é, ñ, etc. and works
// correctly with UTF-8 locales.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LOWERCASE_RANGE: Lazy<Regex> = Lazy::new(|| {
    // Match: [a-z] but not [[:lower:]]
    Regex::new(r"\[a-z\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for [a-z] pattern
        for m in LOWERCASE_RANGE.find_iter(line) {
            // Check if this is already part of [:lower:]
            let pos = m.start();
            let before = if pos > 0 { &line[..pos] } else { "" };
            let after = &line[m.end()..];

            // Skip if it's part of [[:lower:]]
            if before.ends_with("[[:") && after.starts_with(":]]") {
                continue;
            }
            if before.ends_with("[:") && after.starts_with(":]") {
                continue;
            }

            let start_col = pos + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2018",
                Severity::Info,
                "Use '[:lower:]' to support accents and foreign alphabets".to_string(),
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
    fn test_sc2018_az_range() {
        let code = r#"tr [a-z] [A-Z]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2018");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("[:lower:]"));
    }

    #[test]
    fn test_sc2018_in_regex() {
        let code = r#"[[ $var =~ [a-z] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2018_in_grep() {
        let code = r#"grep '[a-z]' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2018_in_case() {
        let code = r#"case $var in [a-z]) echo "lower";; esac"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2018_posix_class_ok() {
        let code = r#"tr '[:lower:]' '[:upper:]'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2018_bracket_posix_ok() {
        let code = r#"[[ $var =~ [[:lower:]] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2018_other_range_ignored() {
        let code = r#"[[ $var =~ [0-9] ]]"#;
        let result = check(code);
        // Only checking for [a-z], not [0-9]
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2018_multiple_issues() {
        let code = r#"
tr [a-z] [A-Z]
grep '[a-z]' file
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2018_in_sed() {
        let code = r#"sed 's/[a-z]/X/g' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2018_no_bracket_ok() {
        let code = r#"echo "a-z""#;
        let result = check(code);
        // Not a character class
        assert_eq!(result.diagnostics.len(), 0);
    }
}
