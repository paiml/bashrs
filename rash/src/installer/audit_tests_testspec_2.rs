#[cfg(test)]
mod tests {
    use super::*;

    /// Simplified test spec for audit testing
    #[derive(Debug, Clone)]
    struct TestSpec {
        name: String,
        version: String,
        description: String,
        author: Option<String>,
        security: Option<TestSecurity>,
        artifacts: Vec<TestArtifact>,
        steps: Vec<TestStep>,
    }

    #[derive(Debug, Clone)]
    struct TestSecurity {
        trust_model: String,
        require_signatures: bool,
    }

    #[derive(Debug, Clone)]
    struct TestArtifact {
        id: String,
        url: String,
        sha256: Option<String>,
        signature: Option<String>,
        signed_by: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct TestStep {
        id: String,
        name: String,
        action: TestAction,
        depends_on: Vec<String>,
        has_postconditions: bool,
        has_checkpoint: bool,
        has_timing: bool,
    }

    #[derive(Debug, Clone)]
    enum TestAction {
        Script { content: String },
        Other,
    }

    fn create_test_spec() -> TestSpec {
        TestSpec {
            name: "test-installer".to_string(),
            version: "1.0.0".to_string(),
            description: "Test installer".to_string(),
            author: Some("Test Author".to_string()),
            security: Some(TestSecurity {
                trust_model: "keyring".to_string(),
                require_signatures: true,
            }),
            artifacts: vec![],
            steps: vec![TestStep {
                id: "step-1".to_string(),
                name: "First Step".to_string(),
                action: TestAction::Script {
                    content: "echo hello".to_string(),
                },
                depends_on: vec![],
                has_postconditions: true,
                has_checkpoint: true,
                has_timing: true,
            }],
        }
    }

    /// Simple audit for test specs (mimics real audit logic)
    fn audit_test_spec(spec: &TestSpec, path: &Path) -> AuditReport {
        let mut report = AuditReport::new(&spec.name, &spec.version, path.to_path_buf());
        audit_test_security(&spec.security, &mut report);
        audit_test_artifacts(&spec.artifacts, &mut report);
        audit_test_steps(&spec.steps, &mut report);
        if spec.description.is_empty() {
            report.add_finding(AuditFinding::new(
                "BP001",
                AuditSeverity::Suggestion,
                AuditCategory::BestPractices,
                "Missing installer description",
                "No description field.",
            ));
        }
        report.metadata.audited_at = chrono_timestamp();
        report.metadata.steps_audited = spec.steps.len();
        report.metadata.artifacts_audited = spec.artifacts.len();
        report
    }

    fn audit_test_security(security: &Option<TestSecurity>, report: &mut AuditReport) {
        let Some(ref security) = security else {
            report.add_finding(AuditFinding::new(
                "SEC003",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "No security configuration",
                "No security section defined.",
            ));
            return;
        };
        if !security.require_signatures {
            report.add_finding(AuditFinding::new(
                "SEC001",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "Signatures not required",
                "Artifact signature verification is disabled.",
            ));
        }
        if security.trust_model == "tofu" {
            report.add_finding(AuditFinding::new(
                "SEC002",
                AuditSeverity::Info,
                AuditCategory::Security,
                "Using TOFU model",
                "TOFU is suitable for development.",
            ));
        }
    }

    fn audit_test_artifacts(artifacts: &[TestArtifact], report: &mut AuditReport) {
        for artifact in artifacts {
            if artifact.signature.is_none() && artifact.signed_by.is_none() {
                report.add_finding(
                    AuditFinding::new(
                        "SEC004",
                        AuditSeverity::Warning,
                        AuditCategory::Security,
                        "Unsigned artifact",
                        format!("Artifact '{}' has no signature.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
            if artifact.sha256.is_none() {
                report.add_finding(
                    AuditFinding::new(
                        "SEC005",
                        AuditSeverity::Error,
                        AuditCategory::Security,
                        "Missing artifact hash",
                        format!("Artifact '{}' has no SHA256.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
            if artifact.url.contains("latest") {
                report.add_finding(
                    AuditFinding::new(
                        "HERM002",
                        AuditSeverity::Warning,
                        AuditCategory::Hermetic,
                        "Unpinned artifact version",
                        format!("Artifact '{}' uses unpinned version.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
        }
    }

    fn audit_test_steps(steps: &[TestStep], report: &mut AuditReport) {
        let step_ids: std::collections::HashSet<&str> =
            steps.iter().map(|s| s.id.as_str()).collect();
        let mut seen_ids: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for step in steps {
            audit_test_step_quality(step, &step_ids, &mut seen_ids, report);
        }
    }

    fn audit_test_step_quality<'a>(
        step: &'a TestStep,
        step_ids: &std::collections::HashSet<&str>,
        seen_ids: &mut std::collections::HashSet<&'a str>,
        report: &mut AuditReport,
    ) {
        if seen_ids.contains(step.id.as_str()) {
            report.add_finding(
                AuditFinding::new(
                    "QUAL004",
                    AuditSeverity::Error,
                    AuditCategory::Quality,
                    "Duplicate step ID",
                    format!("Step ID '{}' is duplicated.", step.id),
                )
                .with_location(&step.id),
            );
        }
        seen_ids.insert(&step.id);

        for dep in &step.depends_on {
            if !step_ids.contains(dep.as_str()) {
                report.add_finding(
                    AuditFinding::new(
                        "QUAL005",
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Invalid dependency reference",
                        format!("Step '{}' depends on non-existent '{}'.", step.id, dep),
                    )
                    .with_location(&step.id),
                );
            }
        }

        if !step.has_postconditions {
            report.add_finding(
                AuditFinding::new(
                    "QUAL001",
                    AuditSeverity::Warning,
                    AuditCategory::Quality,
                    "Missing postconditions",
                    format!("Step '{}' has no postconditions.", step.id),
                )
                .with_location(&step.id),
            );
        }

        if let TestAction::Script { ref content } = step.action {
            if content.contains("curl") && content.contains("| bash") {
                report.add_finding(
                    AuditFinding::new(
                        "SEC007",
                        AuditSeverity::Critical,
                        AuditCategory::Security,
                        "Unsafe curl pipe to bash",
                        "Step contains 'curl ... | bash' pattern.",
                    )
                    .with_location(&step.id),
                );
            }
        }
    }

    include!("audit_tests_extracted_AUDIT.rs");
}
