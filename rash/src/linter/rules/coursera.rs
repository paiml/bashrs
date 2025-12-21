//! Coursera Lab Image Linting Rules (COURSERA001-020)
//!
//! These rules validate Dockerfiles for Coursera Labs compatibility.
//! Reference: <https://www.coursera.support/s/article/360062379011-Coursera-Labs-Requirements-Specifications-and-Limitations>
//!
//! ## Coursera Labs Platform Constraints
//!
//! - Single port support (port 80, 443, or 1025-65535)
//! - Maximum 10GB image size (5GB recommended)
//! - Maximum 4GB memory allocation
//! - HEALTHCHECK required for startup validation
//! - Non-root user recommended

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// COURSERA001: Single port exposed
///
/// Coursera Labs supports only single-port containers.
/// Multiple EXPOSE directives will cause deployment failures.
///
/// ## Example
///
/// ❌ **BAD** (multiple ports):
/// ```dockerfile
/// EXPOSE 80
/// EXPOSE 443
/// EXPOSE 8080
/// ```
///
/// ✅ **GOOD** (single port):
/// ```dockerfile
/// EXPOSE 8888
/// ```
pub fn check_coursera001(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let expose_lines: Vec<(usize, &str)> = source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim().to_uppercase();
            trimmed.starts_with("EXPOSE ")
        })
        .collect();

    if expose_lines.len() > 1 {
        if let Some((first_line, _)) = expose_lines.first() {
            let span = Span::new(*first_line + 1, 1, *first_line + 1, 7);
            let diag = Diagnostic::new(
                "COURSERA001",
                Severity::Warning,
                format!(
                    "Multiple EXPOSE directives detected ({}). Coursera Labs supports only single-port containers. Remove extra EXPOSE directives, keep only the primary port.",
                    expose_lines.len()
                ),
                span,
            );
            result.add(diag);
        }
    }

    result
}

/// COURSERA003: Valid port range
///
/// Coursera Labs only supports ports 80, 443, or 1025-65535.
/// Privileged ports (1-1024 except 80, 443) are not allowed.
///
/// ## Example
///
/// ❌ **BAD** (privileged port):
/// ```dockerfile
/// EXPOSE 22
/// ```
///
/// ✅ **GOOD** (valid port):
/// ```dockerfile
/// EXPOSE 8888
/// ```
pub fn check_coursera003(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim().to_uppercase();
        if trimmed.starts_with("EXPOSE ") {
            let port_str = trimmed.strip_prefix("EXPOSE ").unwrap_or("").trim();
            // Handle multiple ports on same line: EXPOSE 80 443
            for port_part in port_str.split_whitespace() {
                // Handle port/protocol format: 8080/tcp
                let port_num_str = port_part.split('/').next().unwrap_or(port_part);
                if let Ok(port) = port_num_str.parse::<u16>() {
                    if !is_valid_coursera_port(port) {
                        let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                        let diag = Diagnostic::new(
                            "COURSERA003",
                            Severity::Warning,
                            format!(
                                "Port {} is outside allowed range. Coursera Labs only supports ports 80, 443, or 1025-65535. Use a valid port.",
                                port
                            ),
                            span,
                        );
                        result.add(diag);
                    }
                }
            }
        }
    }

    result
}

/// COURSERA006: HEALTHCHECK required
///
/// Coursera Labs requires HEALTHCHECK for startup validation.
/// Without it, the platform cannot determine if the container is ready.
///
/// ## Example
///
/// ❌ **BAD** (missing HEALTHCHECK):
/// ```dockerfile
/// FROM jupyter/base-notebook:latest
/// CMD ["jupyter", "notebook"]
/// ```
///
/// ✅ **GOOD** (with HEALTHCHECK):
/// ```dockerfile
/// FROM jupyter/base-notebook:latest
/// HEALTHCHECK --interval=30s CMD curl -f http://localhost:8888/ || exit 1
/// CMD ["jupyter", "notebook"]
/// ```
pub fn check_coursera006(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let has_healthcheck = source
        .lines()
        .any(|line| line.trim().to_uppercase().starts_with("HEALTHCHECK "));

    if !has_healthcheck {
        // Point to end of file as suggestion location
        let line_count = source.lines().count().max(1);
        let span = Span::new(line_count, 1, line_count, 1);
        let diag = Diagnostic::new(
            "COURSERA006",
            Severity::Warning,
            "Missing HEALTHCHECK directive. Coursera Labs requires HEALTHCHECK for startup validation. Add HEALTHCHECK --interval=30s CMD curl -f http://localhost:PORT/ || exit 1".to_string(),
            span,
        );
        result.add(diag);
    }

    result
}

/// COURSERA014: Running as root
///
/// Containers should not run as root for security.
/// Coursera Labs recommends using a non-root user.
///
/// ## Example
///
/// ❌ **BAD** (runs as root):
/// ```dockerfile
/// FROM ubuntu:22.04
/// USER root
/// ```
///
/// ✅ **GOOD** (non-root user):
/// ```dockerfile
/// FROM jupyter/base-notebook:latest
/// USER jovyan
/// ```
pub fn check_coursera014(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Find last USER directive
    let last_user = source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim().to_uppercase().starts_with("USER "))
        .last();

    match last_user {
        None => {
            // No USER directive - runs as root
            let line_count = source.lines().count().max(1);
            let span = Span::new(line_count, 1, line_count, 1);
            let diag = Diagnostic::new(
                "COURSERA014",
                Severity::Warning,
                "No USER directive found. Container will run as root. Add USER directive to run as non-root user (e.g., 'jovyan')".to_string(),
                span,
            );
            result.add(diag);
        }
        Some((line_num, line)) => {
            let user = line
                .trim()
                .strip_prefix("USER ")
                .or_else(|| line.trim().strip_prefix("user "))
                .unwrap_or("")
                .trim();
            if user == "root" || user == "0" {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                let diag = Diagnostic::new(
                    "COURSERA014",
                    Severity::Warning,
                    "Final USER is root. Container should run as non-root user. Change to non-root user like 'jovyan' or create a dedicated user".to_string(),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

/// COURSERA020: apt cache cleanup
///
/// Dockerfile should clean apt cache to reduce image size.
/// Coursera Labs has a 10GB image size limit (5GB recommended).
///
/// ## Example
///
/// ❌ **BAD** (no cleanup):
/// ```dockerfile
/// RUN apt-get update && apt-get install -y python3
/// ```
///
/// ✅ **GOOD** (with cleanup):
/// ```dockerfile
/// RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*
/// ```
pub fn check_coursera020(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("run ") && trimmed.contains("apt-get install") {
            // Check if cleanup is in the same RUN command
            let has_cleanup = trimmed.contains("rm -rf /var/lib/apt/lists")
                || trimmed.contains("apt-get clean")
                || trimmed.contains("apt-get autoremove");

            if !has_cleanup {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                let diag = Diagnostic::new(
                    "COURSERA020",
                    Severity::Warning,
                    "apt-get install without cache cleanup adds ~200MB bloat. Coursera Labs has 10GB limit. Add '&& rm -rf /var/lib/apt/lists/*' at end of RUN command".to_string(),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

/// Check if port is valid for Coursera Labs
fn is_valid_coursera_port(port: u16) -> bool {
    port == 80 || port == 443 || port >= 1025
}

/// Run all Coursera profile lint rules on a Dockerfile
pub fn lint_dockerfile_coursera(source: &str) -> LintResult {
    let mut result = LintResult::new();

    result.merge(check_coursera001(source));
    result.merge(check_coursera003(source));
    result.merge(check_coursera006(source));
    result.merge(check_coursera014(source));
    result.merge(check_coursera020(source));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // COURSERA001: Single Port Tests
    // ========================================

    #[test]
    fn test_coursera001_multiple_expose_triggers_warning() {
        let dockerfile = r#"
FROM nginx:latest
EXPOSE 80
EXPOSE 443
EXPOSE 8080
"#;
        let result = check_coursera001(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA001 should trigger on multiple EXPOSE"
        );
        assert_eq!(result.diagnostics[0].code, "COURSERA001");
    }

    #[test]
    fn test_coursera001_single_expose_passes() {
        let dockerfile = r#"
FROM nginx:latest
EXPOSE 8888
"#;
        let result = check_coursera001(dockerfile);
        assert!(result.diagnostics.is_empty(), "Single EXPOSE should pass");
    }

    #[test]
    fn test_coursera001_no_expose_passes() {
        let dockerfile = r#"
FROM nginx:latest
CMD ["nginx", "-g", "daemon off;"]
"#;
        let result = check_coursera001(dockerfile);
        assert!(result.diagnostics.is_empty(), "No EXPOSE should pass");
    }

    // ========================================
    // COURSERA003: Valid Port Range Tests
    // ========================================

    #[test]
    fn test_coursera003_invalid_port_22() {
        let dockerfile = r#"
FROM ubuntu:22.04
EXPOSE 22
"#;
        let result = check_coursera003(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA003 should trigger on port 22"
        );
        assert_eq!(result.diagnostics[0].code, "COURSERA003");
    }

    #[test]
    fn test_coursera003_valid_port_8888() {
        let dockerfile = r#"
FROM jupyter/base-notebook:latest
EXPOSE 8888
"#;
        let result = check_coursera003(dockerfile);
        assert!(result.diagnostics.is_empty(), "Port 8888 should be valid");
    }

    #[test]
    fn test_coursera003_valid_port_80() {
        let dockerfile = r#"
FROM nginx:latest
EXPOSE 80
"#;
        let result = check_coursera003(dockerfile);
        assert!(result.diagnostics.is_empty(), "Port 80 should be valid");
    }

    #[test]
    fn test_coursera003_valid_port_443() {
        let dockerfile = r#"
FROM nginx:latest
EXPOSE 443
"#;
        let result = check_coursera003(dockerfile);
        assert!(result.diagnostics.is_empty(), "Port 443 should be valid");
    }

    #[test]
    fn test_coursera003_invalid_port_1024() {
        let dockerfile = r#"
FROM ubuntu:22.04
EXPOSE 1024
"#;
        let result = check_coursera003(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "Port 1024 should be invalid"
        );
    }

    #[test]
    fn test_coursera003_port_with_protocol() {
        let dockerfile = r#"
FROM ubuntu:22.04
EXPOSE 22/tcp
"#;
        let result = check_coursera003(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "Port 22/tcp should be invalid"
        );
    }

    // ========================================
    // COURSERA006: HEALTHCHECK Tests
    // ========================================

    #[test]
    fn test_coursera006_missing_healthcheck() {
        let dockerfile = r#"
FROM jupyter/base-notebook:latest
CMD ["jupyter", "notebook"]
"#;
        let result = check_coursera006(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA006 should trigger on missing HEALTHCHECK"
        );
        assert_eq!(result.diagnostics[0].code, "COURSERA006");
    }

    #[test]
    fn test_coursera006_has_healthcheck() {
        let dockerfile = r#"
FROM jupyter/base-notebook:latest
HEALTHCHECK --interval=30s CMD curl -f http://localhost:8888/ || exit 1
CMD ["jupyter", "notebook"]
"#;
        let result = check_coursera006(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "With HEALTHCHECK should pass"
        );
    }

    #[test]
    fn test_coursera006_healthcheck_lowercase() {
        let dockerfile = r#"
FROM jupyter/base-notebook:latest
healthcheck --interval=30s CMD curl -f http://localhost:8888/ || exit 1
"#;
        let result = check_coursera006(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "Lowercase healthcheck should be detected"
        );
    }

    // ========================================
    // COURSERA014: No Root User Tests
    // ========================================

    #[test]
    fn test_coursera014_no_user_directive() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update
"#;
        let result = check_coursera014(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA014 should trigger without USER"
        );
        assert_eq!(result.diagnostics[0].code, "COURSERA014");
    }

    #[test]
    fn test_coursera014_user_root() {
        let dockerfile = r#"
FROM ubuntu:22.04
USER root
RUN apt-get update
"#;
        let result = check_coursera014(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA014 should trigger on USER root"
        );
    }

    #[test]
    fn test_coursera014_user_uid_zero() {
        let dockerfile = r#"
FROM ubuntu:22.04
USER 0
"#;
        let result = check_coursera014(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA014 should trigger on USER 0"
        );
    }

    #[test]
    fn test_coursera014_non_root_user() {
        let dockerfile = r#"
FROM jupyter/base-notebook:latest
USER jovyan
"#;
        let result = check_coursera014(dockerfile);
        assert!(result.diagnostics.is_empty(), "Non-root user should pass");
    }

    #[test]
    fn test_coursera014_final_user_non_root() {
        let dockerfile = r#"
FROM ubuntu:22.04
USER root
RUN apt-get update
USER appuser
"#;
        let result = check_coursera014(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "Final non-root user should pass"
        );
    }

    // ========================================
    // COURSERA020: apt Cleanup Tests
    // ========================================

    #[test]
    fn test_coursera020_apt_without_cleanup() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3
"#;
        let result = check_coursera020(dockerfile);
        assert!(
            !result.diagnostics.is_empty(),
            "COURSERA020 should trigger without cleanup"
        );
        assert_eq!(result.diagnostics[0].code, "COURSERA020");
    }

    #[test]
    fn test_coursera020_apt_with_rm_cleanup() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3 && rm -rf /var/lib/apt/lists/*
"#;
        let result = check_coursera020(dockerfile);
        assert!(result.diagnostics.is_empty(), "With rm cleanup should pass");
    }

    #[test]
    fn test_coursera020_apt_with_clean() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3 && apt-get clean
"#;
        let result = check_coursera020(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "With apt-get clean should pass"
        );
    }

    #[test]
    fn test_coursera020_apt_with_autoremove() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y python3 && apt-get autoremove
"#;
        let result = check_coursera020(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "With apt-get autoremove should pass"
        );
    }

    #[test]
    fn test_coursera020_no_apt_install() {
        let dockerfile = r#"
FROM ubuntu:22.04
RUN apt-get update
"#;
        let result = check_coursera020(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "Without apt-get install should pass"
        );
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_lint_dockerfile_coursera_all_rules() {
        let dockerfile = r#"
FROM ubuntu:22.04
EXPOSE 22
EXPOSE 80
RUN apt-get update && apt-get install -y python3
CMD ["python3"]
"#;
        let result = lint_dockerfile_coursera(dockerfile);

        // Should trigger:
        // - COURSERA001 (multiple EXPOSE)
        // - COURSERA003 (port 22)
        // - COURSERA006 (missing HEALTHCHECK)
        // - COURSERA014 (no USER directive)
        // - COURSERA020 (apt without cleanup)
        assert!(
            result.diagnostics.len() >= 5,
            "Should detect multiple issues: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_lint_dockerfile_coursera_compliant() {
        let dockerfile = r#"
FROM jupyter/base-notebook:latest
EXPOSE 8888
USER jovyan
HEALTHCHECK --interval=30s CMD curl -f http://localhost:8888/ || exit 1
CMD ["jupyter", "notebook"]
"#;
        let result = lint_dockerfile_coursera(dockerfile);
        assert!(
            result.diagnostics.is_empty(),
            "Compliant Dockerfile should have no warnings: {:?}",
            result.diagnostics
        );
    }

    // ========================================
    // Property-based Tests
    // ========================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]
            #[test]
            fn prop_coursera001_never_panics(dockerfile in ".*") {
                let _ = check_coursera001(&dockerfile);
            }

            #[test]
            fn prop_coursera003_never_panics(dockerfile in ".*") {
                let _ = check_coursera003(&dockerfile);
            }

            #[test]
            fn prop_coursera006_never_panics(dockerfile in ".*") {
                let _ = check_coursera006(&dockerfile);
            }

            #[test]
            fn prop_coursera014_never_panics(dockerfile in ".*") {
                let _ = check_coursera014(&dockerfile);
            }

            #[test]
            fn prop_coursera020_never_panics(dockerfile in ".*") {
                let _ = check_coursera020(&dockerfile);
            }

            #[test]
            fn prop_valid_port_range_80(port in 80u16..=80u16) {
                prop_assert!(is_valid_coursera_port(port));
            }

            #[test]
            fn prop_valid_port_range_443(port in 443u16..=443u16) {
                prop_assert!(is_valid_coursera_port(port));
            }

            #[test]
            fn prop_valid_port_range_high(port in 1025u16..=65535u16) {
                prop_assert!(is_valid_coursera_port(port));
            }

            #[test]
            fn prop_invalid_port_range_privileged(port in 1u16..=79u16) {
                prop_assert!(!is_valid_coursera_port(port));
            }

            #[test]
            fn prop_invalid_port_range_privileged_mid(port in 81u16..=442u16) {
                prop_assert!(!is_valid_coursera_port(port));
            }

            #[test]
            fn prop_invalid_port_range_privileged_high(port in 444u16..=1024u16) {
                prop_assert!(!is_valid_coursera_port(port));
            }
        }
    }
}
