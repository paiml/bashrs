//! PORT001: Array syntax in POSIX sh
//!
//! **Rule**: Detect `arr=()` array syntax in `#!/bin/sh` scripts
//!
//! **Why this matters**:
//! POSIX sh does not support array syntax (`arr=()`). This is a bash/zsh
//! extension that will fail on dash, ash, or other strict POSIX shells.
//!
//! **Auto-fix**: None (manual refactoring required)
//!
//! ## Examples
//!
//! Bad (bash-only syntax in sh script):
//! ```bash
//! #!/bin/sh
//! files=()
//! files=(foo bar baz)
//! ```
//!
//! Good (POSIX-compatible):
//! ```bash
//! #!/bin/sh
//! files="foo bar baz"
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

/// Check for array syntax in POSIX sh scripts
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    if !is_posix_sh(source) {
        return result;
    }

    // Match: var=() or var=(items)
    let pattern = Regex::new(r"\b\w+=\(").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        if let Some(m) = pattern.find(line) {
            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "PORT001",
                Severity::Warning,
                "Array syntax `=()` is not supported in POSIX sh. Use space-separated strings or positional parameters instead.",
                Span::new(line_num + 1, start_col, line_num + 1, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port001_detects_array_in_sh() {
        let script = "#!/bin/sh\nfiles=(foo bar baz)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PORT001");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_port001_detects_empty_array() {
        let script = "#!/bin/sh\narr=()";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_port001_no_flag_in_bash() {
        let script = "#!/bin/bash\nfiles=(foo bar baz)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port001_no_flag_without_shebang() {
        let script = "files=(foo bar baz)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port001_no_false_positive_comment() {
        let script = "#!/bin/sh\n# arr=(foo)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port001_no_fix_provided() {
        let script = "#!/bin/sh\narr=(1 2 3)";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_port001_env_sh_shebang() {
        let script = "#!/usr/bin/env sh\narr=(1 2 3)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
