// SC2019: Use '[:upper:]' to support accents and foreign alphabets
//
// The range [A-Z] only matches ASCII uppercase letters. To match uppercase
// letters including accented characters and foreign alphabets, use [:upper:].
//
// Examples:
// Bad:
//   tr [A-Z] [a-z]                  // Only ASCII
//   [[ $var =~ [A-Z] ]]             // Only ASCII
//   grep '[A-Z]' file               // Only ASCII
//
// Good:
//   tr '[:upper:]' '[:lower:]'      // Supports accents
//   [[ $var =~ [[:upper:]] ]]       // Supports accents
//   grep '[[:upper:]]' file         // Supports accents
//
// Note: [:upper:] includes accented characters like É, Ñ, etc. and works
// correctly with UTF-8 locales.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UPPERCASE_RANGE: Lazy<Regex> = Lazy::new(|| {
    // Match: [A-Z] but not [[:upper:]]
    Regex::new(r"\[A-Z\]").unwrap()
});

/// Check if the match is already part of a POSIX character class like [:upper:]
fn is_part_of_posix_class(line: &str, match_start: usize, match_end: usize) -> bool {
    let before = if match_start > 0 {
        &line[..match_start]
    } else {
        ""
    };
    let after = &line[match_end..];

    // Skip if it's part of [[:upper:]] or [:upper:]
    (before.ends_with("[[:") && after.starts_with(":]]"))
        || (before.ends_with("[:") && after.starts_with(":]"))
}

/// Create diagnostic for ASCII-only character range
fn create_character_class_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2019",
        Severity::Info,
        "Use '[:upper:]' to support accents and foreign alphabets".to_string(),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for [A-Z] pattern
        for m in UPPERCASE_RANGE.find_iter(line) {
            let pos = m.start();

            if is_part_of_posix_class(line, pos, m.end()) {
                continue;
            }

            let start_col = pos + 1;
            let end_col = m.end() + 1;
            let diagnostic = create_character_class_diagnostic(line_num, start_col, end_col);
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2019_az_range() {
        let code = r#"tr [A-Z] [a-z]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2019");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("[:upper:]"));
    }

    #[test]
    fn test_sc2019_in_regex() {
        let code = r#"[[ $var =~ [A-Z] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2019_in_grep() {
        let code = r#"grep '[A-Z]' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2019_in_case() {
        let code = r#"case $var in [A-Z]) echo "upper";; esac"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2019_posix_class_ok() {
        let code = r#"tr '[:upper:]' '[:lower:]'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2019_bracket_posix_ok() {
        let code = r#"[[ $var =~ [[:upper:]] ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2019_other_range_ignored() {
        let code = r#"[[ $var =~ [0-9] ]]"#;
        let result = check(code);
        // Only checking for [A-Z], not [0-9]
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2019_multiple_issues() {
        let code = r#"
tr [A-Z] [a-z]
grep '[A-Z]' file
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2019_in_sed() {
        let code = r#"sed 's/[A-Z]/x/g' file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2019_no_bracket_ok() {
        let code = r#"echo "A-Z""#;
        let result = check(code);
        // Not a character class
        assert_eq!(result.diagnostics.len(), 0);
    }
}
