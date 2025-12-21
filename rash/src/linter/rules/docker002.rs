//! DOCKER002: Unpinned base image (security/reproducibility risk)
//!
//! Equivalent to hadolint DL3006, DL3007
//!
//! **Rule**: Base images should be pinned with SHA256 digest or avoid :latest tag
//!
//! **Why this matters**:
//! Using :latest or unpinned tags means your build is not reproducible.
//! The same Dockerfile can produce different images over time.
//!
//! ## Examples
//!
//! ❌ **BAD** (unpinned):
//! ```dockerfile
//! FROM debian:latest
//! FROM python:3.12
//! ```
//!
//! ✅ **GOOD** (pinned with SHA256):
//! ```dockerfile
//! FROM debian:12-slim@sha256:abc123...
//! FROM python:3.12.0@sha256:def456...
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("FROM ") && !trimmed.contains("scratch") {
            let from_image = trimmed["FROM ".len()..]
                .split_whitespace()
                .next()
                .unwrap_or("");

            // Check for :latest tag (hadolint DL3007)
            if from_image.ends_with(":latest") {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                let diag = Diagnostic::new(
                    "DOCKER002",
                    Severity::Warning,
                    "Base image uses :latest tag - pin to specific version for reproducibility (hadolint DL3007)".to_string(),
                    span,
                );
                result.add(diag);
            }
            // Check if SHA256 pin is missing (hadolint DL3006)
            else if !trimmed.contains("@sha256:") {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                let diag = Diagnostic::new(
                    "DOCKER002",
                    Severity::Warning,
                    "Base image not pinned with SHA256 digest - use image@sha256:... for reproducibility (hadolint DL3006)".to_string(),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DOCKER002_unpinned_warns() {
        let dockerfile = "FROM debian:12-slim\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("SHA256"));
    }

    #[test]
    fn test_DOCKER002_latest_warns() {
        let dockerfile = "FROM debian:latest\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("latest"));
    }

    #[test]
    fn test_DOCKER002_pinned_no_warn() {
        let dockerfile = "FROM debian:12-slim@sha256:abc123\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DOCKER002_scratch_no_warn() {
        let dockerfile = "FROM scratch\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
            #[test]
            fn prop_never_panics(dockerfile in ".*") {
                let _ = check(&dockerfile);
            }

            #[test]
            fn prop_scratch_never_warns(
                commands in prop::collection::vec("(RUN|COPY|CMD) .*", 0..10)
            ) {
                let dockerfile = format!("FROM scratch\n{}", commands.join("\n"));
                let result = check(&dockerfile);
                prop_assert_eq!(result.diagnostics.len(), 0);
            }

            #[test]
            fn prop_pinned_never_warns(
                image in "[a-z]+:[a-z0-9.-]+",
                hash in "[a-f0-9]{64}"
            ) {
                let dockerfile = format!("FROM {}@sha256:{}", image, hash);
                let result = check(&dockerfile);
                prop_assert_eq!(result.diagnostics.len(), 0);
            }

            #[test]
            fn prop_latest_always_warns(
                image in "[a-z]+"
            ) {
                let dockerfile = format!("FROM {}:latest", image);
                let result = check(&dockerfile);
                prop_assert_eq!(result.diagnostics.len(), 1);
            }

            #[test]
            fn prop_unpinned_warns(
                image in "[a-z]+:[0-9.]+"
            ) {
                let dockerfile = format!("FROM {}", image);
                let result = check(&dockerfile);
                prop_assert_eq!(result.diagnostics.len(), 1);
            }
        }
    }
}
