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
                    .map_or_else(|| "null".to_string(), |l| format!("\"{}\"", l));
                let suggestion = f.suggestion.as_ref().map_or_else(
                    || "null".to_string(),
                    |s| format!("\"{}\"", s.replace('\"', "\\\"")),
                );

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

include!("audit_default.rs");
