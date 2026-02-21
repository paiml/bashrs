//! SEC016: Missing Input Validation
//!
//! **Rule**: Detect use of $1, $2, etc. positional parameters without validation
//!
//! **Why this matters**:
//! Using positional parameters directly in file operations, commands, or paths
//! without validation can lead to injection attacks, path traversal, or
//! unexpected behavior with malicious input.
//!
//! ## Examples
//!
//! Bad:
//! ```bash
//! rm -rf "$1"
//! eval "$1"
//! mysql -e "$1"
//! ```
//!
//! Good:
//! ```bash
//! # Validate input first
//! [ -z "$1" ] && echo "Usage: $0 <dir>" && exit 1
//! [ -d "$1" ] || exit 1
//! rm -rf "$1"
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Dangerous commands that should never use unvalidated input
const DANGEROUS_WITH_INPUT: &[&str] = &["eval", "exec", "rm -rf", "dd "];

/// Validation patterns that indicate positional parameters are checked
const VALIDATION_PATTERNS: &[&str] = &[
    "[ -z \"$1\"",
    "[ -z \"$2\"",
    "[[ -z \"$1\"",
    "[[ -z \"$2\"",
    "[ -n \"$1\"",
    "[ -d \"$1\"",
    "[ -f \"$1\"",
    "${1:?",
    "${1:-",
    "if [ $# ",
    "if [[ $# ",
    "getopts",
    "shift",
];

/// Positional parameter references
const POSITIONAL_PARAMS: &[&str] = &[
    "$1", "$2", "$3", "\"$1\"", "\"$2\"", "\"$3\"", "${1", "${2", "${3",
];

/// Check whether the script has existing input validation
fn has_input_validation(lines: &[&str]) -> bool {
    lines.iter().any(|line| {
        let trimmed = line.trim();
        VALIDATION_PATTERNS.iter().any(|pat| trimmed.contains(pat))
    })
}

/// Check whether a line uses positional parameters
fn uses_positional_param(line: &str) -> bool {
    POSITIONAL_PARAMS.iter().any(|p| line.contains(p))
}

/// Check a single line for dangerous command + positional parameter usage
fn check_line(line: &str, line_num: usize) -> Option<Diagnostic> {
    let trimmed = line.trim();

    if trimmed.starts_with('#') || trimmed.is_empty() {
        return None;
    }

    for dangerous_cmd in DANGEROUS_WITH_INPUT {
        if trimmed.contains(dangerous_cmd) && uses_positional_param(trimmed) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            return Some(Diagnostic::new(
                "SEC016",
                Severity::Warning,
                format!(
                    "Dangerous command '{}' uses unvalidated positional parameter - validate input first",
                    dangerous_cmd.trim()
                ),
                span,
            ));
        }
    }

    None
}

/// Check for missing input validation on positional parameters
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    if has_input_validation(&lines) {
        return result;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if let Some(diag) = check_line(line, line_num) {
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sec016_detects_rm_rf_with_param() {
        let script = "rm -rf \"$1\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC016");
    }

    #[test]
    fn test_sec016_detects_eval_with_param() {
        let script = "eval \"$1\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec016_detects_exec_with_param() {
        let script = "exec $1";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec016_safe_with_validation() {
        let script = r#"
[ -z "$1" ] && echo "Usage: $0 <dir>" && exit 1
rm -rf "$1"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec016_safe_with_getopts() {
        let script = r#"
while getopts "d:" opt; do
  case $opt in d) DIR="$OPTARG" ;; esac
done
rm -rf "$1"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec016_safe_no_params() {
        let script = "rm -rf /tmp/build";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec016_ignores_comments() {
        let script = "# eval \"$1\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec016_empty() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec016_dd_with_param() {
        let script = "dd if=$1 of=/dev/sda";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec016_safe_with_argcount_check() {
        let script = r#"
if [ $# -lt 1 ]; then echo "Need arg"; exit 1; fi
rm -rf "$1"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]

        #[test]
        fn prop_sec016_never_panics(s in ".*") {
            let _ = check(&s);
        }
    }
}
