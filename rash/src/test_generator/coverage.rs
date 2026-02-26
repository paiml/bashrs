//! Coverage Tracking and Analysis

use super::core::GeneratedTestSuite;
use std::collections::{HashMap, HashSet};

/// Unique identifier for a code branch
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BranchId {
    pub function: String,
    pub line: usize,
    pub branch_type: BranchType,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum BranchType {
    IfThen,
    IfElse,
    WhileBody,
    ForBody,
    CaseArm(usize),
}

/// Tracks code coverage for generated tests
pub struct CoverageTracker {
    lines_covered: HashSet<usize>,
    branches_covered: HashSet<BranchId>,
    total_lines: usize,
    total_branches: usize,
    #[allow(dead_code)]
    function_coverage: HashMap<String, FunctionCoverage>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FunctionCoverage {
    lines: HashSet<usize>,
    branches: HashSet<BranchId>,
    total_lines: usize,
    total_branches: usize,
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageTracker {
    pub fn new() -> Self {
        Self {
            lines_covered: HashSet::new(),
            branches_covered: HashSet::new(),
            total_lines: 0,
            total_branches: 0,
            function_coverage: HashMap::new(),
        }
    }

    /// Analyze test suite and calculate coverage
    pub fn analyze(&mut self, suite: &GeneratedTestSuite) {
        // For each unit test, determine which lines/branches it covers
        for test in &suite.unit_tests {
            // Extract coverage information from test assertions
            for _assertion in &test.assertions {
                // Mark lines as covered based on assertion targets
                // This is a simplified version - real implementation would
                // use AST analysis or instrumentation
                self.mark_covered(test.name.as_str());
            }
        }

        // Property tests provide additional coverage
        for _prop_test in &suite.property_tests {
            // Property tests tend to cover more edge cases
            // so they increase coverage confidence
        }
    }

    fn mark_covered(&mut self, _test_name: &str) {
        // Simplified - real implementation would track actual execution
        // For now, assume each test covers at least one line
        self.lines_covered.insert(1);
    }

    /// Calculate line coverage percentage
    pub fn coverage_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            return 100.0; // No code to cover
        }

        (self.lines_covered.len() as f64 / self.total_lines as f64) * 100.0
    }

    /// Calculate branch coverage percentage
    pub fn branch_coverage(&self) -> f64 {
        if self.total_branches == 0 {
            return 100.0; // No branches to cover
        }

        (self.branches_covered.len() as f64 / self.total_branches as f64) * 100.0
    }

    /// Check if coverage meets target
    pub fn is_sufficient(&self, target: f64) -> bool {
        self.coverage_percentage() >= target
    }

    /// Get uncovered code paths
    pub fn uncovered_paths(&self) -> Vec<UncoveredPath> {
        let mut uncovered = Vec::new();

        // Find uncovered lines
        for line in 1..=self.total_lines {
            if !self.lines_covered.contains(&line) {
                uncovered.push(UncoveredPath::Line(line));
            }
        }

        // Find uncovered branches
        // TODO: Track branches from AST analysis

        uncovered
    }

    /// Set total lines to track
    pub fn set_total_lines(&mut self, total: usize) {
        self.total_lines = total;
    }

    /// Set total branches to track
    pub fn set_total_branches(&mut self, total: usize) {
        self.total_branches = total;
    }
}

/// Represents an uncovered code path
#[derive(Debug, Clone)]
pub enum UncoveredPath {
    Line(usize),
    Branch(BranchId),
    Function(String),
}

/// Quality metrics report
#[derive(Debug, Clone)]
pub struct QualityReport {
    pub fmt_passed: bool,
    pub clippy_passed: bool,
    pub coverage_percentage: f64,
    pub mutation_score: f64,
    pub meets_quality_gates: bool,
    pub suggestions: Vec<String>,
}

impl QualityReport {
    pub fn display(&self) -> String {
        format!(
            r"
Quality Report
==============
Formatting: {}
Clippy: {}
Coverage: {:.1}%
Mutation Score: {:.1}%
Quality Gates: {}

{}
",
            if self.fmt_passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            },
            if self.clippy_passed {
                "✅ PASS"
            } else {
                "❌ FAIL"
            },
            self.coverage_percentage,
            self.mutation_score,
            if self.meets_quality_gates {
                "✅ PASS"
            } else {
                "❌ FAIL"
            },
            self.suggestions.join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_tracker_creation() {
        let tracker = CoverageTracker::new();
        assert_eq!(tracker.coverage_percentage(), 100.0); // No code = 100% coverage
    }

    #[test]
    fn test_coverage_tracker_default() {
        let tracker = CoverageTracker::default();
        assert_eq!(tracker.coverage_percentage(), 100.0);
    }

    #[test]
    fn test_coverage_calculation() {
        let mut tracker = CoverageTracker::new();
        tracker.set_total_lines(100);

        // Initially 0%
        assert_eq!(tracker.coverage_percentage(), 0.0);

        // Mark 50 lines as covered
        for i in 1..=50 {
            tracker.lines_covered.insert(i);
        }

        assert_eq!(tracker.coverage_percentage(), 50.0);
    }

    #[test]
    fn test_is_sufficient() {
        let mut tracker = CoverageTracker::new();
        tracker.set_total_lines(100);

        for i in 1..=80 {
            tracker.lines_covered.insert(i);
        }

        assert!(tracker.is_sufficient(80.0));
        assert!(!tracker.is_sufficient(85.0));
    }

    #[test]
    fn test_quality_report_display() {
        let report = QualityReport {
            fmt_passed: true,
            clippy_passed: true,
            coverage_percentage: 95.2,
            mutation_score: 91.3,
            meets_quality_gates: true,
            suggestions: vec!["All green!".to_string()],
        };

        let display = report.display();
        assert!(display.contains("95.2%"));
        assert!(display.contains("91.3%"));
        assert!(display.contains("✅"));
    }

    // ============== BranchId tests ==============

    #[test]
    fn test_branch_id_creation() {
        let branch = BranchId {
            function: "my_func".to_string(),
            line: 42,
            branch_type: BranchType::IfThen,
        };
        assert_eq!(branch.function, "my_func");
        assert_eq!(branch.line, 42);
    }

    #[test]
    fn test_branch_id_debug() {
        let branch = BranchId {
            function: "test".to_string(),
            line: 10,
            branch_type: BranchType::IfElse,
        };
        let debug_str = format!("{:?}", branch);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("IfElse"));
    }

    #[test]
    fn test_branch_id_clone() {
        let branch = BranchId {
            function: "func".to_string(),
            line: 5,
            branch_type: BranchType::WhileBody,
        };
        let cloned = branch.clone();
        assert_eq!(branch, cloned);
    }

    #[test]
    fn test_branch_id_hash() {
        use std::collections::HashSet;

        let branch1 = BranchId {
            function: "func".to_string(),
            line: 5,
            branch_type: BranchType::ForBody,
        };
        let branch2 = BranchId {
            function: "func".to_string(),
            line: 5,
            branch_type: BranchType::ForBody,
        };
        let branch3 = BranchId {
            function: "other".to_string(),
            line: 5,
            branch_type: BranchType::ForBody,
        };

        let mut set = HashSet::new();
        set.insert(branch1.clone());
        assert!(set.contains(&branch2));
        assert!(!set.contains(&branch3));
    }

    // ============== BranchType tests ==============

    #[test]
    fn test_branch_type_if_then() {
        let bt = BranchType::IfThen;
        let debug = format!("{:?}", bt);
        assert!(debug.contains("IfThen"));
    }

    #[test]
    fn test_branch_type_if_else() {
        let bt = BranchType::IfElse;
        let debug = format!("{:?}", bt);
        assert!(debug.contains("IfElse"));
    }

    #[test]
    fn test_branch_type_while_body() {
        let bt = BranchType::WhileBody;
        let debug = format!("{:?}", bt);
        assert!(debug.contains("WhileBody"));
    }

    #[test]
    fn test_branch_type_for_body() {
        let bt = BranchType::ForBody;
        let debug = format!("{:?}", bt);
        assert!(debug.contains("ForBody"));
    }

    #[test]
    fn test_branch_type_case_arm() {
        let bt = BranchType::CaseArm(3);
        let debug = format!("{:?}", bt);
        assert!(debug.contains("CaseArm"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn test_branch_type_clone() {
        let bt = BranchType::CaseArm(5);
        let cloned = bt.clone();
        assert_eq!(bt, cloned);
    }

    #[test]
    fn test_branch_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(BranchType::IfThen);
        set.insert(BranchType::IfElse);
        set.insert(BranchType::CaseArm(1));
        set.insert(BranchType::CaseArm(2));

        assert!(set.contains(&BranchType::IfThen));
        assert!(set.contains(&BranchType::CaseArm(1)));
        assert!(!set.contains(&BranchType::ForBody));
    }

    // ============== CoverageTracker tests ==============

    #[test]
    fn test_coverage_tracker_branch_coverage_empty() {
        let tracker = CoverageTracker::new();
        // No branches means 100% branch coverage
        assert_eq!(tracker.branch_coverage(), 100.0);
    }

    #[test]
    fn test_coverage_tracker_branch_coverage() {
        let mut tracker = CoverageTracker::new();
        tracker.set_total_branches(10);

        // Initially 0%
        assert_eq!(tracker.branch_coverage(), 0.0);

        // Cover 5 branches
        for i in 0..5 {
            tracker.branches_covered.insert(BranchId {
                function: format!("func_{}", i),
                line: i,
                branch_type: BranchType::IfThen,
            });
        }

        assert_eq!(tracker.branch_coverage(), 50.0);
    }

    #[test]
    fn test_coverage_tracker_set_total_lines() {
        let mut tracker = CoverageTracker::new();
        tracker.set_total_lines(100);
        assert_eq!(tracker.total_lines, 100);
    }

    #[test]
    fn test_coverage_tracker_set_total_branches() {
        let mut tracker = CoverageTracker::new();
        tracker.set_total_branches(50);
        assert_eq!(tracker.total_branches, 50);
    }

    #[test]
    fn test_coverage_tracker_uncovered_paths_empty() {
        let tracker = CoverageTracker::new();
        let uncovered = tracker.uncovered_paths();
        assert!(uncovered.is_empty());
    }

    #[test]
    fn test_coverage_tracker_uncovered_paths() {
        let mut tracker = CoverageTracker::new();
        tracker.set_total_lines(5);
        tracker.lines_covered.insert(1);
        tracker.lines_covered.insert(3);

        let uncovered = tracker.uncovered_paths();
        assert_eq!(uncovered.len(), 3); // Lines 2, 4, 5
    }

    #[test]
    fn test_coverage_tracker_analyze() {
        use crate::test_generator::core::GeneratedTestSuite;

        let suite = GeneratedTestSuite::new();

        let mut tracker = CoverageTracker::new();
        tracker.analyze(&suite);
        // Should complete without panic
    }

    #[test]
    fn test_coverage_tracker_analyze_with_tests() {
        use crate::test_generator::core::GeneratedTestSuite;
        use crate::test_generator::unit_tests::{Assertion, UnitTest};

        let mut suite = GeneratedTestSuite::new();
        suite.unit_tests.push(UnitTest {
            name: "test_example".to_string(),
            test_fn: "example".to_string(),
            assertions: vec![Assertion::Equals {
                actual: "result".to_string(),
                expected: "42".to_string(),
            }],
        });

        let mut tracker = CoverageTracker::new();
        tracker.analyze(&suite);

        // Should have covered at least one line
        assert!(tracker.lines_covered.contains(&1));
    }

    // ============== UncoveredPath tests ==============

    #[test]
    fn test_uncovered_path_line() {
        let path = UncoveredPath::Line(42);
        let debug = format!("{:?}", path);
        assert!(debug.contains("Line"));
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_uncovered_path_branch() {
        let branch_id = BranchId {
            function: "test".to_string(),
            line: 10,
            branch_type: BranchType::IfThen,
        };
        let path = UncoveredPath::Branch(branch_id);
        let debug = format!("{:?}", path);
        assert!(debug.contains("Branch"));
    }

    #[test]
    fn test_uncovered_path_function() {
        let path = UncoveredPath::Function("my_function".to_string());
        let debug = format!("{:?}", path);
        assert!(debug.contains("Function"));
        assert!(debug.contains("my_function"));
    }

    #[test]
    fn test_uncovered_path_clone() {
        let path = UncoveredPath::Line(5);
        let cloned = path.clone();
        // Both should format the same
        assert_eq!(format!("{:?}", path), format!("{:?}", cloned));
    }

    // ============== QualityReport tests ==============

    #[test]
    fn test_quality_report_display_fail() {
        let report = QualityReport {
            fmt_passed: false,
            clippy_passed: false,
            coverage_percentage: 50.0,
            mutation_score: 40.0,
            meets_quality_gates: false,
            suggestions: vec!["Fix formatting".to_string(), "Add tests".to_string()],
        };

        let display = report.display();
        assert!(display.contains("50.0%"));
        assert!(display.contains("40.0%"));
        assert!(display.contains("❌"));
        assert!(display.contains("Fix formatting"));
        assert!(display.contains("Add tests"));
    }

    #[test]
    fn test_quality_report_clone() {
        let report = QualityReport {
            fmt_passed: true,
            clippy_passed: true,
            coverage_percentage: 90.0,
            mutation_score: 85.0,
            meets_quality_gates: true,
            suggestions: vec![],
        };

        let cloned = report.clone();
        assert_eq!(report.fmt_passed, cloned.fmt_passed);
        assert_eq!(report.clippy_passed, cloned.clippy_passed);
        assert_eq!(report.coverage_percentage, cloned.coverage_percentage);
        assert_eq!(report.mutation_score, cloned.mutation_score);
        assert_eq!(report.meets_quality_gates, cloned.meets_quality_gates);
    }

    #[test]
    fn test_quality_report_debug() {
        let report = QualityReport {
            fmt_passed: true,
            clippy_passed: false,
            coverage_percentage: 75.5,
            mutation_score: 60.2,
            meets_quality_gates: false,
            suggestions: vec!["suggestion1".to_string()],
        };

        let debug = format!("{:?}", report);
        assert!(debug.contains("fmt_passed"));
        assert!(debug.contains("clippy_passed"));
        assert!(debug.contains("75.5"));
    }

    #[test]
    fn test_quality_report_empty_suggestions() {
        let report = QualityReport {
            fmt_passed: true,
            clippy_passed: true,
            coverage_percentage: 100.0,
            mutation_score: 100.0,
            meets_quality_gates: true,
            suggestions: vec![],
        };

        let display = report.display();
        assert!(display.contains("100.0%"));
    }

    #[test]
    fn test_quality_report_multiple_suggestions() {
        let report = QualityReport {
            fmt_passed: true,
            clippy_passed: true,
            coverage_percentage: 85.0,
            mutation_score: 90.0,
            meets_quality_gates: true,
            suggestions: vec![
                "Add edge case tests".to_string(),
                "Consider mutation testing".to_string(),
                "Improve documentation".to_string(),
            ],
        };

        let display = report.display();
        assert!(display.contains("Add edge case tests"));
        assert!(display.contains("Consider mutation testing"));
        assert!(display.contains("Improve documentation"));
    }
}
