//! SEC002: Unquoted Variable in Command
//!
//! **Rule**: Detect unquoted variables in potentially dangerous commands
//!
//! **Why this matters**:
//! Unquoted variables can lead to command injection if they contain spaces
//! or special characters. This is especially dangerous in commands that
//! interact with the network, filesystem, or execute other commands.
//!
//! **Auto-fix**: Safe (add quotes)
//!
//! ## Examples
//!
//! ❌ **UNSAFE**:
//! ```bash
//! curl $URL
//! wget $FILE_PATH
//! ssh $HOST
//! git clone $REPO
//! ```
//!
//! ✅ **SAFE** (auto-fixable):
//! ```bash
//! curl "${URL}"
//! wget "${FILE_PATH}"
//! ssh "${HOST}"
//! git clone "${REPO}"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Dangerous commands that should never have unquoted variables
const DANGEROUS_COMMANDS: &[&str] = &[
    "curl", "wget", "ssh", "scp", "git", "rsync", "docker", "kubectl",
];

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Find which dangerous command (if any) is in the line
fn find_dangerous_command(line: &str) -> Option<&'static str> {
    DANGEROUS_COMMANDS
        .iter()
        .find(|&cmd| line.contains(cmd))
        .copied()
}

/// Find the first unquoted variable in a line
/// Returns (column, variable_name) if found
fn find_unquoted_variable(line: &str) -> Option<usize> {
    let mut chars = line.chars().peekable();
    let mut col = 0;
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    while let Some(ch) = chars.next() {
        col += 1;

        match ch {
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '$' if !in_double_quotes && !in_single_quotes => {
                // Found unquoted $
                // Check if followed by variable name
                if chars
                    .peek()
                    .map(|c| c.is_alphanumeric() || *c == '_')
                    .unwrap_or(false)
                {
                    return Some(col);
                }
            }
            _ => {}
        }
    }

    None
}

/// Create a SEC002 diagnostic for unquoted variable
fn create_sec002_diagnostic(cmd: &str, line_num: usize, col: usize) -> Diagnostic {
    let span = Span::new(line_num + 1, col, line_num + 1, col + 1);

    Diagnostic::new(
        "SEC002",
        Severity::Error,
        format!("Unquoted variable in {} command - add quotes", cmd),
        span,
    )
    .with_fix(Fix::new("\"$VAR\""))
}

/// Check for unquoted variables in dangerous commands
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if is_comment_line(line) {
            continue;
        }

        // Check if line contains a dangerous command
        if let Some(cmd) = find_dangerous_command(line) {
            // Look for unquoted variable
            if let Some(col) = find_unquoted_variable(line) {
                let diag = create_sec002_diagnostic(cmd, line_num, col);
                result.add(diag);
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
    fn prop_sec002_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# curl $URL",
            "  # wget $FILE",
            "\t# ssh $HOST",
            "# git clone $REPO",
        ];

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
    fn prop_sec002_quoted_variables_never_diagnosed() {
        // Property: Properly quoted variables should never be diagnosed
        let test_cases = vec![
            r#"curl "${URL}""#,
            "wget \"$FILE_PATH\"",
            "ssh '$HOST'",
            r#"git clone "${REPO}""#,
            "docker run \"$IMAGE\"",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Quoted variables should be OK: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sec002_unquoted_dangerous_always_diagnosed() {
        // Property: Unquoted variables in dangerous commands should always be diagnosed
        let test_cases = vec![
            ("curl $URL", "curl"),
            ("wget $FILE", "wget"),
            ("ssh $HOST", "ssh"),
            ("git clone $REPO", "git"),
            ("docker run $IMAGE", "docker"),
        ];

        for (code, cmd) in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                1,
                "Unquoted {} should be diagnosed: {}",
                cmd,
                code
            );
            assert!(result.diagnostics[0].message.contains(cmd));
        }
    }

    #[test]
    fn prop_sec002_safe_commands_never_diagnosed() {
        // Property: Non-dangerous commands should not be diagnosed
        let test_cases = vec!["echo $VAR", "printf $FORMAT", "cat $FILE", "ls $DIR"];

        for code in test_cases {
            let result = check(code);
            assert_eq!(
                result.diagnostics.len(),
                0,
                "Safe commands should not be diagnosed: {}",
                code
            );
        }
    }

    #[test]
    fn prop_sec002_diagnostic_code_always_sec002() {
        // Property: All diagnostics must have code "SEC002"
        let code = "curl $A\nwget $B\nssh $C";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SEC002");
        }
    }

    #[test]
    fn prop_sec002_diagnostic_severity_always_error() {
        // Property: All diagnostics must be Error severity
        let code = "curl $A\nwget $B";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Error);
        }
    }

    #[test]
    fn prop_sec002_all_diagnostics_have_fix() {
        // Property: All SEC002 diagnostics must provide a fix
        let code = "curl $URL\nwget $FILE";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert!(
                diagnostic.fix.is_some(),
                "All SEC002 diagnostics should have a fix"
            );
        }
    }

    #[test]
    fn prop_sec002_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_sec002_only_one_diagnostic_per_line() {
        // Property: Only report first unquoted variable per line
        let code = "curl $URL $BACKUP";
        let result = check(code);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should only report once per line"
        );
    }

    // ===== Original Unit Tests =====

    #[test]
    fn test_SEC002_detects_unquoted_curl() {
        let script = "curl $URL";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC002");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("curl"));
    }

    #[test]
    fn test_SEC002_detects_unquoted_wget() {
        let script = "wget $FILE_PATH";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC002_detects_unquoted_ssh() {
        let script = "ssh $HOST";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC002_no_warning_with_quotes() {
        let script = r#"curl "${URL}""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC002_no_warning_with_double_quotes() {
        let script = "wget \"$FILE_PATH\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC002_provides_fix() {
        let script = "curl $URL";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "\"$VAR\"");
    }

    #[test]
    fn test_SEC002_no_false_positive_comment() {
        let script = "# curl $URL";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
