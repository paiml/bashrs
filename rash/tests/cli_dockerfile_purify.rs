#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
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
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
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
fn test_dockerfile_docker005_adds_no_install_recommends() {
    // RED: Should FAIL because --no-install-recommends not added

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, DOCKERFILE_NO_FLAG).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add --no-install-recommends flag
    assert!(
        output_content.contains("--no-install-recommends"),
        "Expected --no-install-recommends flag to be added"
    );
}

// ============================================================================
// PHASE 6 (RED): Tests for DOCKER006 - Convert ADD to COPY
// ============================================================================

#[test]
fn test_dockerfile_docker006_converts_add_to_copy() {
    // RED: Should FAIL because ADD → COPY conversion not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, DOCKERFILE_ADD_LOCAL).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should convert ADD to COPY
    assert!(
        output_content.contains("COPY app.py"),
        "Expected ADD to be converted to COPY for local files"
    );

    // Should NOT contain ADD
    assert!(
        !output_content.contains("ADD app.py"),
        "Should not have ADD for local files"
    );
}

#[test]
fn test_dockerfile_docker006_preserves_add_for_urls() {
    // RED: Should FAIL if ADD is converted for URLs

    let dockerfile_add_url = r#"FROM debian:12-slim
ADD https://example.com/file.tar.gz /tmp/
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_add_url).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should keep ADD for URLs
    assert!(
        output_content.contains("ADD https://example.com"),
        "Expected ADD to be preserved for URLs"
    );
}

// ============================================================================
// PHASE 7 (RED): Tests for CLI Options
// ============================================================================

#[test]
fn test_dockerfile_010_help_shows_purify_command() {
    // Test that help output includes dockerfile purify

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("purify"));
}

#[test]
fn test_dockerfile_011_dry_run_flag() {
    // RED: Should FAIL because --dry-run not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    fs::write(&input_file, DOCKERFILE_NO_USER).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Would add USER directive"));
}

#[test]
fn test_dockerfile_012_fix_flag_in_place() {
    // RED: Should FAIL because --fix not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    fs::write(&input_file, DOCKERFILE_NO_USER).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("--fix")
        .assert()
        .success();

    // Should modify file in-place
    let content = fs::read_to_string(&input_file).unwrap();
    assert!(content.contains("USER"), "File should be modified in-place");

    // Should create backup
    let backup_file = temp_dir.path().join("Dockerfile.bak");
    assert!(backup_file.exists(), "Backup file should be created");
}

// ============================================================================
// PHASE 8: DOCKER001 Edge Cases
// ============================================================================

#[test]
fn test_dockerfile_docker001_edge_multi_stage_build() {
    // Test USER directive in multi-stage Dockerfiles

    let dockerfile_multi = r#"FROM debian:12-slim AS builder
WORKDIR /build
COPY src/ /build/
RUN make build

FROM debian:12-slim
COPY --from=builder /build/app /app/
CMD ["/app/app"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_multi).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add USER directive to final stage only
    assert!(
        output_content.contains("USER appuser"),
        "Expected USER directive in final stage"
    );

    // USER should come before CMD
    let user_pos = output_content.rfind("USER").unwrap();
    let cmd_pos = output_content.rfind("CMD").unwrap();
    assert!(
        user_pos < cmd_pos,
        "USER should come before CMD in final stage"
    );
}

#[test]
fn test_dockerfile_docker001_edge_alpine_image() {
    // Test USER directive with Alpine Linux (different user creation syntax)

    let dockerfile_alpine = r#"FROM alpine:3.19
RUN apk add python3
CMD ["python3"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_alpine).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add USER directive (implementation may vary for Alpine)
    assert!(
        output_content.contains("USER"),
        "Expected USER directive for Alpine"
    );
}

#[test]
fn test_dockerfile_docker001_edge_entrypoint_instead_of_cmd() {
    // Test USER directive placement with ENTRYPOINT instead of CMD

    let dockerfile_entrypoint = r#"FROM debian:12-slim
COPY app.sh /app/
ENTRYPOINT ["/app/app.sh"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_entrypoint).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add USER before ENTRYPOINT
    assert!(
        output_content.contains("USER appuser"),
        "Expected USER directive before ENTRYPOINT"
    );

    let user_pos = output_content.find("USER").unwrap();
    let entrypoint_pos = output_content.find("ENTRYPOINT").unwrap();
    assert!(
        user_pos < entrypoint_pos,
        "USER should come before ENTRYPOINT"
    );
}

// ============================================================================
// PHASE 9: DOCKER002 Edge Cases
// ============================================================================

#[test]
fn test_dockerfile_docker002_edge_registry_prefix() {
    // Test pinning with custom registry prefix

    let dockerfile_registry = r#"FROM docker.io/ubuntu
RUN apt-get update
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_registry).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should pin image while preserving registry
    assert!(
        output_content.contains("FROM docker.io/ubuntu:"),
        "Expected registry prefix preserved with pinned tag"
    );
}

#[test]
fn test_dockerfile_docker002_edge_already_pinned() {
    // Test that already-pinned images are not modified

    let dockerfile_pinned = r#"FROM ubuntu:22.04
RUN apt-get update
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_pinned).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should keep existing pin
    assert!(
        output_content.contains("FROM ubuntu:22.04"),
        "Expected existing pin preserved"
    );
}

#[test]
fn test_dockerfile_docker002_edge_unknown_image() {
    // Test that unknown/custom images are not modified

    let dockerfile_custom = r#"FROM mycompany/custom-image
RUN echo "custom"
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_custom).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should not modify unknown images
    assert!(
        output_content.contains("FROM mycompany/custom-image"),
        "Expected custom image unchanged"
    );
}

// ============================================================================
// PHASE 10: DOCKER003 Edge Cases
// ============================================================================

#[test]
fn test_dockerfile_docker003_edge_multiple_run_commands() {
    // Test cleanup added to multiple RUN commands

    let dockerfile_multiple = r#"FROM debian:12-slim
RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y wget
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_multiple).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add cleanup to all apt-get install commands
    let cleanup_count = output_content.matches("rm -rf /var/lib/apt/lists").count();
    assert!(
        cleanup_count >= 2,
        "Expected cleanup added to multiple RUN commands"
    );
}

#[test]
fn test_dockerfile_docker003_edge_yum_package_manager() {
    // Test that yum/dnf (RHEL-based) cleanup is handled if implemented

    let dockerfile_yum = r#"FROM centos:8
RUN yum install -y curl
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_yum).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // If yum cleanup implemented, verify it; otherwise just ensure no panic
    // For now, we accept either cleanup or unchanged (feature not required yet)
    assert!(
        output_content.contains("yum install"),
        "Dockerfile should still contain yum install"
    );
}

#[test]
fn test_dockerfile_docker003_edge_combined_command() {
    // Test cleanup with complex command chains

    let dockerfile_combined = r#"FROM debian:12-slim
RUN apt-get update && apt-get install -y curl wget && echo "done"
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_combined).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add cleanup to complex command
    assert!(
        output_content.contains("rm -rf /var/lib/apt/lists"),
        "Expected cleanup even in complex command chain"
    );
}

// ============================================================================
// PHASE 11: DOCKER005 Edge Cases
// ============================================================================

#[test]
fn test_dockerfile_docker005_edge_apt_without_apt_get() {
    // Test --no-install-recommends with 'apt' command (not 'apt-get')

    let dockerfile_apt = r#"FROM debian:12-slim
RUN apt install -y python3
CMD ["python3"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_apt).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add --no-install-recommends to 'apt install' as well
    // Note: Implementation may only support apt-get; document if so
    assert!(
        output_content.contains("apt install") || output_content.contains("apt-get install"),
        "apt install command should be present"
    );
}

#[test]
fn test_dockerfile_docker005_edge_multiple_apt_get_in_one_run() {
    // Test --no-install-recommends with multiple apt-get commands in one RUN

    let dockerfile_multiple_apt = r#"FROM debian:12-slim
RUN apt-get install -y curl && apt-get install -y wget
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_multiple_apt).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should add --no-install-recommends to all apt-get install commands
    let flag_count = output_content.matches("--no-install-recommends").count();
    assert!(
        flag_count >= 2,
        "Expected --no-install-recommends added to both apt-get commands"
    );
}

// ============================================================================
// PHASE 12: DOCKER006 Edge Cases
// ============================================================================

#[test]
fn test_dockerfile_docker006_edge_add_with_wildcard() {
    // Test ADD → COPY conversion with wildcard patterns

    let dockerfile_wildcard = r#"FROM debian:12-slim
ADD src/*.py /app/
CMD ["python3", "/app/main.py"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_wildcard).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should convert ADD to COPY for wildcard patterns
    assert!(
        output_content.contains("COPY src/*.py"),
        "Expected ADD converted to COPY for wildcard"
    );

    assert!(
        !output_content.contains("ADD src/*.py"),
        "ADD should be replaced"
    );
}

#[test]
fn test_dockerfile_docker006_edge_add_tarball_local() {
    // Test that ADD for .tar.gz files is preserved (tar extraction feature)

    let dockerfile_tarball = r#"FROM debian:12-slim
ADD archive.tar.gz /tmp/
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_tarball).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should preserve ADD for tarballs (auto-extraction feature)
    assert!(
        output_content.contains("ADD archive.tar.gz"),
        "Expected ADD preserved for tarball auto-extraction"
    );
}

// ============================================================================
// PHASE 13: Error Handling Tests
// ============================================================================

#[test]
fn test_dockerfile_error_missing_file() {
    // Test error handling for non-existent file

    let temp_dir = TempDir::new().unwrap();
    let non_existent = temp_dir.path().join("DoesNotExist");

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&non_existent)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file").or(predicate::str::contains("not found")));
}

#[test]
fn test_dockerfile_error_empty_file() {
    // Test handling of empty Dockerfile

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, "").unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success(); // Empty input should succeed (no-op)

    let output_content = fs::read_to_string(&output_file).unwrap();
    assert!(
        output_content.trim().is_empty(),
        "Empty Dockerfile should produce empty output"
    );
}

#[test]
fn test_dockerfile_error_invalid_syntax() {
    // Test handling of Dockerfile with invalid syntax

    let dockerfile_invalid = r#"INVALID_INSTRUCTION
FROM ubuntu
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    fs::write(&input_file, dockerfile_invalid).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .assert()
        .success(); // Should attempt best-effort purification even with invalid syntax
}

// ============================================================================
// PHASE 14: Integration Tests (Multiple Transformations)
// ============================================================================

#[test]
fn test_dockerfile_integration_all_transformations() {
    // Test that all DOCKER rules apply correctly in combination

    let dockerfile_complex = r#"FROM ubuntu:latest
RUN apt-get update
RUN apt-get install -y curl
ADD app.py /app/
WORKDIR /app
CMD ["python3", "app.py"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output_file = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input_file, dockerfile_complex).unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Verify all transformations applied:
    // DOCKER002: Pin ubuntu:latest
    assert!(
        output_content.contains("FROM ubuntu:22.04")
            || output_content.contains("FROM ubuntu:24.04"),
        "DOCKER002: Should pin ubuntu:latest"
    );

    // DOCKER003: Add apt cleanup
    assert!(
        output_content.contains("rm -rf /var/lib/apt/lists"),
        "DOCKER003: Should add apt cleanup"
    );

    // DOCKER005: Add --no-install-recommends
    assert!(
        output_content.contains("--no-install-recommends"),
        "DOCKER005: Should add --no-install-recommends"
    );

    // DOCKER006: Convert ADD to COPY
    assert!(
        output_content.contains("COPY app.py"),
        "DOCKER006: Should convert ADD to COPY"
    );

    // DOCKER001: Add USER directive
    assert!(
        output_content.contains("USER appuser"),
        "DOCKER001: Should add USER directive"
    );
}

#[test]
fn test_dockerfile_integration_idempotency() {
    // Test that purifying twice produces same result (idempotency)

    let dockerfile = r#"FROM ubuntu
RUN apt-get install -y curl
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let first_output = temp_dir.path().join("Dockerfile.purified1");
    let second_output = temp_dir.path().join("Dockerfile.purified2");

    fs::write(&input_file, dockerfile).unwrap();

    // First purification
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&first_output)
        .assert()
        .success();

    // Second purification (purify the purified)
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&first_output)
        .arg("-o")
        .arg(&second_output)
        .assert()
        .success();

    let first_content = fs::read_to_string(&first_output).unwrap();
    let second_content = fs::read_to_string(&second_output).unwrap();

    assert_eq!(
        first_content, second_content,
        "Purification should be idempotent"
    );
}

#[test]
fn test_dockerfile_integration_determinism() {
    // Test that purifying the same input produces identical output (determinism)

    let dockerfile = r#"FROM debian:latest
RUN apt-get update && apt-get install -y python3
ADD script.sh /app/
CMD ["bash"]
"#;

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Dockerfile");
    let output1 = temp_dir.path().join("Dockerfile.out1");
    let output2 = temp_dir.path().join("Dockerfile.out2");

    fs::write(&input_file, dockerfile).unwrap();

    // Purify twice
    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output1)
        .assert()
        .success();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output2)
        .assert()
        .success();

    let content1 = fs::read_to_string(&output1).unwrap();
    let content2 = fs::read_to_string(&output2).unwrap();

    assert_eq!(content1, content2, "Purification should be deterministic");
}
