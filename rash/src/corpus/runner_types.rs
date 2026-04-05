//! Corpus runner data types: result structs, scoring, convergence, regression.
//!
//! Extracted from runner.rs for file size health (PMAT).

use crate::corpus::registry::{CorpusFormat, Grade};
use serde::{Deserialize, Serialize};

/// Result of transpiling a single corpus entry (v2 scoring).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorpusResult {
    /// Entry ID
    pub id: String,
    /// Whether transpilation succeeded (A: 30 points)
    pub transpiled: bool,
    /// B_L1: Whether output contains expected content (10 points)
    pub output_contains: bool,
    /// B_L2: Whether trimmed output lines match expected exactly (8 points)
    pub output_exact: bool,
    /// B_L3: Reserved for execution-based behavioral equivalence (7 points)
    pub output_behavioral: bool,
    /// Whether a unit test exists for this entry (legacy binary detection)
    pub has_test: bool,
    /// Real LLVM coverage ratio for this entry's format (0.0-1.0, V2-8)
    /// C_score = coverage_ratio × 15
    #[serde(default)]
    pub coverage_ratio: f64,
    /// Whether output conforms to format schema (hard gate: 0 if false)
    pub schema_valid: bool,
    /// Whether output passes lint (D: 10 points)
    pub lint_clean: bool,
    /// Whether output is deterministic across runs (E: 10 points)
    pub deterministic: bool,
    /// F: Metamorphic relation consistency (5 points)
    pub metamorphic_consistent: bool,
    /// G: Reserved for cross-shell agreement (5 points)
    pub cross_shell_agree: bool,
    /// The expected output string from the corpus entry (for diagnostics)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<String>,
    /// The actual transpiled output (if successful)
    pub actual_output: Option<String>,
    /// Error message (if transpilation failed)
    pub error: Option<String>,
    /// ML-classified error category (when oracle feature enabled and entry failed)
    pub error_category: Option<String>,
    /// Confidence of error classification (0.0 - 1.0)
    pub error_confidence: Option<f32>,
    /// Decision trace from the emitter (for fault localization)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_trace: Option<Vec<crate::emitter::trace::TranspilerDecision>>,
}

impl CorpusResult {
    /// Calculate 100-point score for this entry (v2 formula).
    pub fn score(&self) -> f64 {
        let a = if self.transpiled { 30.0 } else { 0.0 };

        // Gateway: if transpilation fails, everything else is 0
        if !self.transpiled {
            return a;
        }

        // Schema hard gate: if output is not structurally valid, score is 0
        if !self.schema_valid {
            return 0.0;
        }

        // B: Output correctness (3 levels, 25 points total)
        let b_l1 = if self.output_contains { 10.0 } else { 0.0 };
        // Secondary gate: if L1 fails, L2 and L3 are 0
        let b_l2 = if self.output_contains && self.output_exact {
            8.0
        } else {
            0.0
        };
        let b_l3 = if self.output_contains && self.output_behavioral {
            7.0
        } else {
            0.0
        };

        // C: real LLVM coverage ratio (V2-8 spec §11.4)
        // C_score = coverage_ratio × 15 (replaces binary has_test)
        let c = self.coverage_ratio * 15.0;
        let d = if self.lint_clean { 10.0 } else { 0.0 };
        let e = if self.deterministic { 10.0 } else { 0.0 };
        let f = if self.metamorphic_consistent {
            5.0
        } else {
            0.0
        };
        let g = if self.cross_shell_agree { 5.0 } else { 0.0 };

        a + b_l1 + b_l2 + b_l3 + c + d + e + f + g
    }

    /// Legacy score method for backward compatibility during migration.
    /// Returns score on the original 100-point scale (A=40, B=25, C=15, D=10, E=10).
    pub fn score_v1(&self) -> f64 {
        let a = if self.transpiled { 40.0 } else { 0.0 };
        if !self.transpiled {
            return a;
        }
        let b = if self.output_contains { 25.0 } else { 0.0 };
        let c = self.coverage_ratio * 15.0;
        let d = if self.lint_clean { 10.0 } else { 0.0 };
        let e = if self.deterministic { 10.0 } else { 0.0 };
        a + b + c + d + e
    }
}

/// Per-format score breakdown (spec §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatScore {
    /// Format name
    pub format: CorpusFormat,
    /// Number of entries in this format
    pub total: usize,
    /// Number that transpiled successfully
    pub passed: usize,
    /// Transpilation rate
    pub rate: f64,
    /// Average v2 score for this format
    pub score: f64,
    /// Quality grade for this format
    pub grade: Grade,
}

/// Aggregate score for a corpus run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusScore {
    /// Total entries in corpus
    pub total: usize,
    /// Entries that transpiled successfully
    pub passed: usize,
    /// Entries that failed transpilation
    pub failed: usize,
    /// Transpilation success rate (0.0 - 1.0)
    pub rate: f64,
    /// Weighted aggregate score (0-100)
    pub score: f64,
    /// Quality grade
    pub grade: Grade,
    /// Per-format score breakdowns (spec §11.3)
    pub format_scores: Vec<FormatScore>,
    /// Per-entry results
    pub results: Vec<CorpusResult>,
}

impl CorpusScore {
    /// Whether gateway threshold is met (>= 60% transpilation).
    pub fn gateway_met(&self) -> bool {
        self.rate >= 0.60
    }

    /// Get format-specific score breakdown.
    pub fn format_score(&self, format: CorpusFormat) -> Option<&FormatScore> {
        self.format_scores.iter().find(|fs| fs.format == format)
    }
}

/// A single convergence log entry (Kaizen tracking).
/// Per-format fields (spec §11.10.5) enable format-specific regression detection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConvergenceEntry {
    /// Iteration number
    pub iteration: u32,
    /// Date of measurement
    pub date: String,
    /// Total entries in corpus
    pub total: usize,
    /// Entries that passed
    pub passed: usize,
    /// Entries that failed
    pub failed: usize,
    /// Transpilation rate
    pub rate: f64,
    /// Delta from previous iteration
    pub delta: f64,
    /// Notes about this iteration
    pub notes: String,
    // --- Per-format breakdown (spec §11.10.5) ---
    /// Bash entries that passed
    #[serde(default)]
    pub bash_passed: usize,
    /// Bash entries total
    #[serde(default)]
    pub bash_total: usize,
    /// Makefile entries that passed
    #[serde(default)]
    pub makefile_passed: usize,
    /// Makefile entries total
    #[serde(default)]
    pub makefile_total: usize,
    /// Dockerfile entries that passed
    #[serde(default)]
    pub dockerfile_passed: usize,
    /// Dockerfile entries total
    #[serde(default)]
    pub dockerfile_total: usize,
    // --- V2 score/grade (spec §5.1) ---
    /// V2 weighted score (0-100)
    #[serde(default)]
    pub score: f64,
    /// Quality grade string (e.g. "A+", "A", "B", ...)
    #[serde(default)]
    pub grade: String,
    /// Per-format V2 scores (spec §11.10.5)
    #[serde(default)]
    pub bash_score: f64,
    #[serde(default)]
    pub makefile_score: f64,
    #[serde(default)]
    pub dockerfile_score: f64,
    // --- Lint pass rate (spec §7.5) ---
    /// Entries that passed lint (D dimension)
    #[serde(default)]
    pub lint_passed: usize,
    /// Lint pass rate (0.0-1.0)
    #[serde(default)]
    pub lint_rate: f64,
}

/// A single regression finding (spec §5.3 -- Jidoka).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    /// Human-readable description of the regression
    pub message: String,
    /// Dimension that regressed (e.g. "score", "bash_passed")
    pub dimension: String,
    /// Previous value
    pub previous: f64,
    /// Current value
    pub current: f64,
}

/// Result of comparing current corpus run against previous convergence entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    /// List of detected regressions (empty = no regressions)
    pub regressions: Vec<Regression>,
}

impl RegressionReport {
    /// True if any regressions were detected.
    pub fn has_regressions(&self) -> bool {
        !self.regressions.is_empty()
    }
}

impl ConvergenceEntry {
    /// Compare current entry against a previous entry to detect regressions (spec §5.3).
    /// Returns a report listing all dimensions that regressed.
    pub fn detect_regressions(&self, previous: &ConvergenceEntry) -> RegressionReport {
        let mut regressions = Vec::new();
        let mut check = |dim: &str, prev: f64, curr: f64, label: &str| {
            if curr < prev {
                regressions.push(Regression {
                    message: format!("{label}: {prev} \u{2192} {curr}"),
                    dimension: dim.to_string(),
                    previous: prev,
                    current: curr,
                });
            }
        };
        check("score", previous.score, self.score, "V2 score dropped");
        check(
            "passed",
            previous.passed as f64,
            self.passed as f64,
            "Total passed dropped",
        );
        check(
            "bash_passed",
            previous.bash_passed as f64,
            self.bash_passed as f64,
            "Bash passed dropped",
        );
        check(
            "makefile_passed",
            previous.makefile_passed as f64,
            self.makefile_passed as f64,
            "Makefile passed dropped",
        );
        check(
            "dockerfile_passed",
            previous.dockerfile_passed as f64,
            self.dockerfile_passed as f64,
            "Dockerfile passed dropped",
        );
        check(
            "bash_score",
            previous.bash_score,
            self.bash_score,
            "Bash score dropped",
        );
        check(
            "makefile_score",
            previous.makefile_score,
            self.makefile_score,
            "Makefile score dropped",
        );
        check(
            "dockerfile_score",
            previous.dockerfile_score,
            self.dockerfile_score,
            "Dockerfile score dropped",
        );
        check(
            "lint_passed",
            previous.lint_passed as f64,
            self.lint_passed as f64,
            "Lint passed dropped",
        );
        RegressionReport { regressions }
    }
}
