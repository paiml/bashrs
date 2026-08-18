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

use crate::linter::taint::{self, TaintKind, TaintMap};
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Commands that operate on file paths and are path traversal vectors
const FILE_COMMANDS: &[&str] = &[
    "cat", "rm", "cp", "mv", "source", ".", "less", "more", "head", "tail", "chmod", "chown", "ln",
    "tar", "unzip",
];

/// Check a single line for path traversal, returning a diagnostic if found
fn check_line(line: &str, line_num: usize, taint: &TaintMap) -> Option<Diagnostic> {
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

        // GH-227: only report when the interpolated path can actually be
        // influenced from outside the script. `cat "/data/$x"` where `x="a"`
        // is not a traversal risk.
        if has_variable_in_path(after_cmd)
            && taint.path_taint(line_num, after_cmd) != TaintKind::Clean
        {
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
    let taint = taint::analyze(source);
    // GH-227: a quoted heredoc body is data, not shell — see sec010::check.
    let heredoc_body = crate::linter::heredoc::quoted_heredoc_lines(source);

    for (line_num, line) in source.lines().enumerate() {
        if heredoc_body.contains(&(line_num + 1)) {
            continue;
        }
        if let Some(diag) = check_line(line, line_num, &taint) {
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

    // ------------------------------------------------------------------
    // GH-227: SEC014 had no dataflow at all — any `<file-cmd> …/"$VAR"…`
    // fired, including paths built entirely from string literals.
    // ------------------------------------------------------------------

    #[test]
    fn test_GH227_sec014_literal_var_in_path_not_flagged() {
        let script = "#!/bin/bash\nx=\"a\"\ncat \"/data/$x\"\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "got: {:?}", result.diagnostics);
    }

    #[test]
    fn test_GH227_sec014_literal_out_dir_not_flagged() {
        let script = "#!/bin/bash\nset -euo pipefail\nOUT_DIR=\"build/results\"\nmkdir -p \"$OUT_DIR\"\ncat > \"$OUT_DIR/report.md\" <<'INNER'\nhello\nINNER\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "got: {:?}", result.diagnostics);
    }

    #[test]
    fn test_GH227_sec014_positional_in_path_flagged() {
        let script = r#"cat "/data/$1""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC014");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_GH227_sec014_env_var_in_path_still_flagged() {
        // Never assigned in this file => unproven external influence => still reported.
        let script = r#"cat "/data/$USER_INPUT""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_GH227_sec014_guarded_positional_not_flagged() {
        let script = "#!/bin/bash\ncase \"$1\" in *..*|/*) exit 2 ;; esac\ncat \"/data/$1\"\n";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "got: {:?}", result.diagnostics);
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
