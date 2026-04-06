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
use crate::linter::{lint_shell, Diagnostic, LintResult};

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

// ===== REPL-014-001: Auto-run bashrs linter on purified output =====

/// Result of purifying and linting bash code
///
/// Combines purification with linting to ensure purified output
/// meets bashrs quality standards (no DET/IDEM/SEC violations).
#[derive(Debug, Clone)]
pub struct PurifiedLintResult {
    /// The purified bash code
    pub purified_code: String,

    /// Lint results for the purified code
    pub lint_result: LintResult,

    /// True if purified code has no critical violations (DET/IDEM/SEC)
    pub is_clean: bool,
}

impl PurifiedLintResult {
    pub fn new(purified_code: String, lint_result: LintResult) -> Self {
        let is_clean = Self::check_is_clean(&lint_result);
        Self {
            purified_code,
            lint_result,
            is_clean,
        }
    }

    fn check_is_clean(lint_result: &LintResult) -> bool {
        // Check for critical violations: DET*, IDEM*, SEC*
        !lint_result.diagnostics.iter().any(|d| {
            d.code.starts_with("DET") || d.code.starts_with("IDEM") || d.code.starts_with("SEC")
        })
    }

    /// Get count of critical violations (DET/IDEM/SEC)
    pub fn critical_violations(&self) -> usize {
        self.lint_result
            .diagnostics
            .iter()
            .filter(|d| {
                d.code.starts_with("DET") || d.code.starts_with("IDEM") || d.code.starts_with("SEC")
            })
            .count()
    }

    /// Get DET violations only
    pub fn det_violations(&self) -> Vec<&Diagnostic> {
        self.lint_result
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with("DET"))
            .collect()
    }

    /// Get IDEM violations only
    pub fn idem_violations(&self) -> Vec<&Diagnostic> {
        self.lint_result
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with("IDEM"))
            .collect()
    }

    /// Get SEC violations only
    pub fn sec_violations(&self) -> Vec<&Diagnostic> {
        self.lint_result
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with("SEC"))
            .collect()
    }
}

/// Purify bash input and lint the purified output
///
/// This combines purification with linting to ensure the purified
/// output meets bashrs quality standards (no DET/IDEM/SEC violations).
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::purify_and_lint;
///
/// // Simple echo is passed through unchanged
/// let result = purify_and_lint("echo hello").unwrap();
/// assert!(result.purified_code.contains("echo"));
/// ```
pub fn purify_and_lint(input: &str) -> anyhow::Result<PurifiedLintResult> {
    // Step 1: Purify the input
    let purified_code = purify_bash(input)?;

    // Step 2: Lint the purified output
    let lint_result = lint_shell(&purified_code);

    // Step 3: Create result
    Ok(PurifiedLintResult::new(purified_code, lint_result))
}

/// Format purified lint result for display in REPL
pub fn format_purified_lint_result(result: &PurifiedLintResult) -> String {
    let mut output = String::new();

    // Show purified code
    output.push_str("Purified:\n");
    output.push_str(&result.purified_code);
    output.push_str("\n\n");

    // Show lint results
    if result.is_clean {
        output.push_str("✓ Purified output is CLEAN (no DET/IDEM/SEC violations)\n");
    } else {
        output.push_str(&format!(
            "✗ Purified output has {} critical violation(s)\n",
            result.critical_violations()
        ));

        if !result.det_violations().is_empty() {
            output.push_str(&format!("  DET: {}\n", result.det_violations().len()));
        }
        if !result.idem_violations().is_empty() {
            output.push_str(&format!("  IDEM: {}\n", result.idem_violations().len()));
        }
        if !result.sec_violations().is_empty() {
            output.push_str(&format!("  SEC: {}\n", result.sec_violations().len()));
        }
    }

    // Show full lint report
    if !result.lint_result.diagnostics.is_empty() {
        output.push_str("\nLint Report:\n");
        output.push_str(&crate::repl::linter::format_lint_results(
            &result.lint_result,
        ));
    }

    output
}

/// Format purified lint result with source code context (REPL-014-003)
///
/// This is an enhanced version of `format_purified_lint_result()` that shows
/// source code context for each violation.
///
/// # Examples
///
/// ```no_run
/// use bashrs::repl::purifier::{format_purified_lint_result_with_context, purify_and_lint};
///
/// let input = "echo $RANDOM";
/// let result = purify_and_lint(input).unwrap();
/// let formatted = format_purified_lint_result_with_context(&result, input);
/// ```
pub fn format_purified_lint_result_with_context(
    result: &PurifiedLintResult,
    _original_source: &str,
) -> String {
    let mut output = String::new();

    // Show purified code
    output.push_str("Purified:\n");
    output.push_str(&result.purified_code);
    output.push_str("\n\n");

    // Show lint results
    if result.is_clean {
        output.push_str("✓ Purified output is CLEAN (no DET/IDEM/SEC violations)\n");
    } else {
        output.push_str(&format!(
            "✗ Purified output has {} critical violation(s)\n",
            result.critical_violations()
        ));

        if !result.det_violations().is_empty() {
            output.push_str(&format!("  DET: {}\n", result.det_violations().len()));
        }
        if !result.idem_violations().is_empty() {
            output.push_str(&format!("  IDEM: {}\n", result.idem_violations().len()));
        }
        if !result.sec_violations().is_empty() {
            output.push_str(&format!("  SEC: {}\n", result.sec_violations().len()));
        }

        // Show violations with context
        output.push('\n');
        output.push_str(&crate::repl::linter::format_violations_with_context(
            &result.lint_result,
            &result.purified_code, // Show context from purified code
        ));
    }

    output
}

/// Error returned when purified output fails zero-tolerance quality gate
#[derive(Debug, Clone)]
pub struct PurificationError {
    /// The purified code (even though it has violations)
    pub purified_code: String,

    /// Count of DET violations
    pub det_violations: usize,

    /// Count of IDEM violations
    pub idem_violations: usize,

    /// Count of SEC violations
    pub sec_violations: usize,

    /// All diagnostics (for detailed reporting)
    pub diagnostics: Vec<Diagnostic>,
}

impl PurificationError {
    pub fn new(result: &PurifiedLintResult) -> Self {
        Self {
            purified_code: result.purified_code.clone(),
            det_violations: result.det_violations().len(),
            idem_violations: result.idem_violations().len(),
            sec_violations: result.sec_violations().len(),
            diagnostics: result.lint_result.diagnostics.clone(),
        }
    }

    pub fn total_violations(&self) -> usize {
        self.det_violations + self.idem_violations + self.sec_violations
    }
}

impl std::fmt::Display for PurificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Purified output failed zero-tolerance quality gate: {} violation(s) (DET: {}, IDEM: {}, SEC: {})",
            self.total_violations(),
            self.det_violations,
            self.idem_violations,
            self.sec_violations
        )
    }
}

impl std::error::Error for PurificationError {}

/// Purify bash input and enforce zero-tolerance quality gate
///
/// This function guarantees that returned purified code has ZERO critical
/// violations (DET/IDEM/SEC). If any critical violations exist, it returns
/// an error with detailed diagnostic information.
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::purify_and_validate;
///
/// // Clean input passes
/// let purified = purify_and_validate("echo hello").unwrap();
/// assert!(purified.contains("echo"));
/// ```
///
/// # Errors
///
/// Returns `Err(PurificationError)` if purified output has any:
/// - DET violations (non-deterministic patterns)
/// - IDEM violations (non-idempotent operations)
/// - SEC violations (security issues)
///
/// Note: SC/MAKE violations are warnings only and don't cause failure.
pub fn purify_and_validate(input: &str) -> anyhow::Result<String> {
    // Step 1: Purify and lint
    let result = purify_and_lint(input)?;

    // Step 2: Enforce zero-tolerance
    if !result.is_clean {
        return Err(PurificationError::new(&result).into());
    }

    // Step 3: Return clean code
    Ok(result.purified_code)
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

pub fn explain_purification_changes(original: &str) -> anyhow::Result<String> {
    let purified = purify_bash(original)?;

    if original.trim() == purified.trim() {
        return Ok("No changes needed - code is already purified.".to_string());
    }

    let explanations = collect_change_explanations(original, &purified);

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

    Ok(format!(
        "Changes made during purification:\n\n\
         Original:\n  {}\n\n\
         Purified:\n  {}\n\n\
         The purified version is more idempotent, deterministic, and safe.",
        original.trim(),
        purified.trim()
    ))
}

/// Collect explanations for each detected purification pattern.
fn collect_change_explanations(original: &str, purified: &str) -> Vec<String> {
    let mut explanations = Vec::new();

    if original.contains("mkdir") && !original.contains("mkdir -p") && purified.contains("mkdir -p")
    {
        explanations.push(
            "✓ Added -p flag to mkdir for idempotency\n  \
             Makes directory creation safe to re-run (won't fail if dir exists)"
                .to_string(),
        );
    }

    if original.contains("rm ") && !original.contains("rm -f") && purified.contains("rm -f") {
        explanations.push(
            "✓ Added -f flag to rm for idempotency\n  \
             Makes file deletion safe to re-run (won't fail if file doesn't exist)"
                .to_string(),
        );
    }

    if original.contains('$') && !original.contains("\"$") && purified.contains("\"$") {
        explanations.push(
            "✓ Added quotes around variables for safety\n  \
             Prevents word splitting and glob expansion issues"
                .to_string(),
        );
    }

    if original.contains("ln -s") && !original.contains("ln -sf") && purified.contains("ln -sf") {
        explanations.push(
            "✓ Added -f flag to ln -s for idempotency\n  \
             Makes symlink creation safe to re-run (forces replacement)"
                .to_string(),
        );
    }

    if original.contains("$RANDOM") && !purified.contains("$RANDOM") {
        explanations.push(
            "✓ Removed $RANDOM for determinism\n  \
             Non-deterministic values make scripts unpredictable"
                .to_string(),
        );
    }

    if (original.contains("date") || original.contains("$SECONDS"))
        && (!purified.contains("date") || !purified.contains("$SECONDS"))
    {
        explanations.push(
            "✓ Removed timestamp for determinism\n  \
             Time-based values make scripts non-reproducible"
                .to_string(),
        );
    }

    explanations
}

// Re-export transformation types that were extracted to purifier_transforms.rs
// so callers using crate::repl::purifier::TransformationCategory etc. still work.
pub use super::purifier_transforms::*;

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "purifier_tests.rs"]
// FIXME(PMAT-238): mod tests;
