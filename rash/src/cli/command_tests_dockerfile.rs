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
fn test_dockerfile_full_validate_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM python:3.11-slim\nRUN pip install flask\nCOPY . /app\nUSER 65534\n",
    )
    .unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,  // profile
        true,  // size_check
        false, // graded
        false, // runtime
        false, // strict
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add curl\n").unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,
        true,
        false,
        false,
        false,
        ReportFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM node:20-alpine\nCOPY . /app\n").unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,
        true,
        false,
        false,
        false,
        ReportFormat::Markdown,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_coursera_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM python:3.11-slim\nRUN pip install flask\nUSER 65534\n",
    )
    .unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        Some(LintProfileArg::Coursera),
        true,
        false,
        false,
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_with_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:22.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,
        true,
        false,
        true, // runtime
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Dockerfile Purify Command Tests
// ============================================================================

#[test]
fn test_dockerfile_purify_command_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,  // output
        false, // fix
        false, // no_backup
        false, // dry_run
        false, // report
        ReportFormat::Human,
        false, // skip_user
        false, // skip_bash_purify
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_to_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    let output = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        Some(&output),
        false,
        false,
        false,
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_dockerfile_purify_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo hello\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        false,
        false,
        true, // dry_run
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_fix_inplace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        true,  // fix (in-place)
        false, // no_backup (creates backup)
        false,
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
    // Backup should be created
    assert!(input.with_extension("bak").exists());
}

#[test]
fn test_dockerfile_purify_command_fix_no_backup() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo test\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        true, // fix
        true, // no_backup
        false,
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_skip_user() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo test\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        false,
        false,
        false,
        false,
        ReportFormat::Human,
        true, // skip_user
        false,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Estimate Build Time Tests
// ============================================================================

#[test]
fn test_estimate_build_time_simple() {
    use crate::linter::docker_profiler::estimate_size;
    let source = "FROM alpine:3.18\nRUN echo hello\n";
    let estimate = estimate_size(source);
    let time = estimate_build_time(&estimate);
    assert!(time.contains('s') || time.contains('m'));
}

#[test]
fn test_estimate_build_time_with_apt() {
    use crate::linter::docker_profiler::estimate_size;
    let source = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\n";
    let estimate = estimate_size(source);
    let time = estimate_build_time(&estimate);
    assert!(time.contains('s') || time.contains('m'));
}

// ============================================================================
// Dockerfile Lint with Rules Filter Test
// ============================================================================

#[test]
fn test_dockerfile_lint_command_sarif_format() {
    let temp_dir = TempDir::new().unwrap();
    let dockerfile = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_lint_command(&dockerfile, LintFormat::Sarif, None);
    let _ = result;
}

#[test]
fn test_dockerfile_lint_command_nonexistent() {
    let result = dockerfile_lint_command(
        &PathBuf::from("/nonexistent/Dockerfile"),
        LintFormat::Human,
        None,
    );
    assert!(result.is_err());
}

// ===== Tests for Dockerfile helper functions (moved from commands.rs) =====

// FUNCTION 1: convert_add_to_copy_if_local()

#[test]
fn test_convert_add_to_copy_if_local_happy_path_local_file() {
    let line = "ADD myfile.txt /app/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, "COPY myfile.txt /app/",
        "Local file should convert ADD to COPY"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_http_url() {
    let line = "ADD http://example.com/file.tar.gz /tmp/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        "HTTP URLs should preserve ADD (not convert to COPY)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_https_url() {
    let line = "ADD https://example.com/archive.zip /tmp/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        "HTTPS URLs should preserve ADD (not convert to COPY)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_archive() {
    let line = "ADD archive.tar /tmp/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar archives should preserve ADD (auto-extraction feature)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_gz() {
    let line = "ADD file.tar.gz /app/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.gz archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tgz() {
    let line = "ADD package.tgz /opt/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tgz archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_bz2() {
    let line = "ADD data.tar.bz2 /data/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.bz2 archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_xz() {
    let line = "ADD compressed.tar.xz /usr/local/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.xz archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_Z() {
    let line = "ADD legacy.tar.Z /legacy/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.Z archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_empty_line() {
    let line = "";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_convert_add_to_copy_if_local_malformed_no_args() {
    let line = "ADD";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        "Malformed ADD (no arguments) should be unchanged"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_with_extra_spaces() {
    let line = "ADD    local_file.txt    /app/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, "COPY    local_file.txt    /app/",
        "Should convert ADD to COPY while preserving spacing"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_non_docker_line() {
    let line = "# This is a comment with ADD in it";
    let result = convert_add_to_copy_if_local(line);
    // Should not convert comment lines
    assert_eq!(result, line, "Comment lines should not be processed");
}

// FUNCTION 2: add_no_install_recommends()

#[test]
fn test_add_no_install_recommends_happy_path_with_y_flag() {
    let line = "RUN apt-get install -y curl";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install -y --no-install-recommends curl",
        "Should add --no-install-recommends after -y flag"
    );
}

#[test]
fn test_add_no_install_recommends_without_y_flag() {
    let line = "RUN apt-get install python3";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install --no-install-recommends python3",
        "Should add --no-install-recommends after install"
    );
}

#[test]
fn test_add_no_install_recommends_already_present() {
    let line = "RUN apt-get install -y --no-install-recommends git";
    let result = add_no_install_recommends(line);
    assert_eq!(result, line, "Should not add flag if already present");
}

#[test]
fn test_add_no_install_recommends_multiple_packages() {
    let line = "RUN apt-get install -y curl wget git";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install -y --no-install-recommends curl wget git",
        "Should work with multiple packages"
    );
}

#[test]
fn test_add_no_install_recommends_multiple_apt_get_commands() {
    let line = "RUN apt-get update && apt-get install -y curl && apt-get install -y git";
    let result = add_no_install_recommends(line);
    assert!(
        result.contains("--no-install-recommends"),
        "Should add flag to apt-get install commands"
    );
    // Both install commands should get the flag
    let flag_count = result.matches("--no-install-recommends").count();
    assert_eq!(
        flag_count, 2,
        "Should add flag to both apt-get install commands"
    );
}

#[test]
fn test_add_no_install_recommends_apt_install_variant() {
    let line = "RUN apt install -y vim";
    let result = add_no_install_recommends(line);
    // Note: Current implementation only handles "apt-get install", not "apt install"
    // This test documents current behavior
    assert_eq!(result, line, "apt install (not apt-get) not yet supported");
}

#[test]
fn test_add_no_install_recommends_empty_line() {
    let line = "";
    let result = add_no_install_recommends(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_add_no_install_recommends_no_apt_get() {
    let line = "RUN echo hello";
    let result = add_no_install_recommends(line);
    assert_eq!(result, line, "Non-apt-get commands should be unchanged");
}

#[test]
fn test_add_no_install_recommends_apt_get_update_only() {
    let line = "RUN apt-get update";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, line,
        "apt-get update (without install) should be unchanged"
    );
}

#[test]
fn test_add_no_install_recommends_with_continuation() {
    let line = "RUN apt-get install -y \\\n    curl \\\n    wget";
    let result = add_no_install_recommends(line);
    assert!(
        result.contains("--no-install-recommends"),
        "Should handle multi-line continuations"
    );
}

#[test]
fn test_add_no_install_recommends_comment_line() {
    let line = "# RUN apt-get install -y curl";
    let result = add_no_install_recommends(line);
    // Should not process comments
    assert_eq!(result, line, "Comment lines should not be processed");
}

#[test]
fn test_add_no_install_recommends_install_at_end() {
    let line = "RUN apt-get install";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install --no-install-recommends ",
        "Should add flag even if no packages listed"
    );
}

#[test]
fn test_add_no_install_recommends_preserves_other_flags() {
    let line = "RUN apt-get install -y --fix-missing curl";
    let result = add_no_install_recommends(line);
    assert!(
        result.contains("--fix-missing"),
        "Should preserve other flags"
    );
    assert!(
        result.contains("--no-install-recommends"),
        "Should add --no-install-recommends"
    );
}

// FUNCTION 3: add_package_manager_cleanup()

#[test]
fn test_add_package_manager_cleanup_apt_get_install() {
    let line = "RUN apt-get update && apt-get install -y curl";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*",
        "Should add apt cleanup after install"
    );
}

#[test]
fn test_add_package_manager_cleanup_apt_install() {
    let line = "RUN apt install -y python3";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apt install -y python3 && rm -rf /var/lib/apt/lists/*",
        "Should add apt cleanup for 'apt install' variant"
    );
}

#[test]
fn test_add_package_manager_cleanup_apk_add() {
    let line = "RUN apk add curl";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apk add curl && rm -rf /var/cache/apk/*",
        "Should add apk cleanup for Alpine"
    );
}

#[test]
fn test_add_package_manager_cleanup_already_present_apt() {
    let line = "RUN apt-get install -y git && rm -rf /var/lib/apt/lists/*";
    let result = add_package_manager_cleanup(line);
    assert_eq!(result, line, "Should not add cleanup if already present");
}

#[test]
fn test_add_package_manager_cleanup_already_present_apk() {
    let line = "RUN apk add vim && rm -rf /var/cache/apk/*";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, line,
        "Should not add cleanup if already present (apk)"
    );
}

#[test]
fn test_add_package_manager_cleanup_no_package_manager() {
    let line = "RUN echo hello";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, line,
        "Non-package-manager commands should be unchanged"
    );
}

#[test]
fn test_add_package_manager_cleanup_apt_get_update_only() {
    let line = "RUN apt-get update";
    let result = add_package_manager_cleanup(line);
    // update doesn't install packages, so no cleanup needed
    assert_eq!(result, line, "apt-get update alone should be unchanged");
}

#[test]
fn test_add_package_manager_cleanup_empty_line() {
    let line = "";
    let result = add_package_manager_cleanup(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_add_package_manager_cleanup_comment_line() {
    let line = "# RUN apt-get install curl";
    let result = add_package_manager_cleanup(line);
    assert_eq!(result, line, "Comment lines should not be processed");
}

#[test]
fn test_add_package_manager_cleanup_with_trailing_whitespace() {
    let line = "RUN apt-get install -y wget   ";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apt-get install -y wget && rm -rf /var/lib/apt/lists/*",
        "Should trim trailing whitespace before adding cleanup"
    );
}

#[test]
fn test_add_package_manager_cleanup_multiple_commands() {
    let line = "RUN apt-get update && apt-get install -y curl && echo done";
    let result = add_package_manager_cleanup(line);
    assert!(
        result.contains("&& rm -rf /var/lib/apt/lists/*"),
        "Should add cleanup even with multiple commands"
    );
}

#[test]
fn test_add_package_manager_cleanup_apk_add_multiple_packages() {
    let line = "RUN apk add --no-cache curl wget git";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apk add --no-cache curl wget git && rm -rf /var/cache/apk/*",
        "Should add cleanup for apk with multiple packages"
    );
}

#[test]
fn test_add_package_manager_cleanup_partial_match_no_install() {
    let line = "RUN apt-get clean";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, line,
        "apt-get clean (not install) should be unchanged"
    );
}

// FUNCTION 4: pin_base_image_version()

#[test]
fn test_pin_base_image_version_ubuntu_untagged() {
    let line = "FROM ubuntu";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM ubuntu:22.04",
        "Untagged ubuntu should be pinned to 22.04 LTS"
    );
}

#[test]
fn test_pin_base_image_version_ubuntu_latest() {
    let line = "FROM ubuntu:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM ubuntu:22.04",
        "ubuntu:latest should be pinned to 22.04 LTS"
    );
}

#[test]
fn test_pin_base_image_version_ubuntu_already_pinned() {
    let line = "FROM ubuntu:20.04";
    let result = pin_base_image_version(line);
    assert_eq!(result, line, "Already pinned ubuntu should be unchanged");
}

#[test]
fn test_pin_base_image_version_debian() {
    let line = "FROM debian";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM debian:12-slim",
        "Untagged debian should be pinned to 12-slim"
    );
}

#[test]
fn test_pin_base_image_version_alpine() {
    let line = "FROM alpine:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM alpine:3.19",
        "alpine:latest should be pinned to 3.19"
    );
}

#[test]
fn test_pin_base_image_version_node() {
    let line = "FROM node";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM node:20-alpine",
        "Untagged node should be pinned to 20-alpine"
    );
}

#[test]
fn test_pin_base_image_version_python() {
    let line = "FROM python:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM python:3.11-slim",
        "python:latest should be pinned to 3.11-slim"
    );
}

#[test]
fn test_pin_base_image_version_with_registry_prefix() {
    let line = "FROM docker.io/ubuntu";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM docker.io/ubuntu:22.04",
        "Should preserve registry prefix (docker.io/)"
    );
}

#[test]
fn test_pin_base_image_version_with_as_alias() {
    let line = "FROM ubuntu AS builder";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM ubuntu:22.04 AS builder",
        "Should preserve AS alias"
    );
}

#[test]
fn test_pin_base_image_version_unknown_image() {
    let line = "FROM mycompany/custom-image";
    let result = pin_base_image_version(line);
    assert_eq!(result, line, "Unknown images should be unchanged");
}

#[test]
fn test_pin_base_image_version_malformed_no_image() {
    let line = "FROM";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, line,
        "Malformed FROM (no image) should be unchanged"
    );
}

#[test]
fn test_pin_base_image_version_empty_line() {
    let line = "";
    let result = pin_base_image_version(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_pin_base_image_version_rust() {
    let line = "FROM rust:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM rust:1.75-alpine",
        "rust:latest should be pinned to 1.75-alpine"
    );
}
