//! SEC003: Command Injection via find -exec with sh -c
//!
//! **Rule**: Detect `{}` embedded in shell command strings within `find -exec sh -c`
//!
//! **Why this matters**:
//! When {} appears inside a shell command string (sh -c '...{}...'), filenames
//! with special characters can lead to command injection. The {} is expanded
//! by find BEFORE the shell parses the string, allowing malicious filenames
//! to inject arbitrary commands.
//!
//! **Important**: Unquoted {} as a separate argument (find -exec rm {} \;) is
//! SAFE because find passes the filename as a single argument. The shell never
//! interprets it.
//!
//! **Auto-fix**: Use positional parameters instead
//!
//! ## Examples
//!
//! ❌ **UNSAFE** (command injection via embedded {}):
//! ```bash
//! find . -exec sh -c 'echo {}' \;
//! find . -exec bash -c "rm {}" \;
//! ```
//!
//! ✅ **SAFE** (use positional parameters):
//! ```bash
//! find . -exec sh -c 'echo "$1"' _ {} \;
//! find . -exec bash -c 'rm "$1"' _ {} \;
//! ```
//!
//! ✅ **SAFE** ({} as separate argument - NOT in shell string):
//! ```bash
//! find . -exec rm {} \;
//! find . -exec chmod 644 {} +
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for {} embedded in shell command strings (dangerous injection vector)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Only flag the dangerous pattern: sh -c or bash -c with {} embedded in the string
        // Pattern: find ... -exec (sh|bash) -c '...{}...' or "...{}..."
        if line.contains("find ") && line.contains("-exec") {
            // Check for sh -c or bash -c with {} inside the command string
            if (line.contains("sh -c") || line.contains("bash -c")) && line.contains("{}") {
                // Check if {} is inside quotes (dangerous)
                // Look for patterns like 'echo {}' or "rm {}"
                if is_braces_in_shell_string(line) {
                    if let Some(col) = find_braces_in_quotes(line) {
                        let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 3);

                        let diag = Diagnostic::new(
                            "SEC003",
                            Severity::Error,
                            "Command injection: {} embedded in shell string. Use positional params: sh -c 'cmd \"$1\"' _ {}",
                            span,
                        )
                        .with_fix(Fix::new("\"$1\""));

                        result.add(diag);
                    }
                }
            }
        }
    }

    result
}

/// Check if {} appears inside a quoted string in the line
fn is_braces_in_shell_string(line: &str) -> bool {
    // Simple heuristic: check for patterns like '...{}...' or "...{}..."
    let single_quote_pattern = |s: &str| {
        let mut in_single = false;
        let mut found_braces_in_quote = false;
        let chars: Vec<char> = s.chars().collect();
        for i in 0..chars.len() {
            if chars[i] == '\'' {
                in_single = !in_single;
            }
            if in_single && i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}' {
                found_braces_in_quote = true;
            }
        }
        found_braces_in_quote
    };

    let double_quote_pattern = |s: &str| {
        let mut in_double = false;
        let mut found_braces_in_quote = false;
        let chars: Vec<char> = s.chars().collect();
        for i in 0..chars.len() {
            if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
                in_double = !in_double;
            }
            if in_double && i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}' {
                found_braces_in_quote = true;
            }
        }
        found_braces_in_quote
    };

    single_quote_pattern(line) || double_quote_pattern(line)
}

/// Find the column position of {} inside quotes
fn find_braces_in_quotes(line: &str) -> Option<usize> {
    let mut in_single = false;
    let mut in_double = false;
    let chars: Vec<char> = line.chars().collect();

    for i in 0..chars.len() {
        if chars[i] == '\'' && !in_double {
            in_single = !in_single;
        }
        if chars[i] == '"' && !in_single && (i == 0 || chars[i - 1] != '\\') {
            in_double = !in_double;
        }
        if (in_single || in_double) && i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}'
        {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Tests for DANGEROUS pattern: sh -c with embedded {} =====

    #[test]
    fn test_SEC003_detects_sh_c_with_embedded_braces_single_quote() {
        // DANGEROUS: {} inside shell string - command injection risk
        let script = "find . -exec sh -c 'echo {}' \\;";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC003");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("Command injection"));
    }

    #[test]
    fn test_SEC003_detects_bash_c_with_embedded_braces_double_quote() {
        // DANGEROUS: {} inside shell string with double quotes
        let script = r#"find . -exec bash -c "rm {}" \;"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC003_provides_fix_for_injection() {
        let script = "find . -exec sh -c 'cat {}' \\;";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "\"$1\"");
    }

    // ===== Tests for SAFE patterns: {} as separate argument =====

    #[test]
    fn test_SEC003_no_warning_for_standard_find_exec() {
        // SAFE: {} as separate argument - find handles it, shell never sees it
        let script = r#"find . -name "*.sh" -exec chmod +x {} \;"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC003_no_warning_for_rm_with_braces() {
        // SAFE: {} as separate argument
        let script = "find /tmp -type f -exec rm {} \\;";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC003_no_warning_for_batch_mode() {
        // SAFE: {} with + batch mode
        let script = "find . -type f -exec chmod 644 {} +";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC003_no_false_positive_no_find() {
        let script = "echo {} something";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    // REQ-FP-006: SEC003 MUST NOT flag unquoted {} in find -exec
    // Rationale: {} is handled by find, not the shell. Quoting provides no security benefit.
    #[test]
    fn test_SEC003_no_false_positive_find_exec_standard() {
        // Standard find -exec patterns - {} is handled by find, NOT the shell
        let script = r#"find . -name "*.txt" -exec rm {} \;
find . -type f -exec chmod 644 {} +"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC003 must NOT flag {{}} in find -exec - it's handled by find, not the shell"
        );
    }

    #[test]
    fn test_SEC003_no_false_positive_find_execdir() {
        // execdir variant - same principle
        let script = "find . -execdir mv {} {}.bak \\;";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC003_safe_positional_params() {
        // SAFE: Using positional parameters correctly
        let script = r#"find . -exec sh -c 'echo "$1"' _ {} \;"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Mutation Coverage Tests - Following SEC001 pattern (100% kill rate) =====

    #[test]
    fn test_mutation_sec003_line_num_calculation() {
        // Tests line number calculation for multiline input
        let bash_code = "# comment\nfind . -exec sh -c 'echo {}' \\;";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        // With +1: line 2, With *1: line 0
        assert_eq!(
            result.diagnostics[0].span.start_line, 2,
            "Line number must use +1, not *1"
        );
    }

    #[test]
    fn test_mutation_sec003_column_calculation() {
        // Tests column calculations
        let bash_code = "find . -exec sh -c 'echo {}' \\;";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // {} is inside 'echo {}' - position should be after 'echo '
        assert!(
            span.start_col > 20,
            "Start column should be inside the quoted string"
        );
        assert_eq!(
            span.end_col - span.start_col,
            2,
            "Span should cover {{}} (2 chars)"
        );
    }

    #[test]
    fn test_mutation_sec003_double_quotes_detection() {
        // Tests detection in double-quoted strings
        let bash_code = r#"find . -exec bash -c "test {}" \;"#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_mutation_sec003_requires_find_keyword() {
        // Ensure we only check find commands, not other sh -c
        let bash_code = "sh -c 'echo {}' \\;";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_mutation_sec003_requires_exec_flag() {
        // Ensure we require -exec, not just find with sh -c
        let bash_code = "find . -name '*.sh' | sh -c 'echo {}'";
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Issue #87: SEC003 should NOT flag standard find -exec with {} as separate arg
    #[test]
    fn test_SEC003_issue_87_no_false_positive_dirname() {
        // From issue #87 reproduction case
        let script = r#"DIRS=$(find "$CORPUS" -name "Cargo.toml" -exec dirname {} \; 2>/dev/null | sort -u)"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "SEC003 must NOT flag {{}} in 'find -exec dirname {{}} \\;' - it's a separate argument, not embedded in shell string"
        );
    }

    #[test]
    fn test_SEC003_issue_87_no_false_positive_command_substitution() {
        // Another common pattern from issue
        let script = "FILES=$(find /path -type f -exec basename {} \\;)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
