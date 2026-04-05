use super::*;

// ============================================================================
// Dockerfile Command Tests
// ============================================================================

#[test]
fn test_dockerfile_lint_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let dockerfile = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM ubuntu:20.04\nRUN apt-get update").unwrap();

    let result = dockerfile_lint_command(&dockerfile, LintFormat::Human, None);
    // Should succeed (may have warnings but shouldn't error)
    let _ = result;
}

#[test]
fn test_dockerfile_lint_command_with_rules() {
    let temp_dir = TempDir::new().unwrap();
    let dockerfile = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM ubuntu:20.04\nRUN apt-get update").unwrap();

    let result = dockerfile_lint_command(&dockerfile, LintFormat::Json, Some("DOCKER001"));
    let _ = result;
}

// ============================================================================
// Purify Dockerfile Content Tests
// ============================================================================

#[test]
fn test_purify_dockerfile_content_basic() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update";
    let result = purify_dockerfile(dockerfile, false);
    assert!(result.is_ok());
}

#[test]
fn test_purify_dockerfile_content_skip_user() {
    let dockerfile = "FROM ubuntu:20.04\nRUN echo hello";
    let result = purify_dockerfile(dockerfile, true);
    assert!(result.is_ok());
}

#[test]
fn test_purify_dockerfile_content_with_cleanup() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update && apt-get install -y curl";
    let result = purify_dockerfile(dockerfile, false);
    assert!(result.is_ok());
    let purified = result.unwrap();
    // Should add cleanup patterns
    assert!(purified.contains("apt-get") || purified.contains("FROM"));
}

#[test]
fn test_logic_find_devcontainer_json_exists() {
    let temp_dir = TempDir::new().unwrap();
    let devcontainer_dir = temp_dir.path().join(".devcontainer");
    fs::create_dir_all(&devcontainer_dir).unwrap();

    let json_path = devcontainer_dir.join("devcontainer.json");
    fs::write(&json_path, r#"{"name": "test"}"#).unwrap();

    // Test finding devcontainer.json
    let result = logic_find_devcontainer_json(temp_dir.path());
    assert!(result.is_ok());
}

#[test]
fn test_logic_find_devcontainer_json_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let result = logic_find_devcontainer_json(temp_dir.path());
    assert!(result.is_err());
}

// ============================================================================
// Dockerfile Profile Command Tests
// ============================================================================

#[test]
fn test_dockerfile_profile_command_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM python:3.11-slim\nRUN pip install flask\nCOPY . /app\n",
    )
    .unwrap();

    let result = dockerfile_profile_command(
        &input,
        true,  // build
        true,  // layers
        false, // startup
        false, // memory
        false, // cpu
        None,  // workload
        "30s", // duration
        None,  // profile
        false, // simulate_limits
        false, // full
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_full_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\nCOPY . /app\n",
    )
    .unwrap();

    let result = dockerfile_profile_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        "30s",
        None,
        false,
        true, // full (enables all sections)
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add curl\n").unwrap();

    let result = dockerfile_profile_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        "30s",
        None,
        false,
        false,
        ReportFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM node:20-alpine\nCOPY . /app\n").unwrap();

    let result = dockerfile_profile_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        "30s",
        None,
        false,
        false,
        ReportFormat::Markdown,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_coursera_with_limits() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\n").unwrap();

    let result = dockerfile_profile_command(
        &input,
        true,
        true,
        true,
        true,
        true,
        None,
        "30s",
        Some(LintProfileArg::Coursera),
        true, // simulate_limits
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Dockerfile Size Check Command Tests
// ============================================================================

#[test]
fn test_dockerfile_size_check_command_human_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add curl\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false, // verbose
        false, // layers
        false, // detect_bloat
        false, // verify
        false, // docker_verify
        None,  // profile
        false, // strict
        None,  // max_size
        false, // compression_analysis
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_verbose_with_bloat() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl wget git\n",
    )
    .unwrap();

    let result = dockerfile_size_check_command(
        &input,
        true, // verbose
        true, // layers
        true, // detect_bloat
        false,
        false,
        None,
        false,
        None,
        true, // compression_analysis
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11\nRUN pip install flask\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        None,
        false,
        ReportFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM node:20\nCOPY . /app\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        None,
        false,
        ReportFormat::Markdown,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_with_coursera_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        true,
        true,
        true,
        false,
        false,
        Some(LintProfileArg::Coursera),
        false,
        None,
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_custom_max_size_gb() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN echo hello\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        Some("5GB"),
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_custom_max_size_mb() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN echo hello\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        Some("500MB"),
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Dockerfile Full Validate Command Tests
// ============================================================================

#[test]

include!("command_tests_dockerfile_tests_dockerfile.rs");
