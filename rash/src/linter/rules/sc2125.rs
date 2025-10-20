// SC2125: Brace expansions and globs are literal in assignments. Assign as array or use * as arguments.
//
// In variable assignments, glob patterns like *.txt are treated as literal strings,
// not expanded. This is often not what the user intended.
//
// Examples:
// Bad:
//   files=*.txt           # Literal string "*.txt", not file list
//   docs={a,b,c}.md       # Literal string "{a,b,c}.md", not expansion
//   paths=/tmp/*.log      # Literal string, not expanded
//
// Good:
//   files=(*.txt)         # Array with expanded files
//   shopt -s nullglob; files=(*.txt)  # Handle no matches gracefully
//   for file in *.txt; do ...; done   # Use glob in loop, not assignment

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static GLOB_IN_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
    // Match: var=*.ext or var=/path/*.ext or var={a,b,c}
    Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*=([^=]*\*[^=\s]*|.*\{[^}]+\}[^=\s]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip array assignments (those with parentheses)
        if trimmed.contains("=(") {
            continue;
        }

        // Skip quoted assignments
        if trimmed.contains("=\"") || trimmed.contains("='") {
            continue;
        }

        for cap in GLOB_IN_ASSIGNMENT.captures_iter(trimmed) {
            if let Some(value) = cap.get(1) {
                let val_text = value.as_str();

                // Skip if it's in a command substitution
                if val_text.contains("$(") || val_text.contains("`") {
                    continue;
                }

                let start_col = cap.get(0).unwrap().start() + 1;
                let end_col = cap.get(0).unwrap().end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2125",
                    Severity::Warning,
                    "Brace expansions and globs are literal in assignments. Assign as array, e.g., arr=(*.txt)",
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2125_glob_assignment() {
        let code = r#"files=*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2125");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("array"));
    }

    #[test]
    fn test_sc2125_brace_expansion() {
        let code = r#"docs={a,b,c}.md"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2125_path_glob() {
        let code = r#"logs=/tmp/*.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2125_array_assignment_ok() {
        let code = r#"files=(*.txt)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2125_quoted_assignment_ok() {
        let code = r#"pattern="*.txt""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2125_single_quoted_ok() {
        let code = r#"glob='*.log'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2125_command_substitution_ok() {
        let code = r#"files=$(ls *.txt)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2125_literal_value_ok() {
        let code = r#"name=value"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2125_multiple_wildcards() {
        let code = r#"all=/var/log/*.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2125_brace_in_path() {
        let code = r#"configs=/etc/{a,b,c}/config"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
