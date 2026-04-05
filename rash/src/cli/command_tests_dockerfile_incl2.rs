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

    let result = dockerfile_purify_command(DockerfilePurifyCommandArgs {
        input: &input,
        output: None,
        fix: false,
        no_backup: false,
        dry_run: false,
        skip_user: false,
    });
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_to_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    let output = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(DockerfilePurifyCommandArgs {
        input: &input,
        output: Some(&output),
        fix: false,
        no_backup: false,
        dry_run: false,
        skip_user: false,
    });
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_dockerfile_purify_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo hello\n").unwrap();

    let result = dockerfile_purify_command(DockerfilePurifyCommandArgs {
        input: &input,
        output: None,
        fix: false,
        no_backup: false,
        dry_run: true,
        skip_user: false,
    });
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_fix_inplace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(DockerfilePurifyCommandArgs {
        input: &input,
        output: None,
        fix: true,
        no_backup: false,
        dry_run: false,
        skip_user: false,
    });
    assert!(result.is_ok());
    // Backup should be created
    assert!(input.with_extension("bak").exists());
}

#[test]
fn test_dockerfile_purify_command_fix_no_backup() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo test\n").unwrap();

    let result = dockerfile_purify_command(DockerfilePurifyCommandArgs {
        input: &input,
        output: None,
        fix: true,
        no_backup: true,
        dry_run: false,
        skip_user: false,
    });
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_skip_user() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo test\n").unwrap();

    let result = dockerfile_purify_command(DockerfilePurifyCommandArgs {
        input: &input,
        output: None,
        fix: false,
        no_backup: false,
        dry_run: false,
        skip_user: true,
    });
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

include!("command_tests_dockerfile_incl2_incl2.rs");
