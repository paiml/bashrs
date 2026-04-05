//! Installer MCP Handlers (§7 MCP-Assisted Generation)
//!
//! MCP tools for AI-assisted installer authoring using the batuta stack.
//!
//! Tools:
//! - `installer_scaffold` - Generate installer skeleton from description
//! - `installer_step_suggest` - Suggest next step based on current state
//! - `installer_validate` - Validate installer spec and suggest improvements
//! - `installer_audit` - Run security/quality audit on installer

use pforge_runtime::{Handler, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// installer_scaffold - Generate installer from natural language description
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InstallerScaffoldInput {
    /// Natural language description of what to install
    pub description: String,
    /// Target operating systems (default: ["ubuntu:22.04"])
    #[serde(default = "default_targets")]
    pub target_os: Vec<String>,
    /// Author name for the installer
    #[serde(default)]
    pub author: Option<String>,
}

fn default_targets() -> Vec<String> {
    vec!["ubuntu >= 22.04".to_string()]
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InstallerScaffoldOutput {
    /// Generated installer.toml content
    pub installer_toml: String,
    /// Suggested project name
    pub project_name: String,
    /// Number of steps generated
    pub step_count: usize,
    /// Suggestions for additional configuration
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

pub struct InstallerScaffoldHandler;

#[async_trait::async_trait]
impl Handler for InstallerScaffoldHandler {
    type Input = InstallerScaffoldInput;
    type Output = InstallerScaffoldOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let project_name = generate_project_name(&input.description);
        let (installer_toml, step_count) = generate_installer_scaffold(
            &input.description,
            &input.target_os,
            input.author.as_deref(),
        );

        let mut suggestions = Vec::new();

        // Add suggestions based on detected patterns
        if input.description.to_lowercase().contains("database") {
            suggestions
                .push("Consider adding a backup step before database modifications".to_string());
        }
        if input.description.to_lowercase().contains("docker") {
            suggestions.push("Ensure Docker daemon is running as a precondition".to_string());
        }
        if step_count > 5 {
            suggestions.push("Consider grouping related steps for better organization".to_string());
        }

        Ok(InstallerScaffoldOutput {
            installer_toml,
            project_name,
            step_count,
            suggestions,
        })
    }
}

// ============================================================================
// installer_step_suggest - Suggest next step based on current state
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InstallerStepSuggestInput {
    /// Current installer steps (as TOML strings)
    #[serde(default)]
    pub current_steps: Vec<String>,
    /// What the installer should achieve
    pub goal: String,
    /// Current step IDs for dependency tracking
    #[serde(default)]
    pub step_ids: Vec<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InstallerStepSuggestOutput {
    /// Suggested step in TOML format
    pub suggested_step: String,
    /// Step ID for the suggestion
    pub step_id: String,
    /// Rationale for the suggestion
    pub rationale: String,
    /// Dependencies on existing steps
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    /// Alternative suggestions
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

pub struct InstallerStepSuggestHandler;

#[async_trait::async_trait]
impl Handler for InstallerStepSuggestHandler {
    type Input = InstallerStepSuggestInput;
    type Output = InstallerStepSuggestOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let (suggested_step, step_id, rationale, depends_on, alternatives) =
            suggest_next_step(&input.current_steps, &input.goal, &input.step_ids);

        Ok(InstallerStepSuggestOutput {
            suggested_step,
            step_id,
            rationale,
            depends_on,
            alternatives,
        })
    }
}

// ============================================================================
// installer_validate - Validate installer spec and suggest improvements
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InstallerValidateInput {
    /// Installer TOML content
    pub installer_toml: String,
    /// Run security-focused validation
    #[serde(default)]
    pub security_focus: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InstallerValidateOutput {
    /// Whether the installer is valid
    pub valid: bool,
    /// Validation errors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ValidationIssue>,
    /// Validation warnings
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ValidationIssue>,
    /// Suggested improvements
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
    /// Quality score (0-100)
    pub quality_score: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ValidationIssue {
    /// Issue code (e.g., "SEC001", "QUAL002")
    pub code: String,
    /// Issue message
    pub message: String,
    /// Location in the TOML (step ID or section)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// How to fix the issue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

pub struct InstallerValidateHandler;

#[async_trait::async_trait]
impl Handler for InstallerValidateHandler {
    type Input = InstallerValidateInput;
    type Output = InstallerValidateOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        // Parse the TOML
        let parse_result: std::result::Result<toml::Value, _> =
            toml::from_str(&input.installer_toml);

        match parse_result {
            Ok(value) => {
                let (errors, warnings, suggestions, score) =
                    validate_installer_toml(&value, input.security_focus);

                Ok(InstallerValidateOutput {
                    valid: errors.is_empty(),
                    errors,
                    warnings,
                    suggestions,
                    quality_score: score,
                })
            }
            Err(e) => Ok(InstallerValidateOutput {
                valid: false,
                errors: vec![ValidationIssue {
                    code: "PARSE001".to_string(),
                    message: format!("Failed to parse TOML: {}", e),
                    location: None,
                    fix: Some("Check TOML syntax".to_string()),
                }],
                warnings: vec![],
                suggestions: vec![],
                quality_score: 0,
            }),
        }
    }
}

// ============================================================================
// installer_audit - Run security/quality audit
// ============================================================================

#[derive(Debug, Deserialize, JsonSchema)]
pub struct InstallerAuditInput {
    /// Installer TOML content
    pub installer_toml: String,
    /// Minimum severity to report (info, warning, error, critical)
    #[serde(default = "default_min_severity")]
    pub min_severity: String,
}

fn default_min_severity() -> String {
    "warning".to_string()
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct InstallerAuditOutput {
    /// Audit findings
    pub findings: Vec<AuditFinding>,
    /// Overall security score (0-100)
    pub security_score: u32,
    /// Overall quality score (0-100)
    pub quality_score: u32,
    /// Combined grade (A+, A, B, C, D, F)
    pub grade: String,
    /// Summary of the audit
    pub summary: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AuditFinding {
    /// Finding code (e.g., "SEC001")
    pub code: String,
    /// Severity level
    pub severity: String,
    /// Category (security, quality, performance)
    pub category: String,
    /// Finding message
    pub message: String,
    /// Affected step or section
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Recommended fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
}

pub struct InstallerAuditHandler;

#[async_trait::async_trait]
impl Handler for InstallerAuditHandler {
    type Input = InstallerAuditInput;
    type Output = InstallerAuditOutput;
    type Error = pforge_runtime::Error;

    async fn handle(&self, input: Self::Input) -> Result<Self::Output> {
        let parse_result: std::result::Result<toml::Value, _> =
            toml::from_str(&input.installer_toml);

        match parse_result {
            Ok(value) => {
                let (findings, security_score, quality_score) =
                    audit_installer_toml(&value, &input.min_severity);

                let combined_score = (security_score + quality_score) / 2;
                let grade = match combined_score {
                    95..=100 => "A+",
                    90..=94 => "A",
                    85..=89 => "A-",
                    80..=84 => "B+",
                    75..=79 => "B",
                    70..=74 => "B-",
                    65..=69 => "C+",
                    60..=64 => "C",
                    55..=59 => "C-",
                    50..=54 => "D",
                    _ => "F",
                };

                let summary = format!(
                    "Audit complete: {} findings, Security: {}/100, Quality: {}/100, Grade: {}",
                    findings.len(),
                    security_score,
                    quality_score,
                    grade
                );

                Ok(InstallerAuditOutput {
                    findings,
                    security_score,
                    quality_score,
                    grade: grade.to_string(),
                    summary,
                })
            }
            Err(e) => Err(pforge_runtime::Error::Handler(format!(
                "Failed to parse installer TOML: {}",
                e
            ))),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn generate_project_name(description: &str) -> String {
    // Extract key words from description
    let lower = description.to_lowercase();
    let words: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .take(3)
        .collect();

    if words.is_empty() {
        "my-installer".to_string()
    } else {
        format!("{}-installer", words.join("-"))
    }
}

fn generate_installer_scaffold(
    description: &str,
    target_os: &[String],
    author: Option<&str>,
) -> (String, usize) {
    let project_name = generate_project_name(description);
    let author_str = author.unwrap_or("bashrs team");
    let os_list: Vec<String> = target_os.iter().map(|s| format!("\"{}\"", s)).collect();

    // Detect common patterns and generate appropriate steps
    let desc_lower = description.to_lowercase();
    let mut steps = Vec::new();
    let mut step_count = 0;

    // System dependencies step
    steps.push(format!(
        r#"[[step]]
id = "install-deps"
name = "Install System Dependencies"
action = "apt-install"
packages = ["curl", "ca-certificates"]

[step.postconditions]
command_succeeds = "which curl"

[step.checkpoint]
enabled = true

[step.timing]
timeout = "10m""#
    ));
    step_count += 1;

    // Add package-specific steps based on description
    if desc_lower.contains("postgresql") || desc_lower.contains("postgres") {
        steps.push(format!(
            r#"
[[step]]
id = "add-postgres-repo"
name = "Add PostgreSQL APT Repository"
action = "script"
depends_on = ["install-deps"]

[step.script]
interpreter = "sh"
content = '''
curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /usr/share/keyrings/postgresql.gpg
echo "deb [signed-by=/usr/share/keyrings/postgresql.gpg] http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list
apt-get update
'''

[step.postconditions]
file_exists = "/etc/apt/sources.list.d/pgdg.list"

[step.checkpoint]
enabled = true

[[step]]
id = "install-postgresql"
name = "Install PostgreSQL"
action = "apt-install"
depends_on = ["add-postgres-repo"]
packages = ["postgresql-16", "postgresql-client-16"]

[step.postconditions]
command_succeeds = "pg_isready"

[step.checkpoint]
enabled = true

[step.timing]
timeout = "15m""#
        ));
        step_count += 2;
    }

    if desc_lower.contains("docker") {
        steps.push(format!(
            r#"
[[step]]
id = "install-docker"
name = "Install Docker"
action = "script"
depends_on = ["install-deps"]

[step.script]
interpreter = "sh"
content = '''
curl -fsSL https://get.docker.com | sh
usermod -aG docker $SUDO_USER || true
'''

[step.postconditions]
command_succeeds = "docker --version"

[step.checkpoint]
enabled = true

[step.timing]
timeout = "15m""#
        ));
        step_count += 1;
    }

    if desc_lower.contains("rust") || desc_lower.contains("cargo") {
        steps.push(format!(
            r#"
[[step]]
id = "install-rust"
name = "Install Rust Toolchain"
action = "script"
depends_on = ["install-deps"]

[step.script]
interpreter = "sh"
content = '''
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
'''

[step.postconditions]
command_succeeds = "rustc --version"

[step.checkpoint]
enabled = true


        include!("installer_part2_incl2.rs");
