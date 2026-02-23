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

/// Check if exec form uses a shell with -c flag, return the matching shell name
fn find_shell_in_exec_form(rest: &str, extra_patterns: bool) -> Option<&'static str> {
    for shell in SHELL_PATHS {
        let mut patterns = vec![
            format!("[\"{}\", \"-c\"", shell),
            format!("[\"{}\" , \"-c\"", shell),
        ];
        if extra_patterns {
            patterns.push(format!("['{}'", shell));
        }
        if patterns.iter().any(|p| rest.contains(p.as_str())) {
            return Some(shell);
        }
    }
    None
}

/// Check CMD directive for shell usage
fn check_cmd_directive(rest: &str, line_num: usize, trimmed: &str, result: &mut LintResult) {
    if rest.starts_with('[') {
        if let Some(shell) = find_shell_in_exec_form(rest, true) {
            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
            result.add(Diagnostic::new(
                "DOCKER008",
                Severity::Warning,
                format!(
                    "CMD uses shell '{}' with -c flag - consider direct execution (F062)",
                    shell
                ),
                span,
            ));
        }
    } else {
        let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
        result.add(Diagnostic::new(
            "DOCKER008",
            Severity::Info,
            "CMD uses shell form - consider exec form for better signal handling (F072)"
                .to_string(),
            span,
        ));
    }
}

/// Check RUN directive for redundant shell exec form
fn check_run_directive(rest: &str, line_num: usize, trimmed: &str, result: &mut LintResult) {
    if rest.starts_with('[') {
        if let Some(shell) = find_shell_in_exec_form(rest, false) {
            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
            result.add(Diagnostic::new(
                "DOCKER008",
                Severity::Info,
                format!(
                    "RUN exec form with '{}' -c is redundant - shell form does the same (F064)",
                    shell
                ),
                span,
            ));
        }
    }
}

/// Check for shell in CMD directives and shell form usage
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("CMD ") {
            let rest = trimmed.strip_prefix("CMD ").unwrap_or("");
            check_cmd_directive(rest, line_num, trimmed, &mut result);
        }

        if trimmed.starts_with("RUN ") {
            let rest = trimmed.strip_prefix("RUN ").unwrap_or("");
            check_run_directive(rest, line_num, trimmed, &mut result);
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
