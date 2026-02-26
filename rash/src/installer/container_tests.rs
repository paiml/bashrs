#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for installer/container.rs
//! Targets uncovered branches in: Architecture, Platform, TestStatus,
//! PlatformResult, MatrixConfig, ContainerTestMatrix, MatrixSummary,
//! ContainerConfig, ResourceLimits, truncate, escape_json.

use super::*;
use std::time::Duration;

#[test]
fn test_COV_CONT_001_architecture_parse_all_variants() {
    assert_eq!(Architecture::parse("x64"), Some(Architecture::Amd64));
    assert_eq!(Architecture::parse("aarch64"), Some(Architecture::Arm64));
    assert_eq!(Architecture::parse("armv7"), Some(Architecture::Armv7));
    assert_eq!(Architecture::parse("arm"), Some(Architecture::Armv7));
    assert_eq!(Architecture::parse("armhf"), Some(Architecture::Armv7));
    assert_eq!(Architecture::parse("AMD64"), Some(Architecture::Amd64));
    assert_eq!(Architecture::parse("ARM64"), Some(Architecture::Arm64));
    assert_eq!(Architecture::parse("X86_64"), Some(Architecture::Amd64));
}

#[test]
fn test_COV_CONT_002_architecture_display_and_platform() {
    assert_eq!(Architecture::Armv7.platform_string(), "linux/arm/v7");
    assert_eq!(Architecture::Amd64.display_name(), "amd64");
    assert_eq!(Architecture::Arm64.display_name(), "arm64");
    assert_eq!(Architecture::Armv7.display_name(), "armv7");
    assert_eq!(Architecture::default(), Architecture::Amd64);
}

#[test]
fn test_COV_CONT_003_platform_construction() {
    let p = Platform::new("alpine:3.19", Architecture::Arm64);
    assert_eq!(p.image, "alpine:3.19");
    assert!(p.notes.is_none());

    let p2 = Platform::with_notes("alpine:3.19", Architecture::Amd64, "musl libc");
    assert_eq!(p2.notes, Some("musl libc".to_string()));

    let p3 = Platform::new("alpine:3.19", Architecture::Armv7);
    assert_eq!(p3.display(), "alpine:3.19@armv7");
}

#[test]
fn test_COV_CONT_004_platform_parse_variants() {
    let p1 = Platform::parse("centos:8@unknown");
    assert_eq!(p1.image, "centos:8");
    assert_eq!(p1.arch, Architecture::Amd64);

    let p2 = Platform::parse("debian:11@armv7");
    assert_eq!(p2.arch, Architecture::Armv7);

    assert_eq!(
        Platform::new("ubuntu:22.04", Architecture::Amd64),
        Platform::new("ubuntu:22.04", Architecture::Amd64)
    );
}

#[test]
fn test_COV_CONT_005_test_status_symbols_and_text() {
    assert_eq!(TestStatus::Passed.symbol(), "✓");
    assert_eq!(TestStatus::Failed.symbol(), "✗");
    assert_eq!(TestStatus::Skipped.symbol(), "⊘");
    assert_eq!(TestStatus::Running.symbol(), "▶");
    assert_eq!(TestStatus::Pending.symbol(), "⏳");
    assert_eq!(TestStatus::TimedOut.symbol(), "⏱");
    assert_eq!(TestStatus::Running.text(), "RUN");
    assert_eq!(TestStatus::Pending.text(), "PEND");
    assert_eq!(TestStatus::TimedOut.text(), "TIMEOUT");
    assert!(!TestStatus::Running.is_success());
    assert!(!TestStatus::Pending.is_success());
}

#[test]
fn test_COV_CONT_006_platform_result_skipped() {
    let platform = Platform::new("alpine:3.19", Architecture::Amd64);
    let result = PlatformResult::skipped(platform, "musl libc incompatible");
    assert_eq!(result.status, TestStatus::Skipped);
    assert_eq!(result.duration, Duration::ZERO);
    assert_eq!(result.steps_passed, 0);
    assert_eq!(result.error, Some("musl libc incompatible".to_string()));
}

#[test]
fn test_COV_CONT_007_platform_result_format_row_variants() {
    let passed = PlatformResult::passed(
        Platform::new("ubuntu:22.04", Architecture::Amd64),
        Duration::from_secs(125),
        7,
    );
    let row = passed.format_row();
    assert!(row.contains("ubuntu:22.04") && row.contains("2m 05s"));

    let failed = PlatformResult::failed(
        Platform::new("fedora:40", Architecture::Amd64),
        Duration::from_secs(30),
        "pkg not found",
    );
    assert!(failed.format_row().contains("pkg not found"));

    let skipped =
        PlatformResult::skipped(Platform::new("test:1", Architecture::Amd64), "unsupported");
    assert!(skipped.format_row().contains("-"));

    let no_steps = PlatformResult::failed(
        Platform::new("test:1", Architecture::Arm64),
        Duration::from_secs(10),
        "err",
    );
    assert!(no_steps.format_row().contains("arm64"));
}

#[test]
fn test_COV_CONT_008_container_config() {
    let default = ContainerConfig::default();
    assert!(default.image.is_empty());
    assert!(default.platform.is_none());
    assert!(default.remove_after);

    let config = ContainerConfig::for_image("ubuntu:22.04")
        .with_volume("/src", "/app/src")
        .with_volume("/data", "/app/data")
        .with_env("HOME", "/root")
        .with_env("DEBIAN_FRONTEND", "noninteractive")
        .with_platform("linux/arm64");
    assert_eq!(config.volumes.len(), 2);
    assert_eq!(config.env.len(), 2);
    assert_eq!(config.platform, Some("linux/arm64".to_string()));
}

#[test]
fn test_COV_CONT_009_resource_limits() {
    let custom = ResourceLimits {
        memory: Some("4G".to_string()),
        cpus: Some(4.0),
        timeout: Duration::from_secs(3600),
    };
    assert_eq!(custom.memory, Some("4G".to_string()));

    let none = ResourceLimits {
        memory: None,
        cpus: None,
        timeout: Duration::from_secs(60),
    };
    assert!(none.memory.is_none());
}

#[test]
fn test_COV_CONT_010_matrix_config_variants() {
    let ext = MatrixConfig::extended_platforms();
    assert_eq!(ext.platforms.len(), 9);
    let alpine = ext
        .platforms
        .iter()
        .find(|p| p.image == "alpine:3.19")
        .unwrap();
    assert_eq!(alpine.notes, Some("musl libc".to_string()));

    let mut config = MatrixConfig::default();
    config.add_platform(Platform::new("centos:8", Architecture::Amd64));
    assert_eq!(config.platforms.len(), 1);

    assert_eq!(MatrixConfig::default().with_parallelism(0).parallelism, 1);
    assert_eq!(MatrixConfig::default().with_parallelism(8).parallelism, 8);
    assert_eq!(
        MatrixConfig::default()
            .with_runtime(ContainerRuntime::Podman)
            .runtime,
        ContainerRuntime::Podman
    );

    let three = MatrixConfig::from_platform_string("a:1, b:2@arm64, c:3@armv7");
    assert_eq!(three.platforms.len(), 3);
    assert_eq!(three.platforms[2].arch, Architecture::Armv7);
}

#[test]
fn test_COV_CONT_011_matrix_accessors_and_validate() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let path = temp_dir.path().join("inst");
    std::fs::create_dir(&path).unwrap();

    let matrix = ContainerTestMatrix::new(&path, MatrixConfig::from_platform_string("u:1"));
    assert_eq!(matrix.installer_path(), path.as_path());
    assert_eq!(matrix.config().platforms.len(), 1);
    assert!(matrix.results().is_empty());

    // Validate: no platforms
    let empty = ContainerTestMatrix::new(&path, MatrixConfig::default());
    assert!(empty.validate().is_err());

    // Validate: missing path
    let missing = ContainerTestMatrix::new(
        "/tmp/bashrs_truly_nonexistent_path_xyz_12345",
        MatrixConfig::from_platform_string("u:1"),
    );
    assert!(missing.validate().is_err());
}

#[test]
fn test_COV_CONT_012_matrix_simulate_with_alpine() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let path = temp_dir.path().join("inst");
    std::fs::create_dir(&path).unwrap();

    let config = MatrixConfig::from_platform_string("ubuntu:22.04,alpine:3.19,debian:12");
    let mut matrix = ContainerTestMatrix::new(&path, config);
    let summary = matrix.simulate();
    assert_eq!(summary.total, 3);
    assert_eq!(summary.passed, 2);
    assert_eq!(summary.skipped, 1);

    let custom_summary = matrix.generate_summary(Duration::from_secs(30));
    assert_eq!(custom_summary.total_duration, Duration::from_secs(30));
}

#[test]
fn test_COV_CONT_013_matrix_format_results_branches() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let path = temp_dir.path().join("inst");
    std::fs::create_dir(&path).unwrap();

    // Skipped result
    let mut m1 = ContainerTestMatrix::new(&path, MatrixConfig::from_platform_string("alpine:3.19"));
    m1.simulate();
    let out1 = m1.format_results();
    assert!(out1.contains("SKIP") && out1.contains("musl"));

    // Failed result
    let mut m2 = ContainerTestMatrix::new(&path, MatrixConfig::from_platform_string("u:1"));
    m2.results.push(PlatformResult::failed(
        Platform::new("centos:7", Architecture::Amd64),
        Duration::from_secs(20),
        "EOL distro",
    ));
    assert!(m2.format_results().contains("EOL distro"));
}

#[test]
fn test_COV_CONT_014_matrix_to_json() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let path = temp_dir.path().join("inst");
    std::fs::create_dir(&path).unwrap();

    // JSON with error
    let mut m1 = ContainerTestMatrix::new(&path, MatrixConfig::from_platform_string("alpine:3.19"));
    m1.simulate();
    let json1 = m1.to_json();
    assert!(json1.contains("\"error\":") && json1.contains("musl"));

    // JSON with multiple platforms
    let mut m2 = ContainerTestMatrix::new(&path, MatrixConfig::from_platform_string("u:1,u:2,u:3"));
    m2.simulate();
    assert_eq!(m2.to_json().matches("\"image\"").count(), 3);
}

#[test]
fn test_COV_CONT_015_summary_format_and_rates() {
    let short = MatrixSummary {
        total: 2,
        passed: 2,
        failed: 0,
        skipped: 0,
        total_duration: Duration::from_secs(45),
        parallelism: 2,
    };
    let out = short.format();
    assert!(out.contains("45s") && !out.contains("failed"));

    assert!(MatrixSummary {
        total: 5,
        passed: 5,
        failed: 0,
        skipped: 0,
        total_duration: Duration::ZERO,
        parallelism: 1,
    }
    .all_passed());

    let empty = MatrixSummary {
        total: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        total_duration: Duration::ZERO,
        parallelism: 1,
    };
    assert!((empty.success_rate()).abs() < f64::EPSILON);
    assert!(
        (MatrixSummary {
            total: 3,
            passed: 3,
            failed: 0,
            skipped: 0,
            total_duration: Duration::ZERO,
            parallelism: 1,
        }
        .success_rate()
            - 100.0)
            .abs()
            < f64::EPSILON
    );

    let with_failed = MatrixSummary {
        total: 3,
        passed: 1,
        failed: 2,
        skipped: 0,
        total_duration: Duration::from_secs(120),
        parallelism: 4,
    };
    let fo = with_failed.format();
    assert!(fo.contains("2 failed") && fo.contains("2m 00s") && fo.contains("4 workers"));
}

#[test]
fn test_COV_CONT_016_runtime_default_and_command() {
    assert_eq!(ContainerRuntime::default(), ContainerRuntime::Docker);
    assert_eq!(ContainerRuntime::Docker.command(), "docker");
    assert_eq!(ContainerRuntime::Podman.command(), "podman");
}

#[test]
fn test_COV_CONT_017_truncate_and_escape_json() {
    assert_eq!(truncate("hello", 10), "hello");
    assert_eq!(truncate("hello", 5), "hello");
    let long = truncate("hello world this is long", 10);
    assert!(long.ends_with("...") && long.len() <= 10);

    assert_eq!(escape_json("path\\to\\file"), "path\\\\to\\\\file");
    assert_eq!(escape_json("col1\tcol2"), "col1\\tcol2");
    assert_eq!(escape_json("line\r\n"), "line\\r\\n");
    let combined = escape_json("say \"hi\"\ttab\nnewline\\bs");
    assert!(combined.contains("\\\"") && combined.contains("\\t"));
}

#[test]
fn test_COV_CONT_018_step_test_result() {
    let passed = StepTestResult {
        step_id: "s1".to_string(),
        step_name: "Install".to_string(),
        passed: true,
        duration: Duration::from_millis(500),
        error: None,
    };
    assert!(passed.passed && passed.error.is_none());

    let failed = StepTestResult {
        step_id: "s2".to_string(),
        step_name: "Configure".to_string(),
        passed: false,
        duration: Duration::from_millis(200),
        error: Some("not found".to_string()),
    };
    assert!(!failed.passed);
}
