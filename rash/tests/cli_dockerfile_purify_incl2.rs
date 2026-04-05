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

include!("cli_dockerfile_purify_incl2_incl2.rs");
