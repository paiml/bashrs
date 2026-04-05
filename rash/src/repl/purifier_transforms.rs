// Transformation explanation types and safety rationale generators.
use super::purifier::purify_bash;

// ===== REPL-013-001: TRANSFORMATION EXPLANATION TYPES =====

/// Category of transformation applied during purification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformationCategory {
    /// Makes code safe to re-run without side effects
    Idempotency,
    /// Makes code produce consistent results across runs
    Determinism,
    /// Prevents injection, race conditions, etc.
    Safety,
}

/// Detailed explanation of a single transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformationExplanation {
    /// Category of transformation
    pub category: TransformationCategory,
    /// Brief title of the transformation
    pub title: String,
    /// Original code snippet
    pub original: String,
    /// Transformed code snippet
    pub transformed: String,
    /// Detailed description of what changed
    pub what_changed: String,
    /// Explanation of why this change improves the code
    pub why_it_matters: String,
    /// Optional line number where transformation occurred
    pub line_number: Option<usize>,
    /// Detailed safety rationale (REPL-013-002)
    pub safety_rationale: SafetyRationale,
    /// Alternative approaches (REPL-013-003)
    pub alternatives: Vec<Alternative>,
}

impl TransformationExplanation {
    /// Create a new transformation explanation
    pub fn new(
        category: TransformationCategory,
        title: impl Into<String>,
        original: impl Into<String>,
        transformed: impl Into<String>,
        what_changed: impl Into<String>,
        why_it_matters: impl Into<String>,
    ) -> Self {
        Self {
            category,
            title: title.into(),
            original: original.into(),
            transformed: transformed.into(),
            what_changed: what_changed.into(),
            why_it_matters: why_it_matters.into(),
            line_number: None,
            safety_rationale: SafetyRationale::new(), // REPL-013-002: Default rationale
            alternatives: Vec::new(),                 // REPL-013-003: Default empty alternatives
        }
    }

    /// Set line number where transformation occurred
    pub fn with_line_number(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Set safety rationale (REPL-013-002)
    pub fn with_safety_rationale(mut self, rationale: SafetyRationale) -> Self {
        self.safety_rationale = rationale;
        self
    }

    /// Set alternatives (REPL-013-003)
    pub fn with_alternatives(mut self, alternatives: Vec<Alternative>) -> Self {
        self.alternatives = alternatives;
        self
    }
}

// ===== REPL-013-002: SAFETY RATIONALE TYPES =====

/// Severity level for safety concerns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetySeverity {
    /// Must fix: Prevents catastrophic failures or critical security issues
    Critical,
    /// Should fix: Prevents serious operational or security problems
    High,
    /// Recommended: Improves robustness and reduces risk
    Medium,
    /// Optional: Minor improvements
    Low,
}

/// Detailed safety rationale for a transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyRationale {
    /// Security vulnerabilities prevented
    pub vulnerabilities_prevented: Vec<String>,
    /// Operational failures eliminated
    pub failures_eliminated: Vec<String>,
    /// Attack vectors closed
    pub attack_vectors_closed: Vec<String>,
    /// Impact if NOT applied
    pub impact_without_fix: String,
    /// Severity level
    pub severity: SafetySeverity,
}

impl SafetyRationale {
    /// Create empty rationale
    pub fn new() -> Self {
        Self {
            vulnerabilities_prevented: Vec::new(),
            failures_eliminated: Vec::new(),
            attack_vectors_closed: Vec::new(),
            impact_without_fix: String::new(),
            severity: SafetySeverity::Low,
        }
    }

    /// Add vulnerability prevented
    pub fn add_vulnerability(mut self, vuln: impl Into<String>) -> Self {
        self.vulnerabilities_prevented.push(vuln.into());
        self
    }

    /// Add failure eliminated
    pub fn add_failure(mut self, failure: impl Into<String>) -> Self {
        self.failures_eliminated.push(failure.into());
        self
    }

    /// Add attack vector closed
    pub fn add_attack_vector(mut self, vector: impl Into<String>) -> Self {
        self.attack_vectors_closed.push(vector.into());
        self
    }

    /// Set impact description
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.impact_without_fix = impact.into();
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: SafetySeverity) -> Self {
        self.severity = severity;
        self
    }
}

impl Default for SafetyRationale {
    fn default() -> Self {
        Self::new()
    }
}

// ===== REPL-013-003: ALTERNATIVE SUGGESTIONS =====

/// A single alternative approach to a transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alternative {
    /// Brief description of this approach
    pub approach: String,
    /// Code example showing this alternative
    pub example: String,
    /// When to prefer this approach
    pub when_to_use: String,
    /// Pros of this approach
    pub pros: Vec<String>,
    /// Cons of this approach
    pub cons: Vec<String>,
}

impl Alternative {
    /// Create new alternative
    pub fn new(
        approach: impl Into<String>,
        example: impl Into<String>,
        when_to_use: impl Into<String>,
    ) -> Self {
        Self {
            approach: approach.into(),
            example: example.into(),
            when_to_use: when_to_use.into(),
            pros: Vec::new(),
            cons: Vec::new(),
        }
    }

    /// Add a pro (advantage)
    pub fn add_pro(mut self, pro: impl Into<String>) -> Self {
        self.pros.push(pro.into());
        self
    }

    /// Add a con (disadvantage)
    pub fn add_con(mut self, con: impl Into<String>) -> Self {
        self.cons.push(con.into());
        self
    }
}

// ===== REPL-013-002: SAFETY RATIONALE GENERATION FUNCTIONS (RED PHASE STUBS) =====

/// Generate safety rationale for idempotency transformations
pub fn generate_idempotency_rationale(transformation_title: &str) -> SafetyRationale {
    match transformation_title {
        title if title.contains("mkdir") && title.contains("-p") => SafetyRationale::new()
            .add_failure("Script fails if directory already exists")
            .add_failure("Non-atomic operations create race conditions")
            .add_failure("Partial failure leaves system in inconsistent state")
            .with_impact(
                "Without -p flag, mkdir fails on re-run, breaking automation \
                 and deployment pipelines. Creates deployment race conditions \
                 in parallel execution environments.",
            )
            .with_severity(SafetySeverity::High),

        title if title.contains("rm") && title.contains("-f") => SafetyRationale::new()
            .add_failure("Script fails if file doesn't exist")
            .add_failure("Cleanup scripts cannot be re-run safely")
            .add_failure("Error handling becomes complex")
            .with_impact(
                "Without -f flag, rm fails if file missing, breaking \
                 cleanup operations and rollback procedures. Requires \
                 manual intervention to recover.",
            )
            .with_severity(SafetySeverity::High),

        title if title.contains("ln") && title.contains("-sf") => SafetyRationale::new()
            .add_failure("Symlink creation fails if link exists")
            .add_failure("Cannot update symlinks atomically")
            .add_failure("Deployment scripts break on re-run")
            .with_impact(
                "Without -sf flags, ln fails on existing symlinks, \
                 breaking blue-green deployments and atomic updates. \
                 Creates deployment downtime.",
            )
            .with_severity(SafetySeverity::High),

        _ => SafetyRationale::new()
            .add_failure("Operation not safe to re-run")
            .with_impact("May fail on subsequent executions")
            .with_severity(SafetySeverity::Medium),
    }
}

/// Generate safety rationale for determinism transformations
pub fn generate_determinism_rationale(transformation_title: &str) -> SafetyRationale {
    match transformation_title {
        title if title.contains("RANDOM") => SafetyRationale::new()
            .add_vulnerability("Non-reproducible builds break security audits")
            .add_vulnerability("Cannot verify script behavior in production")
            .add_failure("Debugging impossible with non-deterministic values")
            .add_failure("Testing cannot catch all edge cases")
            .with_impact(
                "$RANDOM creates unpredictable script behavior, breaking \
                 reproducible builds, security audits, and compliance checks. \
                 Makes debugging production issues nearly impossible.",
            )
            .with_severity(SafetySeverity::Critical),

        title if title.contains("timestamp") || title.contains("date") => SafetyRationale::new()
            .add_vulnerability("Time-based values break reproducibility")
            .add_vulnerability("Cannot verify script output")
            .add_failure("Testing across time zones fails")
            .add_failure("Replay attacks become possible")
            .with_impact(
                "Timestamps make scripts non-reproducible, breaking security \
                 verification and compliance. Creates race conditions in \
                 distributed systems.",
            )
            .with_severity(SafetySeverity::High),

        _ => SafetyRationale::new()
            .add_vulnerability("Non-deterministic behavior breaks verification")
            .with_impact("Cannot guarantee reproducible results")
            .with_severity(SafetySeverity::Medium),
    }
}

/// Generate safety rationale for safety transformations
pub fn generate_safety_rationale(transformation_title: &str) -> SafetyRationale {
    match transformation_title {
        title if title.contains("quot") || title.contains("variable") => SafetyRationale::new()
            .add_vulnerability("Command injection via unquoted variables")
            .add_vulnerability("Path traversal attacks")
            .add_attack_vector("Inject shell metacharacters into variables")
            .add_attack_vector("Word splitting allows arbitrary command execution")
            .add_failure("Filename with spaces breaks script")
            .add_failure("Glob expansion creates unexpected behavior")
            .with_impact(
                "Unquoted variables allow CRITICAL command injection attacks. \
                 Attacker can execute arbitrary commands by controlling \
                 variable content. Enables privilege escalation and data theft.",
            )
            .with_severity(SafetySeverity::Critical),

        _ => SafetyRationale::new()
            .add_vulnerability("Potential security issue")
            .with_impact("May create security or safety problem")
            .with_severity(SafetySeverity::Medium),
    }
}

/// Format safety rationale for display
pub fn format_safety_rationale(rationale: &SafetyRationale) -> String {
    let mut output = String::new();

    // Severity
    let severity_symbol = match rationale.severity {
        SafetySeverity::Critical => "🔴 CRITICAL",
        SafetySeverity::High => "🟠 HIGH",
        SafetySeverity::Medium => "🟡 MEDIUM",
        SafetySeverity::Low => "🟢 LOW",
    };
    output.push_str(&format!("Severity: {}\n\n", severity_symbol));

    // Vulnerabilities prevented
    if !rationale.vulnerabilities_prevented.is_empty() {
        output.push_str("Vulnerabilities Prevented:\n");
        for vuln in &rationale.vulnerabilities_prevented {
            output.push_str(&format!("  • {}\n", vuln));
        }
        output.push('\n');
    }

    // Failures eliminated
    if !rationale.failures_eliminated.is_empty() {
        output.push_str("Failures Eliminated:\n");
        for failure in &rationale.failures_eliminated {
            output.push_str(&format!("  • {}\n", failure));
        }
        output.push('\n');
    }

    // Attack vectors closed
    if !rationale.attack_vectors_closed.is_empty() {
        output.push_str("Attack Vectors Closed:\n");
        for vector in &rationale.attack_vectors_closed {
            output.push_str(&format!("  • {}\n", vector));
        }
        output.push('\n');
    }

    // Impact
    if !rationale.impact_without_fix.is_empty() {
        output.push_str("Impact Without Fix:\n");
        output.push_str(&format!("  {}\n", rationale.impact_without_fix));
    }

    output
}

// ===== REPL-013-003: ALTERNATIVE SUGGESTION GENERATION FUNCTIONS (RED PHASE STUBS) =====

/// Generate alternatives for idempotency transformations
pub fn generate_idempotency_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("mkdir") && title.contains("-p") => vec![
            Alternative::new(
                "Check before creating",
                "[ -d /path ] || mkdir /path",
                "When you want explicit control over error handling",
            )
            .add_pro("Explicit about what's happening")
            .add_pro("Can add custom logic between check and creation")
            .add_con("Not atomic - race condition between check and create")
            .add_con("More verbose than mkdir -p"),
            Alternative::new(
                "Use mkdir with error suppression",
                "mkdir /path 2>/dev/null || true",
                "When you don't care about the reason for failure",
            )
            .add_pro("Simple and concise")
            .add_pro("Idempotent")
            .add_con("Hides all errors, not just 'already exists'")
            .add_con("Can mask real problems like permission issues"),
        ],

        title if title.contains("rm") && title.contains("-f") => vec![
            Alternative::new(
                "Check before removing",
                "[ -e /path ] && rm /path",
                "When you want to know if the file existed",
            )
            .add_pro("Explicit about file existence")
            .add_pro("Can add logging or side effects")
            .add_con("Not atomic - race condition")
            .add_con("More verbose"),
            Alternative::new(
                "Use rm with error check",
                "rm /path 2>/dev/null || true",
                "When you want to suppress errors but keep other rm behavior",
            )
            .add_pro("Simple")
            .add_pro("Idempotent")
            .add_con("Hides all errors")
            .add_con("May mask permission problems"),
        ],

        title if title.contains("ln") && title.contains("-sf") => vec![
            Alternative::new(
                "Remove then create",
                "rm -f /link && ln -s /target /link",
                "When you need two separate operations",
            )
            .add_pro("Very explicit")
            .add_pro("Can add logic between removal and creation")
            .add_con("Not atomic - window where link doesn't exist")
            .add_con("More verbose"),
            Alternative::new(
                "Check before creating",
                "[ -L /link ] || ln -s /target /link",
                "When you want to preserve existing links",
            )
            .add_pro("Won't overwrite existing links")
            .add_pro("Explicit check")
            .add_con("Not idempotent if link points elsewhere")
            .add_con("Race condition between check and create"),
        ],

        _ => vec![Alternative::new(
            "Add explicit idempotency check",
            "[ condition ] || operation",
            "When you want fine-grained control",
        )
        .add_pro("Explicit about preconditions")
        .add_con("Not atomic")],
    }
}

/// Generate alternatives for determinism transformations
pub fn generate_determinism_alternatives(transformation_title: &str) -> Vec<Alternative> {
    match transformation_title {
        title if title.contains("RANDOM") => vec![
            Alternative::new(
                "Use UUID for unique IDs",
                "id=$(uuidgen)  # or $(cat /proc/sys/kernel/random/uuid)",
                "When you need globally unique identifiers",
            )
            .add_pro("Guaranteed unique across machines")
            .add_pro("Standard format")
            .add_pro("Deterministic if you control the seed")
            .add_con("Requires uuidgen or /proc/sys/kernel")
            .add_con("Longer than simple numbers"),
            Alternative::new(
                "Use timestamp-based IDs",
                "id=$(date +%s%N)  # nanoseconds since epoch",
                "When you need time-ordered IDs",
            )
            .add_pro("Sortable by time")
            .add_pro("No external dependencies")
            .add_con("Not unique across machines")
            .add_con("Still non-deterministic (but reproducible with fixed time)"),
            Alternative::new(
                "Use hash of inputs",
                "id=$(echo \"$input\" | sha256sum | cut -d' ' -f1)",
                "When you want IDs derived from content",
            )
            .add_pro("Truly deterministic")
            .add_pro("Same input = same ID")
            .add_con("Requires sha256sum")
            .add_con("Collisions possible (though extremely rare)"),
            Alternative::new(
                "Use sequential counter",
                "id=$((++counter))  # with state file",
                "When you need simple incrementing IDs",
            )
            .add_pro("Simple and predictable")
            .add_pro("Compact")
            .add_con("Requires state management")
            .add_con("Not unique across processes without locking"),
        ],

        title if title.contains("timestamp") || title.contains("date") => vec![
            Alternative::new(
                "Use explicit version parameter",
                "version=$1  # Pass version as argument",
                "When version is known at invocation time",
            )
            .add_pro("Fully deterministic")
            .add_pro("Version controlled externally")
            .add_con("Requires coordination with caller"),
            Alternative::new(
                "Use git commit hash",
                "version=$(git rev-parse --short HEAD)",
                "When deploying from git repository",
            )
            .add_pro("Deterministic for given commit")
            .add_pro("Traceable to source code")
            .add_con("Requires git repository")
            .add_con("Not available in all environments"),
            Alternative::new(
                "Use build number from CI",
                "version=${BUILD_NUMBER:-dev}",
                "When running in CI/CD pipeline",
            )
            .add_pro("Integrates with CI/CD")
            .add_pro("Incrementing version numbers")
            .add_con("Requires CI environment")
            .add_con("May not be available locally"),
        ],

        _ => vec![Alternative::new(
            "Make value an input parameter",
            "value=$1  # Pass as argument",
            "When value should be controlled by caller",
        )
        .add_pro("Fully deterministic")
        .add_con("Requires caller to provide value")],
    }
}

// Re-export functions moved to purifier_transforms_gen.rs
pub use super::purifier_transforms_gen::{
    explain_purification_changes_detailed, format_alternatives, format_transformation_report,
    generate_safety_alternatives,
};

#[cfg(test)]
#[path = "purifier_transforms_tests_inline.rs"]
mod tests_inline;
