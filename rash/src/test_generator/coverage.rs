//! Coverage Tracking and Analysis

use std::collections::{HashSet, HashMap};
use super::core::GeneratedTestSuite;

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
    function_coverage: HashMap<String, FunctionCoverage>,
}

#[derive(Debug, Clone)]
struct FunctionCoverage {
    lines: HashSet<usize>,
    branches: HashSet<BranchId>,
    total_lines: usize,
    total_branches: usize,
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
            for assertion in &test.assertions {
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
            r#"
Quality Report
==============
Formatting: {}
Clippy: {}
Coverage: {:.1}%
Mutation Score: {:.1}%
Quality Gates: {}

{}
"#,
            if self.fmt_passed { "✅ PASS" } else { "❌ FAIL" },
            if self.clippy_passed { "✅ PASS" } else { "❌ FAIL" },
            self.coverage_percentage,
            self.mutation_score,
            if self.meets_quality_gates { "✅ PASS" } else { "❌ FAIL" },
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
}
