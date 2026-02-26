//! SEC010 Pure Logic - Path traversal detection
//!
//! Extracted for EXTREME TDD testability.

use regex::Regex;

/// File operation commands that are path traversal vectors
pub const FILE_OPS: &[&str] = &["cp", "mv", "cat", "tar", "unzip", "rm", "mkdir", "cd", "ln"];

/// Patterns that indicate potential path traversal
pub const TRAVERSAL_PATTERNS: &[&str] = &["..", "../", "/.."];

/// Known-safe patterns that should not trigger SEC010
pub const SAFE_VAR_PATTERNS: &[&str] = &[
    "$PWD",
    "${PWD}",
    "$HOME",
    "${HOME}",
    "$TMPDIR",
    "${TMPDIR}",
    "BASH_SOURCE",
    "dirname",
    "XDG_",
];

/// Patterns suggesting untrusted user input
pub const USER_INPUT_PATTERNS: &[&str] = &[
    "USER",
    "INPUT",
    "UPLOAD",
    "ARCHIVE",
    "UNTRUSTED",
    "EXTERNAL",
    "REMOTE",
    "ARG",
    "NAME",
    "FILE",
    "PATH",
    "DIR",
];

/// Check if line is a comment
pub fn is_comment(line: &str) -> bool {
    line.trim().starts_with('#')
}

/// Check if line contains heredoc pattern (cat <<EOF is not a file read)
pub fn is_heredoc_pattern(line: &str) -> bool {
    line.contains("<<")
        && (line.contains("EOF") || line.contains("'EOF'") || line.contains("\"EOF\""))
}

/// Check if line contains a path validation check
pub fn is_path_validation_check(line: &str) -> bool {
    // Pattern: if [[ "$VAR" == *".."* ]] or if [[ "$VAR" == /* ]]
    (line.contains("[[") || line.contains("[ "))
        && (line.contains("\"..\"") || line.contains("*\"..*") || line.contains("/*"))
}

/// Extract variable being validated from a validation check
pub fn extract_validated_variable(line: &str) -> Option<String> {
    #[allow(clippy::unwrap_used)] // Compile-time regex
    static VAR_PATTERN: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap());

    VAR_PATTERN.captures(line).map(|cap| cap[1].to_string())
}

/// Extract assigned variable from realpath/readlink validation
pub fn extract_assigned_variable(line: &str) -> Option<String> {
    #[allow(clippy::unwrap_used)] // Compile-time regex
    static ASSIGN_PATTERN: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"([A-Za-z_][A-Za-z0-9_]*)=").unwrap());

    ASSIGN_PATTERN.captures(line).map(|cap| cap[1].to_string())
}

/// Check if line uses a validated variable
pub fn is_variable_validated(line: &str, validated_vars: &[String]) -> bool {
    for var in validated_vars {
        if line.contains(&format!("${}", var)) || line.contains(&format!("${{{}}}", var)) {
            return true;
        }
    }
    false
}

/// Find a command in a line with word boundary detection
pub fn find_command(line: &str, cmd: &str) -> Option<usize> {
    if let Some(pos) = line.find(cmd) {
        let before_ok = if pos == 0 {
            true
        } else {
            line.chars()
                .nth(pos - 1)
                .is_some_and(|c| matches!(c, ' ' | '\t' | ';' | '&' | '|' | '(' | '\n'))
        };

        let after_idx = pos + cmd.len();
        let after_ok = if after_idx >= line.len() {
            true
        } else {
            line.chars()
                .nth(after_idx)
                .is_some_and(|c| matches!(c, ' ' | '\t' | ';' | '&' | '|' | ')'))
        };

        if before_ok && after_ok {
            return Some(pos);
        }
    }
    None
}

/// Check if line contains unvalidated variable in file operation
pub fn contains_unvalidated_variable(line: &str) -> bool {
    if !line.contains('$') {
        return false;
    }

    // Skip known-safe patterns
    for safe_pattern in SAFE_VAR_PATTERNS {
        if line.contains(safe_pattern) {
            return false;
        }
    }

    // Script directory parent (..) with BASH_SOURCE is intentional
    if line.contains("dirname") && line.contains("..") {
        return false;
    }

    // Check for user input patterns
    for pattern in USER_INPUT_PATTERNS {
        if line.contains(pattern) && (line.contains('$') || line.contains("${")) {
            // Skip if it's PATH environment variable
            if *pattern == "PATH" && (line.contains("$PATH") || line.contains("${PATH}")) {
                continue;
            }
            return true;
        }
    }

    false
}

/// Check if line contains any file operation command
pub fn contains_file_operation(line: &str) -> bool {
    FILE_OPS.iter().any(|cmd| find_command(line, cmd).is_some())
}

/// Check if line is a validation context (checking for ..)
pub fn is_validation_context(line: &str) -> bool {
    line.contains("==") || line.contains("!=") || line.contains("-n") || line.contains("-z")
}

/// Check if line ends a validation block
pub fn is_validation_block_end(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "fi" || trimmed.starts_with("fi ") || trimmed.starts_with("fi;")
}

/// Check if line is a validation guard (exit/return)
pub fn is_validation_guard(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains("exit") || trimmed.contains("return")
}

/// Check if line contains realpath or readlink validation
pub fn is_realpath_validation(line: &str) -> bool {
    line.contains("realpath") || line.contains("readlink -f")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== IS COMMENT =====

    #[test]
    fn test_is_comment_true() {
        assert!(is_comment("# comment"));
        assert!(is_comment("  # indented"));
    }

    #[test]
    fn test_is_comment_false() {
        assert!(!is_comment("echo hello # inline"));
        assert!(!is_comment("cp file dest"));
    }

    // ===== IS HEREDOC PATTERN =====

    #[test]
    fn test_is_heredoc_pattern_true() {
        assert!(is_heredoc_pattern("cat <<EOF"));
        assert!(is_heredoc_pattern("cat <<'EOF'"));
    }

    #[test]
    fn test_is_heredoc_pattern_false() {
        assert!(!is_heredoc_pattern("cat file.txt"));
    }

    // ===== IS PATH VALIDATION CHECK =====

    #[test]
    fn test_is_path_validation_check_true() {
        assert!(is_path_validation_check(
            r#"if [[ "$VAR" == *".."* ]]; then"#
        ));
    }

    #[test]
    fn test_is_path_validation_check_false() {
        assert!(!is_path_validation_check("cp $FILE dest"));
    }

    // ===== EXTRACT VALIDATED VARIABLE =====

    #[test]
    fn test_extract_validated_variable() {
        let result = extract_validated_variable(r#"if [[ "$USER_PATH" == *".."* ]]; then"#);
        assert_eq!(result, Some("USER_PATH".into()));
    }

    #[test]
    fn test_extract_validated_variable_none() {
        let result = extract_validated_variable("echo hello");
        assert!(result.is_none());
    }

    // ===== EXTRACT ASSIGNED VARIABLE =====

    #[test]
    fn test_extract_assigned_variable() {
        let result = extract_assigned_variable("SAFE_PATH=$(realpath -m \"$FILE\")");
        assert_eq!(result, Some("SAFE_PATH".into()));
    }

    // ===== IS VARIABLE VALIDATED =====

    #[test]
    fn test_is_variable_validated_true() {
        let validated = vec!["SAFE_PATH".into()];
        assert!(is_variable_validated("cp $SAFE_PATH dest", &validated));
        assert!(is_variable_validated("cp ${SAFE_PATH} dest", &validated));
    }

    #[test]
    fn test_is_variable_validated_false() {
        let validated = vec!["SAFE_PATH".into()];
        assert!(!is_variable_validated("cp $OTHER_PATH dest", &validated));
    }

    // ===== FIND COMMAND =====

    #[test]
    fn test_find_command_found() {
        assert_eq!(find_command("cp file dest", "cp"), Some(0));
        assert_eq!(find_command("  cp file dest", "cp"), Some(2));
    }

    #[test]
    fn test_find_command_not_word() {
        // "cpr" should not match "cp"
        assert_eq!(find_command("cpr file", "cp"), None);
    }

    // ===== CONTAINS UNVALIDATED VARIABLE =====

    #[test]
    fn test_contains_unvalidated_variable_user_input() {
        assert!(contains_unvalidated_variable("cp $USER_FILE dest"));
        assert!(contains_unvalidated_variable("cat $INPUT_PATH"));
    }

    #[test]
    fn test_contains_unvalidated_variable_safe() {
        assert!(!contains_unvalidated_variable("cp $PWD/file dest"));
        assert!(!contains_unvalidated_variable("cd $HOME"));
    }

    #[test]
    fn test_contains_unvalidated_variable_no_var() {
        assert!(!contains_unvalidated_variable("cp file dest"));
    }

    // ===== CONTAINS FILE OPERATION =====

    #[test]
    fn test_contains_file_operation_true() {
        assert!(contains_file_operation("cp file dest"));
        assert!(contains_file_operation("cat file.txt"));
    }

    #[test]
    fn test_contains_file_operation_false() {
        assert!(!contains_file_operation("echo hello"));
    }

    // ===== IS VALIDATION CONTEXT =====

    #[test]
    fn test_is_validation_context_true() {
        assert!(is_validation_context(r#"if [[ "$x" == "y" ]]; then"#));
        assert!(is_validation_context("if [ -n \"$x\" ]; then"));
    }

    #[test]
    fn test_is_validation_context_false() {
        assert!(!is_validation_context("cp file dest"));
    }

    // ===== IS VALIDATION BLOCK END =====

    #[test]
    fn test_is_validation_block_end_true() {
        assert!(is_validation_block_end("fi"));
        assert!(is_validation_block_end("fi;"));
        assert!(is_validation_block_end("  fi  "));
    }

    #[test]
    fn test_is_validation_block_end_false() {
        assert!(!is_validation_block_end("if"));
    }

    // ===== IS VALIDATION GUARD =====

    #[test]
    fn test_is_validation_guard_true() {
        assert!(is_validation_guard("exit 1"));
        assert!(is_validation_guard("return 1"));
    }

    #[test]
    fn test_is_validation_guard_false() {
        assert!(!is_validation_guard("echo error"));
    }

    // ===== IS REALPATH VALIDATION =====

    #[test]
    fn test_is_realpath_validation_true() {
        assert!(is_realpath_validation("SAFE=$(realpath -m \"$x\")"));
        assert!(is_realpath_validation("SAFE=$(readlink -f \"$x\")"));
    }

    #[test]
    fn test_is_realpath_validation_false() {
        assert!(!is_realpath_validation("echo $PATH"));
    }
}
