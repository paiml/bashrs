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

/// Check if a command appears as a whole word in the line
fn is_word_boundary(line: &str, pos: usize, cmd_len: usize) -> bool {
    let before_ok = pos == 0
        || !line.as_bytes()[pos - 1].is_ascii_alphanumeric() && line.as_bytes()[pos - 1] != b'_';
    let after_pos = pos + cmd_len;
    let after_ok = after_pos >= line.len()
        || !line.as_bytes()[after_pos].is_ascii_alphanumeric()
            && line.as_bytes()[after_pos] != b'_';
    before_ok && after_ok
}

/// Find which dangerous command (if any) is in the line
fn find_dangerous_command(line: &str) -> Option<&'static str> {
    DANGEROUS_COMMANDS
        .iter()
        .find(|&cmd| {
            line.match_indices(cmd)
                .any(|(pos, _)| is_word_boundary(line, pos, cmd.len()))
        })
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
                    .is_some_and(|c| c.is_alphanumeric() || *c == '_')
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

    // ===== Mutation Coverage Tests - Following SEC001 pattern (100% kill rate) =====

    #[test]
    fn test_mutation_sec002_unquoted_var_start_col_exact() {
        // MUTATION: Line 84:35 - replace + with * in line_num + 1
        // MUTATION: Line 84:63 - Tests start column calculation
        let bash_code = "curl $URL"; // $ at column 6
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // With correct arithmetic: start_col = 6
        // With mutation (+ → *): would produce incorrect column
        assert_eq!(
            span.start_col, 6,
            "Start column must use correct calculation"
        );
    }

    #[test]
    fn test_mutation_sec002_unquoted_var_end_col_exact() {
        // MUTATION: Line 84:63 - replace + with * in col + 1
        // MUTATION: Line 84:63 - replace + with - in col + 1
        // Tests end column calculation
        let bash_code = "curl $URL"; // $ at column 6, ends at column 7
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // With +1: end_col = 7
        // With *1: end_col = 6
        // With -1: end_col = 5
        assert_eq!(
            span.end_col, 7,
            "End column must be col + 1, not col * 1 or col - 1"
        );
    }

    #[test]
    fn test_mutation_sec002_line_num_calculation() {
        // MUTATION: Line 84:35 - replace + with * in line_num + 1
        // Tests line number calculation for multiline input
        let bash_code = "# comment\ncurl $URL"; // curl on line 2
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With +1: line 2
        // With *1: line 0
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Line number must use +1, not *1"
        );
    }

    #[test]
    fn test_mutation_sec002_column_with_offset() {
        // Tests column calculations with leading whitespace
        // Also catches Line 59:13 col += 1 mutation
        let bash_code = "    curl $URL"; // $ at column 10 (4 spaces + "curl " = 9, $ at 10)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_col, 10, "Must account for leading whitespace");
        assert_eq!(span.end_col, 11, "End must be start + 1");
    }

    #[test]
    fn test_mutation_sec002_column_tracking_accuracy() {
        // MUTATION: Line 59:13 - replace += with *= in col += 1
        // Test that column tracking is accurate for variables at various positions
        let bash_code = "curl       $URL"; // $ at column 12 (extra spaces)
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With col += 1: correctly tracks to column 12
        // With col *= 1: would produce incorrect tracking
        assert_eq!(
            result.diagnostics[0].span.start_col, 12,
            "Column tracking must increment correctly"
        );
    }

    #[test]
    fn test_mutation_sec002_quote_detection_single_quotes() {
        // MUTATION: Line 62:20 - replace !in_single_quotes with true
        // Ensure single-quoted variables are not diagnosed
        let bash_code = "curl '$URL'"; // Should be safe (single quotes)
        let result = check(bash_code);
        // With correct logic: 0 diagnostics (single quotes protect variable)
        // With mutation (!in_single_quotes → true): might incorrectly diagnose
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Single-quoted variables should be safe"
        );
    }

    #[test]
    fn test_mutation_sec002_quote_detection_double_quotes() {
        // MUTATION: Line 62:20 - Additional test for quote tracking logic
        // Tests quote tracking logic comprehensively
        let bash_code = r#"curl "${URL}""#; // Should be safe (double quotes)
        let result = check(bash_code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Double-quoted variables should be safe"
        );
    }

    #[test]
    fn test_mutation_sec002_variable_detection_underscore() {
        // MUTATION: Line 69:56 - replace == with != in *c == '_'
        // Tests that underscore is correctly detected as part of variable names
        let bash_code = "curl $MY_VAR"; // Variable with underscore
        let result = check(bash_code);
        // With ==: detects $MY_VAR (correct)
        // With !=: might fail to detect underscore as valid variable char
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should detect variable with underscore"
        );
    }
}
