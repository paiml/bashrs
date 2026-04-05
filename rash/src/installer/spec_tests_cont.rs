/// Postconditions for a step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Postcondition {
    /// File must exist
    #[serde(default)]
    pub file_exists: Option<String>,

    /// File must have specific mode
    #[serde(default)]
    pub file_mode: Option<String>,

    /// Command must succeed
    #[serde(default)]
    pub command_succeeds: Option<String>,

    /// Packages must be absent
    #[serde(default)]
    pub packages_absent: Vec<String>,

    /// Service must be active
    #[serde(default)]
    pub service_active: Option<String>,

    /// User must be in group
    #[serde(default)]
    pub user_in_group: Option<UserInGroupCheck>,

    /// Environment variable must match pattern
    #[serde(default)]
    pub env_matches: HashMap<String, String>,
}

/// User in group check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInGroupCheck {
    pub user: String,
    pub group: String,
}

/// Checkpoint configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepCheckpoint {
    /// Whether checkpointing is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Rollback command
    #[serde(default)]
    pub rollback: Option<String>,

    /// State files to backup
    #[serde(default)]
    pub state_files: Vec<String>,
}

/// Timing configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepTiming {
    /// Timeout for this step (e.g., "5m", "30s")
    #[serde(default)]
    pub timeout: Option<String>,

    /// Retry configuration
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Number of retries
    #[serde(default)]
    pub count: u32,

    /// Delay between retries (e.g., "10s")
    #[serde(default)]
    pub delay: Option<String>,

    /// Backoff strategy: "linear" or "exponential"
    #[serde(default)]
    pub backoff: Option<String>,
}

/// Verification commands
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepVerification {
    /// Commands to run for verification
    #[serde(default)]
    pub commands: Vec<VerificationCommand>,
}

/// A single verification command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCommand {
    /// Command to run
    pub cmd: String,

    /// Expected output substring
    #[serde(default)]
    pub expect: Option<String>,
}

/// Failure action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAction {
    /// Action to take: "stop", "abort", "continue", "retry"
    pub action: String,

    /// Message to display
    #[serde(default)]
    pub message: Option<String>,

    /// Whether to notify (email, etc.)
    #[serde(default)]
    pub notify: Vec<String>,

    /// Whether to preserve state for debugging
    #[serde(default)]
    pub preserve_state: bool,
}

/// Step constraints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepConstraints {
    /// Exclusive resource lock
    #[serde(default)]
    pub exclusive_resource: Option<String>,
}

/// Action types supported by the installer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Run a script
    Script,
    /// Install packages via apt
    AptInstall,
    /// Remove packages via apt
    AptRemove,
    /// Write a file
    FileWrite,
    /// Verify a condition
    Verify,
    /// Add user to group
    UserGroup,
    /// Custom action
    Custom(String),
}

impl Action {
    /// Parse action from string
    pub fn parse(s: &str) -> Self {
        match s {
            "script" => Self::Script,
            "apt-install" => Self::AptInstall,
            "apt-remove" => Self::AptRemove,
            "file-write" => Self::FileWrite,
            "verify" => Self::Verify,
            "user-group" => Self::UserGroup,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Environment variable definition with parsing
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variable name
    pub name: String,
    /// Default value
    pub default: Option<String>,
    /// Source environment variable
    pub from_env: Option<String>,
    /// Whether the variable is required
    pub required: bool,
    /// Validation pattern
    pub validate: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_installer() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"
"#;
        let spec = InstallerSpec::parse(toml).expect("Failed to parse minimal spec");
        assert_eq!(spec.installer.name, "test");
        assert_eq!(spec.installer.version, "1.0.0");
    }

    #[test]
    fn test_parse_installer_with_step() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "hello"
name = "Hello World"
action = "script"

[step.script]
content = "echo hello"
"#;
        let spec = InstallerSpec::parse(toml).expect("Failed to parse spec with step");
        assert_eq!(spec.step.len(), 1);
        assert_eq!(spec.step[0].id, "hello");
        assert_eq!(spec.step[0].action, "script");
    }

    #[test]
    fn test_parse_installer_with_artifact() {
        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[artifact]]
id = "myfile"
url = "https://example.com/file.tar.gz"
sha256 = "abc123"
"#;
        let spec = InstallerSpec::parse(toml).expect("Failed to parse spec with artifact");
        assert_eq!(spec.artifact.len(), 1);
        assert_eq!(spec.artifact[0].id, "myfile");
        assert_eq!(spec.artifact[0].sha256.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_action_parse() {
        assert_eq!(Action::parse("script"), Action::Script);
        assert_eq!(Action::parse("apt-install"), Action::AptInstall);
        assert_eq!(
            Action::parse("custom-action"),
            Action::Custom("custom-action".to_string())
        );
    }

    #[test]
    fn test_invalid_toml() {
        let toml = "INVALID [[[";
        let result = InstallerSpec::parse(toml);
        assert!(result.is_err());
    }
}
