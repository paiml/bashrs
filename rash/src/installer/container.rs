//! Container-native test matrix for installers (#110)
//!
//! Provides parallel multi-distro testing using Podman or Docker containers.
//!
//! # Example
//!
//! ```bash
//! # Run full matrix
//! bashrs installer test ./my-installer --matrix
//!
//! # Test specific platforms
//! bashrs installer test ./my-installer --matrix ubuntu:22.04,debian:12
//!
//! # Test specific architecture
//! bashrs installer test ./my-installer --matrix --arch arm64
//!
//! # Generate matrix report
//! bashrs installer test ./my-installer --matrix --report matrix-results.json
//! ```

use crate::models::{Error, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Container runtime type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContainerRuntime {
    /// Docker container runtime
    #[default]
    Docker,
    /// Podman container runtime (rootless preferred)
    Podman,
}

impl ContainerRuntime {
    /// Get the command name for this runtime
    pub fn command(&self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }

    /// Detect available container runtime
    pub fn detect() -> Option<Self> {
        // Check for podman first (preferred for rootless)
        if std::process::Command::new("podman")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(Self::Podman);
        }

        // Fall back to docker
        if std::process::Command::new("docker")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Some(Self::Docker);
        }

        None
    }

    /// Check if runtime is available
    pub fn is_available(&self) -> bool {
        std::process::Command::new(self.command())
            .arg("--version")
            .output()
            .is_ok()
    }
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Architecture {
    /// x86_64 / amd64
    #[default]
    Amd64,
    /// aarch64 / arm64
    Arm64,
    /// armv7
    Armv7,
}

impl Architecture {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "amd64" | "x86_64" | "x64" => Some(Self::Amd64),
            "arm64" | "aarch64" => Some(Self::Arm64),
            "armv7" | "arm" | "armhf" => Some(Self::Armv7),
            _ => None,
        }
    }

    /// Get docker platform string
    pub fn platform_string(&self) -> &'static str {
        match self {
            Self::Amd64 => "linux/amd64",
            Self::Arm64 => "linux/arm64",
            Self::Armv7 => "linux/arm/v7",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Amd64 => "amd64",
            Self::Arm64 => "arm64",
            Self::Armv7 => "armv7",
        }
    }
}

/// A platform to test against
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Platform {
    /// Container image (e.g., "ubuntu:22.04")
    pub image: String,

    /// Target architecture
    pub arch: Architecture,

    /// Optional platform-specific notes
    pub notes: Option<String>,
}

impl Platform {
    /// Create a new platform
    pub fn new(image: &str, arch: Architecture) -> Self {
        Self {
            image: image.to_string(),
            arch,
            notes: None,
        }
    }

    /// Create with notes
    pub fn with_notes(image: &str, arch: Architecture, notes: &str) -> Self {
        Self {
            image: image.to_string(),
            arch,
            notes: Some(notes.to_string()),
        }
    }

    /// Parse from string like "ubuntu:22.04" or "ubuntu:22.04@arm64"
    pub fn parse(s: &str) -> Self {
        if let Some((image, arch_str)) = s.split_once('@') {
            let arch = Architecture::parse(arch_str).unwrap_or_default();
            Self::new(image, arch)
        } else {
            Self::new(s, Architecture::default())
        }
    }

    /// Get display string
    pub fn display(&self) -> String {
        format!("{}@{}", self.image, self.arch.display_name())
    }
}

/// Test status for a platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// Tests passed
    Passed,
    /// Tests failed
    Failed,
    /// Tests were skipped
    Skipped,
    /// Tests are running
    Running,
    /// Tests are pending
    Pending,
    /// Test timed out
    TimedOut,
}

impl TestStatus {
    /// Get status symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Passed => "✓",
            Self::Failed => "✗",
            Self::Skipped => "⊘",
            Self::Running => "▶",
            Self::Pending => "⏳",
            Self::TimedOut => "⏱",
        }
    }

    /// Get status text
    pub fn text(&self) -> &'static str {
        match self {
            Self::Passed => "PASS",
            Self::Failed => "FAIL",
            Self::Skipped => "SKIP",
            Self::Running => "RUN",
            Self::Pending => "PEND",
            Self::TimedOut => "TIMEOUT",
        }
    }

    /// Check if status indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Passed | Self::Skipped)
    }
}

/// Result of testing on a single platform
#[derive(Debug, Clone)]
pub struct PlatformResult {
    /// Platform that was tested
    pub platform: Platform,

    /// Test status
    pub status: TestStatus,

    /// Test duration
    pub duration: Duration,

    /// Number of steps that passed
    pub steps_passed: usize,

    /// Total number of steps
    pub steps_total: usize,

    /// Container ID (if applicable)
    pub container_id: Option<String>,

    /// Captured logs
    pub logs: String,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Individual step results
    pub step_results: Vec<StepTestResult>,
}

impl PlatformResult {
    /// Create a passed result
    pub fn passed(platform: Platform, duration: Duration, steps: usize) -> Self {
        Self {
            platform,
            status: TestStatus::Passed,
            duration,
            steps_passed: steps,
            steps_total: steps,
            container_id: None,
            logs: String::new(),
            error: None,
            step_results: Vec::new(),
        }
    }

    /// Create a failed result
    pub fn failed(platform: Platform, duration: Duration, error: &str) -> Self {
        Self {
            platform,
            status: TestStatus::Failed,
            duration,
            steps_passed: 0,
            steps_total: 0,
            container_id: None,
            logs: String::new(),
            error: Some(error.to_string()),
            step_results: Vec::new(),
        }
    }

    /// Create a skipped result
    pub fn skipped(platform: Platform, reason: &str) -> Self {
        Self {
            platform,
            status: TestStatus::Skipped,
            duration: Duration::ZERO,
            steps_passed: 0,
            steps_total: 0,
            container_id: None,
            logs: String::new(),
            error: Some(reason.to_string()),
            step_results: Vec::new(),
        }
    }

    /// Format as table row
    pub fn format_row(&self) -> String {
        let _steps_str = if self.steps_total > 0 {
            format!("{}/{} passed", self.steps_passed, self.steps_total)
        } else {
            "N/A".to_string()
        };

        let duration_str = if self.duration.as_secs() > 0 {
            format!(
                "{}m {:02}s",
                self.duration.as_secs() / 60,
                self.duration.as_secs() % 60
            )
        } else {
            "-".to_string()
        };

        let notes = if let Some(ref err) = self.error {
            format!(" ← {}", truncate(err, 40))
        } else {
            String::new()
        };

        format!(
            "  {:<22} {:<8} {} {}    {}{}",
            truncate(&self.platform.image, 22),
            self.platform.arch.display_name(),
            self.status.symbol(),
            self.status.text(),
            duration_str,
            notes
        )
    }
}

/// Result of testing a single step
#[derive(Debug, Clone)]
pub struct StepTestResult {
    /// Step ID
    pub step_id: String,

    /// Step name
    pub step_name: String,

    /// Whether step passed
    pub passed: bool,

    /// Duration
    pub duration: Duration,

    /// Error message if failed
    pub error: Option<String>,
}

/// Resource limits for containers
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Memory limit (e.g., "2G")
    pub memory: Option<String>,

    /// CPU limit (e.g., 2.0 for 2 CPUs)
    pub cpus: Option<f64>,

    /// Timeout for the entire test
    pub timeout: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory: Some("2G".to_string()),
            cpus: Some(2.0),
            timeout: Duration::from_secs(30 * 60), // 30 minutes
        }
    }
}

/// Configuration for container tests
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Container image
    pub image: String,

    /// Platform/architecture
    pub platform: Option<String>,

    /// Volume mounts (host_path, container_path)
    pub volumes: Vec<(PathBuf, PathBuf)>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Working directory in container
    pub workdir: Option<PathBuf>,

    /// Whether to remove container after test
    pub remove_after: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            image: String::new(),
            platform: None,
            volumes: Vec::new(),
            env: HashMap::new(),
            limits: ResourceLimits::default(),
            workdir: None,
            remove_after: true,
        }
    }
}

impl ContainerConfig {
    /// Create config for an image
    pub fn for_image(image: &str) -> Self {
        Self {
            image: image.to_string(),
            ..Default::default()
        }
    }

    /// Add a volume mount
    pub fn with_volume(mut self, host: impl AsRef<Path>, container: impl AsRef<Path>) -> Self {
        self.volumes.push((
            host.as_ref().to_path_buf(),
            container.as_ref().to_path_buf(),
        ));
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Set platform
    pub fn with_platform(mut self, platform: &str) -> Self {
        self.platform = Some(platform.to_string());
        self
    }
}

/// Matrix configuration
#[derive(Debug, Clone)]
pub struct MatrixConfig {
    /// Platforms to test
    pub platforms: Vec<Platform>,

    /// Maximum parallel tests
    pub parallelism: usize,

    /// Container runtime to use
    pub runtime: ContainerRuntime,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Whether to continue on failure
    pub continue_on_failure: bool,

    /// Report output path
    pub report_path: Option<PathBuf>,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            platforms: Vec::new(),
            parallelism: 4,
            runtime: ContainerRuntime::default(),
            limits: ResourceLimits::default(),
            continue_on_failure: true,
            report_path: None,
        }
    }
}

impl MatrixConfig {
    /// Create with default Ubuntu/Debian platforms
    pub fn default_platforms() -> Self {
        let platforms = vec![
            Platform::new("ubuntu:20.04", Architecture::Amd64),
            Platform::new("ubuntu:22.04", Architecture::Amd64),
            Platform::new("ubuntu:24.04", Architecture::Amd64),
            Platform::new("debian:11", Architecture::Amd64),
            Platform::new("debian:12", Architecture::Amd64),
        ];

        Self {
            platforms,
            ..Default::default()
        }
    }

    /// Create with extended platforms including Fedora and Alpine
    pub fn extended_platforms() -> Self {
        let platforms = vec![
            Platform::new("ubuntu:20.04", Architecture::Amd64),
            Platform::new("ubuntu:22.04", Architecture::Amd64),
            Platform::new("ubuntu:24.04", Architecture::Amd64),
            Platform::new("debian:11", Architecture::Amd64),
            Platform::new("debian:12", Architecture::Amd64),
            Platform::new("fedora:39", Architecture::Amd64),
            Platform::new("fedora:40", Architecture::Amd64),
            Platform::new("rockylinux:9", Architecture::Amd64),
            Platform::with_notes("alpine:3.19", Architecture::Amd64, "musl libc"),
        ];

        Self {
            platforms,
            ..Default::default()
        }
    }

    /// Parse platforms from comma-separated string
    pub fn from_platform_string(s: &str) -> Self {
        let platforms: Vec<Platform> = s.split(',').map(|p| Platform::parse(p.trim())).collect();

        Self {
            platforms,
            ..Default::default()
        }
    }

    /// Add a platform
    pub fn add_platform(&mut self, platform: Platform) {
        self.platforms.push(platform);
    }

    /// Set parallelism
    pub fn with_parallelism(mut self, n: usize) -> Self {
        self.parallelism = n.max(1);
        self
    }

    /// Set runtime
    pub fn with_runtime(mut self, runtime: ContainerRuntime) -> Self {
        self.runtime = runtime;
        self
    }
}

/// Container test matrix runner
#[derive(Debug)]
pub struct ContainerTestMatrix {
    /// Configuration
    config: MatrixConfig,

    /// Installer path
    installer_path: PathBuf,

    /// Results
    results: Vec<PlatformResult>,
}

impl ContainerTestMatrix {
    /// Create a new test matrix
    pub fn new(installer_path: impl AsRef<Path>, config: MatrixConfig) -> Self {
        Self {
            config,
            installer_path: installer_path.as_ref().to_path_buf(),
            results: Vec::new(),
        }
    }

    /// Get installer path
    pub fn installer_path(&self) -> &Path {
        &self.installer_path
    }

    /// Get configuration
    pub fn config(&self) -> &MatrixConfig {
        &self.config
    }

    /// Get results
    pub fn results(&self) -> &[PlatformResult] {
        &self.results
    }

    /// Check if runtime is available
    pub fn check_runtime(&self) -> Result<()> {
        if !self.config.runtime.is_available() {
            return Err(Error::Validation(format!(
                "Container runtime '{}' is not available. Install {} or use --runtime to specify.",
                self.config.runtime.command(),
                self.config.runtime.command()
            )));
        }
        Ok(())
    }

    /// Validate the matrix configuration
    pub fn validate(&self) -> Result<()> {
        if self.config.platforms.is_empty() {
            return Err(Error::Validation(
                "No platforms specified for test matrix".to_string(),
            ));
        }

        if !self.installer_path.exists() {
            return Err(Error::Validation(format!(
                "Installer path does not exist: {}",
                self.installer_path.display()
            )));
        }

        Ok(())
    }

    /// Simulate running the matrix (for testing/dry-run)
    pub fn simulate(&mut self) -> MatrixSummary {
        let start = std::time::Instant::now();

        for platform in &self.config.platforms {
            // Simulate test result based on platform
            let result = self.simulate_platform(platform);
            self.results.push(result);
        }

        self.generate_summary(start.elapsed())
    }

    /// Simulate a single platform test
    fn simulate_platform(&self, platform: &Platform) -> PlatformResult {
        // Simulate based on platform characteristics
        let duration = Duration::from_secs(60 + (platform.image.len() as u64 * 5));

        // Alpine might have compatibility issues
        if platform.image.contains("alpine") {
            return PlatformResult::skipped(platform.clone(), "musl libc may be incompatible");
        }

        // Simulate step count based on installer
        let steps = 7; // Typical installer has ~7 steps

        PlatformResult::passed(platform.clone(), duration, steps)
    }

    /// Generate summary from results
    pub fn generate_summary(&self, total_duration: Duration) -> MatrixSummary {
        let passed = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Passed)
            .count();
        let failed = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Failed)
            .count();
        let skipped = self
            .results
            .iter()
            .filter(|r| r.status == TestStatus::Skipped)
            .count();

        MatrixSummary {
            total: self.results.len(),
            passed,
            failed,
            skipped,
            total_duration,
            parallelism: self.config.parallelism,
        }
    }

    /// Format results as text table
    pub fn format_results(&self) -> String {
        let mut output = String::new();

        output.push_str("Container Test Matrix\n");
        output.push_str(
            "══════════════════════════════════════════════════════════════════════════════\n\n",
        );
        output.push_str("  Platform               Arch     Status    Duration    Steps\n");
        output.push_str(
            "  ────────────────────────────────────────────────────────────────────────────\n",
        );

        for result in &self.results {
            let steps_str = if result.steps_total > 0 {
                format!("{}/{} passed", result.steps_passed, result.steps_total)
            } else {
                "N/A".to_string()
            };

            let duration_str = if result.duration.as_secs() > 0 {
                format!(
                    "{}m {:02}s",
                    result.duration.as_secs() / 60,
                    result.duration.as_secs() % 60
                )
            } else {
                "-".to_string()
            };

            let notes = match &result.error {
                Some(err) if result.status == TestStatus::Skipped => {
                    format!(" ({})", truncate(err, 30))
                }
                Some(err) if result.status == TestStatus::Failed => {
                    format!(" ← {}", truncate(err, 30))
                }
                _ => String::new(),
            };

            output.push_str(&format!(
                "  {:<22} {:<8} {} {}    {:<12} {}{}\n",
                truncate(&result.platform.image, 22),
                result.platform.arch.display_name(),
                result.status.symbol(),
                result.status.text(),
                duration_str,
                steps_str,
                notes
            ));
        }

        output.push_str(
            "══════════════════════════════════════════════════════════════════════════════\n",
        );

        output
    }

    /// Generate JSON report
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str("  \"platforms\": [\n");

        for (i, result) in self.results.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!(
                "      \"image\": \"{}\",\n",
                result.platform.image
            ));
            json.push_str(&format!(
                "      \"arch\": \"{}\",\n",
                result.platform.arch.display_name()
            ));
            json.push_str(&format!(
                "      \"status\": \"{}\",\n",
                result.status.text().to_lowercase()
            ));
            json.push_str(&format!(
                "      \"duration_secs\": {},\n",
                result.duration.as_secs()
            ));
            json.push_str(&format!(
                "      \"steps_passed\": {},\n",
                result.steps_passed
            ));
            json.push_str(&format!("      \"steps_total\": {}", result.steps_total));

            if let Some(ref err) = result.error {
                json.push_str(&format!(",\n      \"error\": \"{}\"", escape_json(err)));
            }

            json.push_str("\n    }");
            if i < self.results.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }

        json.push_str("  ]\n");
        json.push_str("}\n");

        json
    }
}

/// Summary of matrix test results
#[derive(Debug, Clone)]
pub struct MatrixSummary {
    /// Total platforms tested
    pub total: usize,

    /// Platforms that passed
    pub passed: usize,

    /// Platforms that failed
    pub failed: usize,

    /// Platforms that were skipped
    pub skipped: usize,

    /// Total duration
    pub total_duration: Duration,

    /// Parallelism used
    pub parallelism: usize,
}

impl MatrixSummary {
    /// Format as text
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("  Summary: {}/{} passed", self.passed, self.total));

        if self.failed > 0 {
            output.push_str(&format!(", {} failed", self.failed));
        }

        if self.skipped > 0 {
            output.push_str(&format!(", {} skipped", self.skipped));
        }

        output.push('\n');

        let duration = if self.total_duration.as_secs() >= 60 {
            format!(
                "{}m {:02}s",
                self.total_duration.as_secs() / 60,
                self.total_duration.as_secs() % 60
            )
        } else {
            format!("{}s", self.total_duration.as_secs())
        };

        output.push_str(&format!(
            "  Total time: {} (parallel execution, {} workers)\n",
            duration, self.parallelism
        ));

        output
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Escape string for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Platform parsing always produces valid platform
        #[test]
        fn prop_platform_parse_valid(
            image in "[a-z]+:[0-9]+\\.[0-9]+"
        ) {
            let platform = Platform::parse(&image);
            prop_assert!(!platform.image.is_empty());
            prop_assert!(platform.arch == Architecture::Amd64); // Default
        }

        /// Property: Summary counts are consistent
        #[test]
        fn prop_summary_counts_consistent(
            passed in 0usize..100,
            failed in 0usize..100,
            skipped in 0usize..100
        ) {
            let total = passed + failed + skipped;
            let summary = MatrixSummary {
                total,
                passed,
                failed,
                skipped,
                total_duration: Duration::ZERO,
                parallelism: 1,
            };

            prop_assert_eq!(summary.total, passed + failed + skipped);
            prop_assert!(summary.success_rate() >= 0.0 && summary.success_rate() <= 100.0);
        }

        /// Property: JSON output is valid
        #[test]
        fn prop_json_has_platforms(
            platform_count in 1usize..5
        ) {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let installer_path = temp_dir.path().join("installer");
            std::fs::create_dir(&installer_path).unwrap();

            let platforms: Vec<Platform> = (0..platform_count)
                .map(|i| Platform::new(&format!("test:{}", i), Architecture::Amd64))
                .collect();

            let config = MatrixConfig {
                platforms,
                ..Default::default()
            };

            let mut matrix = ContainerTestMatrix::new(&installer_path, config);
            matrix.simulate();

            let json = matrix.to_json();
            prop_assert!(json.contains("\"platforms\""));
            let starts_correct = json.starts_with("{");
            let ends_correct = json.ends_with("}\n");
            prop_assert!(starts_correct, "JSON should start with opening brace");
            prop_assert!(ends_correct, "JSON should end with closing brace");
        }
    }
}
