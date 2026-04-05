//! SEC011: Missing Input Validation Before Dangerous Operations
//!
//! **Rule**: Detect missing validation before dangerous operations
//!
//! **Why this matters**:
//! Without input validation, shell scripts can cause catastrophic damage:
//! - `rm -rf "$EMPTY_VAR"` → Deletes current directory
//! - `rm -rf "$VAR"` where `$VAR=/` → Deletes entire filesystem
//! - `chmod -R 777 "$DIR"` with invalid `$DIR` → Opens security holes
//! - SQL injection via unvalidated user input
//!
//! **Examples**:
//!
//! ❌ **DANGEROUS** (no validation):
//! ```bash
//! rm -rf "$BUILD_DIR"  # What if BUILD_DIR is empty or /?
//! chmod -R 777 "$DIR"  # What if DIR is unset?
//! ```
//!
//! ✅ **SAFE** (with validation):
//! ```bash
//! if [ -z "$BUILD_DIR" ] || [ "$BUILD_DIR" = "/" ]; then
//!   echo "Error: Invalid BUILD_DIR"
//!   exit 1
//! fi
//! rm -rf "$BUILD_DIR"
//! ```
//!
//! ## Detection Patterns
//!
//! This rule detects dangerous operations on variables without validation:
//! - `rm -rf "$VAR"` without checking if `$VAR` is empty or `/`
//! - `chmod -R 777 "$VAR"` without validation
//! - File operations on unvalidated paths
//!
//! ## Auto-fix
//!
//! This rule provides **suggestions** but not automatic fixes, because:
//! - Context-dependent validation logic
//! - Different operations need different validation
//! - Requires understanding of script intent

use crate::linter::LintResult;
use crate::linter::{Diagnostic, Severity, Span};

/// Issue #105: Known-safe environment variables that don't need validation
/// These are system-provided or set by the shell, not user input
const SAFE_ENV_VARS: &[&str] = &[
    // User and system info (set by shell/OS)
    "USER",
    "LOGNAME",
    "HOME",
    "SHELL",
    "PWD",
    "OLDPWD",
    "UID",
    "EUID",
    "PPID",
    "HOSTNAME",
    // Temp directories (controlled system paths)
    "TMPDIR",
    "TEMP",
    "TMP",
    // XDG directories (standard locations)
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_RUNTIME_DIR",
    // Common safe paths
    "PATH",
    "MANPATH",
    "LANG",
    "LC_ALL",
];

/// Issue #105: Check if a variable is a known-safe environment variable
fn is_safe_env_var(var_name: &str) -> bool {
    // Direct match
    if SAFE_ENV_VARS.contains(&var_name) {
        return true;
    }
    // XDG_* variables are generally safe (standard locations)
    if var_name.starts_with("XDG_") {
        return true;
    }
    false
}

/// Strip comments from a trimmed line and return the code portion
fn strip_comments(trimmed: &str) -> &str {
    if let Some(pos) = trimmed.find('#') {
        trimmed[..pos].trim()
    } else {
        trimmed
    }
}

/// Track validation patterns and update the validated vars set
fn track_validation(trimmed: &str, validated_vars: &mut std::collections::HashSet<String>) {
    if trimmed.starts_with("if ") && (trimmed.contains("[ -z") || trimmed.contains("[ -n")) {
        if let Some(var_name) = extract_validated_variable(trimmed) {
            validated_vars.insert(var_name);
        }
    }
}

/// Check a dangerous operation and emit diagnostic if variable is unvalidated
fn check_dangerous_op(
    var_name: &str,
    op_desc: &str,
    validated_vars: &std::collections::HashSet<String>,
    inline_validated: &std::collections::HashSet<String>,
    line_num: usize,
    line_len: usize,
    result: &mut LintResult,
) {
    if !validated_vars.contains(var_name) && !inline_validated.contains(var_name) {
        let span = Span::new(line_num + 1, 1, line_num + 1, line_len);
        let diag = Diagnostic::new(
            "SEC011",
            Severity::Error,
            format!(
                "Missing validation for '{}' before '{}' - {}",
                var_name,
                op_desc,
                match op_desc {
                    "rm -rf" => "could delete critical files if variable is empty or '/'",
                    "chmod -R 777" => "could expose sensitive files if variable is unset",
                    _ => "could change ownership of critical files if variable is unset",
                }
            ),
            span,
        );
        result.add(diag);
    }
}

/// Check for missing input validation before dangerous operations
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut validated_vars: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let code_only = strip_comments(trimmed);

        track_validation(trimmed, &mut validated_vars);

        let inline_validated = extract_inline_validated_vars(code_only);

        // Pattern: rm -rf "$VAR"
        if code_only.contains("rm") && code_only.contains("-rf") {
            if let Some(ref var_name) = extract_variable_from_rm(code_only) {
                if is_safe_env_var(var_name) {
                    continue;
                }
                check_dangerous_op(
                    var_name,
                    "rm -rf",
                    &validated_vars,
                    &inline_validated,
                    line_num,
                    line.len(),
                    &mut result,
                );
            }
        }

        // Pattern: chmod -R 777 "$VAR"
        if code_only.contains("chmod") && code_only.contains("-R") && code_only.contains("777") {
            if let Some(ref var_name) = extract_variable_from_chmod(code_only) {
                if is_safe_env_var(var_name) {
                    continue;
                }
                check_dangerous_op(
                    var_name,
                    "chmod -R 777",
                    &validated_vars,
                    &inline_validated,
                    line_num,
                    line.len(),
                    &mut result,
                );
            }
        }

        // Pattern: chown -R user:group "$VAR"
        if code_only.contains("chown") && code_only.contains("-R") {
            if let Some(ref var_name) = extract_variable_from_chown(code_only) {
                if is_safe_env_var(var_name) {
                    continue;
                }
                check_dangerous_op(
                    var_name,
                    "chown -R",
                    &validated_vars,
                    &inline_validated,
                    line_num,
                    line.len(),
                    &mut result,
                );
            }
        }
    }

    result
}

/// Issue #89: Extract variables validated inline with && chains
/// Example: `[ -n "$VAR" ] && [ -d "$VAR" ] && rm -rf "$VAR"` → {"VAR"}
fn extract_inline_validated_vars(line: &str) -> std::collections::HashSet<String> {
    let mut validated = std::collections::HashSet::new();

    // Look for [ -n "$VAR" ] or [ -d "$VAR" ] patterns before && rm/chmod/chown
    // This validates the variable is non-empty or is a directory

    // Find all [ -n "$VAR" ] patterns
    for pattern in ["[ -n \"$", "[ -d \"$", "[ -e \"$", "[ -f \"$"] {
        let mut search_start = 0;
        while let Some(start) = line[search_start..].find(pattern) {
            let abs_start = search_start + start + pattern.len();
            if let Some(end) = line[abs_start..].find('"') {
                let var_name = &line[abs_start..abs_start + end];
                // Only count as validated if this test precedes a dangerous operation via &&
                // Check if there's && after this test and before the dangerous operation
                let after_test = &line[abs_start + end..];
                if after_test.contains("&&")
                    && (after_test.contains("rm ")
                        || after_test.contains("chmod ")
                        || after_test.contains("chown "))
                {
                    validated.insert(var_name.to_string());
                }
            }
            search_start = abs_start;
        }
    }

    validated
}

/// Extract variable name from validation pattern
/// Example: `if [ -z "$VAR" ]` → Some("VAR")
fn extract_validated_variable(line: &str) -> Option<String> {
    // Match: [ -z "$VAR" ] or [ -n "$VAR" ]
    if let Some(start) = line.find("\"$") {
        if let Some(end) = line[start + 2..].find('"') {
            let var_name = &line[start + 2..start + 2 + end];
            return Some(var_name.to_string());
        }
    }
    None
}

/// Extract just the variable name (stop at first non-var character)
/// Example: "HOME/.cache" → "HOME"
fn extract_var_name_only(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Extract variable name from rm command
/// Example: `rm -rf "$BUILD_DIR"` → Some("BUILD_DIR")
/// Example: `rm -rf "$HOME/.cache"` → Some("HOME")
fn extract_variable_from_rm(line: &str) -> Option<String> {
    // Find "$VAR" pattern specifically after rm -rf
    // First find rm -rf or rm -r or rm --recursive
    let rm_pos = if let Some(pos) = line.find("rm -rf") {
        pos
    } else if let Some(pos) = line.find("rm -r ") {
        pos
    } else if let Some(pos) = line.find("rm --recursive") {
        pos
    } else {
        return None;
    };

    // Search for "$VAR" after the rm command
    let after_rm = &line[rm_pos..];
    if let Some(start) = after_rm.find("\"$") {
        let var_start = start + 2;
        let rest = &after_rm[var_start..];
        let var_name = extract_var_name_only(rest);
        if !var_name.is_empty() {
            return Some(var_name);
        }
    }
    None
}

/// Extract variable name from chmod command
/// Example: `chmod -R 777 "$DIR"` → Some("DIR")
/// Example: `chmod -R 777 "$HOME/.local"` → Some("HOME")
fn extract_variable_from_chmod(line: &str) -> Option<String> {
    // Find chmod command position first
    let chmod_pos = line.find("chmod")?;
    let after_chmod = &line[chmod_pos..];

    if let Some(start) = after_chmod.find("\"$") {
        let var_start = start + 2;
        let rest = &after_chmod[var_start..];
        let var_name = extract_var_name_only(rest);
        if !var_name.is_empty() {
            return Some(var_name);
        }
    }
    None
}

/// Extract variable name from chown command
/// Example: `chown -R user:group "$DIR"` → Some("DIR")
fn extract_variable_from_chown(line: &str) -> Option<String> {
    // Find chown command position first
    let chown_pos = line.find("chown")?;
    let after_chown = &line[chown_pos..];

    if let Some(start) = after_chown.find("\"$") {
        let var_start = start + 2;
        let rest = &after_chown[var_start..];
        let var_name = extract_var_name_only(rest);
        if !var_name.is_empty() {
            return Some(var_name);
        }
    }
    None
}

#[cfg(test)]
#[path = "sec011_tests_sec011_detec.rs"]
mod tests_extracted;
