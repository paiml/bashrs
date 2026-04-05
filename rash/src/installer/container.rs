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

include!("container_incl2.rs");
