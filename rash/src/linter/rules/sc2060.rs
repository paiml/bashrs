// SC2060: Quote parameters to tr to prevent glob expansion
//
// The tr command takes character sets as parameters. If these contain
// glob characters like [a-z] and are not quoted, the shell will expand
// them as globs before tr sees them, causing unexpected behavior.
//
// Examples:
// Bad:
//   echo "$str" | tr [a-z] [A-Z]     // Glob expansion [a-z] → files
//   tr [0-9] [x] < file              // If files [0-9] exist, wrong behavior
//   echo "$text" | tr [:lower:] [:upper:]  // Glob expansion
//
// Good:
//   echo "$str" | tr '[a-z]' '[A-Z]'     // Quoted, no glob expansion
//   tr '[0-9]' '[x]' < file              // Safe from glob matching
//   echo "$text" | tr '[:lower:]' '[:upper:]'  // POSIX character classes
//
// Impact:
//   - If files matching the pattern exist, wrong characters used
//   - If no files match, literal bracket characters used
//   - Behavior varies by directory contents (non-deterministic)
//   - Can cause silent data corruption
//
// Note: Always quote tr parameters containing brackets.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TR_UNQUOTED_BRACKETS: Lazy<Regex> = Lazy::new(|| {
    // Match: tr (with optional flags) followed by argument with unquoted bracket
    // Matches: tr [a-z], tr -d [0-9], tr -c [a-z], etc.
    Regex::new(r"\btr\s+(-[a-zA-Z]+\s+)?\S*\[").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for tr commands with unquoted bracket expressions
        if line.contains("tr ") {
            // Find all tr commands
            for mat in TR_UNQUOTED_BRACKETS.find_iter(line) {
                let matched = mat.as_str();

                // Check if brackets are unquoted
                if matched.contains('[') && !matched.contains('\'') && !matched.contains('"') {
                    let start_col = mat.start() + 1;
                    let end_col = mat.end() + 1;

                    let diagnostic = Diagnostic::new(
                        "SC2060",
                        Severity::Warning,
                        "Quote parameters to tr to prevent glob expansion (e.g., tr '[a-z]' '[A-Z]')".to_string(),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2060_unquoted_ranges() {
        let code = r#"echo "$str" | tr [a-z] [A-Z]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2060");
    }

    #[test]
    fn test_sc2060_unquoted_single_range() {
        let code = r#"tr [0-9] [x] < file"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2060_posix_classes_unquoted() {
        let code = r#"echo "$text" | tr [:lower:] [:upper:]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2060_quoted_single_ok() {
        let code = r#"echo "$str" | tr '[a-z]' '[A-Z]'"#;
        let result = check(code);
        // Quoted brackets, safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2060_quoted_double_ok() {
        let code = r#"tr "[0-9]" "[x]" < file"#;
        let result = check(code);
        // Quoted brackets, safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2060_no_brackets_ok() {
        let code = r#"echo "$str" | tr 'abc' 'xyz'"#;
        let result = check(code);
        // No brackets, no issue
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2060_comment_ok() {
        let code = r#"# echo "$str" | tr [a-z] [A-Z]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2060_posix_quoted_ok() {
        let code = r#"echo "$text" | tr '[:lower:]' '[:upper:]'"#;
        let result = check(code);
        // POSIX classes quoted, safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2060_delete_with_brackets() {
        let code = r#"tr -d [0-9] < input"#;
        let result = check(code);
        // Unquoted with -d flag
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2060_complement_unquoted() {
        let code = r#"tr -c [a-z] x"#;
        let result = check(code);
        // Unquoted with -c flag
        assert_eq!(result.diagnostics.len(), 1);
    }
}
