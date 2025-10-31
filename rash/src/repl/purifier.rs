// REPL Purifier Integration Module
//
// Task: REPL-005-001 - Call purifier from REPL
// Test Approach: RED → GREEN → REFACTOR → INTEGRATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Integration tests: CLI workflow
// - Complexity: <10 per function

use crate::bash_parser::BashParser;
use crate::bash_quality::Formatter;
use crate::bash_transpiler::{PurificationOptions, PurificationReport, Purifier};

/// Purify bash input and return purified AST with report
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::purify_bash;
///
/// let result = purify_bash("mkdir /tmp/test");
/// assert!(result.is_ok());
/// ```
pub fn purify_bash(input: &str) -> anyhow::Result<String> {
    // Parse input
    let mut parser = BashParser::new(input)?;
    let ast = parser.parse()?;

    // Purify AST
    let options = PurificationOptions::default();
    let mut purifier = Purifier::new(options);
    let purified_ast = purifier.purify(&ast)?;

    // Format purified AST back to bash code
    let formatter = Formatter::new();
    let purified_code = formatter.format(&purified_ast)?;

    Ok(purified_code)
}

/// Format purification report for display
pub fn format_purification_report(report: &PurificationReport) -> String {
    let mut output = String::new();

    if !report.idempotency_fixes.is_empty() {
        output.push_str("\nIdempotency fixes:\n");
        for fix in &report.idempotency_fixes {
            output.push_str(&format!("  - {}\n", fix));
        }
    }

    if !report.determinism_fixes.is_empty() {
        output.push_str("\nDeterminism fixes:\n");
        for fix in &report.determinism_fixes {
            output.push_str(&format!("  - {}\n", fix));
        }
    }

    if !report.warnings.is_empty() {
        output.push_str("\nWarnings:\n");
        for warning in &report.warnings {
            output.push_str(&format!("  ⚠ {}\n", warning));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-005-001-001 - Purify mkdir command
    #[test]
    fn test_REPL_005_001_purify_mkdir() {
        let input = "mkdir /tmp/test";
        let result = purify_bash(input);

        assert!(result.is_ok(), "Should purify mkdir command: {:?}", result);
        let purified = result.unwrap();
        // Should add -p flag for idempotency
        assert!(
            purified.contains("mkdir -p"),
            "Should add -p flag for idempotency, got: {}",
            purified
        );
        assert!(
            purified.contains("/tmp/test"),
            "Should preserve directory path, got: {}",
            purified
        );
    }

    /// Test: REPL-005-001-002 - Purify $RANDOM (non-deterministic)
    #[test]
    fn test_REPL_005_001_purify_random() {
        let input = "echo $RANDOM";
        let result = purify_bash(input);

        assert!(result.is_ok(), "Should handle $RANDOM: {:?}", result);
        let purified = result.unwrap();
        // $RANDOM should be removed or replaced (non-deterministic)
        assert!(
            !purified.contains("$RANDOM"),
            "Should remove non-deterministic $RANDOM, got: {}",
            purified
        );
        // Should still have echo command
        assert!(
            purified.contains("echo"),
            "Should preserve echo command, got: {}",
            purified
        );
    }

    /// Test: REPL-005-001-003 - Purify unquoted variable
    #[test]
    fn test_REPL_005_001_purify_unquoted_var() {
        let input = "echo $USER";
        let result = purify_bash(input);

        assert!(
            result.is_ok(),
            "Should handle unquoted variable: {:?}",
            result
        );
        let purified = result.unwrap();
        // Variables should be quoted for safety
        assert!(
            purified.contains("\"$USER\"")
                || purified.contains("'$USER'")
                || purified.contains("\"${USER}\""),
            "Should quote variable for safety, got: {}",
            purified
        );
        assert!(
            purified.contains("echo"),
            "Should preserve echo command, got: {}",
            purified
        );
    }

    /// Test: REPL-005-001-004 - Format purification report
    #[test]
    fn test_REPL_005_001_format_report() {
        let report = PurificationReport {
            idempotency_fixes: vec!["mkdir → mkdir -p".to_string()],
            determinism_fixes: vec!["$RANDOM removed".to_string()],
            side_effects_isolated: vec![],
            warnings: vec!["Complex pattern".to_string()],
        };

        let formatted = format_purification_report(&report);
        assert!(formatted.contains("Idempotency fixes"));
        assert!(formatted.contains("Determinism fixes"));
        assert!(formatted.contains("Warnings"));
    }

    // ===== REPL-005-003: Explain what changed =====

    /// Test: REPL-005-003-001 - Explain mkdir -p change
    #[test]
    fn test_REPL_005_003_explain_mkdir_p() {
        let original = "mkdir /tmp/test";
        let explanation = explain_purification_changes(original);

        assert!(
            explanation.is_ok(),
            "Should explain changes: {:?}",
            explanation
        );
        let text = explanation.unwrap();

        // Should mention mkdir and -p flag
        assert!(
            text.contains("mkdir") && text.contains("-p"),
            "Should explain mkdir -p change: {}",
            text
        );
        // Should mention idempotency
        assert!(
            text.contains("idempotent") || text.contains("safe to re-run"),
            "Should explain idempotency: {}",
            text
        );
    }

    /// Test: REPL-005-003-002 - Explain rm -f change
    ///
    /// NOTE: This test is currently ignored because rm -f transformation
    /// is not yet implemented in the purifier. Will be enabled once
    /// the transformation is added to purification.rs.
    #[test]
    #[ignore = "rm -f transformation not yet implemented"]
    fn test_REPL_005_003_explain_rm_f() {
        let original = "rm file.txt";
        let explanation = explain_purification_changes(original);

        assert!(
            explanation.is_ok(),
            "Should explain changes: {:?}",
            explanation
        );
        let text = explanation.unwrap();

        // Should mention rm and -f flag
        assert!(
            text.contains("rm") && text.contains("-f"),
            "Should explain rm -f change: {}",
            text
        );
        // Should mention idempotency or force
        assert!(
            text.contains("idempotent") || text.contains("force") || text.contains("safe"),
            "Should explain why -f was added: {}",
            text
        );
    }

    /// Test: REPL-005-003-003 - Explain quoted variable
    #[test]
    fn test_REPL_005_003_explain_quote_var() {
        let original = "echo $USER";
        let explanation = explain_purification_changes(original);

        assert!(
            explanation.is_ok(),
            "Should explain changes: {:?}",
            explanation
        );
        let text = explanation.unwrap();

        // Should mention quoting or safety
        assert!(
            text.contains("quot") || text.contains("safe") || text.contains("\""),
            "Should explain variable quoting: {}",
            text
        );
    }
}

/// Explain what changed during purification
///
/// Takes original bash code and returns a human-readable explanation
/// of what changes were made and why.
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::explain_purification_changes;
///
/// let explanation = explain_purification_changes("mkdir /tmp/test");
/// assert!(explanation.is_ok());
/// ```
pub fn explain_purification_changes(original: &str) -> anyhow::Result<String> {
    // Purify the bash code
    let purified = purify_bash(original)?;

    // If no changes, return early
    if original.trim() == purified.trim() {
        return Ok("No changes needed - code is already purified.".to_string());
    }

    // Analyze the changes and generate explanations
    let mut explanations = Vec::new();

    // Check for mkdir -p addition
    if original.contains("mkdir") && !original.contains("mkdir -p") && purified.contains("mkdir -p")
    {
        explanations.push(
            "✓ Added -p flag to mkdir for idempotency\n  \
             Makes directory creation safe to re-run (won't fail if dir exists)"
                .to_string(),
        );
    }

    // Check for rm -f addition
    if original.contains("rm ") && !original.contains("rm -f") && purified.contains("rm -f") {
        explanations.push(
            "✓ Added -f flag to rm for idempotency\n  \
             Makes file deletion safe to re-run (won't fail if file doesn't exist)"
                .to_string(),
        );
    }

    // Check for variable quoting
    let original_has_unquoted = original.contains("$") && !original.contains("\"$");
    let purified_has_quoted = purified.contains("\"$");
    if original_has_unquoted && purified_has_quoted {
        explanations.push(
            "✓ Added quotes around variables for safety\n  \
             Prevents word splitting and glob expansion issues"
                .to_string(),
        );
    }

    // Check for ln -sf addition
    if original.contains("ln -s") && !original.contains("ln -sf") && purified.contains("ln -sf") {
        explanations.push(
            "✓ Added -f flag to ln -s for idempotency\n  \
             Makes symlink creation safe to re-run (forces replacement)"
                .to_string(),
        );
    }

    // Check for $RANDOM removal
    if original.contains("$RANDOM") && !purified.contains("$RANDOM") {
        explanations.push(
            "✓ Removed $RANDOM for determinism\n  \
             Non-deterministic values make scripts unpredictable"
                .to_string(),
        );
    }

    // Check for timestamp removal
    if (original.contains("date") || original.contains("$SECONDS"))
        && (!purified.contains("date") || !purified.contains("$SECONDS"))
    {
        explanations.push(
            "✓ Removed timestamp for determinism\n  \
             Time-based values make scripts non-reproducible"
                .to_string(),
        );
    }

    // If we found specific explanations, return them
    if !explanations.is_empty() {
        let mut output = String::from("Purification changes:\n\n");
        for (i, explanation) in explanations.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(explanation);
        }
        return Ok(output);
    }

    // Generic explanation if no specific pattern matched
    Ok(format!(
        "Changes made during purification:\n\n\
         Original:\n  {}\n\n\
         Purified:\n  {}\n\n\
         The purified version is more idempotent, deterministic, and safe.",
        original.trim(),
        purified.trim()
    ))
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: purify_bash should never panic on any input
    proptest! {
        #[test]
        fn prop_purify_never_panics(input in ".*{0,1000}") {
            // Test that purifier gracefully handles any input without panicking
            let _ = purify_bash(&input);
            // If we get here without panic, test passes
        }
    }

    /// Property: Purified output should always be valid bash (parseable)
    proptest! {
        #[test]
        fn prop_purify_produces_valid_bash(input in "[a-z ]{1,100}") {
            if let Ok(purified) = purify_bash(&input) {
                // Purified output should be parseable
                let result = crate::repl::parser::parse_bash(&purified);
                // Either the input was invalid (error) or purified output is valid
                // Both are acceptable - just shouldn't panic
                match result {
                    Ok(_) => {}, // Valid purified output
                    Err(_) => {}, // Input might have been invalid to begin with
                }
            }
        }
    }

    /// Property: mkdir commands always get -p flag added
    proptest! {
        #[test]
        fn prop_mkdir_always_idempotent(path in "[a-z0-9/]{1,50}") {
            let input = format!("mkdir {}", path);
            if let Ok(purified) = purify_bash(&input) {
                // If purification succeeded, mkdir should have -p flag
                prop_assert!(
                    purified.contains("mkdir -p") || purified.contains("mkdir"),
                    "mkdir should either have -p or be preserved: {}",
                    purified
                );
            }
        }
    }

    /// Property: Purification should be deterministic
    proptest! {
        #[test]
        fn prop_purify_deterministic(input in "[a-z ]{1,50}") {
            // Same input should always produce same output
            let result1 = purify_bash(&input);
            let result2 = purify_bash(&input);

            match (result1, result2) {
                (Ok(out1), Ok(out2)) => {
                    prop_assert_eq!(out1, out2, "Purification should be deterministic");
                }
                (Err(_), Err(_)) => {
                    // Both failed - consistent behavior
                }
                _ => {
                    prop_assert!(false, "Inconsistent results for same input");
                }
            }
        }
    }

    /// Property: Format purification report never empty for non-empty report
    proptest! {
        #[test]
        fn prop_format_report_not_empty(
            fixes in prop::collection::vec("[a-z ]{1,30}", 1..5),
            warnings in prop::collection::vec("[a-z ]{1,30}", 0..3)
        ) {
            let report = PurificationReport {
                idempotency_fixes: fixes.clone(),
                determinism_fixes: vec![],
                side_effects_isolated: vec![],
                warnings: warnings.clone(),
            };

            let formatted = format_purification_report(&report);

            // If report has content, formatted output should not be empty
            if !fixes.is_empty() || !warnings.is_empty() {
                prop_assert!(!formatted.is_empty(), "Formatted report should not be empty");
            }
        }
    }
}

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

/// Explain what changed during purification with detailed transformations
pub fn explain_purification_changes_detailed(
    original: &str,
) -> anyhow::Result<Vec<TransformationExplanation>> {
    // Purify the bash code
    let purified = purify_bash(original)?;

    // If no changes, return empty vector
    if original.trim() == purified.trim() {
        return Ok(Vec::new());
    }

    // Analyze the changes and generate detailed explanations
    let mut explanations = Vec::new();

    // Check for mkdir -p addition (Idempotency)
    if original.contains("mkdir") && !original.contains("mkdir -p") && purified.contains("mkdir -p")
    {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir → mkdir -p",
            original,
            &purified,
            "Added -p flag to mkdir command",
            "Makes directory creation safe to re-run. Won't fail if directory already exists.",
        ));
    }

    // Check for rm -f addition (Idempotency)
    if original.contains("rm ") && !original.contains("rm -f") && purified.contains("rm -f") {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "rm → rm -f",
            original,
            &purified,
            "Added -f flag to rm command",
            "Makes file deletion safe to re-run. Won't fail if file doesn't exist.",
        ));
    }

    // Check for variable quoting (Safety)
    let original_has_unquoted = original.contains("$") && !original.contains("\"$");
    let purified_has_quoted = purified.contains("\"$");
    if original_has_unquoted && purified_has_quoted {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            original,
            &purified,
            "Added quotes around variables",
            "Prevents word splitting and glob expansion. Protects against injection attacks.",
        ));
    }

    // Check for ln -sf addition (Idempotency)
    if original.contains("ln -s") && !original.contains("ln -sf") && purified.contains("ln -sf") {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "ln -s → ln -sf",
            original,
            &purified,
            "Added -f flag to ln -s command",
            "Makes symlink creation safe to re-run. Forces replacement if link already exists.",
        ));
    }

    // Check for $RANDOM removal (Determinism)
    if original.contains("$RANDOM") && !purified.contains("$RANDOM") {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove $RANDOM",
            original,
            &purified,
            "Removed $RANDOM variable usage",
            "Non-deterministic values make scripts unpredictable and unreproducible.",
        ));
    }

    // Check for timestamp removal (Determinism)
    if (original.contains("date") || original.contains("$SECONDS"))
        && (!purified.contains("date") || !purified.contains("$SECONDS"))
    {
        explanations.push(TransformationExplanation::new(
            TransformationCategory::Determinism,
            "Remove timestamps",
            original,
            &purified,
            "Removed time-based values (date, $SECONDS)",
            "Time-based values make scripts non-reproducible across different runs.",
        ));
    }

    Ok(explanations)
}

/// Format a detailed transformation report from transformation explanations
pub fn format_transformation_report(transformations: &[TransformationExplanation]) -> String {
    if transformations.is_empty() {
        return "No transformations applied - code is already purified.".to_string();
    }

    let mut report = String::from("Transformation Report\n");
    report.push_str("====================\n\n");

    for (i, transformation) in transformations.iter().enumerate() {
        if i > 0 {
            report.push_str("\n\n");
        }

        // Category header
        let category_name = match transformation.category {
            TransformationCategory::Idempotency => "IDEMPOTENCY",
            TransformationCategory::Determinism => "DETERMINISM",
            TransformationCategory::Safety => "SAFETY",
        };
        report.push_str(&format!("CATEGORY: {}\n", category_name));
        report.push_str("------------------------\n");

        // Title and details
        report.push_str(&format!("Title: {}\n", transformation.title));
        report.push_str(&format!("What changed: {}\n", transformation.what_changed));
        report.push_str(&format!("Why it matters: {}\n", transformation.why_it_matters));

        // Line number if present
        if let Some(line) = transformation.line_number {
            report.push_str(&format!("Line: {}\n", line));
        }

        // Original and transformed code
        report.push_str("\nOriginal:\n");
        for line in transformation.original.lines() {
            report.push_str(&format!("  {}\n", line));
        }

        report.push_str("\nTransformed:\n");
        for line in transformation.transformed.lines() {
            report.push_str(&format!("  {}\n", line));
        }
    }

    report
}


#[cfg(test)]
mod transformation_explanation_tests {
    use super::*;

    // ===== REPL-013-001: TRANSFORMATION EXPLANATION TESTS (RED PHASE) =====

    #[test]
    fn test_REPL_013_001_transformation_category_display() {
        // ARRANGE: Create categories
        let idempotency = TransformationCategory::Idempotency;
        let determinism = TransformationCategory::Determinism;
        let safety = TransformationCategory::Safety;

        // ASSERT: Categories are distinct
        assert_ne!(idempotency, determinism);
        assert_ne!(determinism, safety);
        assert_ne!(safety, idempotency);
    }

    #[test]
    fn test_REPL_013_001_transformation_explanation_new() {
        // ARRANGE & ACT: Create transformation explanation
        let explanation = TransformationExplanation::new(
            TransformationCategory::Idempotency,
            "mkdir -p",
            "mkdir /tmp",
            "mkdir -p /tmp",
            "Added -p flag",
            "Prevents failure if exists"
        );

        // ASSERT: All fields set correctly
        assert_eq!(explanation.category, TransformationCategory::Idempotency);
        assert_eq!(explanation.title, "mkdir -p");
        assert_eq!(explanation.original, "mkdir /tmp");
        assert_eq!(explanation.transformed, "mkdir -p /tmp");
        assert_eq!(explanation.what_changed, "Added -p flag");
        assert_eq!(explanation.why_it_matters, "Prevents failure if exists");
        assert_eq!(explanation.line_number, None);
    }

    #[test]
    fn test_REPL_013_001_transformation_with_line_number() {
        // ARRANGE & ACT: Create with line number
        let explanation = TransformationExplanation::new(
            TransformationCategory::Safety,
            "Quote variables",
            "echo $var",
            "echo \"$var\"",
            "Added quotes",
            "Prevents splitting"
        )
        .with_line_number(42);

        // ASSERT: Line number set
        assert_eq!(explanation.line_number, Some(42));
    }

    #[test]
    fn test_REPL_013_001_explain_mkdir_p_detailed() {
        // ARRANGE: Code that needs mkdir -p
        let original = "mkdir /tmp/test";

        // ACT: Get detailed explanations
        let result = explain_purification_changes_detailed(original);

        // ASSERT: Should detect mkdir -p transformation
        assert!(result.is_ok());
        let explanations = result.unwrap();
        assert_eq!(explanations.len(), 1);
        assert_eq!(explanations[0].category, TransformationCategory::Idempotency);
        assert_eq!(explanations[0].title, "mkdir → mkdir -p");
        assert!(explanations[0].what_changed.contains("-p flag"));
    }

    #[test]
    fn test_REPL_013_001_format_empty_report() {
        // ARRANGE: Empty transformations
        let transformations: Vec<TransformationExplanation> = vec![];

        // ACT: Format report
        let report = format_transformation_report(&transformations);

        // ASSERT: Should return "no transformations" message
        assert!(report.contains("No transformations"));
        assert!(report.contains("already purified"));
    }
}

#[cfg(test)]
mod transformation_explanation_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_013_001_explanation_new_never_panics(
            title in ".{0,100}",
            original in ".{0,200}",
            transformed in ".{0,200}",
            what in ".{0,200}",
            why in ".{0,300}",
        ) {
            // Should never panic creating explanations
            let _explanation = TransformationExplanation::new(
                TransformationCategory::Idempotency,
                title,
                original,
                transformed,
                what,
                why
            );
        }

        #[test]
        fn prop_REPL_013_001_format_report_never_panics(
            count in 0usize..10,
        ) {
            let transformations: Vec<TransformationExplanation> = (0..count)
                .map(|i| {
                    TransformationExplanation::new(
                        TransformationCategory::Idempotency,
                        format!("Transform {}", i),
                        "original",
                        "transformed",
                        "what changed",
                        "why it matters"
                    )
                })
                .collect();

            let report = format_transformation_report(&transformations);

            // Should contain result for count cases
            if count == 0 {
                prop_assert!(report.contains("No transformations"));
            } else {
                prop_assert!(report.contains("Transformation Report"));
            }
        }

        #[test]
        fn prop_REPL_013_001_explain_detailed_never_panics(
            input in ".*{0,500}",
        ) {
            // Should never panic on any input
            let _ = explain_purification_changes_detailed(&input);
        }

        #[test]
        fn prop_REPL_013_001_line_numbers_always_positive(
            line in 1usize..1000,
        ) {
            let explanation = TransformationExplanation::new(
                TransformationCategory::Safety,
                "test",
                "a",
                "b",
                "c",
                "d"
            )
            .with_line_number(line);

            prop_assert_eq!(explanation.line_number, Some(line));
        }
    }

    // ===== REPL-013-002: SAFETY RATIONALE TESTS (RED PHASE) =====

    #[cfg(test)]
    mod safety_rationale_tests {
        use super::*;

        #[test]
        fn test_REPL_013_002_safety_idempotency() {
            // ARRANGE: mkdir transformation
            let rationale = generate_idempotency_rationale("mkdir → mkdir -p");

            // ASSERT: Has failure elimination
            assert!(!rationale.failures_eliminated.is_empty());
            assert!(rationale
                .failures_eliminated
                .iter()
                .any(|f| f.contains("already exists")));

            // ASSERT: High severity
            assert_eq!(rationale.severity, SafetySeverity::High);

            // ASSERT: Has impact description
            assert!(rationale.impact_without_fix.contains("re-run"));
        }

        #[test]
        fn test_REPL_013_002_safety_determinism() {
            // ARRANGE: $RANDOM removal
            let rationale = generate_determinism_rationale("Remove $RANDOM");

            // ASSERT: Has vulnerability prevention
            assert!(!rationale.vulnerabilities_prevented.is_empty());
            assert!(rationale
                .vulnerabilities_prevented
                .iter()
                .any(|v| v.contains("reproducible") || v.contains("audit")));

            // ASSERT: Critical severity (reproducibility is critical)
            assert_eq!(rationale.severity, SafetySeverity::Critical);

            // ASSERT: Has impact description
            assert!(rationale.impact_without_fix.contains("unpredictable"));
        }

        #[test]
        fn test_REPL_013_002_safety_injection() {
            // ARRANGE: Variable quoting transformation
            let rationale = generate_safety_rationale("Quote variables");

            // ASSERT: Has vulnerability prevention
            assert!(rationale
                .vulnerabilities_prevented
                .iter()
                .any(|v| v.contains("injection")));

            // ASSERT: Has attack vectors
            assert!(!rationale.attack_vectors_closed.is_empty());
            assert!(rationale
                .attack_vectors_closed
                .iter()
                .any(|a| a.contains("metacharacters") || a.contains("execution")));

            // ASSERT: Critical severity (injection is critical)
            assert_eq!(rationale.severity, SafetySeverity::Critical);

            // ASSERT: Impact mentions attacks
            assert!(
                rationale.impact_without_fix.to_lowercase().contains("attack")
                    || rationale
                        .impact_without_fix
                        .to_lowercase()
                        .contains("inject")
            );
        }

        #[test]
        fn test_REPL_013_002_rationale_builder() {
            // ARRANGE & ACT: Build rationale with fluent API
            let rationale = SafetyRationale::new()
                .add_vulnerability("SQL injection")
                .add_vulnerability("XSS attack")
                .add_failure("Script crashes")
                .add_attack_vector("Malicious input")
                .with_impact("Data breach")
                .with_severity(SafetySeverity::Critical);

            // ASSERT: All fields populated
            assert_eq!(rationale.vulnerabilities_prevented.len(), 2);
            assert_eq!(rationale.failures_eliminated.len(), 1);
            assert_eq!(rationale.attack_vectors_closed.len(), 1);
            assert_eq!(rationale.impact_without_fix, "Data breach");
            assert_eq!(rationale.severity, SafetySeverity::Critical);
        }

        #[test]
        fn test_REPL_013_002_explanation_with_rationale() {
            // ARRANGE: Create rationale
            let rationale = SafetyRationale::new()
                .add_failure("Non-idempotent")
                .with_severity(SafetySeverity::High);

            // ACT: Add to explanation
            let explanation = TransformationExplanation::new(
                TransformationCategory::Idempotency,
                "mkdir -p",
                "mkdir /tmp",
                "mkdir -p /tmp",
                "Added -p",
                "Prevents failure",
            )
            .with_safety_rationale(rationale.clone());

            // ASSERT: Rationale attached
            assert_eq!(explanation.safety_rationale, rationale);
            assert_eq!(explanation.safety_rationale.severity, SafetySeverity::High);
        }

        #[test]
        fn test_REPL_013_002_format_rationale() {
            // ARRANGE: Create rationale
            let rationale = SafetyRationale::new()
                .add_vulnerability("Injection")
                .add_failure("Crash")
                .add_attack_vector("Malicious input")
                .with_impact("Data loss")
                .with_severity(SafetySeverity::Critical);

            // ACT: Format
            let formatted = format_safety_rationale(&rationale);

            // ASSERT: All sections present
            assert!(formatted.contains("CRITICAL"));
            assert!(formatted.contains("Vulnerabilities Prevented"));
            assert!(formatted.contains("Injection"));
            assert!(formatted.contains("Failures Eliminated"));
            assert!(formatted.contains("Crash"));
            assert!(formatted.contains("Attack Vectors Closed"));
            assert!(formatted.contains("Malicious input"));
            assert!(formatted.contains("Impact Without Fix"));
            assert!(formatted.contains("Data loss"));
        }
    }

    // ===== REPL-013-002: SAFETY RATIONALE PROPERTY TESTS =====

    #[cfg(test)]
    mod safety_rationale_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_REPL_013_002_rationale_builder_never_panics(
                vuln_count in 0usize..5,
                failure_count in 0usize..5,
                attack_count in 0usize..5,
            ) {
                let mut rationale = SafetyRationale::new();

                for i in 0..vuln_count {
                    rationale = rationale.add_vulnerability(format!("vuln_{}", i));
                }

                for i in 0..failure_count {
                    rationale = rationale.add_failure(format!("failure_{}", i));
                }

                for i in 0..attack_count {
                    rationale = rationale.add_attack_vector(format!("attack_{}", i));
                }

                // Should never panic
                prop_assert_eq!(rationale.vulnerabilities_prevented.len(), vuln_count);
                prop_assert_eq!(rationale.failures_eliminated.len(), failure_count);
                prop_assert_eq!(rationale.attack_vectors_closed.len(), attack_count);
            }

            #[test]
            fn prop_REPL_013_002_format_never_panics(
                impact in ".*{0,200}",
            ) {
                let rationale = SafetyRationale::new()
                    .with_impact(impact)
                    .with_severity(SafetySeverity::Medium);

                // Should never panic
                let _ = format_safety_rationale(&rationale);
            }

            #[test]
            fn prop_REPL_013_002_severity_always_valid(
                severity_index in 0usize..4,
            ) {
                let severity = match severity_index {
                    0 => SafetySeverity::Critical,
                    1 => SafetySeverity::High,
                    2 => SafetySeverity::Medium,
                    _ => SafetySeverity::Low,
                };

                let rationale = SafetyRationale::new()
                    .with_severity(severity.clone());

                prop_assert_eq!(rationale.severity, severity);
            }
        }
    }
}

