// SC2153: Possible misspelling: VAR is not assigned, but var is
//
// Detects cases where an uppercase variable is referenced but never assigned,
// while a similar lowercase variable exists. This often indicates a typo.
//
// Examples:
// Bad:
//   var="value"
//   echo "$VAR"  # Typo: should be $var
//
// Good:
//   var="value"
//   echo "$var"

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

static VAR_ASSIGNMENT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=").unwrap());

static VAR_REFERENCE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap());

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Extract all assigned variables from source
fn extract_assigned_variables(source: &str) -> HashSet<String> {
    let mut assigned_vars = HashSet::new();

    for line in source.lines() {
        if is_comment_line(line) {
            continue;
        }

        for cap in VAR_ASSIGNMENT.captures_iter(line) {
            if let Some(var) = cap.get(1) {
                assigned_vars.insert(var.as_str().to_string());
            }
        }
    }

    assigned_vars
}

/// Check if variable is a common environment variable
fn is_env_variable(var_name: &str) -> bool {
    const ENV_VARS: &[&str] = &[
        "PATH", "HOME", "USER", "SHELL", "PWD", "OLDPWD", "LANG", "LC_ALL", "TERM", "EDITOR",
        "PAGER",
    ];
    ENV_VARS.contains(&var_name)
}

/// Create a misspelling diagnostic
fn create_misspelling_diagnostic(
    var_name: &str,
    lowercase: &str,
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2153",
        Severity::Warning,
        format!(
            "Possible misspelling: {} is not assigned, but {} is",
            var_name, lowercase
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Track all assigned variables
    let assigned_vars = extract_assigned_variables(source);

    // Check for references to unassigned uppercase variables
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) {
            continue;
        }

        for cap in VAR_REFERENCE.captures_iter(line) {
            if let Some(var) = cap.get(1) {
                let var_name = var.as_str();

                // Skip if already assigned or is environment variable
                if assigned_vars.contains(var_name) || is_env_variable(var_name) {
                    continue;
                }

                // Check if lowercase version exists
                let lowercase = var_name.to_lowercase();
                if var_name.chars().any(|c| c.is_uppercase()) && assigned_vars.contains(&lowercase)
                {
                    let start_col = var.start() + 1;
                    let end_col = var.end() + 1;

                    let diagnostic = create_misspelling_diagnostic(
                        var_name, &lowercase, line_num, start_col, end_col,
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

    // ===== Manual Property Tests =====
    // Establish invariants before refactoring

    #[test]
    fn prop_sc2153_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec!["# var=value\n# echo $VAR", "  # file=test\n  # echo $FILE"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Comments should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2153_correct_casing_never_diagnosed() {
        // Property: Variables with correct casing should never be diagnosed
        let test_cases = vec![
            "var=value\necho $var",
            "VAR=value\necho $VAR",
            "file=test\necho $file",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Correct casing should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sc2153_env_vars_never_diagnosed() {
        // Property: Common environment variables should never be diagnosed
        let env_vars = ["PATH", "HOME", "USER", "SHELL", "PWD", "TERM"];

        for var in env_vars {
            let code = format!("echo ${}", var);
            let result = check(&code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Environment variable {} should not be diagnosed",
                var
            );
        }
    }

    #[test]
    fn prop_sc2153_uppercase_with_lowercase_always_diagnosed() {
        // Property: Uppercase variables with lowercase equivalents should be diagnosed
        let test_cases = vec![
            ("var=value\necho $VAR", "VAR", "var"),
            ("file=test\necho $FILE", "FILE", "file"),
            ("name=x\necho $NAME", "NAME", "name"),
        ];

        for (code, upper, lower) in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                1,
                "Misspelling should be diagnosed: {}",
                code
            );
            assert!(result.diagnostics[0].message.contains(upper));
            assert!(result.diagnostics[0].message.contains(lower));
        }
    }

    #[test]
    fn prop_sc2153_both_assigned_never_diagnosed() {
        // Property: If both cases assigned, no misspelling
        let code = "var=lower\nVAR=upper\necho $var\necho $VAR";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2153_no_lowercase_never_diagnosed() {
        // Property: Uppercase without lowercase equivalent should not be diagnosed
        let code = "echo $UNDEFINED_VAR";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sc2153_diagnostic_code_always_sc2153() {
        // Property: All diagnostics must have code "SC2153"
        let code = "a=1\nb=2\necho $A\necho $B";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2153");
        }
    }

    #[test]
    fn prop_sc2153_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "var=value\necho $VAR";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2153_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_sc2153_basic_misspelling() {
        let code = r#"
var="value"
echo "$VAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2153");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("VAR"));
        assert!(result.diagnostics[0].message.contains("var"));
    }

    #[test]
    fn test_sc2153_correct_casing_ok() {
        let code = r#"
var="value"
echo "$var"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_uppercase_assigned_ok() {
        let code = r#"
VAR="value"
echo "$VAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_env_var_ok() {
        let code = r#"
echo "$PATH"
echo "$HOME"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_multiple_vars() {
        let code = r#"
file="test.txt"
name="example"
echo "$FILE"
echo "$NAME"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2153_mixed_case() {
        let code = r#"
myVar="value"
echo "$MYVAR"
"#;
        let result = check(code);
        // Note: Simple lowercase comparison won't catch camelCase vs UPPERCASE
        // This is a known limitation - only exact case-insensitive matches work
        // "MYVAR".to_lowercase() = "myvar" != "myVar"
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_no_lowercase_ok() {
        let code = r#"
echo "$UNDEFINED"
"#;
        let result = check(code);
        // No lowercase version exists, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_braces() {
        let code = r#"
var="value"
echo "${VAR}"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2153_both_assigned_ok() {
        let code = r#"
var="lower"
VAR="upper"
echo "$var"
echo "$VAR"
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2153_function_local() {
        let code = r#"
function test() {
    local var="value"
    echo "$VAR"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
