//! PORT002: `local` keyword in POSIX sh
//!
//! **Rule**: Detect `local VAR` in `#!/bin/sh` scripts
//!
//! **Why this matters**:
//! `local` is not defined by POSIX. While many sh implementations support it
//! as an extension, it is not guaranteed to work in all POSIX-compliant shells.
//!
//! **Auto-fix**: Safe - remove `local` keyword
//!
//! ## Examples
//!
//! Bad (non-POSIX):
//! ```bash
//! #!/bin/sh
//! myfunc() {
//!     local x=5
//! }
//! ```
//!
//! Good (POSIX-compatible):
//! ```bash
//! #!/bin/sh
//! myfunc() {
//!     x=5
//! }
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check if the script has a POSIX sh shebang
fn is_posix_sh(source: &str) -> bool {
    let first_line = source.lines().next().unwrap_or("");
    first_line.starts_with("#!/bin/sh")
        || first_line.starts_with("#! /bin/sh")
        || first_line.starts_with("#!/usr/bin/env sh")
}

/// Check for `local` keyword in POSIX sh scripts
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    if !is_posix_sh(source) {
        return result;
    }

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Match: local VAR  or  local VAR=value
        if trimmed.starts_with("local ") {
            let col = line.find("local").unwrap_or(0);
            let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 6);

            // Build fix by removing "local " prefix
            let fixed = trimmed.strip_prefix("local ").unwrap_or(trimmed);

            let diagnostic = Diagnostic::new(
                "PORT002",
                Severity::Warning,
                "`local` is not defined in POSIX sh. Remove `local` keyword for portability.",
                span,
            )
            .with_fix(Fix::new(fixed.to_string()));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port002_detects_local_in_sh() {
        let script = "#!/bin/sh\nmyfunc() {\n    local x=5\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PORT002");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_port002_provides_fix() {
        let script = "#!/bin/sh\nmyfunc() {\n    local x=5\n}";
        let result = check(script);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "x=5");
    }

    #[test]
    fn test_port002_no_flag_in_bash() {
        let script = "#!/bin/bash\nmyfunc() {\n    local x=5\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port002_no_false_positive_comment() {
        let script = "#!/bin/sh\n# local x=5";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port002_multiple_locals() {
        let script = "#!/bin/sh\nf() {\n    local a=1\n    local b=2\n}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
