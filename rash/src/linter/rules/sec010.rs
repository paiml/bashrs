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

/// Track validation patterns on a line, updating validated_vars and in_validation_block.
/// Returns true if the line was a validation-related line and should be skipped.
fn track_validation_patterns(
    trimmed: &str,
    validated_vars: &mut Vec<String>,
    in_validation_block: &mut bool,
) -> bool {
    if is_path_validation_check(trimmed) {
        if let Some(var) = extract_validated_variable(trimmed) {
            validated_vars.push(var);
        }
        *in_validation_block = true;
        return true;
    }

    if trimmed.contains("realpath") || trimmed.contains("readlink -f") {
        if let Some(var) = extract_assigned_variable(trimmed) {
            validated_vars.push(var);
        }
        return true;
    }

    if is_validation_function_call(trimmed) {
        if let Some(var) = extract_function_argument_variable(trimmed) {
            validated_vars.push(var);
        }
        return true;
    }

    false
}

/// Check file operations for path traversal risks with unvalidated variables
fn check_file_ops(line: &str, line_num: usize, validated_vars: &[String], result: &mut LintResult) {
    for file_op in FILE_OPS {
        if let Some(cmd_col) = find_command(line, file_op) {
            if is_variable_validated(line, validated_vars) {
                continue;
            }
            if contains_unvalidated_variable(line, file_op) {
                let span = Span::new(line_num + 1, cmd_col + 1, line_num + 1, line.len());
                let diag = Diagnostic::new(
                    "SEC010",
                    Severity::Error,
                    format!("Path traversal risk in {} - validate paths don't contain '..' or start with '/'", file_op),
                    span,
                );
                result.add(diag);
                break;
            }
        }
    }
}

/// Check for explicit traversal patterns (e.g. ".." in literal paths)
fn check_traversal_patterns(line: &str, line_num: usize, result: &mut LintResult) {
    for pattern in TRAVERSAL_PATTERNS {
        if line.contains(pattern) && contains_file_operation(line) {
            if line.contains("BASH_SOURCE") || line.contains("dirname") {
                continue;
            }
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

/// Check for path traversal vulnerabilities
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut validated_vars: Vec<String> = Vec::new();
    let mut in_validation_block = false;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') || is_heredoc_pattern(line) {
            continue;
        }

        if track_validation_patterns(trimmed, &mut validated_vars, &mut in_validation_block) {
            continue;
        }

        if trimmed == "fi" || trimmed.starts_with("fi ") || trimmed.starts_with("fi;") {
            in_validation_block = false;
        }

        if in_validation_block && (trimmed.contains("exit") || trimmed.contains("return")) {
            continue;
        }

        check_file_ops(line, line_num, &validated_vars, &mut result);
        check_traversal_patterns(line, line_num, &mut result);
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
            matches!(char_before, Some(' ' | '\t' | ';' | '&' | '|' | '(' | '\n'))
        };

        let after_idx = pos + cmd.len();
        let after_ok = if after_idx >= line.len() {
            true
        } else {
            let char_after = line.chars().nth(after_idx);
            matches!(char_after, Some(' ' | '\t' | ';' | '&' | '|' | ')'))
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
    if !line.contains("if") && !line.starts_with("[[") && !line.starts_with('[') {
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
        if var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
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

/// Issue #127: Check if this is a validation function call
/// Patterns: validate_path, validate_input, check_path, sanitize_path, etc.
fn is_validation_function_call(line: &str) -> bool {
    let validation_prefixes = [
        "validate_",
        "check_",
        "verify_",
        "sanitize_",
        "clean_",
        "safe_",
        "is_valid_",
        "is_safe_",
        "assert_",
    ];

    let line_lower = line.to_lowercase();

    // Check if line starts with a validation function call (not a definition)
    // Skip function definitions: validate_path() { ... }
    if line.contains("()") && (line.contains('{') || line.trim().ends_with("()")) {
        return false;
    }

    for prefix in validation_prefixes {
        if line_lower.contains(prefix) {
            // Make sure it's a function call, not just containing the word
            // Should have a variable argument after it
            if line.contains('$') {
                return true;
            }
        }
    }

    false
}

/// Issue #127: Extract variable passed to a validation function
/// Pattern: validate_path "$VAR" or validate_input "${VAR}"
fn extract_function_argument_variable(line: &str) -> Option<String> {
    // Look for quoted variable arguments: "$VAR" or "${VAR}"
    // Find the first variable after a function call

    // Find position of first $
    let dollar_pos = line.find('$')?;
    let rest = &line[dollar_pos..];

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

    None
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
            "cat <<", "cat<<<", "cat <<-", "echo <<", "read <<", "tee <<",
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
#[path = "sec010_tests_ext.rs"]
mod tests_ext;
