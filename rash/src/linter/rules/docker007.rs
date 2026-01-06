//! DOCKER007: Shell entrypoint detection (F061)
//!
//! **Rule**: Detect Dockerfiles that use shell as entrypoint
//!
//! **Why this matters**:
//! Using shell as ENTRYPOINT defeats the purpose of containerization.
//! Containers should run a single process, not a shell.
//!
//! ## Examples
//!
//! ❌ **BAD** (shell entrypoint):
//! ```dockerfile
//! ENTRYPOINT ["/bin/sh"]
//! ENTRYPOINT ["/bin/bash"]
//! ```
//!
//! ✅ **GOOD** (direct process):
//! ```dockerfile
//! ENTRYPOINT ["/app"]
//! ENTRYPOINT ["python", "app.py"]
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Shell paths that indicate shell entrypoint
const SHELL_PATHS: &[&str] = &[
    "/bin/sh",
    "/bin/bash",
    "/bin/ash",
    "/bin/dash",
    "/bin/zsh",
    "sh",
    "bash",
    "ash",
    "dash",
    "zsh",
];

/// Check for shell entrypoints in Dockerfiles
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Check ENTRYPOINT directive
        if trimmed.starts_with("ENTRYPOINT ") {
            let rest = trimmed.strip_prefix("ENTRYPOINT ").unwrap_or("");

            // Check exec form: ENTRYPOINT ["sh", ...] or ENTRYPOINT ["/bin/sh", ...]
            if rest.starts_with('[') {
                for shell in SHELL_PATHS {
                    let patterns = [
                        format!("[\"{}\"]", shell),
                        format!("[\"{}\"", shell),
                        format!("['{}'", shell),
                    ];
                    for pattern in &patterns {
                        if rest.contains(pattern) {
                            let span =
                                Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                            let diag = Diagnostic::new(
                                "DOCKER007",
                                Severity::Warning,
                                format!(
                                    "Shell entrypoint '{}' detected - consider using direct process (F061)",
                                    shell
                                ),
                                span,
                            );
                            result.add(diag);
                            break;
                        }
                    }
                }
            }
            // Check shell form: ENTRYPOINT /bin/sh
            else {
                for shell in SHELL_PATHS {
                    if rest.trim() == *shell || rest.starts_with(&format!("{} ", shell)) {
                        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                        let diag = Diagnostic::new(
                            "DOCKER007",
                            Severity::Warning,
                            format!(
                                "Shell entrypoint '{}' detected - consider using direct process (F061)",
                                shell
                            ),
                            span,
                        );
                        result.add(diag);
                        break;
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F061: Detects shell entrypoints
    #[test]
    fn test_F061_shell_entrypoint_exec_form() {
        let dockerfile = r#"FROM debian:12-slim
ENTRYPOINT ["/bin/sh"]"#;
        let result = check(dockerfile);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "F061: Should detect /bin/sh entrypoint"
        );
        assert_eq!(result.diagnostics[0].code, "DOCKER007");
    }

    #[test]
    fn test_F061_shell_entrypoint_bash() {
        let dockerfile = r#"FROM debian:12-slim
ENTRYPOINT ["/bin/bash"]"#;
        let result = check(dockerfile);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "F061: Should detect /bin/bash entrypoint"
        );
    }

    #[test]
    fn test_F061_shell_entrypoint_shell_form() {
        let dockerfile = r#"FROM debian:12-slim
ENTRYPOINT /bin/sh"#;
        let result = check(dockerfile);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "F061: Should detect shell form entrypoint"
        );
    }

    #[test]
    fn test_F061_no_warning_direct_process() {
        let dockerfile = r#"FROM debian:12-slim
ENTRYPOINT ["/app"]"#;
        let result = check(dockerfile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F061: Should not flag direct process entrypoint"
        );
    }

    #[test]
    fn test_F061_no_warning_python() {
        let dockerfile = r#"FROM python:3.12-slim
ENTRYPOINT ["python", "app.py"]"#;
        let result = check(dockerfile);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "F061: Should not flag python entrypoint"
        );
    }
}
