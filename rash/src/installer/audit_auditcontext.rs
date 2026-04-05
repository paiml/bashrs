//! Installer Audit Command (#120)
//!
//! Security and quality review for installer specifications.
//!
//! This module provides comprehensive auditing capabilities:
//!
//! - **Security Audit**: Check signature requirements, trust model, privilege escalation
//! - **Quality Audit**: Validate idempotency, preconditions, postconditions
//! - **Hermetic Audit**: Verify lockfile integrity, reproducibility settings
//! - **Best Practices**: Check for common anti-patterns and recommendations
//!
//! # Example
//!
//! ```ignore
//! use bashrs::installer::{AuditContext, AuditReport, AuditSeverity};
//!
//! let ctx = AuditContext::new();
//! let report = ctx.audit_installer(&spec)?;
//!
//! if report.has_critical_issues() {
//!     eprintln!("Critical issues found!");
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Severity level for audit findings

impl AuditContext {

    /// Audit security configuration from parsed spec
    fn audit_security_parsed(&self, spec: &super::spec::InstallerSpec, report: &mut AuditReport) {
        let security = &spec.installer.security;
        audit_sec001_signatures(security, report);
        audit_sec002_trust_model(security, report);
        audit_artifact_security(&spec.artifact, report);
        audit_sec006_privileges(spec, report);
        audit_step_script_security(&spec.step, report);
    }

    /// Audit quality configuration from parsed spec
    fn audit_quality_parsed(&self, spec: &super::spec::InstallerSpec, report: &mut AuditReport) {
        audit_qual001_postconditions(&spec.step, report);
        audit_qual002_checkpoints(&spec.step, report);
        audit_qual003_timeouts(&spec.step, report);
        audit_qual004_duplicate_ids(&spec.step, report);
        audit_qual005_dependencies(&spec.step, report);
    }

    /// Audit hermetic configuration from parsed spec
    fn audit_hermetic_parsed(&self, spec: &super::spec::InstallerSpec, report: &mut AuditReport) {
        // HERM001: Check for lockfile
        let has_lockfile_config = spec.installer.hermetic.lockfile.is_some();

        if !has_lockfile_config && !spec.artifact.is_empty() {
            report.add_finding(
                AuditFinding::new(
                    "HERM001",
                    AuditSeverity::Info,
                    AuditCategory::Hermetic,
                    "No lockfile configuration",
                    "Consider using a lockfile for reproducible installations.",
                )
                .with_suggestion("Run 'bashrs installer lock' to generate installer.lock"),
            );
        }

        // HERM002: Check for version pinning in artifacts
        for artifact in &spec.artifact {
            if artifact.url.contains("latest") || artifact.url.contains("${VERSION}") {
                report.add_finding(
                    AuditFinding::new(
                        "HERM002",
                        AuditSeverity::Warning,
                        AuditCategory::Hermetic,
                        "Unpinned artifact version",
                        format!(
                            "Artifact '{}' uses unpinned version (latest or variable).",
                            artifact.id
                        ),
                    )
                    .with_location(&artifact.id)
                    .with_suggestion("Pin to specific version for reproducibility"),
                );
            }
        }

        // HERM003: Check for external network dependencies
        let mut network_steps = 0;
        for step in &spec.step {
            if let Some(ref script) = step.script {
                if script.content.contains("curl")
                    || script.content.contains("wget")
                    || script.content.contains("apt-get update")
                {
                    network_steps += 1;
                }
            }
        }

        if network_steps > 0 {
            report.add_finding(
                AuditFinding::new(
                    "HERM003",
                    AuditSeverity::Info,
                    AuditCategory::Hermetic,
                    "Network-dependent steps",
                    format!(
                        "{} steps may require network access for hermetic builds.",
                        network_steps
                    ),
                )
                .with_suggestion("Pre-download artifacts and use --hermetic mode"),
            );
        }
    }

    /// Audit best practices from parsed spec
    fn audit_best_practices_parsed(
        &self,
        spec: &super::spec::InstallerSpec,
        report: &mut AuditReport,
    ) {
        audit_bp001_description(&spec.installer, report);
        audit_bp002_author(&spec.installer, report);
        audit_bp003_step_names(&spec.step, report);
        audit_bp004_orphan_steps(&spec.step, report);
        audit_bp005_long_scripts(&spec.step, report);
    }
}

/// BP001: Check for description
fn audit_bp001_description(installer: &super::spec::InstallerMetadata, report: &mut AuditReport) {
    if installer.description.is_empty() {
        report.add_finding(
            AuditFinding::new(
                "BP001",
                AuditSeverity::Suggestion,
                AuditCategory::BestPractices,
                "Missing installer description",
                "The installer has no description field.",
            )
            .with_suggestion("Add a description in [installer] section"),
        );
    }
}

/// BP002: Check for author
fn audit_bp002_author(installer: &super::spec::InstallerMetadata, report: &mut AuditReport) {
    if installer.author.is_empty() {
        report.add_finding(
            AuditFinding::new(
                "BP002",
                AuditSeverity::Suggestion,
                AuditCategory::BestPractices,
                "Missing author information",
                "The installer has no author field.",
            )
            .with_suggestion("Add an author in [installer] section"),
        );
    }
}

/// BP003: Check for step names
fn audit_bp003_step_names(steps: &[super::spec::Step], report: &mut AuditReport) {
    for step in steps {
        if step.name.is_empty() {
            report.add_finding(
                AuditFinding::new(
                    "BP003",
                    AuditSeverity::Suggestion,
                    AuditCategory::BestPractices,
                    "Missing step name",
                    format!("Step '{}' has no human-readable name.", step.id),
                )
                .with_location(&step.id)
                .with_suggestion("Add a descriptive name for better progress reporting"),
            );
        }
    }
}

/// BP004: Check for orphan steps (no dependencies and not depended upon)
fn audit_bp004_orphan_steps(steps: &[super::spec::Step], report: &mut AuditReport) {
    if steps.len() <= 1 {
        return;
    }
    let depended_upon: HashSet<&str> = steps
        .iter()
        .flat_map(|s| s.depends_on.iter().map(|d| d.as_str()))
        .collect();
    let first_step = steps.first().map(|s| s.id.as_str());
    for step in steps {
        if step.depends_on.is_empty()
            && !depended_upon.contains(step.id.as_str())
            && Some(step.id.as_str()) != first_step
        {
            report.add_finding(
                AuditFinding::new(
                    "BP004",
                    AuditSeverity::Warning,
                    AuditCategory::BestPractices,
                    "Orphan step detected",
                    format!(
                        "Step '{}' has no dependencies and nothing depends on it.",
                        step.id
                    ),
                )
                .with_location(&step.id)
                .with_suggestion("Add depends_on to establish execution order"),
            );
        }
    }
}

/// BP005: Check for very long scripts
fn audit_bp005_long_scripts(steps: &[super::spec::Step], report: &mut AuditReport) {
    for step in steps {
        let Some(ref script) = step.script else {
            continue;
        };
        let line_count = script.content.lines().count();
        if line_count > 50 {
            report.add_finding(
                AuditFinding::new(
                    "BP005",
                    AuditSeverity::Suggestion,
                    AuditCategory::BestPractices,
                    "Long script step",
                    format!(
                        "Step '{}' has {} lines. Consider breaking into smaller steps.",
                        step.id, line_count
                    ),
                )
                .with_location(&step.id)
                .with_suggestion("Split into multiple smaller, focused steps"),
            );
        }
    }
}

/// Check whether a step has any postconditions defined
fn step_has_postconditions(step: &super::spec::Step) -> bool {
    step.postconditions.file_exists.is_some()
        || step.postconditions.file_mode.is_some()
        || step.postconditions.command_succeeds.is_some()
        || !step.postconditions.packages_absent.is_empty()
        || step.postconditions.service_active.is_some()
        || step.postconditions.user_in_group.is_some()
        || !step.postconditions.env_matches.is_empty()
        || step
            .verification
            .as_ref()
            .is_some_and(|v| !v.commands.is_empty())
}

/// QUAL001: Check for postconditions
fn audit_qual001_postconditions(steps: &[super::spec::Step], report: &mut AuditReport) {
    for step in steps {
        if !step_has_postconditions(step) {
            report.add_finding(
                AuditFinding::new(
                    "QUAL001",
                    AuditSeverity::Warning,
                    AuditCategory::Quality,
                    "Missing postconditions",
                    format!(
                        "Step '{}' has no postconditions to verify success.",
                        step.id
                    ),
                )
                .with_location(&step.id)
                .with_suggestion("Add postconditions to verify step completed successfully"),
            );
        }
    }
}

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


include!("audit_part3_incl2.rs");
