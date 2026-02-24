//! PORT005: `source` command in POSIX sh
//!
//! **Rule**: Detect `source file` in `#!/bin/sh` scripts
//!
//! **Why this matters**:
//! `source` is a bash/zsh extension. The POSIX-compatible equivalent is
//! `. file` (dot command). Scripts using `source` will fail on dash, ash,
//! or other strict POSIX shells.
//!
//! **Auto-fix**: Safe - replace `source` with `.`
//!
//! ## Examples
//!
//! Bad (bash-only):
//! ```bash
//! #!/bin/sh
//! source /etc/profile
//! source ~/.bashrc
//! ```
//!
//! Good (POSIX-compatible):
//! ```bash
//! #!/bin/sh
//! . /etc/profile
//! . ~/.bashrc
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check if the script has a POSIX sh shebang
fn is_posix_sh(source: &str) -> bool {
    let first_line = source.lines().next().unwrap_or("");
    first_line.starts_with("#!/bin/sh")
        || first_line.starts_with("#! /bin/sh")
        || first_line.starts_with("#!/usr/bin/env sh")
}

/// Check for `source` command in POSIX sh scripts
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

        // Match: source FILE at start of line/command
        if trimmed.starts_with("source ") {
            let col = line.find("source").unwrap_or(0);
            let file_arg = trimmed.strip_prefix("source ").unwrap_or("").trim();
            let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 7);

            let fix_text = format!(". {}", file_arg);

            let diagnostic = Diagnostic::new(
                "PORT005",
                Severity::Warning,
                format!(
                    "`source` is not defined in POSIX sh. Use `. {}` instead.",
                    file_arg
                ),
                span,
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port005_detects_source_in_sh() {
        let script = "#!/bin/sh\nsource /etc/profile";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PORT005");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_port005_provides_fix() {
        let script = "#!/bin/sh\nsource /etc/profile";
        let result = check(script);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, ". /etc/profile");
    }

    #[test]
    fn test_port005_no_flag_in_bash() {
        let script = "#!/bin/bash\nsource /etc/profile";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port005_no_false_positive_comment() {
        let script = "#!/bin/sh\n# source /etc/profile";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port005_no_flag_dot_command() {
        let script = "#!/bin/sh\n. /etc/profile";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port005_multiple_sources() {
        let script = "#!/bin/sh\nsource /etc/profile\nsource ~/.config";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
