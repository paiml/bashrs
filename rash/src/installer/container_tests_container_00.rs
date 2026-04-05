#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_CONTAINER_001_runtime_command() {
        assert_eq!(ContainerRuntime::Docker.command(), "docker");
        assert_eq!(ContainerRuntime::Podman.command(), "podman");
    }

    #[test]
    fn test_CONTAINER_002_architecture_parse() {
        assert_eq!(Architecture::parse("amd64"), Some(Architecture::Amd64));
        assert_eq!(Architecture::parse("x86_64"), Some(Architecture::Amd64));
        assert_eq!(Architecture::parse("arm64"), Some(Architecture::Arm64));
        assert_eq!(Architecture::parse("aarch64"), Some(Architecture::Arm64));
        assert_eq!(Architecture::parse("unknown"), None);
    }

    #[test]
    fn test_CONTAINER_003_architecture_platform_string() {
        assert_eq!(Architecture::Amd64.platform_string(), "linux/amd64");
        assert_eq!(Architecture::Arm64.platform_string(), "linux/arm64");
    }

    #[test]
    fn test_CONTAINER_004_platform_parse() {
        let p1 = Platform::parse("ubuntu:22.04");
        assert_eq!(p1.image, "ubuntu:22.04");
        assert_eq!(p1.arch, Architecture::Amd64);

        let p2 = Platform::parse("debian:12@arm64");
        assert_eq!(p2.image, "debian:12");
        assert_eq!(p2.arch, Architecture::Arm64);
    }

    #[test]
    fn test_CONTAINER_005_platform_display() {
        let p = Platform::new("fedora:40", Architecture::Arm64);
        assert_eq!(p.display(), "fedora:40@arm64");
    }

    #[test]
    fn test_CONTAINER_006_test_status() {
        assert!(TestStatus::Passed.is_success());
        assert!(TestStatus::Skipped.is_success());
        assert!(!TestStatus::Failed.is_success());
        assert!(!TestStatus::TimedOut.is_success());

        assert_eq!(TestStatus::Passed.symbol(), "✓");
        assert_eq!(TestStatus::Failed.symbol(), "✗");
    }

    #[test]
    fn test_CONTAINER_007_platform_result_passed() {
        let platform = Platform::new("ubuntu:22.04", Architecture::Amd64);
        let result = PlatformResult::passed(platform.clone(), Duration::from_secs(90), 7);

        assert_eq!(result.status, TestStatus::Passed);
        assert_eq!(result.steps_passed, 7);
        assert_eq!(result.steps_total, 7);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_CONTAINER_008_platform_result_failed() {
        let platform = Platform::new("fedora:40", Architecture::Amd64);
        let result = PlatformResult::failed(platform, Duration::from_secs(45), "Package not found");

        assert_eq!(result.status, TestStatus::Failed);
        assert_eq!(result.error, Some("Package not found".to_string()));
    }

    #[test]
    fn test_CONTAINER_009_matrix_config_default() {
        let config = MatrixConfig::default_platforms();

        assert_eq!(config.platforms.len(), 5);
        assert_eq!(config.parallelism, 4);
        assert!(config.platforms.iter().any(|p| p.image == "ubuntu:22.04"));
    }

    #[test]
    fn test_CONTAINER_010_matrix_config_from_string() {
        let config = MatrixConfig::from_platform_string("ubuntu:22.04, debian:12@arm64");

        assert_eq!(config.platforms.len(), 2);
        assert_eq!(config.platforms[0].image, "ubuntu:22.04");
        assert_eq!(config.platforms[1].image, "debian:12");
        assert_eq!(config.platforms[1].arch, Architecture::Arm64);
    }

    #[test]
    fn test_CONTAINER_011_matrix_simulate() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let installer_path = temp_dir.path().join("installer");
        std::fs::create_dir(&installer_path).unwrap();

        let config = MatrixConfig::from_platform_string("ubuntu:22.04,debian:12");
        let mut matrix = ContainerTestMatrix::new(&installer_path, config);

        let summary = matrix.simulate();

        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_CONTAINER_012_matrix_format_results() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let installer_path = temp_dir.path().join("installer");
        std::fs::create_dir(&installer_path).unwrap();

        let config = MatrixConfig::from_platform_string("ubuntu:22.04");
        let mut matrix = ContainerTestMatrix::new(&installer_path, config);
        matrix.simulate();

        let output = matrix.format_results();

        assert!(output.contains("Container Test Matrix"));
        assert!(output.contains("ubuntu:22.04"));
        assert!(output.contains("PASS"));
    }

    #[test]
    fn test_CONTAINER_013_matrix_to_json() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let installer_path = temp_dir.path().join("installer");
        std::fs::create_dir(&installer_path).unwrap();

        let config = MatrixConfig::from_platform_string("ubuntu:22.04");
        let mut matrix = ContainerTestMatrix::new(&installer_path, config);
        matrix.simulate();

        let json = matrix.to_json();

        assert!(json.contains("\"platforms\""));
        assert!(json.contains("\"image\": \"ubuntu:22.04\""));
        assert!(json.contains("\"status\": \"pass\""));
    }

    #[test]
    fn test_CONTAINER_014_summary_format() {
        let summary = MatrixSummary {
            total: 10,
            passed: 8,
            failed: 1,
            skipped: 1,
            total_duration: Duration::from_secs(245),
            parallelism: 4,
        };

        let output = summary.format();

        assert!(output.contains("8/10 passed"));
        assert!(output.contains("1 failed"));
        assert!(output.contains("1 skipped"));
        assert!(output.contains("4m 05s"));
    }

    #[test]
    fn test_CONTAINER_015_summary_success_rate() {
        let summary = MatrixSummary {
            total: 10,
            passed: 8,
            failed: 1,
            skipped: 1,
            total_duration: Duration::ZERO,
            parallelism: 1,
        };

        assert!((summary.success_rate() - 80.0).abs() < 0.01);
        assert!(!summary.all_passed());
    }

    #[test]
    fn test_CONTAINER_016_resource_limits_default() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.memory, Some("2G".to_string()));
        assert_eq!(limits.cpus, Some(2.0));
        assert_eq!(limits.timeout, Duration::from_secs(30 * 60));
    }

    #[test]
    fn test_CONTAINER_017_container_config_builder() {
        let config = ContainerConfig::for_image("ubuntu:22.04")
            .with_volume("/host/path", "/container/path")
            .with_env("TEST_VAR", "value")
            .with_platform("linux/amd64");

        assert_eq!(config.image, "ubuntu:22.04");
        assert_eq!(config.volumes.len(), 1);
        assert_eq!(config.env.get("TEST_VAR"), Some(&"value".to_string()));
        assert_eq!(config.platform, Some("linux/amd64".to_string()));
    }

    #[test]
    fn test_CONTAINER_018_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
    }
}
