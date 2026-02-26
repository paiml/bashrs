//! SEC014: Path Traversal Vulnerabilities
//!
//! **Rule**: Detect path traversal vulnerabilities in shell scripts
//!
//! **Why this matters**:
//! Using unsanitized user input in file paths allows attackers to access files
//! outside intended directories via `../` sequences.
//!
//! ## Examples
//!
//! Bad:
//! ```bash
//! cat "/data/$USER_INPUT"
//! rm -rf "/uploads/$FILENAME"
//! source "$CONFIG_DIR/$MODULE"
//! ```
//!
//! Good:
//! ```bash
//! # Validate input doesn't contain path traversal
//! case "$USER_INPUT" in
//!   *..* ) echo "Invalid path"; exit 1 ;;
//! esac
//! realpath --relative-to=/data "/data/$USER_INPUT"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Commands that operate on file paths and are path traversal vectors
const FILE_COMMANDS: &[&str] = &[
    "cat", "rm", "cp", "mv", "source", ".", "less", "more", "head", "tail", "chmod", "chown", "ln",
    "tar", "unzip",
];

/// Check a single line for path traversal, returning a diagnostic if found
fn check_line(line: &str, line_num: usize) -> Option<Diagnostic> {
    let trimmed = line.trim();

    if trimmed.starts_with('#') || trimmed.is_empty() {
        return None;
    }

    for cmd in FILE_COMMANDS {
        if !contains_command(trimmed, cmd) {
            continue;
        }

        let cmd_pos = trimmed.find(cmd)?;
        let after_cmd = &trimmed[cmd_pos + cmd.len()..];

        if has_variable_in_path(after_cmd) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            return Some(Diagnostic::new(
                "SEC014",
                Severity::Warning,
                format!(
                    "Potential path traversal: {} with variable in path - validate input doesn't contain '..'",
                    cmd
                ),
                span,
            ));
        }
    }

    None
}

/// Check for path traversal vulnerabilities
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(diag) = check_line(line, line_num) {
            result.add(diag);
        }
    }

    result
}

/// Check if a command appears as a word in the line
fn contains_command(line: &str, cmd: &str) -> bool {
    // Handle "." specially (source alias)
    if cmd == "." {
        return line.starts_with(". ")
            || line.contains(" . ")
            || line.contains("; . ")
            || line.contains("&& . ");
    }

    if let Some(pos) = line.find(cmd) {
        let before_ok = pos == 0 || {
            let c = line.as_bytes().get(pos - 1);
            matches!(c, Some(b' ' | b'\t' | b';' | b'|' | b'&' | b'('))
        };
        let after_idx = pos + cmd.len();
        let after_ok = after_idx >= line.len() || {
            let c = line.as_bytes().get(after_idx);
            matches!(c, Some(b' ' | b'\t' | b';' | b'|' | b'&' | b')'))
        };
        before_ok && after_ok
    } else {
        false
    }
}

/// Check if a path component contains a bare variable reference (not command substitution)
fn has_bare_variable(part: &str) -> bool {
    let trimmed = part.trim_matches('"').trim_matches('\'');
    let bytes = trimmed.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] != b'$' || trimmed[..i].ends_with('\\') {
            continue;
        }
        // $( is command substitution — safe, skip
        let is_cmd_sub = i + 1 < bytes.len() && bytes[i + 1] == b'(';
        if !is_cmd_sub {
            return true;
        }
    }
    false
}

/// Check if the argument portion contains a variable interpolated into a path
fn has_variable_in_path(args: &str) -> bool {
    let has_path_sep = args.contains('/');
    let has_variable = args.contains('$') && !args.contains("\\$");

    if !has_path_sep || !has_variable {
        return false;
    }

    args.split('/')
        .any(|part| part.contains('$') && has_bare_variable(part))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sec014_detects_cat_with_variable_path() {
        let script = r#"cat "/data/$USER_INPUT""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC014");
    }

    #[test]
    fn test_sec014_detects_rm_with_variable_path() {
        let script = r#"rm -rf "/uploads/$FILENAME""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec014_detects_source_with_variable() {
        let script = r#"source "$CONFIG_DIR/$MODULE""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec014_safe_no_variable() {
        let script = "cat /etc/hosts";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec014_safe_command_substitution() {
        let script = r#"cat "$(realpath /data/file)""#;
        let result = check(script);
        // Command substitution without direct variable in path is safer
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec014_ignores_comments() {
        let script = r#"# cat "/data/$USER_INPUT""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec014_empty_input() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec014_cp_with_variable() {
        let script = r#"cp "$SRC/$FILE" /dest/"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]

        #[test]
        fn prop_sec014_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_sec014_no_variables_is_safe(
            cmd in "(cat|rm|cp|mv|head|tail)",
            path in "/[a-z]{1,10}/[a-z]{1,10}",
        ) {
            let script = format!("{} {}", cmd, path);
            let result = check(&script);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
