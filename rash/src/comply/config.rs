//! Compliance configuration (.bashrs/comply.toml)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Artifact scope for compliance tracking
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Project,
    User,
    System,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Project => write!(f, "project"),
            Scope::User => write!(f, "user"),
            Scope::System => write!(f, "system"),
        }
    }
}

/// Top-level comply configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplyConfig {
    pub comply: ComplyMeta,
    pub scopes: ScopeConfig,
    pub project: ArtifactList,
    pub user: ArtifactList,
    pub rules: RuleConfig,
    pub thresholds: ThresholdConfig,
    pub integration: IntegrationConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplyMeta {
    pub version: String,
    pub bashrs_version: String,
    pub created: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScopeConfig {
    pub project: bool,
    pub user: bool,
    pub system: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactList {
    pub artifacts: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleConfig {
    pub posix: bool,
    pub determinism: bool,
    pub idempotency: bool,
    pub security: bool,
    pub quoting: bool,
    pub shellcheck: bool,
    pub makefile_safety: bool,
    pub dockerfile_best: bool,
    pub config_hygiene: bool,
    pub pzsh_budget: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub min_score: u32,
    pub max_violations: u32,
    pub shellcheck_severity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub pzsh: String,
    pub pmat: String,
}

impl ComplyConfig {
    /// Create default configuration for a new project
    pub fn new_default(bashrs_version: &str) -> Self {
        Self {
            comply: ComplyMeta {
                version: "1.0.0".to_string(),
                bashrs_version: bashrs_version.to_string(),
                created: chrono_now(),
            },
            scopes: ScopeConfig {
                project: true,
                user: false,
                system: false,
            },
            project: ArtifactList {
                artifacts: Vec::new(),
            },
            user: ArtifactList {
                artifacts: Vec::new(),
            },
            rules: RuleConfig {
                posix: true,
                determinism: true,
                idempotency: true,
                security: true,
                quoting: true,
                shellcheck: true,
                makefile_safety: true,
                dockerfile_best: true,
                config_hygiene: true,
                pzsh_budget: "auto".to_string(),
            },
            thresholds: ThresholdConfig {
                min_score: 80,
                max_violations: 0,
                shellcheck_severity: "warning".to_string(),
            },
            integration: IntegrationConfig {
                pzsh: "auto".to_string(),
                pmat: "auto".to_string(),
            },
        }
    }

    /// Load from .bashrs/comply.toml
    pub fn load(project_path: &Path) -> Option<Self> {
        let config_path = project_path.join(".bashrs").join("comply.toml");
        let content = std::fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    }

    /// Save to .bashrs/comply.toml
    pub fn save(&self, project_path: &Path) -> std::io::Result<()> {
        let dir = project_path.join(".bashrs");
        std::fs::create_dir_all(&dir)?;
        let config_path = dir.join("comply.toml");
        let content =
            toml::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(config_path, content)
    }

    /// Check if a comply.toml already exists
    pub fn exists(project_path: &Path) -> bool {
        project_path.join(".bashrs").join("comply.toml").exists()
    }

    /// Get the config file path
    pub fn config_path(project_path: &Path) -> PathBuf {
        project_path.join(".bashrs").join("comply.toml")
    }
}

fn chrono_now() -> String {
    // ISO 8601 timestamp without external chrono dependency
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple UTC timestamp format
    format!("{secs}")
}
