//! SEC006: Unsafe Temporary File Creation
//!
//! **Rule**: Detect predictable temporary file names (race condition vulnerability)
//!
//! **Why this matters**:
//! Predictable temp file names enable symlink attacks and race conditions.
//! Attackers can predict the filename and create malicious symlinks.
//!
//! **Auto-fix**: Safe (suggest mktemp)
//!
//! ## Examples
//!
//! ❌ **PREDICTABLE TEMP FILES** (race condition):
//! ```bash
//! TMPFILE="/tmp/myapp.$$"
//! TMPFILE="/tmp/script_temp"
//! TMP_DIR="/tmp/build_cache"
//! ```
//!
//! ✅ **SECURE TEMP FILES** (auto-fixable):
//! ```bash
//! TMPFILE="$(mktemp)"
//! TMPDIR="$(mktemp -d)"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check a single line for predictable /tmp/ assignment and emit diagnostic if found
fn check_tmp_assignment(line: &str, line_num: usize, result: &mut LintResult) {
    if !line.contains("/tmp/") || line.contains("mktemp") {
        return;
    }
    let Some(eq_pos) = line.find('=') else { return };
    let after_eq = &line[eq_pos + 1..].trim_start();

    if !after_eq.starts_with("\"/tmp/") && !after_eq.starts_with("'/tmp/") {
        return;
    }
    if let Some(col) = line.find("/tmp/") {
        let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 6);
        let diag = Diagnostic::new(
            "SEC006",
            Severity::Warning,
            "Unsafe temp file - use mktemp for secure random names",
            span,
        )
        .with_fix(Fix::new("$(mktemp)"));
        result.add(diag);
    }
}

/// Check for unsafe temporary file creation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        check_tmp_assignment(line, line_num, &mut result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC006_detects_predictable_tmp() {
        let script = r#"TMPFILE="/tmp/myapp.$$""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC006");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_SEC006_detects_static_tmp() {
        let script = "TMP_DIR=\"/tmp/build_cache\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC006_no_warning_mktemp() {
        let script = "TMPFILE=\"$(mktemp)\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC006_no_warning_mktemp_dir() {
        let script = "TMPDIR=\"$(mktemp -d)\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC006_provides_fix() {
        let script = "TMPFILE=\"/tmp/script_temp\"";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "$(mktemp)");
    }

    // ===== Mutation Coverage Tests - Following SEC001 pattern (100% kill rate) =====

    #[test]
    fn test_mutation_sec006_tmp_start_col_exact() {
        // MUTATION: Line 44:29 - replace + with * in line_num + 1
        // MUTATION: Line 45:29 - replace + with * in col + 1
        let bash_code = r#"TMPFILE="/tmp/myapp.$$""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "/tmp/" starts at column 9 (TMPFILE=")
        assert_eq!(
            span.start_col, 10,
            "Start column must use col + 1, not col * 1"
        );
    }

    #[test]
    fn test_mutation_sec006_tmp_end_col_exact() {
        // MUTATION: Line 47:29 - replace + with * in col + 6
        // MUTATION: Line 47:29 - replace + with - in col + 6
        let bash_code = r#"TMPFILE="/tmp/myapp.$$""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "/tmp/" is 5 chars, ends at col + 6
        assert_eq!(
            span.end_col, 15,
            "End column must be col + 6, not col * 6 or col - 6"
        );
    }

    #[test]
    fn test_mutation_sec006_line_num_calculation() {
        // MUTATION: Line 44:29 - replace + with * in line_num + 1
        // Tests line number calculation for multiline input
        let bash_code = "# comment\nTMPFILE=\"/tmp/script_temp\"";
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
    fn test_mutation_sec006_column_with_leading_whitespace() {
        // Tests column calculations with offset
        let bash_code = r#"    TMP_DIR="/tmp/build_cache""#;
        let result = check(bash_code);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        // "/tmp/" starts after leading whitespace
        assert!(span.start_col > 10, "Must account for leading whitespace");
        assert_eq!(
            span.end_col - span.start_col,
            5,
            "Span should cover /tmp/ (5 chars)"
        );
    }
}
