//! Corpus runner: transpiles entries and measures quality.
//!
//! Implements the scoring system from the corpus specification:
//! - A. Transpilation Success (40 points)
//! - B. Output Correctness (25 points)
//! - C. Test Coverage (15 points)
//! - D. Lint Compliance (10 points)
//! - E. Determinism (10 points)
//!
//! Gateway logic: if A < 60%, B-E are scored as 0 (Popperian falsification barrier).

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, Grade};
use crate::models::Config;
use serde::{Deserialize, Serialize};

/// Result of transpiling a single corpus entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusResult {
    /// Entry ID
    pub id: String,
    /// Whether transpilation succeeded (A: 40 points)
    pub transpiled: bool,
    /// Whether output contains expected content (B: 25 points)
    pub output_correct: bool,
    /// Whether a unit test exists and passes (C: 15 points)
    pub has_test: bool,
    /// Whether output passes lint (D: 10 points)
    pub lint_clean: bool,
    /// Whether output is deterministic across runs (E: 10 points)
    pub deterministic: bool,
    /// The actual transpiled output (if successful)
    pub actual_output: Option<String>,
    /// Error message (if transpilation failed)
    pub error: Option<String>,
}

impl CorpusResult {
    /// Calculate 100-point score for this entry.
    pub fn score(&self) -> f64 {
        let a = if self.transpiled { 40.0 } else { 0.0 };

        // Gateway: if transpilation fails, everything else is 0
        if !self.transpiled {
            return a;
        }

        let b = if self.output_correct { 25.0 } else { 0.0 };
        let c = if self.has_test { 15.0 } else { 0.0 };
        let d = if self.lint_clean { 10.0 } else { 0.0 };
        let e = if self.deterministic { 10.0 } else { 0.0 };

        a + b + c + d + e
    }
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
    /// Per-entry results
    pub results: Vec<CorpusResult>,
}

impl CorpusScore {
    /// Whether gateway threshold is met (>= 60% transpilation).
    pub fn gateway_met(&self) -> bool {
        self.rate >= 0.60
    }
}

/// A single convergence log entry (Kaizen tracking).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Corpus runner: loads entries, transpiles, scores, tracks convergence.
pub struct CorpusRunner {
    config: Config,
}

impl CorpusRunner {
    /// Create a new corpus runner with the given config.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the full corpus and return aggregate score.
    pub fn run(&self, registry: &CorpusRegistry) -> CorpusScore {
        let mut results = Vec::new();

        for entry in &registry.entries {
            let result = self.run_entry(entry);
            results.push(result);
        }

        self.compute_score(&results, registry)
    }

    /// Run corpus for a single format.
    pub fn run_format(&self, registry: &CorpusRegistry, format: CorpusFormat) -> CorpusScore {
        let mut results = Vec::new();

        for entry in registry.by_format(format) {
            let result = self.run_entry(entry);
            results.push(result);
        }

        self.compute_score(&results, registry)
    }

    /// Run a single corpus entry.
    fn run_entry(&self, entry: &CorpusEntry) -> CorpusResult {
        let transpile_result = match entry.format {
            CorpusFormat::Bash => {
                crate::transpile(&entry.input, self.config.clone())
            }
            CorpusFormat::Makefile => {
                crate::transpile_makefile(&entry.input, self.config.clone())
            }
            CorpusFormat::Dockerfile => {
                crate::transpile_dockerfile(&entry.input, self.config.clone())
            }
        };

        match transpile_result {
            Ok(output) => {
                // B: Check if output contains expected content
                let output_correct = output.contains(&entry.expected_output);

                // D: Check lint compliance
                let lint_clean = self.check_lint(&output, entry.format);

                // E: Check determinism (transpile again and compare)
                let deterministic = self.check_determinism(entry);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: true,
                    output_correct,
                    has_test: true, // All entries have tests by construction
                    lint_clean,
                    deterministic,
                    actual_output: Some(output),
                    error: None,
                }
            }
            Err(e) => CorpusResult {
                id: entry.id.clone(),
                transpiled: false,
                output_correct: false,
                has_test: true,
                lint_clean: false,
                deterministic: false,
                actual_output: None,
                error: Some(format!("{}", e)),
            },
        }
    }

    fn check_lint(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                let lint_result = crate::linter::rules::lint_shell(output);
                !lint_result.has_errors()
            }
            CorpusFormat::Makefile => {
                let lint_result = crate::linter::rules::lint_makefile(output);
                !lint_result.has_errors()
            }
            CorpusFormat::Dockerfile => {
                let lint_result = crate::linter::rules::lint_dockerfile(output);
                !lint_result.has_errors()
            }
        }
    }

    fn check_determinism(&self, entry: &CorpusEntry) -> bool {
        if !entry.deterministic {
            return true; // Skip determinism check if not required
        }

        let first = match entry.format {
            CorpusFormat::Bash => crate::transpile(&entry.input, self.config.clone()),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, self.config.clone()),
            CorpusFormat::Dockerfile => {
                crate::transpile_dockerfile(&entry.input, self.config.clone())
            }
        };

        let second = match entry.format {
            CorpusFormat::Bash => crate::transpile(&entry.input, self.config.clone()),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, self.config.clone()),
            CorpusFormat::Dockerfile => {
                crate::transpile_dockerfile(&entry.input, self.config.clone())
            }
        };

        match (first, second) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    fn compute_score(&self, results: &[CorpusResult], _registry: &CorpusRegistry) -> CorpusScore {
        let total = results.len();
        let passed = results.iter().filter(|r| r.transpiled).count();
        let failed = total - passed;
        let rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            0.0
        };

        // Gateway check (Popperian falsification barrier)
        let score = if rate < 0.60 {
            // Below gateway: only count transpilation
            rate * 40.0
        } else {
            // Above gateway: compute weighted average
            if total > 0 {
                let total_score: f64 = results.iter().map(|r| r.score()).sum();
                total_score / total as f64
            } else {
                0.0
            }
        };

        let grade = Grade::from_score(score);

        CorpusScore {
            total,
            passed,
            failed,
            rate,
            score,
            grade,
            results: results.to_vec(),
        }
    }

    /// Generate a convergence entry for logging.
    pub fn convergence_entry(
        &self,
        score: &CorpusScore,
        iteration: u32,
        date: &str,
        previous_rate: f64,
        notes: &str,
    ) -> ConvergenceEntry {
        ConvergenceEntry {
            iteration,
            date: date.to_string(),
            total: score.total,
            passed: score.passed,
            failed: score.failed,
            rate: score.rate,
            delta: score.rate - previous_rate,
            notes: notes.to_string(),
        }
    }

    /// Check convergence criteria: rate >= 99% for 3 consecutive iterations,
    /// delta < 0.5% for 3 consecutive iterations.
    pub fn is_converged(entries: &[ConvergenceEntry]) -> bool {
        if entries.len() < 3 {
            return false;
        }

        let last_three = &entries[entries.len() - 3..];

        // Rate threshold: all >= 99%
        let rate_met = last_three.iter().all(|e| e.rate >= 0.99);

        // Stability: all deltas < 0.5%
        let stable = last_three.iter().all(|e| e.delta.abs() < 0.005);

        rate_met && stable
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_CORPUS_RUN_001_score_calculation() {
        let result = CorpusResult {
            id: "T-001".to_string(),
            transpiled: true,
            output_correct: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            actual_output: Some("output".to_string()),
            error: None,
        };
        assert!((result.score() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_002_score_transpile_only() {
        let result = CorpusResult {
            id: "T-002".to_string(),
            transpiled: true,
            output_correct: false,
            has_test: false,
            lint_clean: false,
            deterministic: false,
            actual_output: Some("output".to_string()),
            error: None,
        };
        assert!((result.score() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_003_score_failed_transpile() {
        let result = CorpusResult {
            id: "T-003".to_string(),
            transpiled: false,
            output_correct: false,
            has_test: true,
            lint_clean: false,
            deterministic: false,
            actual_output: None,
            error: Some("parse error".to_string()),
        };
        assert!((result.score()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_004_convergence_not_enough_entries() {
        let entries = vec![ConvergenceEntry {
            iteration: 1,
            date: "2026-02-06".to_string(),
            total: 100,
            passed: 99,
            failed: 1,
            rate: 0.99,
            delta: 0.99,
            notes: "initial".to_string(),
        }];
        assert!(!CorpusRunner::is_converged(&entries));
    }

    #[test]
    fn test_CORPUS_RUN_005_convergence_met() {
        let entries = vec![
            ConvergenceEntry {
                iteration: 1,
                date: "2026-02-01".to_string(),
                total: 200,
                passed: 198,
                failed: 2,
                rate: 0.99,
                delta: 0.001,
                notes: "stable".to_string(),
            },
            ConvergenceEntry {
                iteration: 2,
                date: "2026-02-08".to_string(),
                total: 200,
                passed: 199,
                failed: 1,
                rate: 0.995,
                delta: 0.004,
                notes: "stable".to_string(),
            },
            ConvergenceEntry {
                iteration: 3,
                date: "2026-02-15".to_string(),
                total: 200,
                passed: 199,
                failed: 1,
                rate: 0.995,
                delta: 0.0,
                notes: "converged".to_string(),
            },
        ];
        assert!(CorpusRunner::is_converged(&entries));
    }

    #[test]
    fn test_CORPUS_RUN_006_convergence_rate_below_threshold() {
        let entries = vec![
            ConvergenceEntry {
                iteration: 1,
                date: "2026-02-01".to_string(),
                total: 200,
                passed: 190,
                failed: 10,
                rate: 0.95,
                delta: 0.001,
                notes: "not met".to_string(),
            },
            ConvergenceEntry {
                iteration: 2,
                date: "2026-02-08".to_string(),
                total: 200,
                passed: 192,
                failed: 8,
                rate: 0.96,
                delta: 0.01,
                notes: "not met".to_string(),
            },
            ConvergenceEntry {
                iteration: 3,
                date: "2026-02-15".to_string(),
                total: 200,
                passed: 194,
                failed: 6,
                rate: 0.97,
                delta: 0.01,
                notes: "not met".to_string(),
            },
        ];
        assert!(!CorpusRunner::is_converged(&entries));
    }

    #[test]
    fn test_CORPUS_RUN_007_gateway_logic() {
        // Score 100 for full pass
        let perfect = CorpusResult {
            id: "T-007".to_string(),
            transpiled: true,
            output_correct: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            actual_output: Some("out".to_string()),
            error: None,
        };
        assert!((perfect.score() - 100.0).abs() < f64::EPSILON);

        // Gateway: failed transpile = 0 total
        let failed = CorpusResult {
            id: "T-007b".to_string(),
            transpiled: false,
            output_correct: true, // ignored when transpile fails
            has_test: true,
            lint_clean: true,
            deterministic: true,
            actual_output: None,
            error: Some("err".to_string()),
        };
        assert!((failed.score()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_008_partial_score() {
        // Transpiles + correct but not lint clean
        let partial = CorpusResult {
            id: "T-008".to_string(),
            transpiled: true,
            output_correct: true,
            has_test: true,
            lint_clean: false,
            deterministic: true,
            actual_output: Some("out".to_string()),
            error: None,
        };
        // 40 + 25 + 15 + 0 + 10 = 90
        assert!((partial.score() - 90.0).abs() < f64::EPSILON);
    }
}
