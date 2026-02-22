//! Pure logic functions for gate and lint operations.
//!
//! This module extracts testable, side-effect-free computation from
//! `commands.rs` so that the command handlers become thin I/O shells
//! delegating to well-tested pure functions.

use crate::cli::args::{LintFormat, LintLevel};
use crate::gates::{GateConfig, MutationGate, SatdGate};
use crate::linter::ignore_file::IgnoreFile;
use crate::linter::output::OutputFormat;
use crate::linter::Severity;
use std::collections::HashSet;

// =============================================================================
// Lint-rule filtering helpers
// =============================================================================

/// Build a set of ignored rule codes from `--ignore`, `-e` flags, and
/// `.bashrsignore` rule codes.
///
/// All codes are normalised to UPPERCASE and trimmed before insertion.
pub(crate) fn build_ignored_rules(
    ignore_rules: Option<&str>,
    exclude_rules: Option<&[String]>,
    ignore_file_data: Option<&IgnoreFile>,
) -> HashSet<String> {
    let mut rules = HashSet::new();

    // Add from --ignore (comma-separated)
    if let Some(ignore_str) = ignore_rules {
        for code in ignore_str.split(',') {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }

    // Add from -e (can be repeated)
    if let Some(excludes) = exclude_rules {
        for code in excludes {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }

    // Issue #85: Add rule codes from .bashrsignore file
    if let Some(ignore) = ignore_file_data {
        for code in ignore.ignored_rules() {
            rules.insert(code);
        }
    }

    rules
}

/// Determine the minimum severity level based on `--quiet` and `--level` flags.
///
/// When `quiet` is true, informational messages are suppressed regardless of
/// the explicit `level`.
pub(crate) fn determine_min_severity(quiet: bool, level: LintLevel) -> Severity {
    if quiet {
        Severity::Warning // --quiet suppresses info
    } else {
        match level {
            LintLevel::Info => Severity::Info,
            LintLevel::Warning => Severity::Warning,
            LintLevel::Error => Severity::Error,
        }
    }
}

/// Convert the CLI `LintFormat` enum to the linter's internal `OutputFormat`.
pub(crate) fn convert_lint_format(format: LintFormat) -> OutputFormat {
    match format {
        LintFormat::Human => OutputFormat::Human,
        LintFormat::Json => OutputFormat::Json,
        LintFormat::Sarif => OutputFormat::Sarif,
    }
}

// =============================================================================
// Gate threshold / configuration helpers
// =============================================================================

/// Return `true` when a coverage gate check should be skipped because the
/// `check_coverage` flag is not set in the config.
pub(crate) fn coverage_gate_should_skip(config: &GateConfig) -> bool {
    !config.gates.check_coverage
}

/// Return the minimum coverage threshold from the gate configuration.
pub(crate) fn coverage_gate_threshold(config: &GateConfig) -> f64 {
    config.gates.min_coverage
}

/// Return `true` when a SATD gate check should be skipped.
///
/// Skipped when the gate is absent or explicitly disabled.
pub(crate) fn satd_gate_should_skip(satd: Option<&SatdGate>) -> bool {
    satd.map_or(true, |s| !s.enabled)
}

/// Return `true` when SATD patterns are effectively empty (nothing to check).
pub(crate) fn satd_gate_has_no_patterns(satd: &SatdGate) -> bool {
    satd.patterns.is_empty()
}

/// Return `true` when a mutation gate check should be skipped.
///
/// Skipped when the gate is absent or explicitly disabled.
pub(crate) fn mutation_gate_should_skip(mutation: Option<&MutationGate>) -> bool {
    mutation.map_or(true, |m| !m.enabled)
}

/// Return the minimum mutation score from the gate configuration.
///
/// Returns `0.0` when no mutation gate is configured.
pub(crate) fn mutation_gate_min_score(mutation: Option<&MutationGate>) -> f64 {
    mutation.map_or(0.0, |m| m.min_score)
}

// =============================================================================
// Tier / gate selection helpers
// =============================================================================

/// Select the list of gates that should run for a given tier number.
///
/// Returns `None` for invalid tier numbers (anything other than 1, 2, or 3).
pub(crate) fn select_gates_for_tier<'a>(
    config: &'a GateConfig,
    tier: u8,
) -> Option<&'a Vec<String>> {
    match tier {
        1 => Some(&config.tiers.tier1_gates),
        2 => Some(&config.tiers.tier2_gates),
        3 => Some(&config.tiers.tier3_gates),
        _ => None,
    }
}

/// Return `true` when `tier` is a valid gate tier (1, 2, or 3).
#[allow(dead_code)]
pub(crate) fn is_valid_tier(tier: u8) -> bool {
    matches!(tier, 1..=3)
}

// =============================================================================
// Rule-filter helpers
// =============================================================================

/// Return `true` when a diagnostic code matches any rule in the allowed set.
///
/// Matching is performed via substring containment so that partial rule-code
/// prefixes (e.g. `"SEC"`) select all rules in that family.
#[allow(dead_code)]
pub(crate) fn diagnostic_matches_rule_filter(code: &str, allowed_rules: &[String]) -> bool {
    allowed_rules.iter().any(|rule| code.contains(rule.as_str()))
}

// =============================================================================
// Type-diagnostic severity helpers
// =============================================================================

/// Classify whether a type-check diagnostic severity constitutes an error,
/// taking into account the `type_strict` flag.
///
/// In strict mode, warnings are treated as errors.
#[allow(dead_code)]
pub(crate) fn type_diagnostic_is_error(
    is_error_severity: bool,
    is_warning_severity: bool,
    type_strict: bool,
) -> bool {
    is_error_severity || (type_strict && is_warning_severity)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::gates::{Gates, MutationGate, SatdGate, Tiers};

    // Helper: build a minimal GateConfig for testing
    fn make_config(
        check_coverage: bool,
        min_coverage: f64,
        satd: Option<SatdGate>,
        mutation: Option<MutationGate>,
        tier1: Vec<&str>,
        tier2: Vec<&str>,
        tier3: Vec<&str>,
    ) -> GateConfig {
        GateConfig {
            metadata: None,
            gates: Gates {
                run_clippy: false,
                clippy_strict: false,
                run_tests: false,
                test_timeout: 300,
                check_coverage,
                min_coverage,
                check_complexity: false,
                max_complexity: 10,
                satd,
                mutation,
                security: None,
            },
            tiers: Tiers {
                tier1_gates: tier1.iter().map(|s| s.to_string()).collect(),
                tier2_gates: tier2.iter().map(|s| s.to_string()).collect(),
                tier3_gates: tier3.iter().map(|s| s.to_string()).collect(),
            },
        }
    }

    // ------------------------------------------------------------------
    // build_ignored_rules
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_001_build_ignored_rules_from_comma_string() {
        let rules = build_ignored_rules(Some("sec001,sec002,SEC003"), None, None);
        assert!(rules.contains("SEC001"));
        assert!(rules.contains("SEC002"));
        assert!(rules.contains("SEC003"));
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_GATE_LOGIC_002_build_ignored_rules_from_exclude_slice() {
        let excludes = vec!["det001".to_string(), "IDEM001".to_string()];
        let rules = build_ignored_rules(None, Some(&excludes), None);
        assert!(rules.contains("DET001"));
        assert!(rules.contains("IDEM001"));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_GATE_LOGIC_003_build_ignored_rules_deduplicates() {
        // Same code via two sources — must appear only once in the set
        let excludes = vec!["sec001".to_string()];
        let rules = build_ignored_rules(Some("SEC001"), Some(&excludes), None);
        assert_eq!(rules.len(), 1);
        assert!(rules.contains("SEC001"));
    }

    #[test]
    fn test_GATE_LOGIC_004_build_ignored_rules_skips_empty_tokens() {
        // Trailing comma should not produce an empty entry
        let rules = build_ignored_rules(Some("sec001,,"), None, None);
        assert!(!rules.contains(""));
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_GATE_LOGIC_005_build_ignored_rules_normalises_to_uppercase() {
        let rules = build_ignored_rules(Some("sec001"), None, None);
        assert!(rules.contains("SEC001"));
        assert!(!rules.contains("sec001"));
    }

    #[test]
    fn test_GATE_LOGIC_006_build_ignored_rules_all_none_gives_empty() {
        let rules = build_ignored_rules(None, None, None);
        assert!(rules.is_empty());
    }

    // ------------------------------------------------------------------
    // determine_min_severity
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_007_determine_min_severity_quiet_suppresses_to_warning() {
        // --quiet should suppress Info to Warning regardless of the level flag
        assert_eq!(
            determine_min_severity(true, LintLevel::Info),
            Severity::Warning
        );
        assert_eq!(
            determine_min_severity(true, LintLevel::Warning),
            Severity::Warning
        );
    }

    #[test]
    fn test_GATE_LOGIC_008_determine_min_severity_not_quiet_passes_through() {
        assert_eq!(
            determine_min_severity(false, LintLevel::Info),
            Severity::Info
        );
        assert_eq!(
            determine_min_severity(false, LintLevel::Warning),
            Severity::Warning
        );
        assert_eq!(
            determine_min_severity(false, LintLevel::Error),
            Severity::Error
        );
    }

    // ------------------------------------------------------------------
    // convert_lint_format
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_009_convert_lint_format_all_variants() {
        assert_eq!(convert_lint_format(LintFormat::Human), OutputFormat::Human);
        assert_eq!(convert_lint_format(LintFormat::Json), OutputFormat::Json);
        assert_eq!(convert_lint_format(LintFormat::Sarif), OutputFormat::Sarif);
    }

    // ------------------------------------------------------------------
    // coverage gate helpers
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_010_coverage_gate_skip_and_threshold() {
        let disabled = make_config(false, 80.0, None, None, vec![], vec![], vec![]);
        assert!(coverage_gate_should_skip(&disabled));
        let enabled = make_config(true, 92.5, None, None, vec![], vec![], vec![]);
        assert!(!coverage_gate_should_skip(&enabled));
        assert!((coverage_gate_threshold(&enabled) - 92.5).abs() < f64::EPSILON);
    }

    // ------------------------------------------------------------------
    // SATD gate helpers
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_011_satd_gate_should_skip_none_or_disabled() {
        // Absent gate → skip
        assert!(satd_gate_should_skip(None));
        // Disabled gate → skip
        let disabled = SatdGate { enabled: false, max_count: 0, patterns: vec![] };
        assert!(satd_gate_should_skip(Some(&disabled)));
        // Enabled gate → do not skip
        let enabled = SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec!["TODO".to_string()],
        };
        assert!(!satd_gate_should_skip(Some(&enabled)));
    }

    #[test]
    fn test_GATE_LOGIC_012_satd_gate_pattern_detection() {
        let empty = SatdGate { enabled: true, max_count: 0, patterns: vec![] };
        assert!(satd_gate_has_no_patterns(&empty));
        let populated = SatdGate {
            enabled: true,
            max_count: 2,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        assert!(!satd_gate_has_no_patterns(&populated));
    }

    // ------------------------------------------------------------------
    // Mutation gate helpers
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_013_mutation_gate_should_skip_none_or_disabled() {
        assert!(mutation_gate_should_skip(None));
        let disabled = MutationGate { enabled: false, min_score: 0.0 };
        assert!(mutation_gate_should_skip(Some(&disabled)));
        let enabled = MutationGate { enabled: true, min_score: 80.0 };
        assert!(!mutation_gate_should_skip(Some(&enabled)));
    }

    #[test]
    fn test_GATE_LOGIC_014_mutation_gate_min_score() {
        let gate = MutationGate { enabled: true, min_score: 75.5 };
        assert!((mutation_gate_min_score(Some(&gate)) - 75.5).abs() < f64::EPSILON);
        // Default when None
        assert!((mutation_gate_min_score(None) - 0.0).abs() < f64::EPSILON);
    }

    // ------------------------------------------------------------------
    // Tier / gate selection helpers
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_015_select_gates_for_all_tiers() {
        let config = make_config(
            false,
            80.0,
            None,
            None,
            vec!["clippy", "tests"],
            vec!["coverage", "complexity"],
            vec!["mutation", "security"],
        );
        assert_eq!(
            select_gates_for_tier(&config, 1).unwrap(),
            &["clippy", "tests"]
        );
        assert_eq!(
            select_gates_for_tier(&config, 2).unwrap(),
            &["coverage", "complexity"]
        );
        assert_eq!(
            select_gates_for_tier(&config, 3).unwrap(),
            &["mutation", "security"]
        );
        // Invalid tiers return None
        assert!(select_gates_for_tier(&config, 0).is_none());
        assert!(select_gates_for_tier(&config, 4).is_none());
    }

    #[test]
    fn test_GATE_LOGIC_016_is_valid_tier_boundaries() {
        assert!(is_valid_tier(1));
        assert!(is_valid_tier(2));
        assert!(is_valid_tier(3));
        assert!(!is_valid_tier(0));
        assert!(!is_valid_tier(4));
        assert!(!is_valid_tier(100));
    }

    // ------------------------------------------------------------------
    // diagnostic_matches_rule_filter
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_017_diagnostic_rule_filter_matching() {
        // Exact match
        let exact = vec!["SEC001".to_string()];
        assert!(diagnostic_matches_rule_filter("SEC001", &exact));
        // Substring/prefix match — "SEC" matches "SEC001", "SEC002"
        let prefix = vec!["SEC".to_string()];
        assert!(diagnostic_matches_rule_filter("SEC001", &prefix));
        assert!(diagnostic_matches_rule_filter("SEC002", &prefix));
        // Unrelated code does not match
        assert!(!diagnostic_matches_rule_filter("DET001", &exact));
        // Empty allowed list never matches
        assert!(!diagnostic_matches_rule_filter("SEC001", &[]));
    }

    // ------------------------------------------------------------------
    // type_diagnostic_is_error
    // ------------------------------------------------------------------

    #[test]
    fn test_GATE_LOGIC_018_type_diagnostic_error_classification() {
        // Error severity → always an error
        assert!(type_diagnostic_is_error(true, false, false));
        assert!(type_diagnostic_is_error(true, false, true));
        // Warning non-strict → not an error
        assert!(!type_diagnostic_is_error(false, true, false));
        // Warning strict → treated as error
        assert!(type_diagnostic_is_error(false, true, true));
        // Info → never an error
        assert!(!type_diagnostic_is_error(false, false, false));
        assert!(!type_diagnostic_is_error(false, false, true));
    }
}
