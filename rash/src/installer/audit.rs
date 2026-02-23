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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuditSeverity {
    /// Informational finding, no action required
    Info,
    /// Suggestion for improvement
    Suggestion,
    /// Warning that should be addressed
    Warning,
    /// Error that must be fixed
    Error,
    /// Critical security or reliability issue
    Critical,
}

impl AuditSeverity {
    /// Get display symbol for severity
    pub fn symbol(&self) -> &'static str {
        match self {
            AuditSeverity::Info => "ℹ",
            AuditSeverity::Suggestion => "💡",
            AuditSeverity::Warning => "⚠",
            AuditSeverity::Error => "❌",
            AuditSeverity::Critical => "🚨",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            AuditSeverity::Info => "INFO",
            AuditSeverity::Suggestion => "SUGGESTION",
            AuditSeverity::Warning => "WARNING",
            AuditSeverity::Error => "ERROR",
            AuditSeverity::Critical => "CRITICAL",
        }
    }
}

/// Category of audit finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditCategory {
    /// Security-related finding
    Security,
    /// Quality/reliability finding
    Quality,
    /// Hermetic/reproducibility finding
    Hermetic,
    /// Best practices finding
    BestPractices,
    /// Configuration finding
    Configuration,
}

impl AuditCategory {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            AuditCategory::Security => "Security",
            AuditCategory::Quality => "Quality",
            AuditCategory::Hermetic => "Hermetic",
            AuditCategory::BestPractices => "Best Practices",
            AuditCategory::Configuration => "Configuration",
        }
    }
}

/// An individual audit finding
#[derive(Debug, Clone)]
pub struct AuditFinding {
    /// Unique rule ID (e.g., "SEC001", "QUAL003")
    pub rule_id: String,
    /// Severity level
    pub severity: AuditSeverity,
    /// Category of finding
    pub category: AuditCategory,
    /// Short title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Location in the installer (step ID, artifact ID, etc.)
    pub location: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Related documentation URL
    pub doc_url: Option<String>,
}

impl AuditFinding {
    /// Create a new audit finding
    pub fn new(
        rule_id: impl Into<String>,
        severity: AuditSeverity,
        category: AuditCategory,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            category,
            title: title.into(),
            description: description.into(),
            location: None,
            suggestion: None,
            doc_url: None,
        }
    }

    /// Set location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Set documentation URL
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.doc_url = Some(url.into());
        self
    }

    /// Format finding for display
    pub fn format(&self) -> String {
        let mut output = format!(
            "{} [{}] {}: {}\n",
            self.severity.symbol(),
            self.rule_id,
            self.severity.name(),
            self.title
        );

        output.push_str(&format!("   {}\n", self.description));

        if let Some(ref loc) = self.location {
            output.push_str(&format!("   Location: {}\n", loc));
        }

        if let Some(ref suggestion) = self.suggestion {
            output.push_str(&format!("   Suggestion: {}\n", suggestion));
        }

        output
    }
}

/// Complete audit report
#[derive(Debug, Clone)]
pub struct AuditReport {
    /// Installer name
    pub installer_name: String,
    /// Installer version
    pub installer_version: String,
    /// Path to installer
    pub installer_path: PathBuf,
    /// All findings
    pub findings: Vec<AuditFinding>,
    /// Audit metadata
    pub metadata: AuditMetadata,
}

/// Audit metadata
#[derive(Debug, Clone)]
pub struct AuditMetadata {
    /// When the audit was performed
    pub audited_at: String,
    /// Number of steps audited
    pub steps_audited: usize,
    /// Number of artifacts audited
    pub artifacts_audited: usize,
    /// Audit duration in milliseconds
    pub duration_ms: u64,
}

impl AuditReport {
    /// Create a new empty report
    pub fn new(name: impl Into<String>, version: impl Into<String>, path: PathBuf) -> Self {
        Self {
            installer_name: name.into(),
            installer_version: version.into(),
            installer_path: path,
            findings: Vec::new(),
            metadata: AuditMetadata {
                audited_at: String::new(),
                steps_audited: 0,
                artifacts_audited: 0,
                duration_ms: 0,
            },
        }
    }

    /// Add a finding to the report
    pub fn add_finding(&mut self, finding: AuditFinding) {
        self.findings.push(finding);
    }

    /// Check if there are critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == AuditSeverity::Critical)
    }

    /// Check if there are errors or critical issues
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity >= AuditSeverity::Error)
    }

    /// Get findings by severity
    pub fn findings_by_severity(&self, severity: AuditSeverity) -> Vec<&AuditFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .collect()
    }

    /// Get findings by category
    pub fn findings_by_category(&self, category: AuditCategory) -> Vec<&AuditFinding> {
        self.findings
            .iter()
            .filter(|f| f.category == category)
            .collect()
    }

    /// Count findings by severity
    pub fn count_by_severity(&self) -> HashMap<AuditSeverity, usize> {
        let mut counts = HashMap::new();
        for finding in &self.findings {
            *counts.entry(finding.severity).or_insert(0) += 1;
        }
        counts
    }

    /// Get overall score (0-100)
    pub fn score(&self) -> u8 {
        let base_score = 100i32;
        let mut deductions = 0i32;

        for finding in &self.findings {
            deductions += match finding.severity {
                AuditSeverity::Info => 0,
                AuditSeverity::Suggestion => 1,
                AuditSeverity::Warning => 3,
                AuditSeverity::Error => 10,
                AuditSeverity::Critical => 25,
            };
        }

        (base_score - deductions).max(0) as u8
    }

    /// Get grade based on score
    pub fn grade(&self) -> &'static str {
        match self.score() {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }

    /// Format report for display
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Installer Audit Report\n{}\n\n", "═".repeat(60)));

        output.push_str(&format!(
            "Installer: {} v{}\n",
            self.installer_name, self.installer_version
        ));
        output.push_str(&format!("Path: {}\n", self.installer_path.display()));
        output.push_str(&format!(
            "Audited: {} ({} steps, {} artifacts)\n\n",
            self.metadata.audited_at, self.metadata.steps_audited, self.metadata.artifacts_audited
        ));

        // Score summary
        let score = self.score();
        let grade = self.grade();
        output.push_str(&format!("Score: {}/100 (Grade: {})\n\n", score, grade));

        // Severity summary
        let counts = self.count_by_severity();
        output.push_str("Summary:\n");
        for severity in [
            AuditSeverity::Critical,
            AuditSeverity::Error,
            AuditSeverity::Warning,
            AuditSeverity::Suggestion,
            AuditSeverity::Info,
        ] {
            let count = counts.get(&severity).unwrap_or(&0);
            if *count > 0 {
                output.push_str(&format!(
                    "  {} {}: {}\n",
                    severity.symbol(),
                    severity.name(),
                    count
                ));
            }
        }
        output.push('\n');

        // Findings grouped by category
        let mut categories_seen: HashSet<AuditCategory> = HashSet::new();
        for finding in &self.findings {
            categories_seen.insert(finding.category);
        }

        for category in [
            AuditCategory::Security,
            AuditCategory::Quality,
            AuditCategory::Hermetic,
            AuditCategory::BestPractices,
            AuditCategory::Configuration,
        ] {
            if categories_seen.contains(&category) {
                output.push_str(&format!("{}\n{}\n", category.name(), "-".repeat(40)));
                for finding in self.findings_by_category(category) {
                    output.push_str(&finding.format());
                    output.push('\n');
                }
            }
        }

        output
    }

    /// Export to JSON
    pub fn to_json(&self) -> String {
        let findings_json: Vec<String> = self
            .findings
            .iter()
            .map(|f| {
                let location = f
                    .location
                    .as_ref()
                    .map(|l| format!("\"{}\"", l))
                    .unwrap_or_else(|| "null".to_string());
                let suggestion = f
                    .suggestion
                    .as_ref()
                    .map(|s| format!("\"{}\"", s.replace('\"', "\\\"")))
                    .unwrap_or_else(|| "null".to_string());

                format!(
                    r#"    {{
      "rule_id": "{}",
      "severity": "{}",
      "category": "{}",
      "title": "{}",
      "description": "{}",
      "location": {},
      "suggestion": {}
    }}"#,
                    f.rule_id,
                    f.severity.name(),
                    f.category.name(),
                    f.title.replace('\"', "\\\""),
                    f.description.replace('\"', "\\\""),
                    location,
                    suggestion
                )
            })
            .collect();

        format!(
            r#"{{
  "installer_name": "{}",
  "installer_version": "{}",
  "installer_path": "{}",
  "score": {},
  "grade": "{}",
  "metadata": {{
    "audited_at": "{}",
    "steps_audited": {},
    "artifacts_audited": {},
    "duration_ms": {}
  }},
  "findings": [
{}
  ]
}}"#,
            self.installer_name,
            self.installer_version,
            self.installer_path.display(),
            self.score(),
            self.grade(),
            self.metadata.audited_at,
            self.metadata.steps_audited,
            self.metadata.artifacts_audited,
            self.metadata.duration_ms,
            findings_json.join(",\n")
        )
    }
}

/// Audit context for performing audits
pub struct AuditContext {
    /// Check security configuration
    pub check_security: bool,
    /// Check quality/reliability
    pub check_quality: bool,
    /// Check hermetic settings
    pub check_hermetic: bool,
    /// Check best practices
    pub check_best_practices: bool,
    /// Minimum severity to report
    pub min_severity: AuditSeverity,
    /// Issue #110: Rules to ignore
    pub ignored_rules: HashSet<String>,
}

impl Default for AuditContext {
    fn default() -> Self {
        Self {
            check_security: true,
            check_quality: true,
            check_hermetic: true,
            check_best_practices: true,
            min_severity: AuditSeverity::Info,
            ignored_rules: HashSet::new(),
        }
    }
}

impl AuditContext {
    /// Create a new audit context with all checks enabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create security-only audit
    pub fn security_only() -> Self {
        Self {
            check_security: true,
            check_quality: false,
            check_hermetic: false,
            check_best_practices: false,
            min_severity: AuditSeverity::Warning,
            ignored_rules: HashSet::new(),
        }
    }

    /// Set minimum severity
    pub fn with_min_severity(mut self, severity: AuditSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Issue #110: Add a rule to ignore
    pub fn with_ignored_rule(mut self, rule: impl Into<String>) -> Self {
        self.ignored_rules.insert(rule.into().to_uppercase());
        self
    }

    /// Issue #110: Check if a rule should be ignored
    fn should_ignore_rule(&self, rule_id: &str) -> bool {
        self.ignored_rules.contains(&rule_id.to_uppercase())
    }

    /// Audit an installer from a parsed InstallerSpec
    ///
    /// This method works with the actual InstallerSpec from the spec module.
    pub fn audit_parsed_spec(&self, spec: &super::spec::InstallerSpec, path: &Path) -> AuditReport {
        let start = std::time::Instant::now();
        let installer = &spec.installer;
        let mut report = AuditReport::new(&installer.name, &installer.version, path.to_path_buf());

        // Security audits
        if self.check_security {
            self.audit_security_parsed(spec, &mut report);
        }

        // Quality audits
        if self.check_quality {
            self.audit_quality_parsed(spec, &mut report);
        }

        // Hermetic audits
        if self.check_hermetic {
            self.audit_hermetic_parsed(spec, &mut report);
        }

        // Best practices audits
        if self.check_best_practices {
            self.audit_best_practices_parsed(spec, &mut report);
        }

        // Filter by minimum severity
        report.findings.retain(|f| f.severity >= self.min_severity);

        // Issue #110: Filter out ignored rules
        if !self.ignored_rules.is_empty() {
            report
                .findings
                .retain(|f| !self.should_ignore_rule(&f.rule_id));
        }

        // Update metadata
        report.metadata.audited_at = chrono_timestamp();
        report.metadata.steps_audited = spec.step.len();
        report.metadata.artifacts_audited = spec.artifact.len();
        report.metadata.duration_ms = start.elapsed().as_millis() as u64;

        report
    }

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
fn audit_bp001_description(
    installer: &super::spec::InstallerMetadata,
    report: &mut AuditReport,
) {
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
fn audit_bp002_author(
    installer: &super::spec::InstallerMetadata,
    report: &mut AuditReport,
) {
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
fn audit_sec001_signatures(
    security: &super::spec::InstallerSecurity,
    report: &mut AuditReport,
) {
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
fn audit_sec002_trust_model(
    security: &super::spec::InstallerSecurity,
    report: &mut AuditReport,
) {
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
fn audit_artifact_security(
    artifacts: &[super::spec::Artifact],
    report: &mut AuditReport,
) {
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
fn audit_sec006_privileges(
    spec: &super::spec::InstallerSpec,
    report: &mut AuditReport,
) {
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
fn audit_step_script_security(
    steps: &[super::spec::Step],
    report: &mut AuditReport,
) {
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

#[cfg(test)]
#[allow(dead_code)] // Test helper structs may have unused fields
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

    #[test]
    fn test_AUDIT_120_severity_ordering() {
        assert!(AuditSeverity::Info < AuditSeverity::Suggestion);
        assert!(AuditSeverity::Suggestion < AuditSeverity::Warning);
        assert!(AuditSeverity::Warning < AuditSeverity::Error);
        assert!(AuditSeverity::Error < AuditSeverity::Critical);
    }

    #[test]
    fn test_AUDIT_120_severity_symbols() {
        assert_eq!(AuditSeverity::Info.symbol(), "ℹ");
        assert_eq!(AuditSeverity::Critical.symbol(), "🚨");
    }

    #[test]
    fn test_AUDIT_120_category_names() {
        assert_eq!(AuditCategory::Security.name(), "Security");
        assert_eq!(AuditCategory::Quality.name(), "Quality");
        assert_eq!(AuditCategory::Hermetic.name(), "Hermetic");
    }

    #[test]
    fn test_AUDIT_120_finding_creation() {
        let finding = AuditFinding::new(
            "TEST001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test finding",
            "This is a test finding",
        );

        assert_eq!(finding.rule_id, "TEST001");
        assert_eq!(finding.severity, AuditSeverity::Warning);
        assert_eq!(finding.category, AuditCategory::Security);
    }

    #[test]
    fn test_AUDIT_120_finding_with_location() {
        let finding = AuditFinding::new(
            "TEST002",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Test",
            "Description",
        )
        .with_location("step-1")
        .with_suggestion("Fix it");

        assert_eq!(finding.location, Some("step-1".to_string()));
        assert_eq!(finding.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_AUDIT_120_finding_format() {
        let finding = AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test title",
            "Test description",
        );

        let formatted = finding.format();
        assert!(formatted.contains("SEC001"));
        assert!(formatted.contains("WARNING"));
        assert!(formatted.contains("Test title"));
    }

    #[test]
    fn test_AUDIT_120_report_creation() {
        let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert_eq!(report.installer_name, "test");
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_AUDIT_120_report_add_finding() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        report.add_finding(AuditFinding::new(
            "TEST001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test",
            "Description",
        ));

        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn test_AUDIT_120_report_has_critical() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert!(!report.has_critical_issues());

        report.add_finding(AuditFinding::new(
            "CRIT001",
            AuditSeverity::Critical,
            AuditCategory::Security,
            "Critical",
            "Description",
        ));

        assert!(report.has_critical_issues());
    }

    #[test]
    fn test_AUDIT_120_report_has_errors() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert!(!report.has_errors());

        report.add_finding(AuditFinding::new(
            "ERR001",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Error",
            "Description",
        ));

        assert!(report.has_errors());
    }

    #[test]
    fn test_AUDIT_120_report_score_perfect() {
        let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert_eq!(report.score(), 100);
        assert_eq!(report.grade(), "A");
    }

    #[test]
    fn test_AUDIT_120_report_score_with_findings() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        // Add warning (-3) and error (-10)
        report.add_finding(AuditFinding::new(
            "WARN001",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "Warning",
            "Description",
        ));

        report.add_finding(AuditFinding::new(
            "ERR001",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Error",
            "Description",
        ));

        assert_eq!(report.score(), 87); // 100 - 3 - 10
        assert_eq!(report.grade(), "B");
    }

    #[test]
    fn test_AUDIT_120_report_count_by_severity() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        report.add_finding(AuditFinding::new(
            "W1",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "W1",
            "D",
        ));
        report.add_finding(AuditFinding::new(
            "W2",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "W2",
            "D",
        ));
        report.add_finding(AuditFinding::new(
            "E1",
            AuditSeverity::Error,
            AuditCategory::Security,
            "E1",
            "D",
        ));

        let counts = report.count_by_severity();
        assert_eq!(counts.get(&AuditSeverity::Warning), Some(&2));
        assert_eq!(counts.get(&AuditSeverity::Error), Some(&1));
    }

    #[test]
    fn test_AUDIT_120_report_format() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        report.metadata.audited_at = "2025-12-28T10:00:00Z".to_string();

        report.add_finding(AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test Warning",
            "Description",
        ));

        let formatted = report.format();
        assert!(formatted.contains("Installer Audit Report"));
        assert!(formatted.contains("test v1.0.0"));
        assert!(formatted.contains("SEC001"));
    }

    #[test]
    fn test_AUDIT_120_report_to_json() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        report.metadata.audited_at = "2025-12-28T10:00:00Z".to_string();

        report.add_finding(
            AuditFinding::new(
                "SEC001",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "Test",
                "Description",
            )
            .with_location("step-1"),
        );

        let json = report.to_json();
        assert!(json.contains("\"installer_name\": \"test\""));
        assert!(json.contains("\"rule_id\": \"SEC001\""));
        assert!(json.contains("\"location\": \"step-1\""));
    }

    #[test]
    fn test_AUDIT_120_context_default() {
        let ctx = AuditContext::default();
        assert!(ctx.check_security);
        assert!(ctx.check_quality);
        assert!(ctx.check_hermetic);
        assert!(ctx.check_best_practices);
    }

    #[test]
    fn test_AUDIT_120_context_security_only() {
        let ctx = AuditContext::security_only();
        assert!(ctx.check_security);
        assert!(!ctx.check_quality);
        assert!(!ctx.check_hermetic);
    }

    #[test]
    fn test_AUDIT_120_audit_spec_basic() {
        let spec = create_test_spec();
        let path = PathBuf::from("/test/installer.toml");

        let report = audit_test_spec(&spec, &path);

        assert_eq!(report.installer_name, "test-installer");
        assert_eq!(report.metadata.steps_audited, 1);
    }

    #[test]
    fn test_AUDIT_120_audit_missing_security() {
        let mut spec = create_test_spec();
        spec.security = None;

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC003 finding
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC003"));
    }

    #[test]
    fn test_AUDIT_120_audit_unsigned_artifact() {
        let mut spec = create_test_spec();
        spec.artifacts.push(TestArtifact {
            id: "test-artifact".to_string(),
            url: "https://example.com/file.tar.gz".to_string(),
            sha256: Some("abc123".to_string()),
            signature: None,
            signed_by: None,
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC004 finding
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC004"));
    }

    #[test]
    fn test_AUDIT_120_audit_missing_hash() {
        let mut spec = create_test_spec();
        spec.artifacts.push(TestArtifact {
            id: "test-artifact".to_string(),
            url: "https://example.com/file.tar.gz".to_string(),
            sha256: None,
            signature: Some("sig.sig".to_string()),
            signed_by: Some("key-id".to_string()),
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC005 finding (missing hash)
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC005"));
    }

    #[test]
    fn test_AUDIT_120_audit_unsafe_curl_pipe() {
        let mut spec = create_test_spec();
        spec.steps[0].action = TestAction::Script {
            content: "curl https://example.com/install.sh | bash".to_string(),
        };

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC007 critical finding
        let sec007 = report.findings.iter().find(|f| f.rule_id == "SEC007");
        assert!(sec007.is_some());
        assert_eq!(sec007.unwrap().severity, AuditSeverity::Critical);
    }

    #[test]
    fn test_AUDIT_120_audit_missing_postconditions() {
        let mut spec = create_test_spec();
        spec.steps[0].has_postconditions = false;

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have QUAL001 finding
        assert!(report.findings.iter().any(|f| f.rule_id == "QUAL001"));
    }

    #[test]
    fn test_AUDIT_120_audit_duplicate_step_ids() {
        let mut spec = create_test_spec();
        spec.steps.push(TestStep {
            id: "step-1".to_string(), // Duplicate!
            name: "Duplicate Step".to_string(),
            action: TestAction::Script {
                content: "echo dup".to_string(),
            },
            depends_on: vec![],
            has_postconditions: true,
            has_checkpoint: true,
            has_timing: true,
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have QUAL004 error
        let qual004 = report.findings.iter().find(|f| f.rule_id == "QUAL004");
        assert!(qual004.is_some());
        assert_eq!(qual004.unwrap().severity, AuditSeverity::Error);
    }

    #[test]
    fn test_AUDIT_120_audit_invalid_dependency() {
        let mut spec = create_test_spec();
        spec.steps[0].depends_on = vec!["nonexistent-step".to_string()];

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have QUAL005 error
        let qual005 = report.findings.iter().find(|f| f.rule_id == "QUAL005");
        assert!(qual005.is_some());
    }

    #[test]
    fn test_AUDIT_120_audit_unpinned_version() {
        let mut spec = create_test_spec();
        spec.artifacts.push(TestArtifact {
            id: "unpinned".to_string(),
            url: "https://example.com/app-latest.tar.gz".to_string(),
            sha256: Some("abc".to_string()),
            signature: Some("sig".to_string()),
            signed_by: Some("key".to_string()),
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have HERM002 warning
        assert!(report.findings.iter().any(|f| f.rule_id == "HERM002"));
    }

    #[test]
    fn test_AUDIT_120_audit_missing_description() {
        let mut spec = create_test_spec();
        spec.description = String::new();

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have BP001 suggestion
        assert!(report.findings.iter().any(|f| f.rule_id == "BP001"));
    }

    #[test]
    fn test_AUDIT_120_min_severity_filter() {
        let mut spec = create_test_spec();
        spec.description = String::new(); // BP001 suggestion
        spec.security = None; // SEC003 warning

        let mut report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Filter findings by minimum severity
        report
            .findings
            .retain(|f| f.severity >= AuditSeverity::Warning);

        // Should not have suggestions, only warnings and above
        assert!(report
            .findings
            .iter()
            .all(|f| f.severity >= AuditSeverity::Warning));
    }

    // Property tests
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_score_in_valid_range(
                warning_count in 0usize..20,
                error_count in 0usize..10
            ) {
                let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

                for i in 0..warning_count {
                    report.add_finding(AuditFinding::new(
                        format!("W{}", i),
                        AuditSeverity::Warning,
                        AuditCategory::Quality,
                        "Warning",
                        "D",
                    ));
                }

                for i in 0..error_count {
                    report.add_finding(AuditFinding::new(
                        format!("E{}", i),
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Error",
                        "D",
                    ));
                }

                let score = report.score();
                prop_assert!(score <= 100);
            }

            #[test]
            fn prop_grade_matches_score(score in 0u8..=100) {
                // Create report with enough findings to reach target score
                let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

                // Add errors to reduce score (each -10)
                let errors_needed = ((100 - score as i32) / 10).max(0) as usize;
                for i in 0..errors_needed {
                    report.add_finding(AuditFinding::new(
                        format!("E{}", i),
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Error",
                        "D",
                    ));
                }

                let actual_score = report.score();
                let grade = report.grade();

                let expected_grade = match actual_score {
                    90..=100 => "A",
                    80..=89 => "B",
                    70..=79 => "C",
                    60..=69 => "D",
                    _ => "F",
                };

                prop_assert_eq!(grade, expected_grade);
            }

            #[test]
            fn prop_json_is_valid_structure(name in "[a-z]{5,10}", version in "[0-9]\\.[0-9]\\.[0-9]") {
                let report = AuditReport::new(&name, &version, PathBuf::from("/test"));
                let json = report.to_json();

                // Basic JSON structure validation
                let starts_correct = json.starts_with("{");
                let ends_correct = json.ends_with("}");
                let expected_name = format!("\"installer_name\": \"{}\"", name);
                let contains_name = json.contains(&expected_name);
                prop_assert!(starts_correct, "JSON should start with opening brace");
                prop_assert!(ends_correct, "JSON should end with closing brace");
                prop_assert!(contains_name, "JSON should contain installer name");
            }
        }
    }

    /// Tests for Issue #112: audit postconditions not recognized with commands format
    mod issue_112_tests {
        use super::*;
        use crate::installer::spec::InstallerSpec;

        #[test]
        fn test_112_postconditions_verification_commands_recognized() {
            // Issue #112: Step with verification.commands should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"
description = "Test installer"

[[step]]
id = "install-app"
name = "Install Application"
action = "script"

[step.script]
content = "apt-get install app"

[step.verification]
commands = [
    { cmd = "which app", expect = "/usr/bin/app" }
]
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            // Should NOT have QUAL001 finding - verification.commands counts as postconditions
            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when verification.commands is present"
            );
        }

        #[test]
        fn test_112_postconditions_file_mode_recognized() {
            // Issue #112: Step with file_mode postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "set-perms"
name = "Set Permissions"
action = "script"

[step.script]
content = "chmod 755 /app"

[step.postconditions]
file_mode = "/app:755"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when file_mode postcondition is present"
            );
        }

        #[test]
        fn test_112_postconditions_service_active_recognized() {
            // Issue #112: Step with service_active postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "start-service"
name = "Start Service"
action = "script"

[step.script]
content = "systemctl start myapp"

[step.postconditions]
service_active = "myapp"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when service_active postcondition is present"
            );
        }

        #[test]
        fn test_112_postconditions_env_matches_recognized() {
            // Issue #112: Step with env_matches postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "setup-env"
name = "Setup Environment"
action = "script"

[step.script]
content = "export PATH=/app/bin:$PATH"

[step.postconditions.env_matches]
PATH = "/app/bin"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when env_matches postcondition is present"
            );
        }

        #[test]
        fn test_112_postconditions_user_in_group_recognized() {
            // Issue #112: Step with user_in_group postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "add-group"
name = "Add User to Group"
action = "script"

[step.script]
content = "usermod -aG docker $USER"

[step.postconditions.user_in_group]
user = "deploy"
group = "docker"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when user_in_group postcondition is present"
            );
        }

        #[test]
        fn test_112_no_postconditions_triggers_qual001() {
            // Sanity check: Step with NO postconditions SHOULD trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "no-post"
name = "Step Without Postconditions"
action = "script"

[step.script]
content = "echo hello"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_some(),
                "QUAL001 should be raised when no postconditions are present"
            );
        }
    }

    // ============================================================================
    // Coverage Tests - audit_security_parsed (SEC_COV_001-012)
    // ============================================================================
    mod security_parsed_tests {
        use super::*;
        use crate::installer::spec::InstallerSpec;

        fn sec_audit(toml: &str) -> AuditReport {
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            // Use new() not security_only() — security_only has min_severity=Warning
            // which filters out Info-level findings like SEC002 and SEC006
            let ctx = AuditContext::new();
            ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"))
        }

        #[test]
        fn test_SEC_COV_001_signatures_not_required() {
            // SEC001: require_signatures = false triggers warning
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = false
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC001"));
        }

        #[test]
        fn test_SEC_COV_002_signatures_required_no_sec001() {
            // SEC001: require_signatures = true should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = true
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC001"));
        }

        #[test]
        fn test_SEC_COV_003_tofu_trust_model() {
            // SEC002: trust_model = "tofu" triggers info
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "tofu"
require_signatures = true
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC002"));
        }

        #[test]
        fn test_SEC_COV_004_keyring_trust_model_no_sec002() {
            // SEC002: trust_model = "keyring" should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = true
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC002"));
        }

        #[test]
        fn test_SEC_COV_005_unsigned_artifact() {
            // SEC004: artifact without signature or signed_by
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
sha256 = "abc123"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC004"));
        }

        #[test]
        fn test_SEC_COV_006_signed_artifact_no_sec004() {
            // SEC004: signed artifact should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
sha256 = "abc123"
signature = "myapp.sig"
signed_by = "key-001"
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC004"));
        }

        #[test]
        fn test_SEC_COV_007_missing_sha256() {
            // SEC005: artifact without sha256
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
signature = "myapp.sig"
signed_by = "key-001"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC005"));
        }

        #[test]
        fn test_SEC_COV_008_root_privileges() {
            // SEC006: privileges = "root" triggers info
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "root"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC006"));
        }

        #[test]
        fn test_SEC_COV_009_user_privileges_no_sec006() {
            // SEC006: privileges = "user" should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "user"
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC006"));
        }

        #[test]
        fn test_SEC_COV_010_curl_pipe_bash() {
            // SEC007: curl ... | bash pattern
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "curl https://example.com/setup.sh | bash"
"#,
            );
            let sec007 = report.findings.iter().find(|f| f.rule_id == "SEC007");
            assert!(sec007.is_some());
            assert_eq!(
                sec007.expect("has sec007").severity,
                AuditSeverity::Critical
            );
        }

        #[test]
        fn test_SEC_COV_011_eval_in_script() {
            // SEC008: eval in script
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "eval $DYNAMIC_CMD"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC008"));
        }

        #[test]
        fn test_SEC_COV_012_clean_spec_no_findings() {
            // Fully clean spec should have no security findings
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "user"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp-1.0.0.tar.gz"
sha256 = "abc123def456"
signature = "myapp.sig"
signed_by = "key-001"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "tar xf myapp.tar.gz && ./install.sh"
"#,
            );
            let sec_findings: Vec<_> = report
                .findings
                .iter()
                .filter(|f| f.rule_id.starts_with("SEC"))
                .collect();
            assert!(
                sec_findings.is_empty(),
                "Clean spec should have no SEC findings, got: {sec_findings:?}"
            );
        }
    }
}
