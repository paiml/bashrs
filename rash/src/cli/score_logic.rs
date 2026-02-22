// Score & Audit Logic Module - Extracted for testability
//
// This module contains pure functions for score computation, grade
// comparison, runtime scoring, and audit checking. No I/O or printing.
//
// Architecture:
// - score_logic.rs: Pure score/audit computation (no I/O)
// - commands.rs: Thin I/O shim (reads files, calls logic, prints output)

use crate::linter::docker_profiler::{PlatformProfile, SizeEstimate};

// =============================================================================
// RUNTIME SCORE (Docker image runtime performance scoring)
// =============================================================================

/// Runtime performance score for Docker images
#[derive(Debug, Clone)]
pub struct RuntimeScore {
    /// Overall runtime score (0-100)
    pub score: f64,
    /// Image size in bytes
    pub estimated_size: u64,
    /// Size score component (0-100)
    pub size_score: f64,
    /// Layer optimization score (0-100)
    pub layer_score: f64,
    /// Number of bloat patterns detected
    pub bloat_count: usize,
    /// Whether Docker is available for actual measurement
    pub docker_available: bool,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
}

impl RuntimeScore {
    pub fn new(
        estimate: &SizeEstimate,
        profile: PlatformProfile,
        docker_available: bool,
    ) -> Self {
        let max_size = profile.max_size_bytes();
        let mut suggestions = Vec::new();

        // Calculate size score
        let size_score = compute_size_score(estimate.total_estimated, max_size);

        // Calculate layer score (penalize many layers and bloat)
        let layer_count = estimate.layer_estimates.len();
        let bloat_count = estimate.bloat_patterns.len();
        let layer_score = compute_layer_score(layer_count, bloat_count);

        // Add suggestions based on analysis
        for pattern in &estimate.bloat_patterns {
            suggestions.push(format!("{}: {}", pattern.code, pattern.remediation));
        }

        if layer_count > 10 {
            suggestions.push("Consider combining RUN commands to reduce layer count".to_string());
        }

        if estimate.total_estimated > max_size {
            suggestions.push(format!(
                "Image size ({:.1}GB) exceeds limit ({:.1}GB) - use smaller base image or multi-stage build",
                estimate.total_estimated as f64 / 1_000_000_000.0,
                max_size as f64 / 1_000_000_000.0
            ));
        }

        // Overall score is weighted average
        let score = (size_score * 0.6 + layer_score * 0.4).clamp(0.0, 100.0);

        Self {
            score,
            estimated_size: estimate.total_estimated,
            size_score,
            layer_score,
            bloat_count,
            docker_available,
            suggestions,
        }
    }

    pub fn grade(&self) -> &'static str {
        grade_from_score_100(self.score)
    }
}

// =============================================================================
// GRADE HELPERS
// =============================================================================

/// Convert a 0-100 score to a letter grade string
pub fn grade_from_score_100(score: f64) -> &'static str {
    match score as u32 {
        95..=100 => "A+",
        90..=94 => "A",
        85..=89 => "A-",
        80..=84 => "B+",
        75..=79 => "B",
        70..=74 => "B-",
        65..=69 => "C+",
        60..=64 => "C",
        55..=59 => "C-",
        50..=54 => "D",
        _ => "F",
    }
}

/// Compute combined grade from static analysis score and runtime score.
/// Static analysis score is on a 0-10 scale (multiplied by 10 internally).
/// Runtime score is on a 0-100 scale.
/// Combined = 60% static + 40% runtime.
pub fn compute_combined_score(static_score_10: f64, runtime_score_100: f64) -> f64 {
    static_score_10 * 10.0 * 0.6 + runtime_score_100 * 0.4
}

/// Compare two grades using a canonical ordering.
/// Returns true if `actual` grade is at least as good as `min_grade`.
pub fn grade_meets_minimum(actual: &str, min_grade: &str) -> Option<bool> {
    let grade_order = ["F", "D", "C", "C+", "B", "B+", "A", "A+"];
    let actual_pos = grade_order.iter().position(|&g| g == actual);
    let min_pos = grade_order.iter().position(|&g| g == min_grade);
    match (actual_pos, min_pos) {
        (Some(a), Some(m)) => Some(a >= m),
        _ => None,
    }
}

// =============================================================================
// SCORE COMPONENT COMPUTATION (pure functions)
// =============================================================================

/// Compute size score (0-100) based on estimated size and max allowed.
pub fn compute_size_score(estimated: u64, max_size: u64) -> f64 {
    if max_size == u64::MAX {
        // No limit - base on reasonable defaults (5GB good, 10GB average)
        let five_gb = 5_000_000_000u64;
        if estimated < five_gb {
            100.0
        } else {
            let ratio = estimated as f64 / five_gb as f64;
            (100.0 / ratio).clamp(0.0, 100.0)
        }
    } else {
        let ratio = estimated as f64 / max_size as f64;
        if ratio > 1.0 {
            0.0 // Over limit
        } else if ratio > 0.8 {
            (1.0 - ratio) * 500.0 // 0-100 for 80-100% of limit
        } else {
            100.0 - (ratio * 50.0) // 50-100 for under 80%
        }
    }
}

/// Compute layer optimization score (0-100) based on layer count and bloat.
pub fn compute_layer_score(layer_count: usize, bloat_count: usize) -> f64 {
    let base = if layer_count <= 5 {
        100.0
    } else if layer_count <= 10 {
        80.0
    } else {
        60.0
    };
    (base - (bloat_count as f64 * 20.0)).max(0.0)
}

// =============================================================================
// AUDIT RESULTS (pure data structure + logic)
// =============================================================================

/// Comprehensive quality audit results
#[derive(Debug, Clone)]
pub struct AuditResults {
    pub parse_success: bool,
    pub parse_error: Option<String>,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub test_passed: usize,
    pub test_failed: usize,
    pub test_total: usize,
    pub score: Option<crate::bash_quality::scoring::QualityScore>,
    pub overall_pass: bool,
    pub failure_reason: Option<String>,
}

impl AuditResults {
    /// Create a new AuditResults with default values (all passing)
    pub fn new() -> Self {
        Self {
            parse_success: true,
            parse_error: None,
            lint_errors: 0,
            lint_warnings: 0,
            test_passed: 0,
            test_failed: 0,
            test_total: 0,
            score: None,
            overall_pass: true,
            failure_reason: None,
        }
    }
}

impl Default for AuditResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Check lint results and update audit pass/fail status.
/// Pure logic: modifies `results` in place based on lint counts and strict mode.
pub fn audit_check_lint(results: &mut AuditResults, strict: bool) {
    if results.lint_errors > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!("{} lint errors found", results.lint_errors));
    }
    if strict && results.lint_warnings > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!(
            "Strict mode: {} warnings found",
            results.lint_warnings
        ));
    }
}

/// Run tests from source and update audit results.
/// Pure logic: discovers and runs tests, updates results accordingly.
pub fn audit_run_tests(source: &str, results: &mut AuditResults) {
    use crate::bash_quality::testing::{discover_tests, run_tests, TestResult};

    let tests = match discover_tests(source) {
        Ok(t) => t,
        Err(_) => return,
    };
    let test_report = match run_tests(source, &tests) {
        Ok(r) => r,
        Err(_) => return,
    };

    results.test_total = test_report.results.len();
    results.test_passed = test_report
        .results
        .iter()
        .filter(|(_, result)| matches!(result, TestResult::Pass))
        .count();
    results.test_failed = test_report
        .results
        .iter()
        .filter(|(_, result)| matches!(result, TestResult::Fail(_)))
        .count();

    if results.test_failed > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!(
            "{}/{} tests failed",
            results.test_failed, results.test_total
        ));
    }
}

/// Check score against minimum grade requirement and update audit results.
/// Pure logic: scores the source and compares against min_grade.
pub fn audit_check_score(source: &str, min_grade: Option<&str>, results: &mut AuditResults) {
    use crate::bash_quality::scoring::score_script;

    let score = match score_script(source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to score script: {}", e);
            return;
        }
    };

    if let Some(min_grade_str) = min_grade {
        if let Some(false) = grade_meets_minimum(&score.grade, min_grade_str) {
            results.overall_pass = false;
            results.failure_reason = Some(format!(
                "Quality grade {} below minimum required grade {}",
                score.grade, min_grade_str
            ));
        }
    }

    results.score = Some(score);
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    // ===== grade_from_score_100 tests =====

    #[test]
    fn test_grade_from_score_100_all_boundaries() {
        // Top boundaries for each grade (inclusive)
        assert_eq!(grade_from_score_100(100.0), "A+");
        assert_eq!(grade_from_score_100(95.0), "A+");
        assert_eq!(grade_from_score_100(94.0), "A");
        assert_eq!(grade_from_score_100(90.0), "A");
        assert_eq!(grade_from_score_100(89.0), "A-");
        assert_eq!(grade_from_score_100(85.0), "A-");
        assert_eq!(grade_from_score_100(84.0), "B+");
        assert_eq!(grade_from_score_100(80.0), "B+");
        assert_eq!(grade_from_score_100(79.0), "B");
        assert_eq!(grade_from_score_100(75.0), "B");
        assert_eq!(grade_from_score_100(74.0), "B-");
        assert_eq!(grade_from_score_100(70.0), "B-");
        assert_eq!(grade_from_score_100(69.0), "C+");
        assert_eq!(grade_from_score_100(65.0), "C+");
        assert_eq!(grade_from_score_100(64.0), "C");
        assert_eq!(grade_from_score_100(60.0), "C");
        assert_eq!(grade_from_score_100(59.0), "C-");
        assert_eq!(grade_from_score_100(55.0), "C-");
        assert_eq!(grade_from_score_100(54.0), "D");
        assert_eq!(grade_from_score_100(50.0), "D");
        assert_eq!(grade_from_score_100(49.0), "F");
        assert_eq!(grade_from_score_100(0.0), "F");
    }

    // ===== grade_meets_minimum tests =====

    #[test]
    fn test_grade_meets_minimum_ordering() {
        // A+ meets every grade
        assert_eq!(grade_meets_minimum("A+", "F"), Some(true));
        assert_eq!(grade_meets_minimum("A+", "D"), Some(true));
        assert_eq!(grade_meets_minimum("A+", "C"), Some(true));
        assert_eq!(grade_meets_minimum("A+", "B"), Some(true));
        assert_eq!(grade_meets_minimum("A+", "A"), Some(true));
        assert_eq!(grade_meets_minimum("A+", "A+"), Some(true));
        // F only meets F
        assert_eq!(grade_meets_minimum("F", "F"), Some(true));
        assert_eq!(grade_meets_minimum("F", "D"), Some(false));
        assert_eq!(grade_meets_minimum("F", "A+"), Some(false));
        // Mid-range grade
        assert_eq!(grade_meets_minimum("C+", "C"), Some(true));
        assert_eq!(grade_meets_minimum("C+", "C+"), Some(true));
        assert_eq!(grade_meets_minimum("C+", "B"), Some(false));
        // Unknown grades return None
        assert_eq!(grade_meets_minimum("Z", "A"), None);
        assert_eq!(grade_meets_minimum("A", "Z"), None);
    }

    // ===== compute_combined_score tests =====

    #[test]
    fn test_compute_combined_score_cases() {
        // Perfect: 10.0*10*0.6 + 100.0*0.4 = 100.0
        assert!((compute_combined_score(10.0, 100.0) - 100.0).abs() < 0.001);
        // Zero: 0.0
        assert!((compute_combined_score(0.0, 0.0) - 0.0).abs() < 0.001);
        // Mixed: 10.0*10*0.6 + 50.0*0.4 = 80.0
        assert!((compute_combined_score(10.0, 50.0) - 80.0).abs() < 0.001);
        // Static only: 8.0*10*0.6 + 0.0*0.4 = 48.0
        assert!((compute_combined_score(8.0, 0.0) - 48.0).abs() < 0.001);
    }

    // ===== compute_size_score tests =====

    #[test]
    fn test_compute_size_score_cases() {
        // Under 5GB with no limit → 100
        assert!((compute_size_score(1_000_000_000, u64::MAX) - 100.0).abs() < 0.001);
        // Exactly 5GB with no limit → 100
        assert!((compute_size_score(5_000_000_000, u64::MAX) - 100.0).abs() < 0.001);
        // 10GB with no limit → 50 (ratio 2.0, 100/2.0)
        assert!((compute_size_score(10_000_000_000, u64::MAX) - 50.0).abs() < 0.001);
        // Over limit → 0
        assert!((compute_size_score(2_000_000_000, 1_000_000_000) - 0.0).abs() < 0.001);
        // At limit: ratio=1.0, >0.8 branch: (1.0-1.0)*500 = 0
        assert!((compute_size_score(1_000_000_000, 1_000_000_000) - 0.0).abs() < 0.001);
        // Half of limit: ratio=0.5, <0.8 branch: 100-(0.5*50)=75
        assert!((compute_size_score(500_000_000, 1_000_000_000) - 75.0).abs() < 0.001);
        // 90% of limit: ratio=0.9, >0.8 branch: (1.0-0.9)*500=50
        assert!((compute_size_score(900_000_000, 1_000_000_000) - 50.0).abs() < 0.001);
    }

    // ===== compute_layer_score tests =====

    #[test]
    fn test_compute_layer_score_cases() {
        // ≤5 layers, no bloat → 100
        assert!((compute_layer_score(3, 0) - 100.0).abs() < 0.001);
        assert!((compute_layer_score(5, 0) - 100.0).abs() < 0.001);
        // 6-10 layers, no bloat → 80
        assert!((compute_layer_score(6, 0) - 80.0).abs() < 0.001);
        assert!((compute_layer_score(10, 0) - 80.0).abs() < 0.001);
        // >10 layers, no bloat → 60
        assert!((compute_layer_score(11, 0) - 60.0).abs() < 0.001);
        // 5 layers, 2 bloat: 100-40=60
        assert!((compute_layer_score(5, 2) - 60.0).abs() < 0.001);
        // 5 layers, 6 bloat: 100-120 clamped to 0
        assert!((compute_layer_score(5, 6) - 0.0).abs() < 0.001);
        // 11 layers, 1 bloat: 60-20=40
        assert!((compute_layer_score(11, 1) - 40.0).abs() < 0.001);
    }

    // ===== AuditResults + audit_check_lint tests =====

    #[test]
    fn test_audit_results_default_passes() {
        let results = AuditResults::new();
        assert!(results.overall_pass);
        assert!(results.parse_success);
        assert!(results.failure_reason.is_none());
    }

    #[test]
    fn test_audit_check_lint_cases() {
        // No issues → pass
        let mut r = AuditResults::new();
        audit_check_lint(&mut r, false);
        assert!(r.overall_pass);

        // Lint errors → fail
        let mut r = AuditResults::new();
        r.lint_errors = 3;
        audit_check_lint(&mut r, false);
        assert!(!r.overall_pass);
        assert!(r.failure_reason.unwrap().contains("3 lint errors"));

        // Warnings in non-strict → pass
        let mut r = AuditResults::new();
        r.lint_warnings = 5;
        audit_check_lint(&mut r, false);
        assert!(r.overall_pass);

        // Warnings in strict → fail
        let mut r = AuditResults::new();
        r.lint_warnings = 2;
        audit_check_lint(&mut r, true);
        assert!(!r.overall_pass);
        assert!(r.failure_reason.unwrap().contains("Strict mode: 2 warnings"));

        // Both errors and warnings in strict → fail (reason is set)
        let mut r = AuditResults::new();
        r.lint_errors = 1;
        r.lint_warnings = 3;
        audit_check_lint(&mut r, true);
        assert!(!r.overall_pass);
        assert!(r.failure_reason.is_some());
    }

    // ===== RuntimeScore grade tests =====

    #[test]
    fn test_runtime_score_grade_boundaries() {
        let make_score = |s: f64| RuntimeScore {
            score: s,
            estimated_size: 0,
            size_score: 0.0,
            layer_score: 0.0,
            bloat_count: 0,
            docker_available: false,
            suggestions: vec![],
        };

        assert_eq!(make_score(100.0).grade(), "A+");
        assert_eq!(make_score(95.0).grade(), "A+");
        assert_eq!(make_score(94.0).grade(), "A");
        assert_eq!(make_score(90.0).grade(), "A");
        assert_eq!(make_score(80.0).grade(), "B+");
        assert_eq!(make_score(75.0).grade(), "B");
        assert_eq!(make_score(70.0).grade(), "B-");
        assert_eq!(make_score(60.0).grade(), "C");
        assert_eq!(make_score(50.0).grade(), "D");
        assert_eq!(make_score(40.0).grade(), "F");
        assert_eq!(make_score(0.0).grade(), "F");
    }
}
