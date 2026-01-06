//! DOCKER011: USER directive validation (F069)
//!
//! **Rule**: Validate USER directive for non-root execution
//!
//! **Why this matters**:
//! Running containers as root is a security risk. Best practice is to
//! run as a non-root user to limit potential damage from vulnerabilities.
//!
//! ## Examples
//!
//! ❌ **BAD** (running as root):
//! ```dockerfile
//! FROM python:3.12
//! COPY app.py /
//! USER root
//! CMD ["python", "app.py"]
//! ```
//!
//! ✅ **GOOD** (non-root user):
//! ```dockerfile
//! FROM python:3.12
//! RUN useradd -r -u 1001 appuser
//! COPY app.py /
//! USER appuser
//! CMD ["python", "app.py"]
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Root user identifiers
const ROOT_USERS: &[&str] = &["root", "0"];

/// Check for USER directive validation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut last_user_line = 0;
    let mut last_user_value = String::new();
    let mut has_user_directive = false;
    let mut has_cmd_or_entrypoint = false;
    let mut cmd_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if upper.starts_with("USER ") {
            has_user_directive = true;
            last_user_line = line_num + 1;
            last_user_value = trimmed[5..].trim().to_string();

            // Check for root user
            let user_lower = last_user_value.to_lowercase();
            if ROOT_USERS.contains(&user_lower.as_str()) {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                let diag = Diagnostic::new(
                    "DOCKER011",
                    Severity::Warning,
                    format!(
                        "USER {} runs container as root - consider non-root user (F069)",
                        last_user_value
                    ),
                    span,
                );
                result.add(diag);
            }

            // Check for numeric UID 0
            if let Ok(uid) = last_user_value.parse::<u32>() {
                if uid == 0 {
                    let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                    let diag = Diagnostic::new(
                        "DOCKER011",
                        Severity::Warning,
                        "USER 0 runs container as root - consider non-root UID (F069)".to_string(),
                        span,
                    );
                    result.add(diag);
                }
            }
        }

        if upper.starts_with("CMD ") || upper.starts_with("ENTRYPOINT ") {
            has_cmd_or_entrypoint = true;
            cmd_line = line_num + 1;
        }
    }

    // Check if container has CMD/ENTRYPOINT but no USER directive
    if has_cmd_or_entrypoint && !has_user_directive {
        let span = Span::new(cmd_line, 1, cmd_line, 1);
        let diag = Diagnostic::new(
            "DOCKER011",
            Severity::Warning,
            "No USER directive - container will run as root (F069)".to_string(),
            span,
        );
        result.add(diag);
    }

    // Check if final USER is root (even if changed earlier)
    if has_user_directive && ROOT_USERS.contains(&last_user_value.to_lowercase().as_str()) {
        let span = Span::new(last_user_line, 1, last_user_line, 1);
        let diag = Diagnostic::new(
            "DOCKER011",
            Severity::Warning,
            "Final USER is root - consider switching to non-root before CMD (F069)".to_string(),
            span,
        );
        result.add(diag);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F069: USER directive validation
    #[test]
    fn test_F069_nonroot_user() {
        let dockerfile = r#"FROM python:3.12
RUN useradd -r appuser
USER appuser
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        // Non-root user should not have root warnings
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("as root")),
            "F069: Non-root user should not warn about root"
        );
    }

    #[test]
    fn test_F069_root_user() {
        let dockerfile = r#"FROM python:3.12
USER root
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        // Should warn about root user
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("root")),
            "F069: Should warn about root user"
        );
    }

    #[test]
    fn test_F069_uid_zero() {
        let dockerfile = r#"FROM python:3.12
USER 0
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        // Should warn about UID 0
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("USER 0")),
            "F069: Should warn about UID 0"
        );
    }

    #[test]
    fn test_F069_no_user_directive() {
        let dockerfile = r#"FROM python:3.12
COPY app.py /
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        // Should warn about missing USER
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("No USER directive")),
            "F069: Should warn about missing USER directive"
        );
    }

    #[test]
    fn test_F069_numeric_uid() {
        let dockerfile = r#"FROM python:3.12
USER 1001
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        // Non-root UID should not warn
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("as root")),
            "F069: Non-root UID should not warn"
        );
    }
}
