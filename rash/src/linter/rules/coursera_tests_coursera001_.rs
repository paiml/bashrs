
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
