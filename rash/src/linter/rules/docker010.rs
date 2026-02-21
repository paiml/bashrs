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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ===== HEALTHCHECK NONE =====

    #[test]
    fn test_DOCKER010_COV_001_healthcheck_none_triggers_info() {
        let dockerfile = "FROM ubuntu:22.04\nHEALTHCHECK NONE\nCMD echo hello";
        let result = check(dockerfile);
        assert!(result.diagnostics.iter().any(|d| d
            .message
            .contains("HEALTHCHECK NONE disables health monitoring")));
    }

    #[test]
    fn test_DOCKER010_COV_002_healthcheck_present_no_none_warning() {
        let dockerfile =
            "FROM ubuntu:22.04\nHEALTHCHECK --interval=30s CMD curl -f http://localhost/\nCMD echo";
        let result = check(dockerfile);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("HEALTHCHECK NONE")));
    }

    // ===== AGGRESSIVE INTERVAL =====

    #[test]
    fn test_DOCKER010_COV_003_aggressive_interval_triggers_warning() {
        let dockerfile =
            "FROM ubuntu:22.04\nHEALTHCHECK --interval=2s CMD curl localhost\nCMD echo";
        let result = check(dockerfile);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("may be too aggressive")));
    }

    #[test]
    fn test_DOCKER010_COV_004_normal_interval_no_warning() {
        let dockerfile =
            "FROM ubuntu:22.04\nHEALTHCHECK --interval=30s CMD curl localhost\nCMD echo";
        let result = check(dockerfile);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("too aggressive")));
    }

    #[test]
    fn test_DOCKER010_COV_005_boundary_interval_5s_no_warning() {
        let dockerfile =
            "FROM ubuntu:22.04\nHEALTHCHECK --interval=5s CMD curl localhost\nCMD echo";
        let result = check(dockerfile);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("too aggressive")));
    }

    // ===== MISSING HEALTHCHECK SUGGESTION =====

    #[test]
    fn test_DOCKER010_COV_006_missing_healthcheck_with_cmd() {
        let dockerfile = "FROM ubuntu:22.04\nRUN apt-get update\nCMD echo hello";
        let result = check(dockerfile);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("Consider adding HEALTHCHECK")));
    }

    #[test]
    fn test_DOCKER010_COV_007_missing_healthcheck_with_entrypoint() {
        let dockerfile = "FROM ubuntu:22.04\nENTRYPOINT [\"./app\"]\n";
        let result = check(dockerfile);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("Consider adding HEALTHCHECK")));
    }

    #[test]
    fn test_DOCKER010_COV_008_no_cmd_no_entrypoint_no_suggestion() {
        let dockerfile = "FROM ubuntu:22.04\nRUN echo hello\n";
        let result = check(dockerfile);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("Consider adding HEALTHCHECK")));
    }

    // ===== ORDERING =====

    #[test]
    fn test_DOCKER010_COV_009_healthcheck_after_cmd_triggers_info() {
        let dockerfile =
            "FROM ubuntu:22.04\nCMD echo hello\nHEALTHCHECK --interval=30s CMD curl localhost";
        let result = check(dockerfile);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("should typically come before CMD")));
    }

    #[test]
    fn test_DOCKER010_COV_010_healthcheck_before_cmd_no_ordering_issue() {
        let dockerfile =
            "FROM ubuntu:22.04\nHEALTHCHECK --interval=30s CMD curl localhost\nCMD echo";
        let result = check(dockerfile);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("should typically come before CMD")));
    }

    // ===== COMBINED / CLEAN =====

    #[test]
    fn test_DOCKER010_COV_011_clean_dockerfile_no_diagnostics() {
        let dockerfile = "FROM ubuntu:22.04\nHEALTHCHECK --interval=30s --timeout=10s --retries=3 CMD curl -f http://localhost/\nCMD [\"./app\"]";
        let result = check(dockerfile);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DOCKER010_COV_012_multiple_issues() {
        // HEALTHCHECK NONE + after CMD = 2 diagnostics (NONE + ordering)
        let dockerfile = "FROM ubuntu:22.04\nCMD echo hello\nHEALTHCHECK NONE";
        let result = check(dockerfile);
        assert!(result.diagnostics.len() >= 2);
    }

    #[test]
    fn test_DOCKER010_COV_013_no_interval_specified() {
        // HEALTHCHECK without --interval should not trigger aggressive interval warning
        let dockerfile = "FROM ubuntu:22.04\nHEALTHCHECK CMD curl localhost\nCMD echo";
        let result = check(dockerfile);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("too aggressive")));
    }

    #[test]
    fn test_DOCKER010_COV_014_all_diagnostics_have_docker010_code() {
        let dockerfile = "FROM ubuntu:22.04\nCMD echo hello\nHEALTHCHECK NONE";
        let result = check(dockerfile);
        for diag in &result.diagnostics {
            assert_eq!(diag.code, "DOCKER010");
        }
    }
}
