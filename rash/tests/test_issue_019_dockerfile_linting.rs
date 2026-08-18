#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Test for Issue #19: Dockerfile-specific linting support
//!
//! GitHub Issue: https://github.com/paiml/bashrs/issues/19
//! Feature Request: "Dockerfile-specific linting support in bashrs"
//!
//! GOAL:
//! Add Dockerfile-specific linting beyond just scoring bash in RUN commands.
//! Detect Dockerfile best practices, security issues, and optimization opportunities.
//!
//! Test methodology: EXTREME TDD (RED → GREEN → REFACTOR)

use bashrs::linter::rules::lint_dockerfile;
use bashrs::linter::Severity;

/// Issue #19.1: Detect missing USER directive (security risk)
///
/// RED PHASE: This test should FAIL initially
#[test]
fn test_issue_019_docker001_missing_user_directive() {
    let dockerfile = r#"
FROM debian:12-slim

WORKDIR /app
COPY app.py .

CMD ["python3", "app.py"]
"#;

    let result = lint_dockerfile(dockerfile);

    // Should warn about running as root
    let docker001_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER001")
        .collect();

    assert_eq!(
        docker001_errors.len(),
        1,
        "DOCKER001 should warn about missing USER directive. Found {} errors",
        docker001_errors.len()
    );

    let diag = &docker001_errors[0];
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.message.contains("USER"));
}

/// Issue #19.1b: No warning for scratch images (no users in scratch)
#[test]
fn test_issue_019_docker001_scratch_no_warning() {
    let dockerfile = r#"
FROM scratch

COPY binary /app

ENTRYPOINT ["/app"]
"#;

    let result = lint_dockerfile(dockerfile);

    // Should NOT warn for scratch images
    let docker001_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER001")
        .collect();

    assert_eq!(
        docker001_errors.len(),
        0,
        "DOCKER001 should not warn for scratch images"
    );
}

/// Issue #19.1c: No warning when USER directive present
#[test]
fn test_issue_019_docker001_user_present() {
    let dockerfile = r#"
FROM debian:12-slim

WORKDIR /app
COPY app.py .

RUN useradd -m appuser
USER appuser

CMD ["python3", "app.py"]
"#;

    let result = lint_dockerfile(dockerfile);

    // Should NOT warn when USER is present
    let docker001_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER001")
        .collect();

    assert_eq!(
        docker001_errors.len(),
        0,
        "DOCKER001 should not warn when USER directive present"
    );
}

/// Issue #19.2: Detect unpinned base images (security risk)
#[test]
fn test_issue_019_docker002_unpinned_base_image() {
    let dockerfile = r#"
FROM debian:12-slim

WORKDIR /app
"#;

    let result = lint_dockerfile(dockerfile);

    // Should warn about unpinned base image
    let docker002_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER002")
        .collect();

    assert_eq!(
        docker002_errors.len(),
        1,
        "DOCKER002 should warn about unpinned base image (no SHA256). Found {} errors",
        docker002_errors.len()
    );

    let diag = &docker002_errors[0];
    assert!(diag.message.contains("SHA256") || diag.message.contains("sha256"));
}

/// Issue #19.2b: No warning for pinned base images
#[test]
fn test_issue_019_docker002_pinned_base_image() {
    let dockerfile = r#"
FROM debian:12-slim@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef

WORKDIR /app
"#;

    let result = lint_dockerfile(dockerfile);

    // Should NOT warn for pinned images
    let docker002_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER002")
        .collect();

    assert_eq!(
        docker002_errors.len(),
        0,
        "DOCKER002 should not warn for SHA256-pinned images"
    );
}

/// Issue #19.3: Detect missing apt-get cleanup
#[test]
fn test_issue_019_docker003_missing_apt_cleanup() {
    let dockerfile = r#"
FROM debian:12-slim

RUN apt-get update && apt-get install -y curl
"#;

    let result = lint_dockerfile(dockerfile);

    // Should warn about missing cleanup
    let docker003_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER003")
        .collect();

    assert_eq!(
        docker003_errors.len(),
        1,
        "DOCKER003 should warn about missing apt cleanup. Found {} errors",
        docker003_errors.len()
    );
}

/// Issue #19.3b: No warning when cleanup present
#[test]
fn test_issue_019_docker003_cleanup_present() {
    let dockerfile = r#"
FROM debian:12-slim

RUN apt-get update && \
    apt-get install -y curl && \
    rm -rf /var/lib/apt/lists/*
"#;

    let result = lint_dockerfile(dockerfile);

    // Should NOT warn when cleanup present
    let docker003_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER003")
        .collect();

    assert_eq!(
        docker003_errors.len(),
        0,
        "DOCKER003 should not warn when apt cleanup present"
    );
}

/// Issue #19.4: Detect invalid COPY --from references
#[test]
fn test_issue_019_docker004_invalid_copy_from() {
    let dockerfile = r#"
FROM debian:12-slim AS builder

WORKDIR /build
RUN echo "build"

FROM debian:12-slim

COPY --from=nonexistent /build/app /app
"#;

    let result = lint_dockerfile(dockerfile);

    // Should warn about invalid --from reference
    let docker004_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER004")
        .collect();

    assert_eq!(
        docker004_errors.len(),
        1,
        "DOCKER004 should warn about invalid COPY --from reference. Found {} errors",
        docker004_errors.len()
    );
}

/// Issue #19.4b: No warning for valid COPY --from
#[test]
fn test_issue_019_docker004_valid_copy_from() {
    let dockerfile = r#"
FROM debian:12-slim AS builder

WORKDIR /build
RUN echo "build"

FROM debian:12-slim

COPY --from=builder /build/app /app
"#;

    let result = lint_dockerfile(dockerfile);

    // Should NOT warn for valid --from reference
    let docker004_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER004")
        .collect();

    assert_eq!(
        docker004_errors.len(),
        0,
        "DOCKER004 should not warn for valid COPY --from reference"
    );
}

/// Issue #19.5: Comprehensive test on ruchy-docker Dockerfile
#[test]
fn test_issue_019_comprehensive_rust_dockerfile() {
    // Real Dockerfile from ruchy-docker (rust/fibonacci.Dockerfile)
    let dockerfile = r#"
# Multi-stage Dockerfile for Fibonacci benchmark (Rust)
FROM rust:1.83-slim AS builder

WORKDIR /build
COPY benchmarks/fibonacci/main.rs .

RUN echo 'cargo config' > Cargo.toml && \
    cargo build --release

FROM scratch

COPY --from=builder /build/target/release/fibonacci /fibonacci

ENTRYPOINT ["/fibonacci"]
"#;

    let result = lint_dockerfile(dockerfile);

    println!("\n=== Issue #19 Dockerfile Linting ===");
    println!("Total diagnostics: {}", result.diagnostics.len());
    for diag in &result.diagnostics {
        println!("  - {}: {}", diag.code, diag.message);
    }
    println!("===================================\n");

    // This Dockerfile is well-structured:
    // - Multi-stage build ✓
    // - Uses scratch (no USER needed) ✓
    // - COPY --from=builder is valid ✓
    // - Should have minimal warnings

    // Should warn about unpinned base images (DOCKER002)
    let docker002_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER002")
        .count();

    assert!(
        docker002_count >= 1,
        "Should warn about unpinned base image (rust:1.83-slim without SHA256)"
    );

    // Should NOT warn about missing USER (scratch image)
    let docker001_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER001")
        .count();

    assert_eq!(
        docker001_count, 0,
        "Should not warn about missing USER for scratch image"
    );

    // Should NOT warn about invalid COPY --from
    let docker004_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER004")
        .count();

    assert_eq!(
        docker004_count, 0,
        "Should not warn about COPY --from=builder (valid reference)"
    );
}
