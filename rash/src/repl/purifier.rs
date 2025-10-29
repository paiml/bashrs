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

    // For now, return summary
    // TODO: Implement full bash code generation from purified AST
    let output = format!("Purified {} statement(s)\n(Full bash output coming soon)", purified_ast.statements.len());

    Ok(output)
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
        assert!(
            purified.contains("Purified"),
            "Should return purification message"
        );
    }

    /// Test: REPL-005-001-002 - Purify $RANDOM (non-deterministic)
    #[test]
    fn test_REPL_005_001_purify_random() {
        let input = "echo $RANDOM";
        let result = purify_bash(input);

        assert!(result.is_ok(), "Should handle $RANDOM: {:?}", result);
        let purified = result.unwrap();
        assert!(purified.contains("Purified"), "Should purify successfully");
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
        assert!(purified.contains("Purified"), "Should purify successfully");
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
}
