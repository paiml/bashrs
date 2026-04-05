fn audit_installer_toml(value: &toml::Value, min_severity: &str) -> (Vec<AuditFinding>, u32, u32) {
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
            let step_id = step.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

            // Check for script content
            if let Some(script) = step.get("script").and_then(|s| s.get("content")) {
                if let Some(content) = script.as_str() {
                    if content.contains("curl")
                        && content.contains("| sh")
                        && severity_threshold <= 2
                    {
                        findings.push(AuditFinding {
                            code: "SEC001".to_string(),
                            severity: "error".to_string(),
                            category: "security".to_string(),
                            message: "Piping curl output directly to shell".to_string(),
                            location: Some(step_id.to_string()),
                            recommendation: Some(
                                "Download, verify checksum, then execute".to_string(),
                            ),
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
"#
            .to_string(),
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
"#
            .to_string(),
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
"#
            .to_string(),
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
"#
            .to_string(),
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
"#
            .to_string(),
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
"#
            .to_string(),
            min_severity: "info".to_string(),
        };

        let result = handler.handle(input).await.unwrap();
        assert!(result.findings.iter().any(|f| f.code == "QUAL001"));
        assert!(result.findings.iter().any(|f| f.code == "QUAL002"));
        assert!(result.findings.iter().any(|f| f.code == "QUAL003"));
    }

    #[test]
    fn test_MCP_013_generate_project_name() {
        assert_eq!(
            generate_project_name("Install Docker"),
            "install-docker-installer"
        );
        assert_eq!(
            generate_project_name("PostgreSQL database"),
            "postgresql-database-installer"
        );
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
