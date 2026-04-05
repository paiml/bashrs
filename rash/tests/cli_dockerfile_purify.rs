#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)] // Tests can use expect() for simplicity
//! Tests for Dockerfile purification (EXTREME TDD)
//!
//! Tests the `bashrs dockerfile purify` command for auto-fixing Dockerfile issues.
//!
//! ## Test Coverage (RED Phase)
//! - DOCKER001: Add missing USER directive
//! - DOCKER002: Pin unpinned base images
//! - DOCKER003: Add apt/apk cleanup
//! - DOCKER005: Add --no-install-recommends
//! - DOCKER006: Convert ADD to COPY
//!
//! ## Related
//! - docs/specifications/purify-dockerfile-spec.md
//! - Existing lint rules: DOCKER001-DOCKER010

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create test command
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

// Sample Dockerfiles for testing

/// Dockerfile missing USER directive (DOCKER001)
const DOCKERFILE_NO_USER: &str = r#"FROM debian:12-slim
WORKDIR /app
COPY app.py /app/
CMD ["python3", "app.py"]
"#;

/// Dockerfile with unpinned base image (DOCKER002)
const DOCKERFILE_UNPINNED: &str = r#"FROM ubuntu
RUN apt-get update && apt-get install -y curl
CMD ["bash"]
"#;

/// Dockerfile with apt-get but no cleanup (DOCKER003)
const DOCKERFILE_NO_CLEANUP: &str = r#"FROM debian:12-slim
RUN apt-get update && apt-get install -y curl
CMD ["bash"]
"#;

/// Dockerfile with apt-get but missing --no-install-recommends (DOCKER005)
const DOCKERFILE_NO_FLAG: &str = r#"FROM debian:12-slim
RUN apt-get install -y python3
CMD ["python3"]
"#;

/// Dockerfile using ADD for local files (DOCKER006)
const DOCKERFILE_ADD_LOCAL: &str = r#"FROM debian:12-slim
ADD app.py /app/
CMD ["python3", "/app/app.py"]
"#;

// ============================================================================
// PHASE 1 (RED): Tests for CLI interface
// ============================================================================

#[test]
fn test_dockerfile_001_purify_command_exists() {
    // RED: This should FAIL because 'dockerfile purify' subcommand doesn't exist yet

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    fs::write(&input_file, DOCKERFILE_NO_USER).unwrap();

    // Test that 'dockerfile purify' command is recognized
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .assert()
        .success(); // Should succeed (not fail with "unknown subcommand")
}

#[test]
fn test_dockerfile_002_purify_to_stdout() {
    // RED: This should FAIL because purify not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    fs::write(&input_file, DOCKERFILE_NO_USER).unwrap();

    // Purify to stdout (default behavior)
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("USER")); // Should add USER directive
}

#[test]
fn test_dockerfile_003_purify_to_file() {
    // RED: This should FAIL because -o flag not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, DOCKERFILE_NO_USER).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Output file should exist
    assert!(output_file.exists());
}

// ============================================================================
// PHASE 2 (RED): Tests for DOCKER001 - Add Missing USER Directive
// ============================================================================

#[test]
fn test_dockerfile_docker001_adds_missing_user_directive() {
    // RED: This should FAIL because DOCKER001 transformation not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, DOCKERFILE_NO_USER).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add USER directive
    assert!(
        output_content.contains("USER appuser"),
        "Expected USER directive to be added"
    );

    // Should add user creation command
    assert!(
        output_content.contains("RUN groupadd -r appuser"),
        "Expected user creation RUN command"
    );

    // USER directive should come before CMD
    let user_pos = output_content.find("USER").unwrap();
    let cmd_pos = output_content.find("CMD").unwrap();
    assert!(user_pos < cmd_pos, "USER directive should come before CMD");
}

#[test]
fn test_dockerfile_docker001_preserves_existing_user() {
    // RED: Should FAIL if existing USER is removed

    let dockerfile_with_user = r#"FROM debian:12-slim
WORKDIR /app
USER www-data
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_with_user).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should keep existing USER
    assert!(
        output_content.contains("USER www-data"),
        "Expected existing USER directive to be preserved"
    );

    // Should NOT add new user creation
    assert!(
        !output_content.contains("groupadd -r appuser"),
        "Should not add new user when one already exists"
    );
}

#[test]
fn test_dockerfile_docker001_skip_for_scratch_image() {
    // RED: Should FAIL if USER added to scratch images

    let dockerfile_scratch = r#"FROM scratch
COPY app /app
CMD ["/app"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_scratch).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should NOT add USER for scratch images
    assert!(
        !output_content.contains("USER"),
        "Should not add USER directive for scratch images"
    );
}

// ============================================================================
// PHASE 3 (RED): Tests for DOCKER002 - Pin Unpinned Base Images
// ============================================================================

#[test]
fn test_dockerfile_docker002_pins_untagged_image() {
    // RED: Should FAIL because DOCKER002 transformation not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, DOCKERFILE_UNPINNED).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should pin ubuntu to LTS version
    assert!(
        output_content.contains("FROM ubuntu:22.04")
            || output_content.contains("FROM ubuntu:24.04"),
        "Expected ubuntu to be pinned to LTS version"
    );

    // Should NOT contain unpinned ubuntu
    assert!(
        !output_content.contains("FROM ubuntu\n") && !output_content.contains("FROM ubuntu "),
        "Should not have unpinned ubuntu"
    );
}

#[test]
fn test_dockerfile_docker002_pins_latest_tag() {
    // RED: Should FAIL because :latest pinning not implemented

    let dockerfile_latest = r#"FROM debian:latest
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_latest).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should replace :latest with stable version
    assert!(
        output_content.contains("FROM debian:12") || output_content.contains("FROM debian:11"),
        "Expected :latest to be replaced with stable version"
    );
}

// ============================================================================
// PHASE 4 (RED): Tests for DOCKER003 - Add apt/apk Cleanup
// ============================================================================

#[test]
fn test_dockerfile_docker003_adds_apt_cleanup() {
    // RED: Should FAIL because apt cleanup transformation not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, DOCKERFILE_NO_CLEANUP).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add cleanup command
    assert!(
        output_content.contains("rm -rf /var/lib/apt/lists/*"),
        "Expected apt cleanup command to be added"
    );

    // Should be in same RUN command (combined with &&)
    assert!(
        output_content.contains("apt-get install") && output_content.contains("rm -rf"),
        "Cleanup should be in same RUN command"
    );
}

#[test]
fn test_dockerfile_docker003_adds_apk_cleanup() {
    // RED: Should FAIL because apk cleanup not implemented

    let dockerfile_apk = r#"FROM alpine:3.19
RUN apk add curl
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_apk).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add apk cleanup
    assert!(
        output_content.contains("rm -rf /var/cache/apk/*"),
        "Expected apk cleanup command to be added"
    );
}

// ============================================================================
// PHASE 5 (RED): Tests for DOCKER005 - Add --no-install-recommends
// ============================================================================

#[test]

include!("cli_dockerfile_purify_incl2.rs");
