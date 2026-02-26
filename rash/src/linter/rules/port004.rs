//! PORT004: Process substitution in POSIX sh
//!
//! **Rule**: Detect `<(...)` or `>(...)` process substitution in `#!/bin/sh` scripts
//!
//! **Why this matters**:
//! Process substitution (`<(...)` and `>(...)`) is a bash/zsh extension
//! not available in POSIX sh. Scripts using this will fail on dash, ash,
//! or other strict POSIX shells.
//!
//! **Auto-fix**: None (manual refactoring required - use temp files or pipes)
//!
//! ## Examples
//!
//! Bad (bash-only):
//! ```bash
//! #!/bin/sh
//! diff <(sort file1) <(sort file2)
//! ```
//!
//! Good (POSIX-compatible):
//! ```bash
//! #!/bin/sh
//! sort file1 > /tmp/sorted1
//! sort file2 > /tmp/sorted2
//! diff /tmp/sorted1 /tmp/sorted2
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check if the script has a POSIX sh shebang
fn is_posix_sh(source: &str) -> bool {
    let first_line = source.lines().next().unwrap_or("");
    first_line.starts_with("#!/bin/sh")
        || first_line.starts_with("#! /bin/sh")
        || first_line.starts_with("#!/usr/bin/env sh")
}

/// Check for process substitution in POSIX sh scripts
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    if !is_posix_sh(source) {
        return result;
    }

    // Match <( or >(  - process substitution
    let pattern = Regex::new(r"[<>]\(").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        for m in pattern.find_iter(line) {
            let col = m.start();

            // Make sure it's not inside quotes
            let before = &line[..col];
            let single_quotes = before.matches('\'').count();
            let double_quotes = before.matches('"').count();
            if single_quotes % 2 != 0 || double_quotes % 2 != 0 {
                continue;
            }

            // Make sure < is not part of a redirect like 2>(
            // Actually <( and >( are the process substitution forms
            let ch = line.as_bytes()[col] as char;
            let subst_type = if ch == '<' { "input" } else { "output" };

            let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 3);

            let diagnostic = Diagnostic::new(
                "PORT004",
                Severity::Warning,
                format!(
                    "Process substitution `{}(...)` is not supported in POSIX sh. Use temporary files or pipes instead.",
                    ch
                ),
                span,
            );

            let _ = subst_type; // used for documentation context
            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port004_detects_input_process_subst() {
        let script = "#!/bin/sh\ndiff <(sort file1) <(sort file2)";
        let result = check(script);
        assert!(result.diagnostics.len() >= 1);
        assert_eq!(result.diagnostics[0].code, "PORT004");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_port004_detects_output_process_subst() {
        let script = "#!/bin/sh\ntee >(wc -l)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_port004_no_flag_in_bash() {
        let script = "#!/bin/bash\ndiff <(sort file1) <(sort file2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port004_no_false_positive_comment() {
        let script = "#!/bin/sh\n# diff <(sort file1) <(sort file2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port004_no_fix_provided() {
        let script = "#!/bin/sh\ndiff <(sort file1) <(sort file2)";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_port004_no_flag_without_shebang() {
        let script = "diff <(sort file1) <(sort file2)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
