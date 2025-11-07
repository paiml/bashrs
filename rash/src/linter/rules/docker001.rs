//! DOCKER001: Missing USER directive (security risk)
//!
//! **Rule**: Detect Dockerfiles running as root user
//!
//! **Why this matters**:
//! Running containers as root is a security risk. If the container is compromised,
//! the attacker has root privileges. Always use a non-root USER.
//!
//! **Exceptions**:
//! - `FROM scratch` images (no users exist in scratch)
//! - Build stages (intermediate stages can run as root)
//!
//! ## Examples
//!
//! ❌ **BAD** (runs as root):
//! ```dockerfile
//! FROM debian:12-slim
//! COPY app /app
//! CMD ["/app"]
//! ```
//!
//! ✅ **GOOD** (non-root user):
//! ```dockerfile
//! FROM debian:12-slim
//! RUN useradd -m appuser
//! USER appuser
//! COPY app /app
//! CMD ["/app"]
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for missing USER directive in non-scratch Dockerfiles
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();

    // Parse Dockerfile to find FROM and USER directives
    let mut stages = Vec::new();
    let mut current_stage: Option<DockerStage> = None;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Detect FROM directive (new stage)
        if let Some(stripped) = trimmed.strip_prefix("FROM ") {
            // Save previous stage
            if let Some(stage) = current_stage.take() {
                stages.push(stage);
            }

            // Start new stage
            let from_image = stripped.trim();
            let is_scratch = from_image.starts_with("scratch");
            let is_named_stage = from_image.contains(" AS ");

            current_stage = Some(DockerStage {
                line: line_num + 1,
                from_image: from_image.to_string(),
                is_scratch,
                is_named_stage,
                has_user: false,
            });
        }

        // Detect USER directive
        if trimmed.starts_with("USER ") {
            if let Some(ref mut stage) = current_stage {
                stage.has_user = true;
            }
        }
    }

    // Save last stage
    if let Some(stage) = current_stage {
        stages.push(stage);
    }

    // Check the final stage (runtime stage)
    if let Some(final_stage) = stages.last() {
        // Only warn for non-scratch final stages
        if !final_stage.is_scratch && !final_stage.has_user {
            let span = Span::new(final_stage.line, 1, final_stage.line, 5);

            let diag = Diagnostic::new(
                "DOCKER001",
                Severity::Warning,
                "Missing USER directive - container runs as root (security risk). Add USER directive before CMD/ENTRYPOINT".to_string(),
                span,
            );

            result.add(diag);
        }
    }

    result
}

#[derive(Debug)]
struct DockerStage {
    line: usize,
    from_image: String,
    is_scratch: bool,
    is_named_stage: bool,
    has_user: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DOCKER001_missing_user_directive() {
        let dockerfile = "FROM debian:12-slim\nCOPY app /app\n";
        let result = check(dockerfile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DOCKER001");
        assert!(diag.message.contains("USER"));
    }

    #[test]
    fn test_DOCKER001_scratch_no_warning() {
        let dockerfile = "FROM scratch\nCOPY app /app\n";
        let result = check(dockerfile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DOCKER001_user_present() {
        let dockerfile = "FROM debian:12-slim\nUSER appuser\nCOPY app /app\n";
        let result = check(dockerfile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DOCKER001_multi_stage_final_no_user() {
        let dockerfile =
            "FROM debian AS builder\nRUN build\n\nFROM debian\nCOPY --from=builder /app /app\n";
        let result = check(dockerfile);

        // Should warn about final stage
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DOCKER001_multi_stage_final_scratch() {
        let dockerfile =
            "FROM debian AS builder\nRUN build\n\nFROM scratch\nCOPY --from=builder /app /app\n";
        let result = check(dockerfile);

        // Should NOT warn (final stage is scratch)
        assert_eq!(result.diagnostics.len(), 0);
    }
}
