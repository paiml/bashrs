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
