//! DOCKER005: Missing --no-install-recommends (image size optimization)
//!
//! Equivalent to hadolint DL3015
//!
//! **Rule**: apt-get install should use --no-install-recommends
//!
//! **Why this matters**:
//! By default, apt-get installs recommended packages, significantly
//! increasing image size. Use --no-install-recommends to install only
//! required dependencies.
//!
//! ## Examples
//!
//! ❌ **BAD** (installs recommended packages):
//! ```dockerfile
//! RUN apt-get update && apt-get install -y curl
//! ```
//!
//! ✅ **GOOD** (minimal installation):
//! ```dockerfile
//! RUN apt-get update && \
//!     apt-get install -y --no-install-recommends curl && \
//!     rm -rf /var/lib/apt/lists/*
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("apt-get install") && !trimmed.contains("--no-install-recommends") {
            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
            let diag = Diagnostic::new(
                "DOCKER005",
                Severity::Info,
                "apt-get install without --no-install-recommends increases image size (hadolint DL3015)".to_string(),
                span,
            );
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DOCKER005_missing_no_install_recommends() {
        let dockerfile = "RUN apt-get install -y curl\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DOCKER005");
    }

    #[test]
    fn test_DOCKER005_has_no_install_recommends() {
        let dockerfile = "RUN apt-get install -y --no-install-recommends curl\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
