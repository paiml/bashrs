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

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line is an array assignment
fn is_array_assignment(line: &str) -> bool {
    line.contains("=(")
}

/// Check if line is a quoted assignment
fn is_quoted_assignment(line: &str) -> bool {
    line.contains("=\"") || line.contains("='")
}

/// Check if value contains command substitution
fn contains_command_substitution(value: &str) -> bool {
    value.contains("$(") || value.contains("`")
}

/// Create diagnostic for glob in assignment
fn create_glob_assignment_diagnostic(
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2125",
        Severity::Warning,
        "Brace expansions and globs are literal in assignments. Assign as array, e.g., arr=(*.txt)",
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        let trimmed = line.trim_start();
        if is_comment_line(trimmed) || is_array_assignment(trimmed) || is_quoted_assignment(trimmed)
        {
            continue;
        }

        for cap in GLOB_IN_ASSIGNMENT.captures_iter(trimmed) {
            if let Some(value) = cap.get(1) {
                let val_text = value.as_str();

                if contains_command_substitution(val_text) {
                    continue;
                }

                let start_col = cap.get(0).unwrap().start() + 1;
                let end_col = cap.get(0).unwrap().end() + 1;

                let diagnostic = create_glob_assignment_diagnostic(line_num, start_col, end_col);
                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2125_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# files=*.txt",
            "  # docs={a,b,c}.md",
            "\t# logs=/tmp/*.log",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2125_array_assignments_never_diagnosed() {
        // Property: Array assignments should never be diagnosed
        let test_cases = vec!["files=(*.txt)", "docs=({a,b,c}.md)", "logs=(/tmp/*.log)"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2125_quoted_assignments_never_diagnosed() {
        // Property: Quoted assignments should never be diagnosed
        let test_cases = vec![
            "pattern=\"*.txt\"",
            "glob='*.log'",
            "brace=\"{a,b,c}\"",
            "path='/tmp/*.log'",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2125_command_substitution_never_diagnosed() {
        // Property: Command substitution should never be diagnosed
        let test_cases = vec![
            "files=$(ls *.txt)",
            "docs=`find . -name *.md`",
            "logs=$(echo /tmp/*.log)",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2125_literal_values_never_diagnosed() {
        // Property: Literal values without globs should never be diagnosed
        let test_cases = vec!["name=value", "path=/usr/bin", "file=test.txt"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2125_unquoted_globs_always_diagnosed() {
        // Property: Unquoted globs in assignments should always be diagnosed
        let test_cases = vec![
            "files=*.txt",
            "logs=/tmp/*.log",
            "all=/var/log/*.log",
            "docs=*.md",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains("array"));
        }
    }

    #[test]
    fn prop_sc2125_brace_expansions_always_diagnosed() {
        // Property: Brace expansions in assignments should always be diagnosed
        let test_cases = vec![
            "docs={a,b,c}.md",
            "configs=/etc/{a,b,c}/config",
            "files={1,2,3}.txt",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
        }
    }

    #[test]
    fn prop_sc2125_diagnostic_code_always_sc2125() {
        // Property: All diagnostics must have code "SC2125"
        let code = "files=*.txt\nlogs=*.log";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2125");
        }
    }

    #[test]
    fn prop_sc2125_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "files=*.txt";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2125_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
