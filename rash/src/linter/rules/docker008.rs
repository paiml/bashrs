//! DOCKER008: Shell in CMD detection (F062) and shell form vs exec form (F072)
//!
//! **Rule**: Detect CMD directives that use shell interpreters
//!
//! **Why this matters**:
//! Using `sh -c` in CMD adds an unnecessary shell layer, prevents proper
//! signal handling, and makes the container harder to manage.
//!
//! ## Examples
//!
//! ❌ **BAD** (shell in CMD):
//! ```dockerfile
//! CMD ["sh", "-c", "python app.py"]
//! CMD /bin/sh -c "python app.py"
//! ```
//!
//! ✅ **GOOD** (direct execution):
//! ```dockerfile
//! CMD ["python", "app.py"]
//! CMD python app.py
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Shell paths that indicate shell usage
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

/// Check for shell in CMD directives and shell form usage
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Check CMD directive
        if trimmed.starts_with("CMD ") {
            let rest = trimmed.strip_prefix("CMD ").unwrap_or("");

            // F062: Check exec form: CMD ["sh", "-c", ...] or CMD ["/bin/sh", "-c", ...]
            if rest.starts_with('[') {
                for shell in SHELL_PATHS {
                    // Check for shell as first argument with -c
                    let patterns = [
                        format!("[\"{}\", \"-c\"", shell),
                        format!("[\"{}\" , \"-c\"", shell),
                        format!("['{}'", shell),
                    ];
                    for pattern in &patterns {
                        if rest.contains(pattern) {
                            let span =
                                Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                            let diag = Diagnostic::new(
                                "DOCKER008",
                                Severity::Warning,
                                format!(
                                    "CMD uses shell '{}' with -c flag - consider direct execution (F062)",
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
            // F072: Shell form: CMD command (without brackets)
            else {
                // Shell form is not necessarily bad but is worth noting
                // It runs through /bin/sh -c by default
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                let diag = Diagnostic::new(
                    "DOCKER008",
                    Severity::Info,
                    "CMD uses shell form - consider exec form for better signal handling (F072)"
                        .to_string(),
                    span,
                );
                result.add(diag);
            }
        }

        // Also check RUN for unnecessary sh -c
        if trimmed.starts_with("RUN ") {
            let rest = trimmed.strip_prefix("RUN ").unwrap_or("");

            // Check exec form RUN: RUN ["sh", "-c", ...]
            if rest.starts_with('[') {
                for shell in SHELL_PATHS {
                    let patterns = [
                        format!("[\"{}\", \"-c\"", shell),
                        format!("[\"{}\" , \"-c\"", shell),
                    ];
                    for pattern in &patterns {
                        if rest.contains(pattern) {
                            let span =
                                Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                            let diag = Diagnostic::new(
                                "DOCKER008",
                                Severity::Info,
                                format!(
                                    "RUN exec form with '{}' -c is redundant - shell form does the same (F064)",
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
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F062: Detects shell in CMD
    #[test]
    fn test_F062_shell_in_cmd_exec_form() {
        let dockerfile = r#"FROM debian:12-slim
CMD ["sh", "-c", "python app.py"]"#;
        let result = check(dockerfile);

        assert!(
            result.diagnostics.iter().any(|d| d.code == "DOCKER008"
                && d.message.contains("sh")
                && d.message.contains("-c")),
            "F062: Should detect sh -c in CMD exec form. Got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_F062_shell_in_cmd_bash() {
        let dockerfile = r#"FROM debian:12-slim
CMD ["/bin/bash", "-c", "python app.py"]"#;
        let result = check(dockerfile);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("bash")),
            "F062: Should detect /bin/bash -c in CMD"
        );
    }

    /// F072: Shell form vs exec form
    #[test]
    fn test_F072_cmd_shell_form() {
        let dockerfile = r#"FROM debian:12-slim
CMD python app.py"#;
        let result = check(dockerfile);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("shell form")),
            "F072: Should note shell form usage"
        );
    }

    #[test]
    fn test_F062_no_warning_exec_form_direct() {
        let dockerfile = r#"FROM debian:12-slim
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        // Should have no warnings about shell
        assert!(
            !result.diagnostics.iter().any(|d| d.message.contains("sh")),
            "F062: Should not flag direct exec form"
        );
    }

    /// F064: RUN with shell
    #[test]
    fn test_F064_run_exec_form_with_shell() {
        let dockerfile = r#"FROM debian:12-slim
RUN ["sh", "-c", "apt-get update"]"#;
        let result = check(dockerfile);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("RUN exec form")),
            "F064: Should note RUN exec form with sh -c is redundant"
        );
    }
}
