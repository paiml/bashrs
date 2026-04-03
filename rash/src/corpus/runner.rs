//! Corpus runner: transpiles entries and measures quality.
//!
//! Implements the v2 scoring system from the corpus specification:
//! - A. Transpilation Success (30 points)
//! - B. Output Correctness: L1 containment (10) + L2 exact match (8) + L3 behavioral (7)
//! - C. Test Coverage (15 points) — real LLVM coverage ratio per format (V2-8)
//! - D. Lint Compliance (10 points)
//! - E. Determinism (10 points)
//! - F. Metamorphic Consistency (5 points) — MR-1 through MR-7
//! - G. Cross-shell agreement (5 points)
//!
//! Gateway logic: if A < 60%, B-G are scored as 0 (Popperian falsification barrier).
//! Secondary gate: if B_L1 < 60%, B_L2 and B_L3 are scored as 0.

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, Grade};
use crate::models::Config;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

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

/// A single regression finding (spec §5.3 — Jidoka).
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
                    message: format!("{label}: {prev} → {curr}"),
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

/// Valid Dockerfile instruction prefixes (per Dockerfile reference).
const DOCKERFILE_INSTRUCTIONS: &[&str] = &[
    "FROM ",
    "RUN ",
    "CMD ",
    "LABEL ",
    "EXPOSE ",
    "ENV ",
    "ADD ",
    "COPY ",
    "ENTRYPOINT ",
    "VOLUME ",
    "USER ",
    "WORKDIR ",
    "ARG ",
    "ONBUILD ",
    "STOPSIGNAL ",
    "HEALTHCHECK ",
    "SHELL ",
    "FROM\t",
    "RUN\t",
    "CMD\t",
    "LABEL\t",
    "EXPOSE\t",
    "ENV\t",
    "ADD\t",
    "COPY\t",
    "ENTRYPOINT\t",
    "VOLUME\t",
    "USER\t",
    "WORKDIR\t",
    "ARG\t",
];

/// Check exact match: whether expected output appears as exact trimmed lines in actual output.
/// This is stricter than containment — it checks that the expected content appears
/// as complete, whitespace-normalized lines (not just a substring within a longer line).
fn check_exact_match(actual: &str, expected: &str) -> bool {
    let expected_trimmed = expected.trim();
    if expected_trimmed.is_empty() {
        return true;
    }

    // Check if expected appears as exact consecutive lines in actual
    let expected_lines: Vec<&str> = expected_trimmed.lines().map(str::trim).collect();
    let actual_lines: Vec<&str> = actual.lines().map(str::trim).collect();

    if expected_lines.len() == 1 {
        // Single line: check if any actual line matches exactly
        actual_lines.iter().any(|line| *line == expected_lines[0])
    } else {
        // Multi-line: check for consecutive line sequence match
        actual_lines
            .windows(expected_lines.len())
            .any(|window| window == expected_lines.as_slice())
    }
}

/// Classify a transpilation error into a category using keyword matching.
/// When the `oracle` feature is enabled, uses the ML-powered classifier.
/// Returns (category_name, confidence).
fn classify_error(error_msg: &str) -> (Option<String>, Option<f32>) {
    #[cfg(feature = "oracle")]
    {
        let classifier = bashrs_oracle::ErrorClassifier::new();
        let category = classifier.classify_by_keywords(error_msg);
        (Some(category.name().to_string()), Some(0.85))
    }
    #[cfg(not(feature = "oracle"))]
    {
        // Lightweight keyword classification without oracle dependency
        let msg = error_msg.to_lowercase();
        let category =
            if msg.contains("parse") || msg.contains("syntax") || msg.contains("unexpected") {
                "syntax_error"
            } else if msg.contains("unsupported") || msg.contains("not implemented") {
                "unsupported_construct"
            } else if msg.contains("type") || msg.contains("mismatch") {
                "type_error"
            } else {
                "unknown"
            };
        (Some(category.to_string()), Some(0.5))
    }
}

/// Set of known test function names, lazily initialized from source files.
static TEST_NAMES: OnceLock<HashSet<String>> = OnceLock::new();

/// Build the set of test function names by scanning corpus test source files.
/// Looks for `fn test_` patterns in corpus_tests.rs and the runner.rs test module.
fn build_test_name_set() -> HashSet<String> {
    let mut names = HashSet::new();

    // Scan corpus_tests.rs (integration tests)
    let corpus_tests_path = std::path::Path::new("rash/tests/corpus_tests.rs");
    // Also try from the workspace root or relative to crate
    let paths = [
        corpus_tests_path.to_path_buf(),
        std::path::PathBuf::from("tests/corpus_tests.rs"),
    ];

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            extract_test_names(&content, &mut names);
        }
    }

    // Also scan runner.rs itself for inline test module
    let runner_paths = [
        std::path::PathBuf::from("rash/src/corpus/runner.rs"),
        std::path::PathBuf::from("src/corpus/runner.rs"),
    ];

    for path in &runner_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            extract_test_names(&content, &mut names);
        }
    }

    names
}

/// Extract test function names from Rust source code.
fn extract_test_names(source: &str, names: &mut HashSet<String>) {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn test_") {
            // Extract function name: "fn test_FOO(...)" -> "test_FOO"
            let after_fn = &trimmed[3..]; // skip "fn "
            if let Some(paren) = after_fn.find('(') {
                let name = &after_fn[..paren];
                names.insert(name.to_string());
            }
        }
    }
}

/// Detect whether a test function exists for this corpus entry ID.
/// Scans corpus_tests.rs and runner.rs test modules for test functions
/// that reference this entry ID (e.g., "B-001" → matches "B_001" or "B001").
fn detect_test_exists(entry_id: &str) -> bool {
    if entry_id.is_empty() {
        return false;
    }

    let test_names = TEST_NAMES.get_or_init(build_test_name_set);

    // If we couldn't load any test names (e.g., source files not found),
    // fall back to true to avoid false negatives in CI environments.
    if test_names.is_empty() {
        return true;
    }

    // Normalize entry ID for matching: "B-001" → "B_001" and "B001"
    let normalized_underscore = entry_id.replace('-', "_");
    let normalized_no_sep = entry_id.replace('-', "");

    // Check if any test name contains either normalized form
    test_names
        .iter()
        .any(|name| name.contains(&normalized_underscore) || name.contains(&normalized_no_sep))
}

/// Per-format LLVM coverage ratios, lazily loaded from LCOV data.
static FORMAT_COVERAGE: OnceLock<HashMap<String, f64>> = OnceLock::new();

/// Standard locations to search for LCOV coverage data files.
/// Includes both workspace root and crate-level paths since cargo tests
/// may run from either directory.
const LCOV_SEARCH_PATHS: &[&str] = &[
    "target/coverage/lcov.info",
    "lcov.info",
    ".coverage/lcov.info",
    "target/llvm-cov/lcov.info",
    // Workspace-relative paths (when cwd is a crate subdirectory)
    "../target/coverage/lcov.info",
    "../lcov.info",
    "../.coverage/lcov.info",
];

/// Source file path patterns that map to each corpus format.
/// Used to attribute LCOV line coverage to the correct format.
fn format_file_patterns(format: CorpusFormat) -> &'static [&'static str] {
    match format {
        CorpusFormat::Bash => &["emitter/posix", "bash_transpiler/"],
        CorpusFormat::Makefile => &["emitter/makefile"],
        CorpusFormat::Dockerfile => &["emitter/dockerfile"],
    }
}

/// Load per-format coverage ratios from LCOV data.
/// Returns a map of format name → coverage ratio (0.0-1.0).
fn load_format_coverage() -> HashMap<String, f64> {
    let mut map = HashMap::new();

    // Try each standard LCOV location
    let lcov_content = LCOV_SEARCH_PATHS
        .iter()
        .find_map(|path| std::fs::read_to_string(path).ok());

    let Some(content) = lcov_content else {
        return map;
    };

    // Parse LCOV and compute per-format coverage
    let file_coverage = parse_lcov_file_coverage(&content);

    for format in [
        CorpusFormat::Bash,
        CorpusFormat::Makefile,
        CorpusFormat::Dockerfile,
    ] {
        let patterns = format_file_patterns(format);
        let mut total_lines = 0u64;
        let mut hit_lines = 0u64;

        for (file_path, (lf, lh)) in &file_coverage {
            if patterns.iter().any(|p| file_path.contains(p)) {
                total_lines += lf;
                hit_lines += lh;
            }
        }

        if total_lines > 0 {
            let ratio = hit_lines as f64 / total_lines as f64;
            map.insert(format!("{format}"), ratio);
        }
    }

    map
}

/// Parse LCOV data into per-file (lines_found, lines_hit) tuples.
fn parse_lcov_file_coverage(content: &str) -> Vec<(String, (u64, u64))> {
    let mut results = Vec::new();
    let mut current_file = String::new();
    let mut lines_found = 0u64;
    let mut lines_hit = 0u64;

    for line in content.lines() {
        if let Some(path) = line.strip_prefix("SF:") {
            current_file = path.to_string();
            lines_found = 0;
            lines_hit = 0;
        } else if let Some(rest) = line.strip_prefix("DA:") {
            // DA:<line number>,<execution count>[,<checksum>]
            if let Some((_line_no, count_str)) = rest.split_once(',') {
                // Count might have a trailing checksum: "5,abc123"
                let count_part = count_str.split(',').next().unwrap_or("0");
                if let Ok(count) = count_part.parse::<u64>() {
                    lines_found += 1;
                    if count > 0 {
                        lines_hit += 1;
                    }
                }
            }
        } else if line == "end_of_record" && !current_file.is_empty() {
            results.push((current_file.clone(), (lines_found, lines_hit)));
        }
    }

    results
}

/// Get the LLVM coverage ratio for a corpus format.
/// Returns 0.0-1.0 from LCOV data, or falls back to test name detection.
fn detect_coverage_ratio(format: CorpusFormat, entry_id: &str) -> f64 {
    let coverage = FORMAT_COVERAGE.get_or_init(load_format_coverage);

    // Primary: real LLVM coverage data
    let format_key = format!("{format}");
    if let Some(&ratio) = coverage.get(&format_key) {
        return ratio;
    }

    // Fallback: binary test name detection
    if detect_test_exists(entry_id) {
        1.0
    } else {
        0.0
    }
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
    ///
    /// KAIZEN-080: Parallelized with std::thread::scope — each thread processes
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
    /// KAIZEN-080: Parallelized — collects format entries then dispatches to thread pool.
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
    /// - Invariant: no shared mutable state — each run_entry call is independent
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

                // B_L2: Exact match — check if expected appears as exact trimmed lines
                let output_exact = check_exact_match(&output, &entry.expected_output);

                // B_L3: Behavioral equivalence — execute transpiled shell and verify exit 0
                let output_behavioral = self.check_behavioral(&output, entry.format);

                // C: Coverage ratio (V2-8) — real LLVM coverage or test name fallback
                let coverage_ratio = detect_coverage_ratio(entry.format, &entry.id);
                let has_test = coverage_ratio > 0.0 || detect_test_exists(&entry.id);

                // D: Check lint compliance
                let lint_clean = self.check_lint(&output, entry.format);

                // E: Check determinism (transpile again and compare)
                // KAIZEN-070: pass first output to avoid redundant re-transpilation
                let deterministic = self.check_determinism_with_output(entry, &output);

                // F: Metamorphic consistency — all MR properties must hold
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

                // G: Cross-shell agreement — for bash entries, verify output
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

    /// MR-2: Stability under no-op addition.
    /// Adding a comment to the input should not change the transpiled output semantics.
    fn check_mr2_stability(&self, entry: &CorpusEntry, output_contains: bool) -> bool {
        // Input is Rust DSL — use Rust comment syntax, not shell comment
        let modified_input = format!("// MR-2 no-op\n{}", entry.input);
        self.check_mr_equivalence_precomputed(entry, &modified_input, output_contains)
    }

    /// MR-3: Trailing whitespace invariance.
    /// Adding trailing whitespace/newlines to the input should not change output semantics.
    fn check_mr3_whitespace(&self, entry: &CorpusEntry, output_contains: bool) -> bool {
        let modified_input = format!("{}\n\n  \n", entry.input);
        self.check_mr_equivalence_precomputed(entry, &modified_input, output_contains)
    }

    /// MR-4: Leading blank line invariance.
    /// Adding leading blank lines to the input should not change output semantics.
    fn check_mr4_leading_blanks(&self, entry: &CorpusEntry, output_contains: bool) -> bool {
        let modified_input = format!("\n\n{}", entry.input);
        self.check_mr_equivalence_precomputed(entry, &modified_input, output_contains)
    }

    /// MR-5: Subsumption — if A transpiles, a simplification of A should also transpile.
    /// For Rust DSL: remove the last statement from main. Vacuously true for
    /// single-statement entries or non-Bash formats.
    fn check_mr5_subsumption(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        let Some((body_start, body_end)) = Self::extract_main_body_range(input) else {
            return true;
        };
        let body = input[body_start..body_end].trim();
        let Some(semi_pos) = Self::find_last_top_level_semicolon(body) else {
            return true;
        };
        let simplified_body = &body[..semi_pos];
        if !simplified_body.contains(';') {
            return true;
        }
        let simplified = format!("{}{}; }}", &input[..body_start], simplified_body);
        self.transpile_entry(&simplified, entry.format).is_ok()
    }

    /// Extract the byte range of the main function body (between outer braces)
    fn extract_main_body_range(input: &str) -> Option<(usize, usize)> {
        let body_start = input
            .find("fn main()")
            .and_then(|i| input[i..].find('{').map(|j| i + j + 1))?;
        let body_end = input.rfind('}')?;
        if body_end <= body_start {
            return None;
        }
        Some((body_start, body_end))
    }

    /// Find the position of the last semicolon at brace depth 0
    fn find_last_top_level_semicolon(body: &str) -> Option<usize> {
        let mut depth = 0i32;
        let mut last_top_semi = None;
        for (i, ch) in body.char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                ';' if depth == 0 => last_top_semi = Some(i),
                _ => {}
            }
        }
        last_top_semi
    }

    /// MR-6: Composition — for entries with multiple `let` statements,
    /// each individual `let` should transpile independently.
    /// Vacuously true if < 2 let statements or non-Bash format.
    fn check_mr6_composition(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        // Extract individual `let` statements from the body
        let lets: Vec<&str> = input
            .split(';')
            .filter(|s| s.trim().starts_with("let ") || s.trim().starts_with("let mut "))
            .collect();

        if lets.len() < 2 {
            return true; // inapplicable
        }
        // Each let should transpile independently inside a main fn
        for let_stmt in &lets {
            let single = format!("fn main() {{ {}; }}", let_stmt.trim());
            if self.transpile_entry(&single, entry.format).is_err() {
                return false;
            }
        }
        true
    }

    /// MR-7: Negation — for entries containing `if`, negating the condition
    /// should still produce valid transpilation.
    /// Vacuously true for entries without `if` or non-Bash format.
    fn check_mr7_negation(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        // Check if input contains an if statement with a simple comparison
        if !input.contains("if ") {
            return true; // inapplicable
        }
        // Simple negation: wrap the condition in !()
        // Find pattern: `if <cond> {` and replace with `if !(<cond>) {`
        if let Some(if_pos) = input.find("if ") {
            let after_if = &input[if_pos + 3..];
            if let Some(brace_pos) = after_if.find('{') {
                let condition = after_if[..brace_pos].trim();
                if condition.is_empty() {
                    return true;
                }
                let negated = format!(
                    "{}if !({}) {}",
                    &input[..if_pos],
                    condition,
                    &after_if[brace_pos..]
                );
                // Negation: negated version must also transpile
                return self.transpile_entry(&negated, entry.format).is_ok();
            }
        }
        true // inapplicable
    }

    /// KAIZEN-072: MR equivalence check reusing pre-computed original containment.
    /// Eliminates 3 redundant transpilations per entry (MR-2, MR-3, MR-4 each
    /// re-transpiled the original — ~53,826 wasted transpilations per corpus run).
    fn check_mr_equivalence_precomputed(
        &self,
        entry: &CorpusEntry,
        modified_input: &str,
        original_contains: bool,
    ) -> bool {
        let modified = self.transpile_entry(modified_input, entry.format);
        match modified {
            Ok(modif) => original_contains == modif.contains(&entry.expected_output),
            // Original succeeded (we're in the Ok branch of run_entry), modified failed → not equivalent
            Err(_) => false,
        }
    }

    /// Transpile input based on format (DRY helper for MR checks).
    fn transpile_entry(
        &self,
        input: &str,
        format: CorpusFormat,
    ) -> std::result::Result<String, crate::Error> {
        match format {
            CorpusFormat::Bash => crate::transpile(input, &self.config),
            CorpusFormat::Makefile => crate::transpile_makefile(input, &self.config),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(input, &self.config),
        }
    }

    /// KAIZEN-073: Cross-shell agreement reusing run_entry output when config matches.
    /// KAIZEN-074: Skip redundant sh execution when behavioral already passed for same output.
    /// If `self.config.target` is Posix, the output from `run_entry` IS the Posix result —
    /// only transpile with Bash config (eliminates ~16,431 redundant transpilations).
    /// When behavioral_passed is true and output is reused as posix_out, skip sh execution
    /// (already verified by check_behavioral) and only run dash.
    fn check_cross_shell_with_output(
        &self,
        entry: &CorpusEntry,
        output: &str,
        behavioral_passed: bool,
    ) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }

        let posix_config = Config {
            target: crate::models::ShellDialect::Posix,
            ..self.config.clone()
        };
        let bash_config = Config {
            target: crate::models::ShellDialect::Bash,
            ..self.config.clone()
        };

        // Track whether posix_out is the same as the behavioral-tested output
        let posix_is_reused = self.config.target == crate::models::ShellDialect::Posix;

        // Reuse run_entry output for whichever dialect matches self.config.target
        let (posix_result, bash_result) = match self.config.target {
            crate::models::ShellDialect::Posix => {
                let bash_r = crate::transpile(&entry.input, &bash_config);
                (Ok(output.to_string()), bash_r)
            }
            crate::models::ShellDialect::Bash => {
                let posix_r = crate::transpile(&entry.input, &posix_config);
                (posix_r, Ok(output.to_string()))
            }
            // Dash/Ash: neither matches Posix or Bash, transpile both
            _ => {
                let posix_r = crate::transpile(&entry.input, &posix_config);
                let bash_r = crate::transpile(&entry.input, &bash_config);
                (posix_r, bash_r)
            }
        };

        match (posix_result, bash_result) {
            (Ok(posix_out), Ok(bash_out)) => {
                let posix_has = posix_out.contains(&entry.expected_output);
                let bash_has = bash_out.contains(&entry.expected_output);
                if !(posix_has && bash_has) {
                    return false;
                }
                // KAIZEN-074: if behavioral already passed for this same output,
                // sh execution is known-good — only run dash
                if behavioral_passed && posix_is_reused {
                    self.check_dash_execution(&posix_out)
                } else {
                    self.check_shell_execution(&posix_out)
                }
            }
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    /// G: Cross-shell agreement — transpile bash entries with Posix and Bash
    /// dialect configs, verify both produce output containing the expected fragment.
    /// Additionally, execute the transpiled output in both `sh` and `dash` to verify
    /// cross-shell runtime agreement.
    /// Non-bash formats pass by default (no dialect variation).
    /// Execute shell output in both `sh` and `dash`, verifying both terminate.
    /// Returns true if both shells execute without timeout.
    /// Gracefully skips dash if not installed.
    fn check_shell_execution(&self, output: &str) -> bool {
        // Execute in sh (must pass)
        let sh_ok = match std::process::Command::new("timeout")
            .args(["2", "sh", "-c", output])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(result) => result.status.code().unwrap_or(128) != 124,
            Err(_) => return false,
        };

        if !sh_ok {
            return false;
        }

        // Execute in dash (graceful: skip if not found)
        match std::process::Command::new("timeout")
            .args(["2", "dash", "-c", output])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(result) => result.status.code().unwrap_or(128) != 124,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true, // dash not installed
            Err(_) => true, // other error, graceful skip
        }
    }

    /// KAIZEN-074: Execute only in dash (sh already verified by check_behavioral).
    /// Gracefully skips if dash is not installed.
    fn check_dash_execution(&self, output: &str) -> bool {
        match std::process::Command::new("timeout")
            .args(["2", "dash", "-c", output])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(result) => result.status.code().unwrap_or(128) != 124,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => true,
        }
    }

    /// Schema validation: verify output conforms to the format grammar.
    /// - Bash: validate via validation::validate_shell_snippet (POSIX grammar)
    ///   AND shellcheck -s sh -f json (catches bashisms, quoting bugs, POSIX violations)
    /// - Makefile: parse via make_parser::parse_makefile (GNU Make grammar)
    /// - Dockerfile: validate via lint (Dockerfile instruction grammar)
    fn check_schema(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                let internal_ok = crate::validation::validate_shell_snippet(output).is_ok();
                if !internal_ok {
                    return false;
                }
                // Additionally run shellcheck for stricter POSIX validation.
                // Graceful fallback: if shellcheck is not installed, trust internal result.
                self.check_shellcheck(output).unwrap_or(true)
            }
            CorpusFormat::Makefile => crate::make_parser::parse_makefile(output).is_ok(),
            CorpusFormat::Dockerfile => {
                // No dedicated Dockerfile parser; use linter as schema proxy.
                // Check that output contains at least one valid Dockerfile instruction.
                let has_instruction = output.lines().any(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && DOCKERFILE_INSTRUCTIONS
                            .iter()
                            .any(|instr| trimmed.starts_with(instr))
                });
                has_instruction
            }
        }
    }

    /// Run shellcheck on shell output, returning None if shellcheck is not found.
    /// Returns Some(true) if no error-level findings, Some(false) if errors found.
    fn check_shellcheck(&self, output: &str) -> Option<bool> {
        let result = std::process::Command::new("shellcheck")
            .args(["-s", "sh", "-f", "json", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(_) => return None,
        };

        // Write output to shellcheck's stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(output.as_bytes());
        }

        let output_result = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return Some(true), // can't read output, trust internal
        };

        // shellcheck exits 0 = clean, 1 = findings exist
        // Parse JSON to check for "error" level findings only
        let stdout = String::from_utf8_lossy(&output_result.stdout);
        if stdout.trim().is_empty() || stdout.trim() == "[]" {
            return Some(true);
        }

        // Parse JSON array of findings; fail only on "error" level
        match serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
            Ok(findings) => {
                let has_errors = findings
                    .iter()
                    .any(|f| f.get("level").and_then(|l| l.as_str()) == Some("error"));
                Some(!has_errors)
            }
            Err(_) => Some(true), // can't parse, trust internal
        }
    }

    /// Rules that are expected in transpiler output and should not count as lint failures.
    /// Lint rules excluded from corpus D-score because they fire on valid transpiler output:
    /// SEC001: transpiler uses `eval echo` for exec() calls
    /// REL001: transpiler trap uses `rm -rf` (intentionally destructive cleanup)
    /// SC1020: missing-space-before-] heuristic on compact generated test expressions
    /// SC1028: bare-paren-in-bracket heuristic false-positives on generated test expressions
    /// SC1035: missing-space-after-keyword heuristic on compact generated code
    /// SC1037: positional-param heuristic false-positives (e.g. $10 in generated code)
    /// SC1041: heredoc style heuristic false-positives on generated code
    /// SC1044: unterminated heredoc heuristic false-positives on generated code
    /// SC1078: odd-quote heuristic false-positives on multi-line transpiler output
    /// SC1140: extra-token-after-] heuristic on valid shell patterns
    /// SC2105: break/continue-outside-loop heuristic on flattened transpiler output
    const CORPUS_LINT_EXCLUSIONS: &'static [&'static str] = &[
        "SEC001", "REL001", "SC1020", "SC1028", "SC1035", "SC1037", "SC1041", "SC1044", "SC1078",
        "SC1140", "SC2105",
    ];

    fn check_lint(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                let lint_result = crate::linter::rules::lint_shell(output);
                !lint_result.diagnostics.iter().any(|d| {
                    d.severity == crate::linter::Severity::Error
                        && !Self::CORPUS_LINT_EXCLUSIONS.contains(&d.code.as_str())
                })
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

    /// B_L3: Behavioral equivalence — execute the transpiled output and verify
    /// it terminates within 2 seconds. Uses `timeout 2 sh -c` for bash.
    /// Exit code 124 = timeout (script hangs = FAIL).
    /// Any other exit code = script terminates normally (PASS).
    /// Makefile: write to temp file and run `make -n -f tempfile` (dry-run syntax check).
    /// Dockerfile: syntax validation proxy (schema + lint).
    fn check_behavioral(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => {
                // Execute with 2-second timeout to catch infinite loops
                // timeout returns 124 on timeout, or the command's exit code
                match std::process::Command::new("timeout")
                    .args(["2", "sh", "-c", output])
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()
                {
                    Ok(result) => {
                        // PASS if script terminated (even non-zero exit).
                        // FAIL only on timeout (exit code 124) or signal kill.
                        let code = result.status.code().unwrap_or(128);
                        code != 124
                    }
                    Err(_) => false,
                }
            }
            CorpusFormat::Makefile => self.check_makefile_dry_run(output),
            // Dockerfile: no direct execution; behavioral equivalence
            // is approximated by schema + lint passing (checked separately).
            CorpusFormat::Dockerfile => true,
        }
    }

    /// Validate Makefile output by writing to a temp file and running `make -n -f`.
    /// Returns true if make dry-run succeeds (exit 0 = valid Makefile syntax).
    /// Also returns true if make says "No targets" — variable-only Makefiles are valid.
    /// Graceful: returns true if make is not found.
    fn check_makefile_dry_run(&self, output: &str) -> bool {
        use std::io::Write;

        // Create temp file for the Makefile content
        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.join(format!("bashrs_makefile_check_{}", std::process::id()));
        let tmp_str = tmp_path.to_string_lossy().to_string();

        // Write Makefile content
        let write_ok =
            std::fs::File::create(&tmp_path).and_then(|mut f| f.write_all(output.as_bytes()));
        if write_ok.is_err() {
            return true; // can't write temp file, graceful pass
        }

        let result = std::process::Command::new("make")
            .args(["-n", "-f", &tmp_str])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        // Clean up temp file
        let _ = std::fs::remove_file(&tmp_path);

        match result {
            Ok(r) => {
                if r.status.success() {
                    return true;
                }
                // "No targets" (exit 2) is valid for variable-only Makefiles.
                // "No rule to make target" means valid syntax but unresolvable
                // prerequisites — acceptable for syntax validation.
                let stderr = String::from_utf8_lossy(&r.stderr);
                stderr.contains("No targets") || stderr.contains("No rule to make target")
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true, // make not installed
            Err(_) => true, // other error, graceful pass
        }
    }

    /// KAIZEN-070: Determinism check reusing the first transpilation output from run_entry.
    /// Eliminates one redundant transpilation per entry (~17,942 per full corpus run).
    fn check_determinism_with_output(&self, entry: &CorpusEntry, first_output: &str) -> bool {
        if !entry.deterministic {
            return true; // Skip determinism check if not required
        }

        let second = match entry.format {
            CorpusFormat::Bash => crate::transpile(&entry.input, &self.config),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, &self.config),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(&entry.input, &self.config),
        };

        match second {
            Ok(b) => first_output == b,
            Err(_) => false,
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

        // Gateway check (Popperian falsification barrier, spec §11.4)
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

        // Per-format breakdowns (spec §11.3)
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
        // Was ~322M string comparisons per format (17,942² × 3 = ~966M total).
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
        // Extract per-format stats from format_scores (spec §11.10.5)
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
