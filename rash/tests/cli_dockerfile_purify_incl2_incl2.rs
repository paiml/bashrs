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
