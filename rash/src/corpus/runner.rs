//! Corpus runner: transpiles entries and measures quality.
//!
//! Implements the v2 scoring system from the corpus specification:
//! - A. Transpilation Success (30 points)
//! - B. Output Correctness: L1 containment (10) + L2 exact match (8) + L3 behavioral (7)
//! - C. Test Coverage (15 points) -- real LLVM coverage ratio per format (V2-8)
//! - D. Lint Compliance (10 points)
//! - E. Determinism (10 points)
//! - F. Metamorphic Consistency (5 points) -- MR-1 through MR-7
//! - G. Cross-shell agreement (5 points)
//!
//! Gateway logic: if A < 60%, B-G are scored as 0 (Popperian falsification barrier).
//! Secondary gate: if B_L1 < 60%, B_L2 and B_L3 are scored as 0.
//!
//! Split into:
//! - `runner_types.rs`: Data types (CorpusResult, FormatScore, CorpusScore, etc.)
//! - `runner_helpers.rs`: Free functions for coverage detection, error classification
//! - `runner_checks.rs`: Validation methods (MR relations, schema, lint, behavioral)
//! - `runner.rs` (this file): CorpusRunner struct and core run/compute/convergence methods

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, Grade};
use crate::models::Config;
use std::collections::HashMap;

// Re-export types so `crate::corpus::runner::TypeName` paths keep working.
pub(crate) use super::runner_helpers::{
    check_exact_match, classify_error, detect_coverage_ratio, detect_test_exists,
    extract_test_names, format_file_patterns, parse_lcov_file_coverage,
};
pub use super::runner_types::{
    ConvergenceEntry, CorpusResult, CorpusScore, FormatScore, Regression, RegressionReport,
};

/// Corpus runner: loads entries, transpiles, scores, tracks convergence.
pub struct CorpusRunner {
    pub(crate) config: Config,
}

impl CorpusRunner {
    /// Create a new corpus runner with the given config.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the full corpus and return aggregate score.
    ///
    /// KAIZEN-080: Parallelized with std::thread::scope -- each thread processes
    /// a chunk of entries independently. CorpusRunner is Send+Sync (Config is scalar,
    /// OnceLock caches are thread-safe, run_entry takes &self).
    pub fn run(&self, registry: &CorpusRegistry) -> CorpusScore {
        let entry_refs: Vec<&CorpusEntry> = registry.entries.iter().collect();
        let results = self.run_entries_parallel(&entry_refs);
        // KAIZEN-071: pass owned Vec to avoid cloning 17,942 CorpusResult structs
        self.compute_score(results, registry)
    }

    /// Run corpus for a single format.
    ///
    /// KAIZEN-080: Parallelized -- collects format entries then dispatches to thread pool.
    pub fn run_format(&self, registry: &CorpusRegistry, format: CorpusFormat) -> CorpusScore {
        let entries: Vec<&CorpusEntry> = registry.by_format(format);
        let results = self.run_entries_parallel(&entries);
        self.compute_score(results, registry)
    }

    /// Run entries in parallel using std::thread::scope.
    ///
    /// Contract:
    /// - Pre: entries is a slice of corpus entry references
    /// - Post: returns Vec<CorpusResult> with len == entries.len(), in same order
    /// - Invariant: no shared mutable state -- each run_entry call is independent
    fn run_entries_parallel(&self, entries: &[&CorpusEntry]) -> Vec<CorpusResult> {
        if entries.is_empty() {
            return Vec::new();
        }

        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // For small entry counts or single-thread systems, skip thread overhead
        if entries.len() < n_threads * 2 || n_threads <= 1 {
            return entries.iter().map(|e| self.run_entry(e)).collect();
        }

        let chunk_size = entries.len().div_ceil(n_threads);
        let chunks: Vec<&[&CorpusEntry]> = entries.chunks(chunk_size).collect();

        std::thread::scope(|s| {
            let handles: Vec<_> = chunks
                .into_iter()
                .map(|chunk| {
                    s.spawn(move || chunk.iter().map(|e| self.run_entry(e)).collect::<Vec<_>>())
                })
                .collect();

            let mut results = Vec::with_capacity(entries.len());
            for handle in handles {
                results.extend(handle.join().expect("corpus runner thread panicked"));
            }
            results
        })
    }

    /// Run a single corpus entry and return its detailed result.
    pub fn run_single(&self, entry: &CorpusEntry) -> CorpusResult {
        self.run_entry(entry)
    }

    /// Run a single entry with decision tracing enabled.
    /// For Bash entries, uses `transpile_with_trace()` to collect emitter decisions.
    /// Makefile/Dockerfile entries fall back to the normal path (no trace).
    pub fn run_entry_with_trace(&self, entry: &CorpusEntry) -> CorpusResult {
        if entry.format != CorpusFormat::Bash {
            return self.run_entry(entry);
        }

        let transpile_result = crate::transpile_with_trace(&entry.input, &self.config);

        match transpile_result {
            Ok((output, trace)) => {
                let schema_valid = self.check_schema(&output, entry.format);
                let output_contains = output.contains(&entry.expected_output);
                let output_exact = check_exact_match(&output, &entry.expected_output);
                let output_behavioral = self.check_behavioral(&output, entry.format);
                let coverage_ratio = detect_coverage_ratio(entry.format, &entry.id);
                let has_test = coverage_ratio > 0.0 || detect_test_exists(&entry.id);
                let lint_clean = self.check_lint(&output, entry.format);
                // KAIZEN-070: reuse output from run_entry_with_trace
                let deterministic = self.check_determinism_with_output(entry, &output);
                // KAIZEN-072: pass output_contains to MR checks to avoid re-transpiling original
                let metamorphic_consistent = deterministic
                    && self.check_mr2_stability(entry, output_contains)
                    && self.check_mr3_whitespace(entry, output_contains)
                    && self.check_mr4_leading_blanks(entry, output_contains)
                    && self.check_mr5_subsumption(entry)
                    && self.check_mr6_composition(entry)
                    && self.check_mr7_negation(entry);
                // KAIZEN-073/074: pass output + behavioral result
                let cross_shell_agree =
                    self.check_cross_shell_with_output(entry, &output, output_behavioral);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: true,
                    output_contains,
                    output_exact,
                    output_behavioral,
                    schema_valid,
                    has_test,
                    coverage_ratio,
                    lint_clean,
                    deterministic,
                    metamorphic_consistent,
                    cross_shell_agree,
                    expected_output: Some(entry.expected_output.clone()),
                    actual_output: Some(output),
                    error: None,
                    error_category: None,
                    error_confidence: None,
                    decision_trace: Some(trace),
                }
            }
            Err(e) => {
                let error_msg = format!("{e}");
                let (error_category, error_confidence) = classify_error(&error_msg);
                let cov = detect_coverage_ratio(entry.format, &entry.id);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: false,
                    output_contains: false,
                    output_exact: false,
                    output_behavioral: false,
                    schema_valid: false,
                    has_test: cov > 0.0 || detect_test_exists(&entry.id),
                    coverage_ratio: cov,
                    lint_clean: false,
                    deterministic: false,
                    metamorphic_consistent: false,
                    cross_shell_agree: false,
                    expected_output: Some(entry.expected_output.clone()),
                    actual_output: None,
                    error: Some(error_msg),
                    error_category,
                    error_confidence,
                    decision_trace: None,
                }
            }
        }
    }

    /// Run a single corpus entry with v2 multi-level correctness checking.
    fn run_entry(&self, entry: &CorpusEntry) -> CorpusResult {
        let transpile_result = match entry.format {
            CorpusFormat::Bash => crate::transpile(&entry.input, &self.config),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, &self.config),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(&entry.input, &self.config),
        };

        match transpile_result {
            Ok(output) => {
                // Schema hard gate: validate output conforms to format grammar
                let schema_valid = self.check_schema(&output, entry.format);

                // B_L1: Containment check (original metric)
                let output_contains = output.contains(&entry.expected_output);

                // B_L2: Exact match -- check if expected appears as exact trimmed lines
                let output_exact = check_exact_match(&output, &entry.expected_output);

                // B_L3: Behavioral equivalence -- execute transpiled shell and verify exit 0
                let output_behavioral = self.check_behavioral(&output, entry.format);

                // C: Coverage ratio (V2-8) -- real LLVM coverage or test name fallback
                let coverage_ratio = detect_coverage_ratio(entry.format, &entry.id);
                let has_test = coverage_ratio > 0.0 || detect_test_exists(&entry.id);

                // D: Check lint compliance
                let lint_clean = self.check_lint(&output, entry.format);

                // E: Check determinism (transpile again and compare)
                // KAIZEN-070: pass first output to avoid redundant re-transpilation
                let deterministic = self.check_determinism_with_output(entry, &output);

                // F: Metamorphic consistency -- all MR properties must hold
                //    MR-1: determinism (already checked as E)
                //    MR-2: stability under no-op comment addition
                //    MR-3: trailing whitespace invariance
                //    MR-4: leading blank line invariance
                //    MR-5: subsumption (simplification preserves transpilability)
                //    MR-6: composition (independent stmts transpile separately)
                //    MR-7: negation (negated condition still transpiles)
                // KAIZEN-072: pass output_contains to MR checks to avoid re-transpiling original
                let metamorphic_consistent = deterministic
                    && self.check_mr2_stability(entry, output_contains)
                    && self.check_mr3_whitespace(entry, output_contains)
                    && self.check_mr4_leading_blanks(entry, output_contains)
                    && self.check_mr5_subsumption(entry)
                    && self.check_mr6_composition(entry)
                    && self.check_mr7_negation(entry);

                // G: Cross-shell agreement -- for bash entries, verify output
                // equivalence across Posix and Bash dialect configs
                // KAIZEN-073: pass output to avoid re-transpiling with matching dialect
                // KAIZEN-074: pass behavioral result to skip redundant sh execution
                let cross_shell_agree =
                    self.check_cross_shell_with_output(entry, &output, output_behavioral);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: true,
                    output_contains,
                    output_exact,
                    output_behavioral,
                    schema_valid,
                    has_test,
                    coverage_ratio,
                    lint_clean,
                    deterministic,
                    metamorphic_consistent,
                    cross_shell_agree,
                    expected_output: Some(entry.expected_output.clone()),
                    actual_output: Some(output),
                    error: None,
                    error_category: None,
                    error_confidence: None,
                    decision_trace: None,
                }
            }
            Err(e) => {
                let error_msg = format!("{e}");
                let (error_category, error_confidence) = classify_error(&error_msg);
                let cov = detect_coverage_ratio(entry.format, &entry.id);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: false,
                    output_contains: false,
                    output_exact: false,
                    output_behavioral: false,
                    schema_valid: false,
                    has_test: cov > 0.0 || detect_test_exists(&entry.id),
                    coverage_ratio: cov,
                    lint_clean: false,
                    deterministic: false,
                    metamorphic_consistent: false,
                    cross_shell_agree: false,
                    expected_output: Some(entry.expected_output.clone()),
                    actual_output: None,
                    error: Some(error_msg),
                    error_category,
                    error_confidence,
                    decision_trace: None,
                }
            }
        }
    }

    fn compute_score(&self, results: Vec<CorpusResult>, registry: &CorpusRegistry) -> CorpusScore {
        let total = results.len();
        let passed = results.iter().filter(|r| r.transpiled).count();
        let failed = total - passed;
        let rate = if total > 0 {
            passed as f64 / total as f64
        } else {
            0.0
        };

        // Gateway check (Popperian falsification barrier, spec SS11.4)
        let score = if rate < 0.60 {
            // Below gateway: only count transpilation component (A=30 max)
            rate * 30.0
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

        // Per-format breakdowns (spec SS11.3)
        let format_scores = self.compute_format_scores(&results, registry);

        // KAIZEN-071: move owned Vec directly instead of cloning
        CorpusScore {
            total,
            passed,
            failed,
            rate,
            score,
            grade,
            format_scores,
            results,
        }
    }

    fn compute_format_scores(
        &self,
        results: &[CorpusResult],
        registry: &CorpusRegistry,
    ) -> Vec<FormatScore> {
        let mut scores = Vec::new();

        // KAIZEN-075: O(1) format lookup instead of O(n) linear search per result.
        // Was ~322M string comparisons per format (17,942^2 x 3 = ~966M total).
        let format_by_id: HashMap<&str, CorpusFormat> = registry
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e.format))
            .collect();

        for format in &[
            CorpusFormat::Bash,
            CorpusFormat::Makefile,
            CorpusFormat::Dockerfile,
        ] {
            let format_results: Vec<&CorpusResult> = results
                .iter()
                .filter(|r| format_by_id.get(r.id.as_str()) == Some(format))
                .collect();

            if format_results.is_empty() {
                continue;
            }

            let ft = format_results.len();
            let fp = format_results.iter().filter(|r| r.transpiled).count();
            let fr = if ft > 0 { fp as f64 / ft as f64 } else { 0.0 };
            let fs = if ft > 0 {
                let ts: f64 = format_results.iter().map(|r| r.score()).sum();
                ts / ft as f64
            } else {
                0.0
            };

            scores.push(FormatScore {
                format: *format,
                total: ft,
                passed: fp,
                rate: fr,
                score: fs,
                grade: Grade::from_score(fs),
            });
        }

        scores
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
        // Extract per-format stats from format_scores (spec SS11.10.5)
        let (bash_passed, bash_total) = score
            .format_score(CorpusFormat::Bash)
            .map_or((0, 0), |fs| (fs.passed, fs.total));
        let (makefile_passed, makefile_total) = score
            .format_score(CorpusFormat::Makefile)
            .map_or((0, 0), |fs| (fs.passed, fs.total));
        let (dockerfile_passed, dockerfile_total) = score
            .format_score(CorpusFormat::Dockerfile)
            .map_or((0, 0), |fs| (fs.passed, fs.total));

        let bash_score = score
            .format_score(CorpusFormat::Bash)
            .map_or(0.0, |fs| fs.score);
        let makefile_score = score
            .format_score(CorpusFormat::Makefile)
            .map_or(0.0, |fs| fs.score);
        let dockerfile_score = score
            .format_score(CorpusFormat::Dockerfile)
            .map_or(0.0, |fs| fs.score);

        let lint_passed = score.results.iter().filter(|r| r.lint_clean).count();
        let lint_rate = if score.total > 0 {
            lint_passed as f64 / score.total as f64
        } else {
            0.0
        };

        ConvergenceEntry {
            iteration,
            date: date.to_string(),
            total: score.total,
            passed: score.passed,
            failed: score.failed,
            rate: score.rate,
            delta: score.rate - previous_rate,
            notes: notes.to_string(),
            bash_passed,
            bash_total,
            makefile_passed,
            makefile_total,
            dockerfile_passed,
            dockerfile_total,
            score: score.score,
            grade: score.grade.to_string(),
            bash_score,
            makefile_score,
            dockerfile_score,
            lint_passed,
            lint_rate,
        }
    }

    /// Append a convergence entry to a JSONL log file.
    /// Creates parent directories if needed.
    pub fn append_convergence_log(
        entry: &ConvergenceEntry,
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        use std::io::Write;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let json = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        writeln!(file, "{json}")?;
        Ok(())
    }

    /// Load convergence entries from a JSONL log file.
    /// Returns empty vec if file does not exist.
    pub fn load_convergence_log(path: &std::path::Path) -> std::io::Result<Vec<ConvergenceEntry>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut entries = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: ConvergenceEntry = serde_json::from_str(trimmed)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            entries.push(entry);
        }
        Ok(entries)
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
#[path = "runner_tests.rs"]
mod tests;
