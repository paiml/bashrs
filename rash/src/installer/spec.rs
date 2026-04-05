//! Installer specification parsing
//!
//! This module handles parsing of installer.toml files into strongly-typed structures.

use crate::models::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete installer specification parsed from installer.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerSpec {
    /// Core installer metadata
    pub installer: InstallerMetadata,

    /// Artifacts to download and verify
    #[serde(default)]
    pub artifact: Vec<Artifact>,

    /// Installation steps
    #[serde(default)]
    pub step: Vec<Step>,
}

impl InstallerSpec {
    /// Parse an installer specification from TOML content
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| Error::Validation(format!("Invalid TOML: {e}")))
    }

    /// Serialize the specification back to TOML
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| Error::Internal(format!("Failed to serialize to TOML: {e}")))
    }
}

/// Installer metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerMetadata {
    /// Installer name
    pub name: String,

    /// Installer version (semver)
    pub version: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Author name or email
    #[serde(default)]
    pub author: String,

    /// System requirements
    #[serde(default)]
    pub requirements: Requirements,

    /// Environment variable definitions
    #[serde(default)]
    pub environment: HashMap<String, EnvVarSpec>,

    /// Security configuration
    #[serde(default)]
    pub security: InstallerSecurity,

    /// Hermetic build configuration
    #[serde(default)]
    pub hermetic: HermeticConfig,

    /// Distributed execution configuration
    #[serde(default)]
    pub distributed: DistributedConfig,

    /// Test matrix configuration
    #[serde(default)]
    pub test_matrix: TestMatrixConfig,

    /// Golden trace configuration
    #[serde(default)]
    pub golden_traces: GoldenTraceConfig,
}

/// System requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Requirements {
    /// Supported operating systems (e.g., "ubuntu >= 20.04")
    #[serde(default)]
    pub os: Vec<String>,

    /// Supported architectures (e.g., "x86_64", "aarch64")
    #[serde(default)]
    pub arch: Vec<String>,

    /// Required privileges: "root" or "user"
    #[serde(default = "default_privileges")]
    pub privileges: String,

    /// Whether network access is required
    #[serde(default)]
    pub network: bool,
}

fn default_privileges() -> String {
    "user".to_string()
}

/// Environment variable specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvVarSpec {
    /// Simple default value
    Simple(String),
    /// Complex specification with validation
    Complex {
        /// Default value
        #[serde(default)]
        default: Option<String>,
        /// Value from another environment variable
        #[serde(default)]
        from_env: Option<String>,
        /// Whether this variable is required
        #[serde(default)]
        required: bool,
        /// Validation pattern
        #[serde(default)]
        validate: Option<String>,
    },
}

/// Security configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallerSecurity {
    /// Trust model: "keyring" or "tofu"
    #[serde(default = "default_trust_model")]
    pub trust_model: String,

    /// Path to keyring file
    #[serde(default)]
    pub keyring: Option<String>,

    /// Whether signatures are required for all artifacts
    #[serde(default)]
    pub require_signatures: bool,

    /// Transparency log URL (Sigstore-compatible)
    #[serde(default)]
    pub transparency_log: Option<String>,
}

fn default_trust_model() -> String {
    "tofu".to_string()
}

/// Hermetic build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HermeticConfig {
    /// Path to lockfile
    #[serde(default)]
    pub lockfile: Option<String>,

    /// SOURCE_DATE_EPOCH setting: "auto" or a Unix timestamp
    #[serde(default)]
    pub source_date_epoch: Option<String>,
}

/// Distributed execution configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Maximum parallel steps
    #[serde(default)]
    pub max_parallel_steps: Option<u32>,

    /// sccache server address
    #[serde(default)]
    pub sccache_server: Option<String>,
}

/// Test matrix configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestMatrixConfig {
    /// Platforms to test (e.g., "ubuntu:22.04")
    #[serde(default)]
    pub platforms: Vec<String>,

    /// Architectures to test
    #[serde(default)]
    pub architectures: Vec<String>,

    /// Maximum parallel containers
    #[serde(default)]
    pub parallelism: Option<u32>,

    /// Container runtime preference
    #[serde(default)]
    pub runtime: Option<String>,
}

/// Golden trace configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoldenTraceConfig {
    /// Whether golden traces are enabled
    #[serde(default)]
    pub enabled: bool,

    /// Directory for golden trace files
    #[serde(default)]
    pub trace_dir: Option<String>,

    /// Syscall categories to capture
    #[serde(default)]
    pub capture: Vec<String>,

    /// Paths to ignore
    #[serde(default)]
    pub ignore_paths: Vec<String>,
}

/// Artifact definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier for this artifact
    pub id: String,

    /// Download URL
    pub url: String,

    /// Expected SHA-256 hash
    #[serde(default)]
    pub sha256: Option<String>,

    /// URL to SHA-256 sums file
    #[serde(default)]
    pub sha256_url: Option<String>,

    /// Signature URL or path
    #[serde(default)]
    pub signature: Option<String>,

    /// Key ID that signed this artifact
    #[serde(default)]
    pub signed_by: Option<String>,
}

/// Installation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique step identifier
    pub id: String,

    /// Human-readable step name
    pub name: String,

    /// Action type
    pub action: String,

    /// Dependencies (step IDs that must complete first)
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Script content (for action = "script")
    #[serde(default)]
    pub script: Option<StepScript>,

    /// Packages to install (for action = "apt-install")
    #[serde(default)]
    pub packages: Vec<String>,

    /// Target path (for file operations)
    #[serde(default)]
    pub path: Option<String>,

    /// Content (for file-write action)
    #[serde(default)]
    pub content: Option<String>,

    /// User (for user-group action)
    #[serde(default)]
    pub user: Option<String>,

    /// Group (for user-group action)
    #[serde(default)]
    pub group: Option<String>,

    /// Privilege level for this step
    #[serde(default)]
    pub privileges: Option<String>,

    /// Preconditions that must be met
    #[serde(default)]
    pub preconditions: Precondition,

    /// Postconditions that must be verified
    #[serde(default)]
    pub postconditions: Postcondition,

    /// Checkpoint configuration
    #[serde(default)]
    pub checkpoint: StepCheckpoint,

    /// Timing configuration
    #[serde(default)]
    pub timing: StepTiming,

    /// Artifacts used by this step
    #[serde(default)]
    pub uses_artifacts: ArtifactRefs,

    /// Verification commands
    #[serde(default)]
    pub verification: Option<StepVerification>,

    /// Failure handling
    #[serde(default)]
    pub on_failure: Option<FailureAction>,

    /// Resource constraints
    #[serde(default)]
    pub constraints: StepConstraints,

    /// Environment variables for this step
    #[serde(default)]
    pub environment: HashMap<String, EnvVarSpec>,
}

/// Script definition for a step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepScript {
    /// Interpreter to use (e.g., "sh", "bash")
    #[serde(default = "default_interpreter")]
    pub interpreter: String,

    /// Script content
    #[serde(default)]
    pub content: String,
}

fn default_interpreter() -> String {
    "sh".to_string()
}

/// Artifact references for a step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactRefs {
    /// List of artifact IDs
    #[serde(default)]
    pub artifacts: Vec<String>,
}

/// Preconditions for a step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Precondition {
    /// File must exist
    #[serde(default)]
    pub file_exists: Option<String>,

    /// Command must succeed
    #[serde(default)]
    pub command_succeeds: Option<String>,

    /// Environment variable must match pattern
    #[serde(default)]
    pub env_matches: HashMap<String, String>,
}

include!("spec_tests_cont.rs");
