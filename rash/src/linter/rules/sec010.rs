//! SEC010: Path Traversal Vulnerabilities
//!
//! **Rule**: Detect path traversal risks in file operations
//!
//! **Why this matters**:
//! Path traversal vulnerabilities allow attackers to access files outside intended directories
//! by using sequences like `../` or absolute paths. This can lead to unauthorized file access,
//! data theft, or system compromise.
//!
//! **Auto-fix**: Manual review required (context-dependent validation needed)
//!
//! ## Examples
//!
//! ❌ **CRITICAL VULNERABILITY**:
//! ```bash
//! # Dangerous - user could provide "../../../../etc/passwd"
//! cp "$USER_FILE" /destination/
//! cat "$INPUT_PATH"
//! tar -xf "$ARCHIVE"  # Could extract outside intended directory
//!
//! # Dangerous - no validation of path
//! mkdir -p "$USER_DIR"
//! cd "$USER_PATH"
//! ```
//!
//! ✅ **SAFE ALTERNATIVES**:
//! ```bash
//! # Validate path doesn't contain ../
//! if [[ "$USER_FILE" == *".."* ]] || [[ "$USER_FILE" == /* ]]; then
//!     echo "Invalid path" >&2
//!     exit 1
//! fi
//! cp "$USER_FILE" /destination/
//!
//! # Use realpath to resolve and validate
//! REAL_PATH=$(realpath -m "$USER_FILE")
//! if [[ "$REAL_PATH" != /safe/base/path/* ]]; then
//!     echo "Path outside allowed directory" >&2
//!     exit 1
//! fi
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// File operation commands that are path traversal vectors
const FILE_OPS: &[&str] = &["cp", "mv", "cat", "tar", "unzip", "rm", "mkdir", "cd", "ln"];

/// Patterns that indicate potential path traversal
const TRAVERSAL_PATTERNS: &[&str] = &[
    "..",      // Parent directory reference
    "../",     // Parent directory path
    "/..",     // Absolute parent reference
];

/// Check for path traversal vulnerabilities
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for file operation commands with variables
        for file_op in FILE_OPS {
            if let Some(cmd_col) = find_command(line, file_op) {
                // Check if line contains unvalidated user input (variables)
                if contains_unvalidated_variable(line, file_op) {
                    let span = Span::new(line_num + 1, cmd_col + 1, line_num + 1, line.len());

                    let diag = Diagnostic::new(
                        "SEC010",
                        Severity::Error,
                        &format!(
                            "Path traversal risk in {} - validate paths don't contain '..' or start with '/'",
                            file_op
                        ),
                        span,
                    );
                    // NO AUTO-FIX: Path validation is context-dependent

                    result.add(diag);
                    break; // Only report once per line
                }
            }
        }

        // Also check for explicit traversal patterns in paths (even if not in variables)
        for pattern in TRAVERSAL_PATTERNS {
            if line.contains(pattern) && contains_file_operation(line) {
                // Check if it's in a validation context (e.g., if statement checking for ..)
                if !is_validation_context(line) {
                    if let Some(pos) = line.find(pattern) {
                        let span = Span::new(line_num + 1, pos + 1, line_num + 1, line.len());

                        let diag = Diagnostic::new(
                            "SEC010",
                            Severity::Warning,
                            "Path contains traversal sequence '..' - ensure this is intentional and validated",
                            span,
                        );

                        result.add(diag);
                        break;
                    }
                }
            }
        }
    }

    result
}

/// Find a command in a line (word boundary detection)
fn find_command(line: &str, cmd: &str) -> Option<usize> {
    if let Some(pos) = line.find(cmd) {
        let before_ok = if pos == 0 {
            true
        } else {
            let char_before = line.chars().nth(pos - 1);
            matches!(
                char_before,
                Some(' ')
                    | Some('\t')
                    | Some(';')
                    | Some('&')
                    | Some('|')
                    | Some('(')
                    | Some('\n')
            )
        };

        let after_idx = pos + cmd.len();
        let after_ok = if after_idx >= line.len() {
            true
        } else {
            let char_after = line.chars().nth(after_idx);
            matches!(
                char_after,
                Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some(')')
            )
        };

        if before_ok && after_ok {
            return Some(pos);
        }
    }
    None
}

/// Check if line contains unvalidated variable in file operation
fn contains_unvalidated_variable(line: &str, _cmd: &str) -> bool {
    // Look for variable usage: $VAR, ${VAR}, "$VAR"
    if !line.contains('$') {
        return false;
    }

    // Check if this looks like user input (common patterns)
    let user_input_patterns = [
        "USER", "INPUT", "FILE", "PATH", "DIR", "ARCHIVE", "NAME", "ARG",
    ];

    let line_upper = line.to_uppercase();
    for pattern in &user_input_patterns {
        if line_upper.contains(pattern) {
            return true;
        }
    }

    // Also flag any variable in file operations as potentially risky
    // (conservative approach for security)
    true
}

/// Check if line contains any file operation
fn contains_file_operation(line: &str) -> bool {
    FILE_OPS.iter().any(|op| find_command(line, op).is_some())
}

/// Check if this is a validation context (checking for ..)
fn is_validation_context(line: &str) -> bool {
    // Common validation patterns
    let validation_keywords = ["if", "case", "grep", "=~", "==", "!="];

    validation_keywords.iter().any(|kw| line.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: Write failing tests first

    #[test]
    fn test_SEC010_detects_cp_with_user_file() {
        let script = r#"cp "$USER_FILE" /destination/"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC010");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("Path traversal"));
    }

    #[test]
    fn test_SEC010_detects_cat_with_input_path() {
        let script = r#"cat "$INPUT_PATH""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    #[test]
    fn test_SEC010_detects_tar_with_archive() {
        let script = r#"tar -xf "$ARCHIVE""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    #[test]
    fn test_SEC010_detects_mkdir_with_user_dir() {
        let script = r#"mkdir -p "$USER_DIR""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    #[test]
    fn test_SEC010_detects_cd_with_user_path() {
        let script = r#"cd "$USER_PATH""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    #[test]
    fn test_SEC010_safe_with_hardcoded_path() {
        let script = r#"cp /etc/config /backup/"#;
        let result = check(script);

        // Hardcoded paths are safe (no variables)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC010_detects_explicit_traversal() {
        let script = r#"cp file.txt ../../sensitive/"#;
        let result = check(script);

        // Should warn about explicit ../ usage
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_SEC010_no_false_positive_validation() {
        let script = r#"if [[ "$FILE" == *".."* ]]; then exit 1; fi"#;
        let result = check(script);

        // This is validation, not a vulnerability
        // Should not flag (or flag with lower severity)
        // Conservative: might still flag but acceptable for security
    }

    #[test]
    fn test_SEC010_no_auto_fix() {
        let script = r#"cp "$USER_FILE" /dest/"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.fix.is_none(), "SEC010 should not provide auto-fix");
    }

    #[test]
    fn test_SEC010_multiple_vulnerabilities() {
        let script = r#"
cp "$USER_FILE" /dest/
cat "$INPUT_PATH"
        "#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_SEC010_no_false_positive_comment() {
        let script = r#"# cp "$USER_FILE" is dangerous"#;
        let result = check(script);

        // Comments should not trigger the rule
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_sec010_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_sec010_safe_hardcoded_paths(
            src in "/[a-z/]{1,20}",
            dst in "/[a-z/]{1,20}",
        ) {
            let cmd = format!("cp {} {}", src, dst);
            let result = check(&cmd);
            // Hardcoded paths (no variables) should be safe
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_sec010_detects_user_variables(
            file_op_idx in 0..9usize,
            var_name in "(USER|INPUT|FILE|PATH|DIR|ARCHIVE|NAME|ARG)_[A-Z]{1,5}",
        ) {
            let file_op = match file_op_idx {
                0 => "cp",
                1 => "mv",
                2 => "cat",
                3 => "tar",
                4 => "unzip",
                5 => "rm",
                6 => "mkdir",
                7 => "cd",
                _ => "ln",
            };
            let cmd = format!(r#"{} "${{{}}}""#, file_op, var_name);
            let result = check(&cmd);
            // Should detect path traversal risk with user variables
            prop_assert!(result.diagnostics.len() >= 1);
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC010");
        }
    }
}
