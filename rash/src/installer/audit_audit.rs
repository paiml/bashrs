/// QUAL002: Check for checkpoints
fn audit_qual002_checkpoints(steps: &[super::spec::Step], report: &mut AuditReport) {
    let steps_without_checkpoint = steps.iter().filter(|s| !s.checkpoint.enabled).count();
    if steps_without_checkpoint > 0 && steps.len() > 1 {
        report.add_finding(
            AuditFinding::new(
                "QUAL002",
                AuditSeverity::Suggestion,
                AuditCategory::Quality,
                "Steps without checkpoints",
                format!(
                    "{} of {} steps have no checkpoint configuration.",
                    steps_without_checkpoint,
                    steps.len()
                ),
            )
            .with_suggestion("Enable checkpoints for resumable installations"),
        );
    }
}

/// QUAL003: Check for timeouts
fn audit_qual003_timeouts(steps: &[super::spec::Step], report: &mut AuditReport) {
    for step in steps {
        if step.timing.timeout.is_none() {
            report.add_finding(
                AuditFinding::new(
                    "QUAL003",
                    AuditSeverity::Suggestion,
                    AuditCategory::Quality,
                    "No timeout specified",
                    format!("Step '{}' has no timeout configuration.", step.id),
                )
                .with_location(&step.id)
                .with_suggestion("Add timing configuration with appropriate timeout"),
            );
        }
    }
}

/// QUAL004: Check for duplicate step IDs
fn audit_qual004_duplicate_ids(steps: &[super::spec::Step], report: &mut AuditReport) {
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for step in steps {
        if seen_ids.contains(step.id.as_str()) {
            report.add_finding(
                AuditFinding::new(
                    "QUAL004",
                    AuditSeverity::Error,
                    AuditCategory::Quality,
                    "Duplicate step ID",
                    format!("Step ID '{}' is used more than once.", step.id),
                )
                .with_location(&step.id),
            );
        }
        seen_ids.insert(&step.id);
    }
}

/// QUAL005: Check dependency references
fn audit_qual005_dependencies(steps: &[super::spec::Step], report: &mut AuditReport) {
    let step_ids: HashSet<&str> = steps.iter().map(|s| s.id.as_str()).collect();
    for step in steps {
        for dep in &step.depends_on {
            if !step_ids.contains(dep.as_str()) {
                report.add_finding(
                    AuditFinding::new(
                        "QUAL005",
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Invalid dependency reference",
                        format!(
                            "Step '{}' depends on '{}' which does not exist.",
                            step.id, dep
                        ),
                    )
                    .with_location(&step.id),
                );
            }
        }
    }
}

/// SEC001: Check if signatures are required
fn audit_sec001_signatures(security: &super::spec::InstallerSecurity, report: &mut AuditReport) {
    if !security.require_signatures {
        report.add_finding(
            AuditFinding::new(
                "SEC001",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "Signatures not required",
                "Artifact signature verification is disabled. This allows potentially tampered artifacts.",
            )
            .with_suggestion("Set require_signatures = true in [installer.security]"),
        );
    }
}

/// SEC002: Check trust model
fn audit_sec002_trust_model(security: &super::spec::InstallerSecurity, report: &mut AuditReport) {
    if security.trust_model == "tofu" {
        report.add_finding(
            AuditFinding::new(
                "SEC002",
                AuditSeverity::Info,
                AuditCategory::Security,
                "Using Trust-On-First-Use model",
                "TOFU is suitable for development but explicit keyring is recommended for production.",
            )
            .with_suggestion("Consider using trust_model = \"keyring\" for production"),
        );
    }
}

/// SEC004/SEC005: Check artifacts for signatures and hashes
fn audit_artifact_security(artifacts: &[super::spec::Artifact], report: &mut AuditReport) {
    for artifact in artifacts {
        if artifact.signature.is_none() && artifact.signed_by.is_none() {
            report.add_finding(
                AuditFinding::new(
                    "SEC004",
                    AuditSeverity::Warning,
                    AuditCategory::Security,
                    "Unsigned artifact",
                    format!(
                        "Artifact '{}' has no signature or signer specified.",
                        artifact.id
                    ),
                )
                .with_location(&artifact.id)
                .with_suggestion("Add signature and signed_by fields to artifact"),
            );
        }
        if artifact.sha256.is_none() {
            report.add_finding(
                AuditFinding::new(
                    "SEC005",
                    AuditSeverity::Error,
                    AuditCategory::Security,
                    "Missing artifact hash",
                    format!(
                        "Artifact '{}' has no SHA256 hash for integrity verification.",
                        artifact.id
                    ),
                )
                .with_location(&artifact.id)
                .with_suggestion("Add sha256 field with the artifact's content hash"),
            );
        }
    }
}

/// SEC006: Check for privilege escalation
fn audit_sec006_privileges(spec: &super::spec::InstallerSpec, report: &mut AuditReport) {
    if spec.installer.requirements.privileges == "root" {
        report.add_finding(
            AuditFinding::new(
                "SEC006",
                AuditSeverity::Info,
                AuditCategory::Security,
                "Root privileges required",
                "This installer requires root privileges. Ensure this is necessary.",
            )
            .with_suggestion("Review if all steps truly require root access"),
        );
    }
}

/// SEC007/SEC008: Check for unsafe script patterns in steps
fn audit_step_script_security(steps: &[super::spec::Step], report: &mut AuditReport) {
    for step in steps {
        let Some(ref script) = step.script else {
            continue;
        };
        if script.content.contains("curl") && script.content.contains("| bash") {
            report.add_finding(
                AuditFinding::new(
                    "SEC007",
                    AuditSeverity::Critical,
                    AuditCategory::Security,
                    "Unsafe curl pipe to bash",
                    "Step contains 'curl ... | bash' pattern which is vulnerable to MITM attacks.",
                )
                .with_location(&step.id)
                .with_suggestion("Download artifact first, verify signature, then execute"),
            );
        }
        if script.content.contains("eval") {
            report.add_finding(
                AuditFinding::new(
                    "SEC008",
                    AuditSeverity::Warning,
                    AuditCategory::Security,
                    "Use of eval",
                    "Step contains 'eval' which can execute arbitrary code.",
                )
                .with_location(&step.id)
                .with_suggestion("Avoid eval; use direct commands or safe alternatives"),
            );
        }
    }
}

/// Generate ISO-8601 timestamp
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple ISO-8601 format
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let remaining_days = days % 365;
    let months = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;

    let day_secs = secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        years, months, day, hours, minutes, seconds
    )
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "audit_tests_testspec_2.rs"]
// FIXME(PMAT-238): mod tests_extracted;
