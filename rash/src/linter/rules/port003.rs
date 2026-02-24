//! PORT003: `[[ ]]` in POSIX sh
//!
//! **Rule**: Detect `[[ ]]` conditional syntax in `#!/bin/sh` scripts
//!
//! **Why this matters**:
//! `[[ ]]` is a bash/zsh/ksh extension not available in POSIX sh.
//! POSIX sh only supports `[ ]` (test) syntax. Scripts using `[[ ]]`
//! will fail on dash, ash, or other strict POSIX shells.
//!
//! **Auto-fix**: Safe - suggest replacing with `[ ]`
//!
//! ## Examples
//!
//! Bad (bash-only):
//! ```bash
//! #!/bin/sh
//! if [[ -f /etc/config ]]; then
//!     echo "exists"
//! fi
//! ```
//!
//! Good (POSIX-compatible):
//! ```bash
//! #!/bin/sh
//! if [ -f /etc/config ]; then
//!     echo "exists"
//! fi
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check if the script has a POSIX sh shebang
fn is_posix_sh(source: &str) -> bool {
    let first_line = source.lines().next().unwrap_or("");
    first_line.starts_with("#!/bin/sh")
        || first_line.starts_with("#! /bin/sh")
        || first_line.starts_with("#!/usr/bin/env sh")
}

/// Check for `[[ ]]` in POSIX sh scripts
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

        if let Some(col) = line.find("[[") {
            // Verify it's [[ not just [ by checking not preceded by another [
            // and followed by a space (to avoid matching things like array[[$i]])
            let after = &line[col + 2..];
            if after.starts_with(' ') || after.starts_with('\t') {
                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 3);

                let diagnostic = Diagnostic::new(
                    "PORT003",
                    Severity::Warning,
                    "`[[ ]]` is not supported in POSIX sh. Use `[ ]` (test) instead.",
                    span,
                )
                .with_fix(Fix::new("Replace [[ ]] with [ ]"));

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port003_detects_double_bracket_in_sh() {
        let script = "#!/bin/sh\nif [[ -f /etc/config ]]; then\n    echo exists\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "PORT003");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_port003_provides_fix() {
        let script = "#!/bin/sh\n[[ -d /tmp ]]";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_port003_no_flag_in_bash() {
        let script = "#!/bin/bash\nif [[ -f /etc/config ]]; then echo yes; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port003_no_flag_single_bracket() {
        let script = "#!/bin/sh\nif [ -f /etc/config ]; then echo yes; fi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port003_no_false_positive_comment() {
        let script = "#!/bin/sh\n# if [[ -f file ]]; then";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_port003_detects_standalone() {
        let script = "#!/bin/sh\n[[ $x == y ]]";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
