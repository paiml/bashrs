//! DOCKER009: Multi-stage build validation (F063)
//!
//! **Rule**: Validate multi-stage Dockerfiles for shell-free final images
//!
//! **Why this matters**:
//! Multi-stage builds should produce minimal final images without shell access.
//! This is critical for security in production environments.
//!
//! ## Examples
//!
//! ❌ **BAD** (shell in final stage):
//! ```dockerfile
//! FROM rust:1.82 AS builder
//! RUN cargo build --release
//!
//! FROM debian:12-slim
//! RUN apt-get install -y bash
//! COPY --from=builder /app/target/release/app /
//! ```
//!
//! ✅ **GOOD** (distroless final stage):
//! ```dockerfile
//! FROM rust:1.82 AS builder
//! RUN cargo build --release
//!
//! FROM gcr.io/distroless/cc-debian12
//! COPY --from=builder /app/target/release/app /
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Distroless and minimal base images that don't have shells
const SHELL_FREE_BASES: &[&str] = &[
    "gcr.io/distroless/",
    "distroless/",
    "scratch",
    "busybox:uclibc",
    "chainguard/",
    "cgr.dev/",
];

/// Shell installation patterns
const SHELL_INSTALL_PATTERNS: &[&str] = &[
    "apt-get install",
    "apt install",
    "apk add",
    "yum install",
    "dnf install",
    "bash",
    "/bin/sh",
    "/bin/bash",
];

/// Validate the final stage of a multi-stage build
fn validate_final_stage(
    lines: &[&str],
    final_line: usize,
    has_shell_install: bool,
    result: &mut LintResult,
) {
    let final_from_line = lines
        .get(final_line.saturating_sub(1))
        .unwrap_or(&"")
        .trim();

    let is_shell_free_base = SHELL_FREE_BASES
        .iter()
        .any(|base| final_from_line.to_lowercase().contains(&base.to_lowercase()));

    if has_shell_install && !is_shell_free_base {
        let span = Span::new(final_line, 1, final_line, 80);
        result.add(Diagnostic::new(
            "DOCKER009",
            Severity::Warning,
            "Final stage may install shell - consider using distroless base image (F063)"
                .to_string(),
            span,
        ));
    }
    if !is_shell_free_base && !final_from_line.is_empty() {
        let span = Span::new(final_line, 1, final_line, 80);
        result.add(Diagnostic::new(
            "DOCKER009",
            Severity::Info,
            "Consider using distroless/scratch base for shell-free final image (F063)".to_string(),
            span,
        ));
    }
}

/// Check for multi-stage build shell-free validation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();
    let mut stages: Vec<(usize, String, bool)> = Vec::new();
    let mut current_stage_line = 0;
    let mut current_stage_name = String::new();
    let mut has_shell_install_in_current = false;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.to_uppercase().starts_with("FROM ") {
            if !current_stage_name.is_empty() || current_stage_line > 0 {
                stages.push((current_stage_line, current_stage_name.clone(), false));
            }
            current_stage_line = line_num + 1;
            has_shell_install_in_current = false;
            let upper = trimmed.to_uppercase();
            current_stage_name = upper
                .find(" AS ")
                .map(|pos| trimmed[pos + 4..].trim().to_string())
                .unwrap_or_default();
        }

        if trimmed.to_uppercase().starts_with("RUN ") {
            let run_content = trimmed[4..].to_lowercase();
            if SHELL_INSTALL_PATTERNS
                .iter()
                .any(|p| run_content.contains(p))
            {
                has_shell_install_in_current = true;
            }
        }
    }

    if !current_stage_name.is_empty() || current_stage_line > 0 {
        stages.push((current_stage_line, current_stage_name, true));
    }

    if stages.len() > 1 {
        if let Some((final_line, _final_name, true)) = stages.last() {
            validate_final_stage(&lines, *final_line, has_shell_install_in_current, &mut result);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F063: Multi-stage build validation
    #[test]
    fn test_F063_distroless_final_stage() {
        let dockerfile = r#"FROM rust:1.82 AS builder
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/app /"#;
        let result = check(dockerfile);

        // Distroless final stage should not warn
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Warning),
            "F063: Distroless final stage should not warn"
        );
    }

    #[test]
    fn test_F063_debian_final_stage() {
        let dockerfile = r#"FROM rust:1.82 AS builder
RUN cargo build --release

FROM debian:12-slim
COPY --from=builder /app/target/release/app /"#;
        let result = check(dockerfile);

        // Non-distroless should have info message
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("distroless")),
            "F063: Non-distroless should suggest distroless"
        );
    }

    #[test]
    fn test_F063_shell_install_in_final() {
        let dockerfile = r#"FROM rust:1.82 AS builder
RUN cargo build --release

FROM debian:12-slim
RUN apt-get install -y bash
COPY --from=builder /app/target/release/app /"#;
        let result = check(dockerfile);

        // Shell installation in final stage should warn
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Warning),
            "F063: Shell install in final stage should warn"
        );
    }

    #[test]
    fn test_F063_scratch_base() {
        let dockerfile = r#"FROM golang:1.22 AS builder
RUN go build -o /app

FROM scratch
COPY --from=builder /app /app"#;
        let result = check(dockerfile);

        // scratch base should not warn
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Warning),
            "F063: scratch base should not warn"
        );
    }

    #[test]
    fn test_F063_single_stage_not_flagged() {
        let dockerfile = r#"FROM debian:12-slim
RUN apt-get update
COPY app /app"#;
        let result = check(dockerfile);

        // Single stage builds should not have multi-stage warnings
        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Final stage")),
            "F063: Single stage should not have multi-stage warnings"
        );
    }
}
