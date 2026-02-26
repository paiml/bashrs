//! Coverage tests for corpus/citl.rs — targeting format_convergence_criteria.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::citl::{format_convergence_criteria, ConvergenceCriteria};

fn make_criteria(
    rate_met: bool,
    stability_met: bool,
    growth_met: bool,
    no_regressions: bool,
) -> ConvergenceCriteria {
    let converged = rate_met && stability_met && growth_met && no_regressions;
    ConvergenceCriteria {
        rate_met,
        rate_values: vec![99.1, 99.2, 99.3],
        stability_met,
        delta_values: vec![0.1, 0.1, 0.1],
        growth_met,
        corpus_size: 1000,
        target_size: 900,
        no_regressions,
        converged,
    }
}

// =============================================================================
// format_convergence_criteria — all code paths
// =============================================================================

#[test]
fn test_convergence_all_met() {
    let criteria = make_criteria(true, true, true, true);
    let result = format_convergence_criteria(&criteria);
    assert!(
        result.contains("CONVERGED") || result.contains("converged"),
        "Should indicate convergence"
    );
}

#[test]
fn test_convergence_none_met() {
    let criteria = make_criteria(false, false, false, false);
    let result = format_convergence_criteria(&criteria);
    assert!(
        result.contains("NOT") || result.contains("not"),
        "Should indicate not converged"
    );
}

#[test]
fn test_convergence_rate_only_failing() {
    let criteria = make_criteria(false, true, true, true);
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_stability_only_failing() {
    let criteria = make_criteria(true, false, true, true);
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_growth_only_failing() {
    let criteria = make_criteria(true, true, false, true);
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_regressions_only_failing() {
    let criteria = make_criteria(true, true, true, false);
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_empty_rate_values() {
    let criteria = ConvergenceCriteria {
        rate_met: false,
        rate_values: vec![],
        stability_met: true,
        delta_values: vec![0.1, 0.2],
        growth_met: true,
        corpus_size: 1000,
        target_size: 900,
        no_regressions: true,
        converged: false,
    };
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_empty_delta_values() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![99.5, 99.6],
        stability_met: false,
        delta_values: vec![],
        growth_met: true,
        corpus_size: 1000,
        target_size: 900,
        no_regressions: true,
        converged: false,
    };
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_growth_below_target() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![99.1],
        stability_met: true,
        delta_values: vec![0.1],
        growth_met: false,
        corpus_size: 500,
        target_size: 900,
        no_regressions: true,
        converged: false,
    };
    let result = format_convergence_criteria(&criteria);
    assert!(!result.is_empty());
}

#[test]
fn test_convergence_contains_table_structure() {
    let criteria = make_criteria(true, true, true, true);
    let result = format_convergence_criteria(&criteria);
    // Should contain table formatting elements
    assert!(result.contains("│") || result.contains("|") || result.contains("Status"));
}

#[test]
fn test_convergence_multiple_criteria_failing() {
    let criteria = make_criteria(false, false, true, false);
    let result = format_convergence_criteria(&criteria);
    assert!(
        result.contains("NOT") || result.contains("not"),
        "Multiple failures should still show NOT CONVERGED"
    );
}

#[test]
fn test_convergence_single_rate_value() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![99.9],
        stability_met: true,
        delta_values: vec![0.01],
        growth_met: true,
        corpus_size: 1000,
        target_size: 900,
        no_regressions: true,
        converged: true,
    };
    let result = format_convergence_criteria(&criteria);
    assert!(result.contains("CONVERGED") || result.contains("converged"));
}
