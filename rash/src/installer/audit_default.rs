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


include!("audit_audit.rs");
