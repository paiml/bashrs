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


include!("container_incl2_incl2.rs");
