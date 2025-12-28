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
        let (installer_toml, step_count) =
            generate_installer_scaffold(&input.description, &input.target_os, input.author.as_deref());

        let mut suggestions = Vec::new();

        // Add suggestions based on detected patterns
        if input.description.to_lowercase().contains("database") {
            suggestions.push("Consider adding a backup step before database modifications".to_string());
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
        let parse_result: std::result::Result<toml::Value, _> = toml::from_str(&input.installer_toml);

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
        let parse_result: std::result::Result<toml::Value, _> = toml::from_str(&input.installer_toml);

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

[step.timing]
timeout = "10m""#
        ));
        step_count += 1;
    }

    if desc_lower.contains("node") || desc_lower.contains("npm") {
        steps.push(format!(
            r#"
[[step]]
id = "install-node"
name = "Install Node.js"
action = "script"
depends_on = ["install-deps"]

[step.script]
interpreter = "sh"
content = '''
curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
apt-get install -y nodejs
'''

[step.postconditions]
command_succeeds = "node --version"

[step.checkpoint]
enabled = true

[step.timing]
timeout = "10m""#
        ));
        step_count += 1;
    }

    // Verification step
    steps.push(format!(
        r#"
[[step]]
id = "verify-installation"
name = "Verify Installation"
action = "script"
depends_on = [{}]

[step.script]
interpreter = "sh"
content = '''
echo "Installation complete!"
echo "Verifying components..."
'''

[step.checkpoint]
enabled = true

[step.timing]
timeout = "2m""#,
        if step_count > 1 {
            steps
                .iter()
                .skip(1)
                .filter_map(|s| {
                    s.lines()
                        .find(|l| l.starts_with("id = "))
                        .map(|l| l.replace("id = ", "").replace("\"", ""))
                })
                .map(|id| format!("\"{}\"", id))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "\"install-deps\"".to_string()
        }
    ));
    step_count += 1;

    let installer_toml = format!(
        r#"# Installer generated by bashrs MCP
# Description: {description}

[installer]
name = "{project_name}"
version = "1.0.0"
description = "{description}"
author = "{author_str}"

[installer.requirements]
os = [{os_list}]
arch = ["x86_64", "aarch64"]
privileges = "root"
network = true

[installer.security]
trust_model = "tofu"
require_signatures = false

{steps}
"#,
        os_list = os_list.join(", "),
        steps = steps.join("\n")
    );

    (installer_toml, step_count)
}

fn suggest_next_step(
    current_steps: &[String],
    goal: &str,
    step_ids: &[String],
) -> (String, String, String, Vec<String>, Vec<String>) {
    let goal_lower = goal.to_lowercase();
    let has_deps = current_steps
        .iter()
        .any(|s| s.contains("apt-install") || s.contains("install-deps"));
    let has_config = current_steps.iter().any(|s| s.contains("configure"));

    // Determine the best next step based on goal and current state
    let (step_template, step_id, rationale) = if !has_deps {
        (
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
timeout = "10m""#.to_string(),
            "install-deps".to_string(),
            "Every installer should start with system dependencies".to_string(),
        )
    } else if goal_lower.contains("config") && !has_config {
        (
            r#"[[step]]
id = "configure-app"
name = "Configure Application"
action = "script"

[step.script]
interpreter = "sh"
content = '''
# Add configuration here
mkdir -p /etc/myapp
'''

[step.postconditions]
file_exists = "/etc/myapp"

[step.checkpoint]
enabled = true

[step.timing]
timeout = "5m""#.to_string(),
            "configure-app".to_string(),
            "Configuration step needed based on goal".to_string(),
        )
    } else {
        (
            r#"[[step]]
id = "verify-installation"
name = "Verify Installation"
action = "script"

[step.script]
interpreter = "sh"
content = '''
echo "Verifying installation..."
'''

[step.checkpoint]
enabled = true

[step.timing]
timeout = "2m""#.to_string(),
            "verify-installation".to_string(),
            "Verification step ensures installation completed correctly".to_string(),
        )
    };

    let depends_on: Vec<String> = if !step_ids.is_empty() {
        vec![step_ids.last().cloned().unwrap_or_default()]
    } else {
        vec![]
    };

    let alternatives = vec![
        "Consider adding a rollback step for safety".to_string(),
        "Add postconditions to verify step success".to_string(),
    ];

    (step_template, step_id, rationale, depends_on, alternatives)
}

fn validate_installer_toml(
    value: &toml::Value,
    security_focus: bool,
) -> (Vec<ValidationIssue>, Vec<ValidationIssue>, Vec<String>, u32) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    let mut score = 100u32;

    // Check for [installer] section
    if value.get("installer").is_none() {
        errors.push(ValidationIssue {
            code: "STRUCT001".to_string(),
            message: "Missing [installer] section".to_string(),
            location: None,
            fix: Some("Add [installer] section with name, version, description".to_string()),
        });
        score = score.saturating_sub(20);
    } else {
        let installer = value.get("installer").expect("checked above");
        if installer.get("name").is_none() {
            errors.push(ValidationIssue {
                code: "STRUCT002".to_string(),
                message: "Missing installer.name".to_string(),
                location: Some("installer".to_string()),
                fix: Some("Add name = \"my-installer\"".to_string()),
            });
            score = score.saturating_sub(10);
        }
        if installer.get("version").is_none() {
            warnings.push(ValidationIssue {
                code: "STRUCT003".to_string(),
                message: "Missing installer.version".to_string(),
                location: Some("installer".to_string()),
                fix: Some("Add version = \"1.0.0\"".to_string()),
            });
            score = score.saturating_sub(5);
        }
    }

    // Check for steps
    let steps = value.get("step").and_then(|s| s.as_array());
    if steps.is_none() || steps.is_some_and(|s| s.is_empty()) {
        errors.push(ValidationIssue {
            code: "STRUCT010".to_string(),
            message: "No steps defined".to_string(),
            location: None,
            fix: Some("Add at least one [[step]] section".to_string()),
        });
        score = score.saturating_sub(30);
    }

    // Security checks
    if security_focus {
        if let Some(steps) = steps {
            for (i, step) in steps.iter().enumerate() {
                let default_id = format!("step-{}", i);
                let step_id = step
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&default_id);

                // Check for script content
                if let Some(script) = step.get("script").and_then(|s| s.get("content")) {
                    if let Some(content) = script.as_str() {
                        // Check for dangerous patterns
                        if content.contains("curl") && content.contains("| sh") {
                            warnings.push(ValidationIssue {
                                code: "SEC001".to_string(),
                                message: "Piping curl to shell is risky".to_string(),
                                location: Some(step_id.to_string()),
                                fix: Some(
                                    "Download script first, verify, then execute".to_string(),
                                ),
                            });
                            score = score.saturating_sub(10);
                        }
                        if content.contains("chmod 777") {
                            warnings.push(ValidationIssue {
                                code: "SEC002".to_string(),
                                message: "chmod 777 is overly permissive".to_string(),
                                location: Some(step_id.to_string()),
                                fix: Some("Use chmod 755 for executables, 644 for files".to_string()),
                            });
                            score = score.saturating_sub(5);
                        }
                    }
                }

                // Check for postconditions
                if step.get("postconditions").is_none() {
                    suggestions.push(format!("Step '{}' has no postconditions", step_id));
                }

                // Check for checkpoint
                if step.get("checkpoint").is_none() {
                    suggestions.push(format!("Step '{}' has no checkpoint", step_id));
                }
            }
        }
    }

    (errors, warnings, suggestions, score)
}

fn audit_installer_toml(
    value: &toml::Value,
    min_severity: &str,
) -> (Vec<AuditFinding>, u32, u32) {
    let mut findings = Vec::new();
    let mut security_deductions = 0u32;
    let mut quality_deductions = 0u32;

    let severity_threshold = match min_severity {
        "info" => 0,
        "warning" => 1,
        "error" => 2,
        "critical" => 3,
        _ => 1,
    };

    // Security audit
    if let Some(steps) = value.get("step").and_then(|s| s.as_array()) {
        for step in steps {
            let step_id = step
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Check for script content
            if let Some(script) = step.get("script").and_then(|s| s.get("content")) {
                if let Some(content) = script.as_str() {
                    if content.contains("curl") && content.contains("| sh") && severity_threshold <= 2 {
                        findings.push(AuditFinding {
                            code: "SEC001".to_string(),
                            severity: "error".to_string(),
                            category: "security".to_string(),
                            message: "Piping curl output directly to shell".to_string(),
                            location: Some(step_id.to_string()),
                            recommendation: Some("Download, verify checksum, then execute".to_string()),
                        });
                        security_deductions += 15;
                    }

                    if content.contains("chmod 777") && severity_threshold <= 1 {
                        findings.push(AuditFinding {
                            code: "SEC002".to_string(),
                            severity: "warning".to_string(),
                            category: "security".to_string(),
                            message: "Using chmod 777 (world-writable)".to_string(),
                            location: Some(step_id.to_string()),
                            recommendation: Some("Use more restrictive permissions".to_string()),
                        });
                        security_deductions += 5;
                    }

                    if content.contains("eval") && severity_threshold <= 2 {
                        findings.push(AuditFinding {
                            code: "SEC003".to_string(),
                            severity: "error".to_string(),
                            category: "security".to_string(),
                            message: "Use of eval is dangerous".to_string(),
                            location: Some(step_id.to_string()),
                            recommendation: Some("Avoid eval, use direct execution".to_string()),
                        });
                        security_deductions += 10;
                    }
                }
            }

            // Quality checks
            if step.get("postconditions").is_none() && severity_threshold <= 1 {
                findings.push(AuditFinding {
                    code: "QUAL001".to_string(),
                    severity: "warning".to_string(),
                    category: "quality".to_string(),
                    message: "Step has no postconditions".to_string(),
                    location: Some(step_id.to_string()),
                    recommendation: Some("Add postconditions to verify step success".to_string()),
                });
                quality_deductions += 5;
            }

            if step.get("checkpoint").is_none() && severity_threshold <= 0 {
                findings.push(AuditFinding {
                    code: "QUAL002".to_string(),
                    severity: "info".to_string(),
                    category: "quality".to_string(),
                    message: "Step has no checkpoint".to_string(),
                    location: Some(step_id.to_string()),
                    recommendation: Some("Enable checkpoints for resume support".to_string()),
                });
                quality_deductions += 2;
            }

            if step.get("timing").is_none() && severity_threshold <= 0 {
                findings.push(AuditFinding {
                    code: "QUAL003".to_string(),
                    severity: "info".to_string(),
                    category: "quality".to_string(),
                    message: "Step has no timeout".to_string(),
                    location: Some(step_id.to_string()),
                    recommendation: Some("Add [step.timing] with timeout".to_string()),
                });
                quality_deductions += 2;
            }
        }
    }

    let security_score = 100u32.saturating_sub(security_deductions);
    let quality_score = 100u32.saturating_sub(quality_deductions);

    (findings, security_score, quality_score)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_MCP_001_scaffold_simple() {
        let handler = InstallerScaffoldHandler;
        let input = InstallerScaffoldInput {
            description: "Install Docker on Ubuntu".to_string(),
            target_os: vec!["ubuntu >= 22.04".to_string()],
            author: Some("Test Author".to_string()),
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.installer_toml.contains("[installer]"));
        assert!(result.installer_toml.contains("docker"));
        assert!(result.step_count > 0);
    }

    #[tokio::test]
    async fn test_MCP_002_scaffold_postgres() {
        let handler = InstallerScaffoldHandler;
        let input = InstallerScaffoldInput {
            description: "PostgreSQL 16 database server".to_string(),
            target_os: vec!["ubuntu >= 22.04".to_string()],
            author: None,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.installer_toml.contains("postgresql"));
        assert!(result.suggestions.iter().any(|s| s.contains("backup")));
    }

    #[tokio::test]
    async fn test_MCP_003_scaffold_rust() {
        let handler = InstallerScaffoldHandler;
        let input = InstallerScaffoldInput {
            description: "Rust development environment".to_string(),
            target_os: vec!["ubuntu >= 22.04".to_string()],
            author: None,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.installer_toml.contains("rustup"));
        assert!(result.installer_toml.contains("rustc --version"));
    }

    #[tokio::test]
    async fn test_MCP_004_step_suggest_no_deps() {
        let handler = InstallerStepSuggestHandler;
        let input = InstallerStepSuggestInput {
            current_steps: vec![],
            goal: "Install an application".to_string(),
            step_ids: vec![],
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.step_id.contains("install-deps"));
        assert!(result.rationale.contains("dependencies"));
    }

    #[tokio::test]
    async fn test_MCP_005_step_suggest_with_deps() {
        let handler = InstallerStepSuggestHandler;
        let input = InstallerStepSuggestInput {
            current_steps: vec!["action = \"apt-install\"".to_string()],
            goal: "Configure the application".to_string(),
            step_ids: vec!["install-deps".to_string()],
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.step_id.contains("configure"));
    }

    #[tokio::test]
    async fn test_MCP_006_validate_valid() {
        let handler = InstallerValidateHandler;
        let input = InstallerValidateInput {
            installer_toml: r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-step"
name = "Test"
action = "script"

[step.script]
content = "echo hello"

[step.postconditions]
command_succeeds = "true"

[step.checkpoint]
enabled = true
"#.to_string(),
            security_focus: true,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_MCP_007_validate_invalid_toml() {
        let handler = InstallerValidateHandler;
        let input = InstallerValidateInput {
            installer_toml: "not valid toml {{{".to_string(),
            security_focus: false,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "PARSE001"));
    }

    #[tokio::test]
    async fn test_MCP_008_validate_missing_installer() {
        let handler = InstallerValidateHandler;
        let input = InstallerValidateInput {
            installer_toml: r#"
[[step]]
id = "test"
name = "Test"
action = "script"
"#.to_string(),
            security_focus: false,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "STRUCT001"));
    }

    #[tokio::test]
    async fn test_MCP_009_validate_security_curl_pipe() {
        let handler = InstallerValidateHandler;
        let input = InstallerValidateInput {
            installer_toml: r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "risky"
name = "Risky Step"
action = "script"

[step.script]
content = "curl https://example.com/script.sh | sh"
"#.to_string(),
            security_focus: true,
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.warnings.iter().any(|w| w.code == "SEC001"));
    }

    #[tokio::test]
    async fn test_MCP_010_audit_clean() {
        let handler = InstallerAuditHandler;
        let input = InstallerAuditInput {
            installer_toml: r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "safe-step"
name = "Safe Step"
action = "script"

[step.script]
content = "echo hello"

[step.postconditions]
command_succeeds = "true"

[step.checkpoint]
enabled = true

[step.timing]
timeout = "5m"
"#.to_string(),
            min_severity: "warning".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.security_score >= 90);
        assert!(result.quality_score >= 90);
    }

    #[tokio::test]
    async fn test_MCP_011_audit_security_issues() {
        let handler = InstallerAuditHandler;
        let input = InstallerAuditInput {
            installer_toml: r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "risky"
name = "Risky"
action = "script"

[step.script]
content = '''
curl https://example.com/script.sh | sh
chmod 777 /tmp/file
eval "$DYNAMIC_CMD"
'''
"#.to_string(),
            min_severity: "info".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.findings.iter().any(|f| f.code == "SEC001"));
        assert!(result.findings.iter().any(|f| f.code == "SEC002"));
        assert!(result.findings.iter().any(|f| f.code == "SEC003"));
        assert!(result.security_score < 80);
    }

    #[tokio::test]
    async fn test_MCP_012_audit_quality_issues() {
        let handler = InstallerAuditHandler;
        let input = InstallerAuditInput {
            installer_toml: r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "no-quality"
name = "Low Quality Step"
action = "script"

[step.script]
content = "echo hello"
"#.to_string(),
            min_severity: "info".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.findings.iter().any(|f| f.code == "QUAL001"));
        assert!(result.findings.iter().any(|f| f.code == "QUAL002"));
        assert!(result.findings.iter().any(|f| f.code == "QUAL003"));
    }

    #[test]
    fn test_MCP_013_generate_project_name() {
        assert_eq!(generate_project_name("Install Docker"), "docker-installer");
        assert_eq!(generate_project_name("PostgreSQL database"), "postgresql-database-installer");
        assert_eq!(generate_project_name("hi"), "my-installer");
    }

    #[test]
    fn test_MCP_014_default_targets() {
        let targets = default_targets();
        assert_eq!(targets, vec!["ubuntu >= 22.04".to_string()]);
    }

    #[test]
    fn test_MCP_015_default_min_severity() {
        assert_eq!(default_min_severity(), "warning");
    }
}
