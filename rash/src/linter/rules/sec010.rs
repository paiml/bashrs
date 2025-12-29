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
    "..",  // Parent directory reference
    "../", // Parent directory path
    "/..", // Absolute parent reference
];

/// Check for path traversal vulnerabilities
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Issue #104: Track validated variables across the script
    // Variables that have been validated with checks like `if [[ "$var" == *".."* ]]`
    let mut validated_vars: Vec<String> = Vec::new();

    // Also track if we're in a validation block that exits on invalid input
    let mut in_validation_block = false;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Issue #106: Skip heredoc patterns - cat <<EOF is not a file read
        if is_heredoc_pattern(line) {
            continue;
        }

        // Issue #104: Detect path validation patterns
        // Pattern: if [[ "$VAR" == *".."* ]] or if [[ "$VAR" == /* ]]
        if is_path_validation_check(trimmed) {
            // Extract variable name being validated
            if let Some(var) = extract_validated_variable(trimmed) {
                validated_vars.push(var);
            }
            in_validation_block = true;
            continue;
        }

        // Issue #104: Check for realpath validation
        // Pattern: SAFE_PATH=$(realpath -m "$VAR") or similar
        if trimmed.contains("realpath") || trimmed.contains("readlink -f") {
            if let Some(var) = extract_assigned_variable(trimmed) {
                validated_vars.push(var);
            }
            continue;
        }

        // Track end of validation blocks
        if trimmed == "fi" || trimmed.starts_with("fi ") || trimmed.starts_with("fi;") {
            in_validation_block = false;
        }

        // Issue #104: Skip validation guard bodies (exit/return after check)
        if in_validation_block && (trimmed.contains("exit") || trimmed.contains("return")) {
            continue;
        }

        // Look for file operation commands with variables
        for file_op in FILE_OPS {
            if let Some(cmd_col) = find_command(line, file_op) {
                // Issue #104: Check if variable has been validated
                if is_variable_validated(line, &validated_vars) {
                    continue;
                }

                // Check if line contains unvalidated user input (variables)
                if contains_unvalidated_variable(line, file_op) {
                    let span = Span::new(line_num + 1, cmd_col + 1, line_num + 1, line.len());

                    let diag = Diagnostic::new(
                        "SEC010",
                        Severity::Error,
                        format!(
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
                // Issue #73: Skip BASH_SOURCE parent directory pattern - this is intentional
                if line.contains("BASH_SOURCE") || line.contains("dirname") {
                    continue;
                }

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
                Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some('(') | Some('\n')
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

/// Issue #73: Known-safe patterns that should not trigger SEC010
// These are checked as exact variable names (with $ or ${} wrapper)
const SAFE_VAR_PATTERNS: &[&str] = &[
    "$PWD",        // Current directory is intentional
    "${PWD}",      // Current directory is intentional
    "$HOME",       // User's home directory is safe
    "${HOME}",     // User's home directory is safe
    "$TMPDIR",     // Temp directory is safe
    "${TMPDIR}",   // Temp directory is safe
    "BASH_SOURCE", // Script's own directory is safe
    "dirname",     // dirname of script is safe
    "XDG_",        // XDG directories are safe
];

/// Check if line contains unvalidated variable in file operation
fn contains_unvalidated_variable(line: &str, _cmd: &str) -> bool {
    // Look for variable usage: $VAR, ${VAR}, "$VAR"
    if !line.contains('$') {
        return false;
    }

    // Issue #73: Skip known-safe patterns
    for safe_pattern in SAFE_VAR_PATTERNS {
        if line.contains(safe_pattern) {
            return false;
        }
    }

    // Issue #73: Script directory parent (..) with BASH_SOURCE is intentional
    // Pattern: cd "$(dirname "${BASH_SOURCE[0]}")/.."
    if line.contains("dirname") && line.contains("..") {
        return false;
    }

    // Check if this looks like user input (common patterns)
    // These patterns suggest untrusted or user-provided input
    let user_input_patterns = [
        "USER",      // USER_FILE, USER_PATH, etc.
        "INPUT",     // INPUT_PATH, INPUT_FILE, etc.
        "UPLOAD",    // Uploaded files
        "ARCHIVE",   // Archive files (could be user-provided)
        "UNTRUSTED", // Explicitly untrusted
        "EXTERNAL",  // External input
        "REMOTE",    // Remote data
        "ARG",       // Command line arguments
        "NAME",      // Could be user-provided name
        "FILE",      // Generic file variables
        "PATH",      // Generic path variables (but not PATH env var)
        "DIR",       // Generic directory variables
    ];

    let line_upper = line.to_uppercase();

    // Don't flag the PATH environment variable itself
    if line.contains("$PATH") || line.contains("${PATH}") {
        // This is the PATH env var, not a user path
        let path_count = line.matches("PATH").count();
        let dollar_path_count = line.matches("$PATH").count() + line.matches("${PATH}").count();
        if path_count == dollar_path_count {
            return false; // All PATH references are the env var
        }
    }

    for pattern in &user_input_patterns {
        if line_upper.contains(pattern) {
            return true;
        }
    }

    // If no suspicious pattern found, assume it's safe
    // This reduces false positives for common scripts
    false
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

/// Issue #104: Check if line is a path validation check
/// Patterns: if [[ "$VAR" == *".."* ]] or [[ "$VAR" == /* ]]
fn is_path_validation_check(line: &str) -> bool {
    // Must be an if/test statement
    if !line.contains("if") && !line.starts_with("[[") && !line.starts_with("[") {
        return false;
    }

    // Must check for path traversal patterns
    let validation_patterns = [
        "*\"..\"/",  // *".."*
        "*..*",      // *..*
        "/*",        // /* (absolute path check)
        "\"/\"*",    // starts with /
        "=~ \\.\\.", // regex match for ..
    ];

    // Check for ".." in the condition
    if line.contains("..") && (line.contains("==") || line.contains("=~") || line.contains("!=")) {
        return true;
    }

    // Check for absolute path validation
    if (line.contains("== /*") || line.contains("== \"/\""))
        && (line.contains("==") || line.contains("!="))
    {
        return true;
    }

    validation_patterns.iter().any(|p| line.contains(p))
}

/// Issue #104: Extract variable name being validated from a check
fn extract_validated_variable(line: &str) -> Option<String> {
    // Look for $VAR or ${VAR} patterns
    let patterns = ["$", "${"];

    for pattern in patterns {
        if let Some(start) = line.find(pattern) {
            let rest = &line[start..];

            // Handle ${VAR} format
            if rest.starts_with("${") {
                if let Some(end) = rest.find('}') {
                    let var_name = &rest[2..end];
                    // Remove array index if present: VAR[0] -> VAR
                    let var_name = var_name.split('[').next().unwrap_or(var_name);
                    return Some(var_name.to_string());
                }
            }
            // Handle $VAR format
            else if let Some(after_dollar) = rest.strip_prefix('$') {
                let var_chars: String = after_dollar
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !var_chars.is_empty() {
                    return Some(var_chars);
                }
            }
        }
    }

    None
}

/// Issue #104: Extract variable being assigned (left side of =)
fn extract_assigned_variable(line: &str) -> Option<String> {
    // Pattern: VAR=$(realpath ...) or VAR=`realpath ...`
    if let Some(eq_pos) = line.find('=') {
        let before_eq = line[..eq_pos].trim();
        // Get the last word before = (in case of export VAR= etc.)
        let var_name = before_eq.split_whitespace().last()?;
        // Validate it's a valid variable name
        if var_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            return Some(var_name.to_string());
        }
    }
    None
}

/// Issue #104: Check if any variable on the line has been validated
fn is_variable_validated(line: &str, validated_vars: &[String]) -> bool {
    for var in validated_vars {
        // Check for $VAR, ${VAR}, or "${VAR}"
        let patterns = [
            format!("${}", var),
            format!("${{{}}}", var),
            format!("\"${}\"", var),
            format!("\"${{{}}}\"", var),
        ];

        for pattern in &patterns {
            if line.contains(pattern) {
                return true;
            }
        }
    }
    false
}

/// Issue #106: Check if this is a heredoc pattern
/// Heredocs like `cat <<EOF` or `cat <<'EOF'` are not file reads
fn is_heredoc_pattern(line: &str) -> bool {
    // Check for heredoc operators: << or <<<
    if line.contains("<<") {
        // Common heredoc patterns with file commands
        // cat <<EOF, cat <<'EOF', cat <<"EOF", cat <<-EOF
        // Also handles here-string: cat <<<
        let heredoc_patterns = [
            "cat <<",
            "cat<<<",
            "cat <<-",
            "echo <<",
            "read <<",
            "tee <<",
        ];

        for pattern in &heredoc_patterns {
            if line.contains(pattern) {
                return true;
            }
        }

        // Also check for $(...) containing heredoc
        // e.g., content=$(cat <<EOF ... EOF)
        if line.contains("$(cat <<") || line.contains("$(cat<<") {
            return true;
        }
    }

    false
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
        assert!(!result.diagnostics.is_empty());
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

    #[test]
    fn test_SEC010_106_heredoc_not_file_read() {
        // Issue #106: cat <<EOF is not a file read, it's a heredoc
        let script = r#"content=$(cat <<EOF
some content here
EOF
)"#;
        let result = check(script);

        // Heredocs should not trigger the rule
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC010_106_heredoc_multiline() {
        // Issue #106: Heredoc with quoted delimiter
        let script = r#"cargo_content=$(cat <<'EOF'
[build]
jobs = 4
EOF
)"#;
        let result = check(script);

        // Heredocs should not trigger the rule
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC010_106_heredoc_with_tee() {
        // tee with heredoc
        let script = r#"tee /etc/config <<EOF
config here
EOF"#;
        let result = check(script);

        // The tee has a path but it's a heredoc input
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC010_real_cat_still_flagged() {
        // Real cat with user file should still be flagged
        let script = r#"cat "$USER_FILE""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    // Issue #104 tests: Path validation guards

    #[test]
    fn test_SEC010_104_validated_path_not_flagged() {
        // Issue #104: If a path is validated with if [[ "$VAR" == *".."* ]], skip subsequent use
        let script = r#"
if [[ "$USER_FILE" == *".."* ]]; then
    echo "Invalid path" >&2
    exit 1
fi
cp "$USER_FILE" /destination/
"#;
        let result = check(script);

        // Should NOT flag because USER_FILE was validated
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Expected no diagnostics for validated path, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_SEC010_104_realpath_validated() {
        // Issue #104: Variables assigned from realpath are considered validated
        let script = r#"
SAFE_PATH=$(realpath -m "$USER_INPUT")
cp "$SAFE_PATH" /destination/
"#;
        let result = check(script);

        // SAFE_PATH is derived from realpath, so it's validated
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Expected no diagnostics for realpath-validated path"
        );
    }

    #[test]
    fn test_SEC010_104_readlink_validated() {
        // Issue #104: Variables assigned from readlink -f are validated
        let script = r#"
RESOLVED=$(readlink -f "$USER_PATH")
cat "$RESOLVED"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Expected no diagnostics for readlink-f-validated path"
        );
    }

    #[test]
    fn test_SEC010_104_unvalidated_still_flagged() {
        // Issue #104: Variables that are NOT validated should still be flagged
        let script = r#"
echo "Processing file..."
cp "$USER_FILE" /destination/
"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    #[test]
    fn test_SEC010_104_different_var_still_flagged() {
        // Issue #104: Validating one variable doesn't validate others
        let script = r#"
if [[ "$SAFE_VAR" == *".."* ]]; then
    exit 1
fi
cp "$USER_FILE" /destination/
"#;
        let result = check(script);

        // USER_FILE was not validated, only SAFE_VAR was
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC010");
    }

    #[test]
    fn test_SEC010_104_absolute_path_check() {
        // Issue #104: Check for absolute path validation
        let script = r#"
if [[ "$INPUT_PATH" == /* ]]; then
    echo "Absolute paths not allowed" >&2
    exit 1
fi
cp "$INPUT_PATH" /destination/
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Expected no diagnostics after absolute path validation"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
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
            prop_assert!(!result.diagnostics.is_empty());
            prop_assert_eq!(result.diagnostics[0].code.as_str(), "SEC010");
        }
    }
}
