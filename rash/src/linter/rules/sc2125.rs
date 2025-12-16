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

/// Issue #93: Check if braces in value are only parameter expansion (not brace expansion)
/// Parameter expansion: ${VAR}, ${VAR:-default}, ${VAR:+alt}, ${#VAR}, ${VAR%pattern}, etc.
/// Brace expansion: {a,b,c}, {1..10} (NOT preceded by $)
/// Returns true only if the value contains braces AND all braces are parameter expansion
fn has_brace_expansion(value: &str) -> bool {
    // If no braces at all, this is NOT a brace issue (might be glob like *.txt)
    if !value.contains('{') {
        return false;
    }

    // Find all braces and check if any are brace expansion
    let chars: Vec<char> = value.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '{' {
            // Check if preceded by $ (parameter expansion)
            if i == 0 || chars[i - 1] != '$' {
                // Found a brace not preceded by $ - check if it's brace expansion
                // Look for comma or .. inside braces (brace expansion patterns)
                let mut depth = 1;
                let mut j = i + 1;
                while j < chars.len() && depth > 0 {
                    if chars[j] == '{' {
                        depth += 1;
                    } else if chars[j] == '}' {
                        depth -= 1;
                    } else if depth == 1 && chars[j] == ',' {
                        // Found comma at depth 1 - this is brace expansion
                        return true;
                    } else if depth == 1
                        && j + 1 < chars.len()
                        && chars[j] == '.'
                        && chars[j + 1] == '.'
                    {
                        // Found .. at depth 1 - this is brace expansion range
                        return true;
                    }
                    j += 1;
                }
            }
        }
    }
    // No brace expansion patterns found (all braces are parameter expansion)
    false
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

                // Issue #93: Check what was matched
                // The regex matches: glob (*) OR brace expansion ({...})
                let has_glob = val_text.contains('*');
                let has_brace = has_brace_expansion(val_text);

                // Skip if no glob and no brace expansion (only parameter expansion like ${VAR:-default})
                if !has_glob && !has_brace {
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

    // Issue #93: Parameter expansion ${VAR:-default} should NOT be flagged
    #[test]
    fn test_issue_93_param_expansion_default_ok() {
        // From issue #93: ${VAR:-default} is parameter expansion, not brace expansion
        let code = r#"TEST_LINE=${TEST_LINE:-99999}"#;
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "SC2125 must NOT flag parameter expansion ${{VAR:-default}}"
        );
    }

    #[test]
    fn test_issue_93_param_expansion_multiple_ok() {
        let code = r#"PROBAR_COUNT=${PROBAR_COUNT:-0}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_param_expansion_alt_value_ok() {
        let code = r#"VALUE=${OTHER:+replacement}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_param_expansion_error_ok() {
        let code = r#"REQUIRED=${VAR:?error message}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_param_expansion_assign_default_ok() {
        let code = r#"VAR=${VAR:=default}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_param_expansion_length_ok() {
        let code = r#"LEN=${#VAR}"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_mixed_param_and_brace_flagged() {
        // Parameter expansion at start, but brace expansion in middle - should be flagged
        let code = r#"FILE=${DIR}/{a,b}.txt"#;
        let result = check(code);
        // The {a,b} part is brace expansion and SHOULD be flagged
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Mixed param expansion + brace expansion should be flagged"
        );
    }
}
