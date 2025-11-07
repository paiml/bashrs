//! DOCKER006: Use COPY instead of ADD (best practice)
//!
//! Equivalent to hadolint DL3020
//!
//! **Rule**: Prefer COPY over ADD for files/directories
//!
//! **Why this matters**:
//! ADD has implicit magic behavior (tar extraction, URL fetching).
//! COPY is explicit and predictable. Use ADD only when you need
//! tar extraction.
//!
//! ## Examples
//!
//! ❌ **BAD** (ADD for regular files):
//! ```dockerfile
//! ADD app.py /app/
//! ADD config.json /etc/
//! ```
//!
//! ✅ **GOOD** (COPY for files, ADD for archives):
//! ```dockerfile
//! COPY app.py /app/
//! COPY config.json /etc/
//! ADD archive.tar.gz /tmp/  # OK - extracting archive
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("ADD ") {
            // Allow ADD for .tar.gz, .tar, .zip archives
            let is_archive = trimmed.contains(".tar.gz")
                || trimmed.contains(".tar")
                || trimmed.contains(".tgz")
                || trimmed.contains(".zip");

            // Allow ADD for URLs
            let is_url = trimmed.contains("http://") || trimmed.contains("https://");

            if !is_archive && !is_url {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len());
                let diag = Diagnostic::new(
                    "DOCKER006",
                    Severity::Warning,
                    "Use COPY instead of ADD for files/directories - ADD has implicit behavior (hadolint DL3020)".to_string(),
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
    fn test_DOCKER006_add_regular_file() {
        let dockerfile = "ADD app.py /app/\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "DOCKER006");
    }

    #[test]
    fn test_DOCKER006_add_tar_allowed() {
        let dockerfile = "ADD archive.tar.gz /tmp/\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DOCKER006_add_url_allowed() {
        let dockerfile = "ADD https://example.com/file.txt /app/\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DOCKER006_copy_is_fine() {
        let dockerfile = "COPY app.py /app/\n";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
