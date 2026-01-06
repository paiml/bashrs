//! DOCKER010 Pure Logic - HEALTHCHECK validation
//!
//! Extracted for EXTREME TDD testability.

/// Analysis result for a Dockerfile
#[derive(Debug, Default)]
pub struct HealthcheckAnalysis {
    pub has_healthcheck: bool,
    pub has_cmd_or_entrypoint: bool,
    pub healthcheck_line: usize,
    pub cmd_line: usize,
    pub is_healthcheck_none: bool,
    pub interval_seconds: Option<u32>,
    pub timeout_seconds: Option<u32>,
    pub retries: Option<u32>,
}

/// Check if line starts with HEALTHCHECK
pub fn is_healthcheck_line(line: &str) -> bool {
    line.trim().to_uppercase().starts_with("HEALTHCHECK ")
}

/// Check if line is HEALTHCHECK NONE
pub fn is_healthcheck_none(line: &str) -> bool {
    line.trim().to_uppercase().contains("HEALTHCHECK") && line.to_uppercase().contains(" NONE")
}

/// Check if line starts with CMD or ENTRYPOINT
pub fn is_cmd_or_entrypoint(line: &str) -> bool {
    let upper = line.trim().to_uppercase();
    upper.starts_with("CMD ") || upper.starts_with("ENTRYPOINT ")
}

/// Extract duration from option like --interval=30s
pub fn extract_duration(line: &str, option: &str) -> Option<u32> {
    line.find(option).and_then(|start| {
        let value_start = start + option.len();
        let rest = &line[value_start..];
        let value: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        value.parse().ok()
    })
}

/// Validate interval value
pub fn is_interval_too_aggressive(seconds: u32) -> bool {
    seconds < 5
}

/// Validate timeout value
pub fn is_timeout_too_short(seconds: u32) -> bool {
    seconds < 1
}

/// Validate retries value
pub fn is_retries_too_low(retries: u32) -> bool {
    retries < 1
}

/// Check if healthcheck should come before CMD
pub fn is_healthcheck_after_cmd(healthcheck_line: usize, cmd_line: usize) -> bool {
    healthcheck_line > cmd_line && cmd_line > 0
}

/// Analyze a Dockerfile for HEALTHCHECK issues
pub fn analyze_dockerfile(source: &str) -> HealthcheckAnalysis {
    let mut analysis = HealthcheckAnalysis::default();

    for (line_num, line) in source.lines().enumerate() {
        if is_healthcheck_line(line) {
            analysis.has_healthcheck = true;
            analysis.healthcheck_line = line_num + 1;
            analysis.is_healthcheck_none = is_healthcheck_none(line);
            analysis.interval_seconds = extract_duration(line, "--interval=");
            analysis.timeout_seconds = extract_duration(line, "--timeout=");
            analysis.retries = extract_duration(line, "--retries=");
        }

        if is_cmd_or_entrypoint(line) {
            analysis.has_cmd_or_entrypoint = true;
            analysis.cmd_line = line_num + 1;
        }
    }

    analysis
}

/// Check if HEALTHCHECK is missing but should be present
pub fn should_suggest_healthcheck(analysis: &HealthcheckAnalysis) -> bool {
    analysis.has_cmd_or_entrypoint && !analysis.has_healthcheck
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== IS HEALTHCHECK LINE =====

    #[test]
    fn test_is_healthcheck_line_true() {
        assert!(is_healthcheck_line("HEALTHCHECK CMD curl localhost"));
        assert!(is_healthcheck_line(
            "  HEALTHCHECK --interval=30s CMD curl localhost"
        ));
        assert!(is_healthcheck_line("healthcheck cmd curl")); // case insensitive
    }

    #[test]
    fn test_is_healthcheck_line_false() {
        assert!(!is_healthcheck_line("# HEALTHCHECK"));
        assert!(!is_healthcheck_line("CMD echo"));
        assert!(!is_healthcheck_line("RUN echo HEALTHCHECK"));
    }

    // ===== IS HEALTHCHECK NONE =====

    #[test]
    fn test_is_healthcheck_none_true() {
        assert!(is_healthcheck_none("HEALTHCHECK NONE"));
        assert!(is_healthcheck_none("  HEALTHCHECK NONE  "));
        assert!(is_healthcheck_none("healthcheck none"));
    }

    #[test]
    fn test_is_healthcheck_none_false() {
        assert!(!is_healthcheck_none("HEALTHCHECK CMD curl localhost"));
        assert!(!is_healthcheck_none("CMD echo none"));
    }

    // ===== IS CMD OR ENTRYPOINT =====

    #[test]
    fn test_is_cmd_or_entrypoint_cmd() {
        assert!(is_cmd_or_entrypoint("CMD echo hello"));
        assert!(is_cmd_or_entrypoint("  CMD [\"python\", \"app.py\"]"));
        assert!(is_cmd_or_entrypoint("cmd echo")); // case insensitive
    }

    #[test]
    fn test_is_cmd_or_entrypoint_entrypoint() {
        assert!(is_cmd_or_entrypoint("ENTRYPOINT [\"./app\"]"));
        assert!(is_cmd_or_entrypoint("  entrypoint ./run.sh"));
    }

    #[test]
    fn test_is_cmd_or_entrypoint_false() {
        assert!(!is_cmd_or_entrypoint("RUN echo"));
        assert!(!is_cmd_or_entrypoint("# CMD echo"));
        assert!(!is_cmd_or_entrypoint("FROM ubuntu"));
    }

    // ===== EXTRACT DURATION =====

    #[test]
    fn test_extract_duration_interval() {
        assert_eq!(
            extract_duration("--interval=30s CMD", "--interval="),
            Some(30)
        );
        assert_eq!(extract_duration("--interval=5s", "--interval="), Some(5));
        assert_eq!(
            extract_duration("--interval=120s --timeout=3s", "--interval="),
            Some(120)
        );
    }

    #[test]
    fn test_extract_duration_timeout() {
        assert_eq!(extract_duration("--timeout=3s CMD", "--timeout="), Some(3));
        assert_eq!(
            extract_duration("--interval=30s --timeout=10s", "--timeout="),
            Some(10)
        );
    }

    #[test]
    fn test_extract_duration_retries() {
        assert_eq!(extract_duration("--retries=3", "--retries="), Some(3));
        assert_eq!(extract_duration("--retries=5 CMD", "--retries="), Some(5));
    }

    #[test]
    fn test_extract_duration_missing() {
        assert_eq!(extract_duration("CMD curl localhost", "--interval="), None);
        assert_eq!(extract_duration("--timeout=3s", "--interval="), None);
    }

    // ===== VALIDATION FUNCTIONS =====

    #[test]
    fn test_is_interval_too_aggressive() {
        assert!(is_interval_too_aggressive(1));
        assert!(is_interval_too_aggressive(4));
        assert!(!is_interval_too_aggressive(5));
        assert!(!is_interval_too_aggressive(30));
    }

    #[test]
    fn test_is_timeout_too_short() {
        assert!(is_timeout_too_short(0));
        assert!(!is_timeout_too_short(1));
        assert!(!is_timeout_too_short(3));
    }

    #[test]
    fn test_is_retries_too_low() {
        assert!(is_retries_too_low(0));
        assert!(!is_retries_too_low(1));
        assert!(!is_retries_too_low(3));
    }

    // ===== IS HEALTHCHECK AFTER CMD =====

    #[test]
    fn test_is_healthcheck_after_cmd() {
        assert!(is_healthcheck_after_cmd(10, 5)); // healthcheck on line 10, cmd on line 5
        assert!(!is_healthcheck_after_cmd(5, 10)); // healthcheck before cmd
        assert!(!is_healthcheck_after_cmd(5, 0)); // no cmd
    }

    // ===== ANALYZE DOCKERFILE =====

    #[test]
    fn test_analyze_dockerfile_with_healthcheck() {
        let dockerfile = "FROM ubuntu\nHEALTHCHECK --interval=30s CMD curl localhost\nCMD echo";
        let analysis = analyze_dockerfile(dockerfile);
        assert!(analysis.has_healthcheck);
        assert!(analysis.has_cmd_or_entrypoint);
        assert_eq!(analysis.healthcheck_line, 2);
        assert_eq!(analysis.cmd_line, 3);
        assert_eq!(analysis.interval_seconds, Some(30));
    }

    #[test]
    fn test_analyze_dockerfile_without_healthcheck() {
        let dockerfile = "FROM ubuntu\nCMD echo hello";
        let analysis = analyze_dockerfile(dockerfile);
        assert!(!analysis.has_healthcheck);
        assert!(analysis.has_cmd_or_entrypoint);
    }

    #[test]
    fn test_analyze_dockerfile_healthcheck_none() {
        let dockerfile = "FROM ubuntu\nHEALTHCHECK NONE\nCMD echo";
        let analysis = analyze_dockerfile(dockerfile);
        assert!(analysis.has_healthcheck);
        assert!(analysis.is_healthcheck_none);
    }

    // ===== SHOULD SUGGEST HEALTHCHECK =====

    #[test]
    fn test_should_suggest_healthcheck_true() {
        let analysis = HealthcheckAnalysis {
            has_cmd_or_entrypoint: true,
            has_healthcheck: false,
            ..Default::default()
        };
        assert!(should_suggest_healthcheck(&analysis));
    }

    #[test]
    fn test_should_suggest_healthcheck_false_has_healthcheck() {
        let analysis = HealthcheckAnalysis {
            has_cmd_or_entrypoint: true,
            has_healthcheck: true,
            ..Default::default()
        };
        assert!(!should_suggest_healthcheck(&analysis));
    }

    #[test]
    fn test_should_suggest_healthcheck_false_no_cmd() {
        let analysis = HealthcheckAnalysis {
            has_cmd_or_entrypoint: false,
            has_healthcheck: false,
            ..Default::default()
        };
        assert!(!should_suggest_healthcheck(&analysis));
    }
}
