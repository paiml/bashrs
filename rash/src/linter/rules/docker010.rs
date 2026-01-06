// DOCKER010: HEALTHCHECK validation - THIN SHIM
// All logic extracted to docker010_logic.rs

use super::docker010_logic::*;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let analysis = analyze_dockerfile(source);

    // Check HEALTHCHECK NONE
    if analysis.is_healthcheck_none {
        result.add(Diagnostic::new(
            "DOCKER010",
            Severity::Info,
            "HEALTHCHECK NONE disables health monitoring - ensure this is intentional (F065)",
            Span::new(analysis.healthcheck_line, 1, analysis.healthcheck_line, 80),
        ));
    }

    // Validate interval
    if let Some(interval) = analysis.interval_seconds {
        if is_interval_too_aggressive(interval) {
            result.add(Diagnostic::new(
                "DOCKER010",
                Severity::Warning,
                format!(
                    "HEALTHCHECK interval {}s may be too aggressive - consider 10s+ (F065)",
                    interval
                ),
                Span::new(analysis.healthcheck_line, 1, analysis.healthcheck_line, 80),
            ));
        }
    }

    // Suggest HEALTHCHECK if missing
    if should_suggest_healthcheck(&analysis) {
        result.add(Diagnostic::new(
            "DOCKER010",
            Severity::Info,
            "Consider adding HEALTHCHECK for container health monitoring (F065)",
            Span::new(analysis.cmd_line, 1, analysis.cmd_line, 1),
        ));
    }

    // Check ordering
    if analysis.has_healthcheck
        && is_healthcheck_after_cmd(analysis.healthcheck_line, analysis.cmd_line)
    {
        result.add(Diagnostic::new(
            "DOCKER010",
            Severity::Info,
            "HEALTHCHECK should typically come before CMD for readability (F065)",
            Span::new(analysis.healthcheck_line, 1, analysis.healthcheck_line, 1),
        ));
    }

    result
}
