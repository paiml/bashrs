fn default_very_high_risk_target() -> f64 {
    92.5
}
fn default_high_risk_target() -> f64 {
    87.5
}

impl Default for RiskBasedConfig {
    fn default() -> Self {
        Self {
            very_high_risk_mutation_target: 92.5,
            very_high_risk_components: vec![],
            high_risk_mutation_target: 87.5,
            high_risk_components: vec![],
        }
    }
}

impl GateConfig {
    /// Load configuration from `.pmat-gates.toml` file
    pub fn load(path: &Path) -> Result<Self, GateConfigError> {
        let content = fs::read_to_string(path).map_err(|e| GateConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| GateConfigError::Parse {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Load from default locations or return default config
    pub fn load_or_default() -> Self {
        // Try current directory first
        if let Ok(config) = Self::load(Path::new(".pmat-gates.toml")) {
            return config;
        }

        // Try project root (look for Cargo.toml)
        let mut current = std::env::current_dir().unwrap_or_default();
        loop {
            let candidate = current.join(".pmat-gates.toml");
            if candidate.exists() {
                if let Ok(config) = Self::load(&candidate) {
                    return config;
                }
            }
            if !current.pop() {
                break;
            }
        }

        // Return default configuration
        Self::default()
    }

    /// Get gates for a specific tier
    pub fn gates_for_tier(&self, tier: Tier) -> &[String] {
        match tier {
            Tier::Tier1 => &self.tiers.tier1_gates,
            Tier::Tier2 => &self.tiers.tier2_gates,
            Tier::Tier3 => &self.tiers.tier3_gates,
        }
    }
}

/// Errors that can occur during gate configuration
#[derive(Debug)]
pub enum GateConfigError {
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },
}

impl std::fmt::Display for GateConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateConfigError::Io { path, source } => {
                write!(f, "Failed to read {}: {}", path.display(), source)
            }
            GateConfigError::Parse { path, source } => {
                write!(f, "Failed to parse {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for GateConfigError {}

/// Result of running a quality gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Name of the gate
    pub gate_name: String,

    /// Whether the gate passed
    pub passed: bool,

    /// Duration of the check
    pub duration: Duration,

    /// Detailed message
    pub message: String,

    /// Metrics collected (e.g., coverage percentage)
    pub metrics: HashMap<String, f64>,

    /// Violations found
    pub violations: Vec<GateViolation>,
}

/// A specific violation found by a gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateViolation {
    /// File where violation occurred
    pub file: Option<String>,

    /// Line number
    pub line: Option<usize>,

    /// Violation description
    pub description: String,

    /// Severity level
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

/// Quality gate executor
pub struct QualityGate {
    config: GateConfig,
}


include!("gates_default_qualitygate.rs");
