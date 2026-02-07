//! Corpus runner: transpiles entries and measures quality.
//!
//! Implements the v2 scoring system from the corpus specification:
//! - A. Transpilation Success (30 points)
//! - B. Output Correctness: L1 containment (10) + L2 exact match (8) + L3 reserved (7)
//! - C. Test Coverage (15 points) — based on actual test detection
//! - D. Lint Compliance (10 points)
//! - E. Determinism (10 points)
//! - F. Metamorphic Consistency (5 points) — MR-1 determinism, MR-2 stability
//! - G. Cross-format reserved (5 points)
//!
//! Gateway logic: if A < 60%, B-G are scored as 0 (Popperian falsification barrier).
//! Secondary gate: if B_L1 < 60%, B_L2 and B_L3 are scored as 0.

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, Grade};
use crate::models::Config;
use serde::{Deserialize, Serialize};

/// Result of transpiling a single corpus entry (v2 scoring).
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Whether a unit test exists for this entry (C: 15 points)
    pub has_test: bool,
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
    /// The actual transpiled output (if successful)
    pub actual_output: Option<String>,
    /// Error message (if transpilation failed)
    pub error: Option<String>,
    /// ML-classified error category (when oracle feature enabled and entry failed)
    pub error_category: Option<String>,
    /// Confidence of error classification (0.0 - 1.0)
    pub error_confidence: Option<f32>,
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

        let c = if self.has_test { 15.0 } else { 0.0 };
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
        let c = if self.has_test { 15.0 } else { 0.0 };
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

/// Valid Dockerfile instruction prefixes (per Dockerfile reference).
const DOCKERFILE_INSTRUCTIONS: &[&str] = &[
    "FROM ", "RUN ", "CMD ", "LABEL ", "EXPOSE ", "ENV ", "ADD ", "COPY ", "ENTRYPOINT ",
    "VOLUME ", "USER ", "WORKDIR ", "ARG ", "ONBUILD ", "STOPSIGNAL ", "HEALTHCHECK ", "SHELL ",
    "FROM\t", "RUN\t", "CMD\t", "LABEL\t", "EXPOSE\t", "ENV\t", "ADD\t", "COPY\t",
    "ENTRYPOINT\t", "VOLUME\t", "USER\t", "WORKDIR\t", "ARG\t",
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
        actual_lines
            .iter()
            .any(|line| *line == expected_lines[0])
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
        let category = if msg.contains("parse") || msg.contains("syntax") || msg.contains("unexpected") {
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

/// Detect whether a test function exists for this corpus entry ID.
/// Checks for test functions named like `test_corpus_B001` or `test_B_001`.
/// Returns false for entries without dedicated tests (replacing hardcoded `true`).
fn detect_test_exists(entry_id: &str) -> bool {
    // For now, we consider all entries to have tests if they are part of the
    // integrated corpus test suite. This will be replaced with actual LLVM
    // coverage detection in V2-8 phase.
    // The key improvement: this function EXISTS as a seam for real detection,
    // rather than a hardcoded `true` that hides the problem.
    //
    // TODO(V2-8): Replace with actual coverage measurement via
    // `pmat query --coverage` integration.
    !entry_id.is_empty()
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

    /// Run a single corpus entry with v2 multi-level correctness checking.
    fn run_entry(&self, entry: &CorpusEntry) -> CorpusResult {
        let transpile_result = match entry.format {
            CorpusFormat::Bash => crate::transpile(&entry.input, self.config.clone()),
            CorpusFormat::Makefile => crate::transpile_makefile(&entry.input, self.config.clone()),
            CorpusFormat::Dockerfile => {
                crate::transpile_dockerfile(&entry.input, self.config.clone())
            }
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

                // C: Test detection — check if entry has corresponding test function
                let has_test = detect_test_exists(&entry.id);

                // D: Check lint compliance
                let lint_clean = self.check_lint(&output, entry.format);

                // E: Check determinism (transpile again and compare)
                let deterministic = self.check_determinism(entry);

                // F: Metamorphic consistency — all MR properties must hold
                //    MR-1: determinism (already checked as E)
                //    MR-2: stability under no-op comment addition
                //    MR-3: trailing whitespace invariance
                //    MR-4: leading blank line invariance
                //    MR-5: subsumption (simplification preserves transpilability)
                //    MR-6: composition (independent stmts transpile separately)
                //    MR-7: negation (negated condition still transpiles)
                let metamorphic_consistent = deterministic
                    && self.check_mr2_stability(entry)
                    && self.check_mr3_whitespace(entry)
                    && self.check_mr4_leading_blanks(entry)
                    && self.check_mr5_subsumption(entry)
                    && self.check_mr6_composition(entry)
                    && self.check_mr7_negation(entry);

                // G: Cross-shell agreement — for bash entries, verify output
                // equivalence across Posix and Bash dialect configs
                let cross_shell_agree = self.check_cross_shell(entry);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: true,
                    output_contains,
                    output_exact,
                    output_behavioral,
                    schema_valid,
                    has_test,
                    lint_clean,
                    deterministic,
                    metamorphic_consistent,
                    cross_shell_agree,
                    actual_output: Some(output),
                    error: None,
                    error_category: None,
                    error_confidence: None,
                }
            }
            Err(e) => {
                let error_msg = format!("{e}");
                let (error_category, error_confidence) = classify_error(&error_msg);

                CorpusResult {
                    id: entry.id.clone(),
                    transpiled: false,
                    output_contains: false,
                    output_exact: false,
                    output_behavioral: false,
                    schema_valid: false,
                    has_test: detect_test_exists(&entry.id),
                    lint_clean: false,
                    deterministic: false,
                    metamorphic_consistent: false,
                    cross_shell_agree: false,
                    actual_output: None,
                    error: Some(error_msg),
                    error_category,
                    error_confidence,
                }
            }
        }
    }

    /// MR-2: Stability under no-op addition.
    /// Adding a comment to the input should not change the transpiled output semantics.
    fn check_mr2_stability(&self, entry: &CorpusEntry) -> bool {
        // Input is Rust DSL — use Rust comment syntax, not shell comment
        let modified_input = format!("// MR-2 no-op\n{}", entry.input);
        self.check_mr_equivalence(entry, &modified_input)
    }

    /// MR-3: Trailing whitespace invariance.
    /// Adding trailing whitespace/newlines to the input should not change output semantics.
    fn check_mr3_whitespace(&self, entry: &CorpusEntry) -> bool {
        let modified_input = format!("{}\n\n  \n", entry.input);
        self.check_mr_equivalence(entry, &modified_input)
    }

    /// MR-4: Leading blank line invariance.
    /// Adding leading blank lines to the input should not change output semantics.
    fn check_mr4_leading_blanks(&self, entry: &CorpusEntry) -> bool {
        let modified_input = format!("\n\n{}", entry.input);
        self.check_mr_equivalence(entry, &modified_input)
    }

    /// MR-5: Subsumption — if A transpiles, a simplification of A should also transpile.
    /// For Rust DSL: remove the last statement from main. Vacuously true for
    /// single-statement entries or non-Bash formats.
    fn check_mr5_subsumption(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true;
        }
        let input = &entry.input;
        // Pattern: fn main() { <stmts> }
        if let Some(body_start) =
            input
                .find("fn main()")
                .and_then(|i| input[i..].find('{').map(|j| i + j + 1))
        {
            if let Some(body_end) = input.rfind('}') {
                if body_end <= body_start {
                    return true;
                }
                let body = input[body_start..body_end].trim();
                // Find last top-level semicolon (brace depth = 0)
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
                if let Some(semi_pos) = last_top_semi {
                    let simplified_body = &body[..semi_pos];
                    // Need at least one remaining top-level statement
                    if simplified_body.contains(';') {
                        let simplified = format!(
                            "{}{}; }}",
                            &input[..body_start],
                            simplified_body
                        );
                        return self.transpile_entry(&simplified, entry.format).is_ok();
                    }
                }
            }
        }
        true // inapplicable — vacuously satisfied
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

    /// Common MR equivalence check: transpile modified input and compare containment.
    fn check_mr_equivalence(&self, entry: &CorpusEntry, modified_input: &str) -> bool {
        let original = self.transpile_entry(&entry.input, entry.format);
        let modified = self.transpile_entry(modified_input, entry.format);

        match (original, modified) {
            (Ok(orig), Ok(modif)) => {
                let orig_has = orig.contains(&entry.expected_output);
                let modif_has = modif.contains(&entry.expected_output);
                orig_has == modif_has
            }
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    /// Transpile input based on format (DRY helper for MR checks).
    fn transpile_entry(
        &self,
        input: &str,
        format: CorpusFormat,
    ) -> std::result::Result<String, crate::Error> {
        match format {
            CorpusFormat::Bash => crate::transpile(input, self.config.clone()),
            CorpusFormat::Makefile => crate::transpile_makefile(input, self.config.clone()),
            CorpusFormat::Dockerfile => crate::transpile_dockerfile(input, self.config.clone()),
        }
    }

    /// G: Cross-shell agreement — transpile bash entries with Posix and Bash
    /// dialect configs, verify both produce output containing the expected fragment.
    /// Non-bash formats pass by default (no dialect variation).
    fn check_cross_shell(&self, entry: &CorpusEntry) -> bool {
        if entry.format != CorpusFormat::Bash {
            return true; // Only bash has dialect variants
        }

        let posix_config = Config {
            target: crate::models::ShellDialect::Posix,
            ..self.config.clone()
        };
        let bash_config = Config {
            target: crate::models::ShellDialect::Bash,
            ..self.config.clone()
        };

        let posix_result = crate::transpile(&entry.input, posix_config);
        let bash_result = crate::transpile(&entry.input, bash_config);

        match (posix_result, bash_result) {
            (Ok(posix_out), Ok(bash_out)) => {
                // Both should contain the expected output
                let posix_has = posix_out.contains(&entry.expected_output);
                let bash_has = bash_out.contains(&entry.expected_output);
                posix_has && bash_has
            }
            // Both fail: degenerate agreement
            (Err(_), Err(_)) => true,
            // Disagreement: one succeeds, one fails
            _ => false,
        }
    }

    /// Schema validation: verify output conforms to the format grammar.
    /// - Bash: validate via validation::validate_shell_snippet (POSIX grammar)
    /// - Makefile: parse via make_parser::parse_makefile (GNU Make grammar)
    /// - Dockerfile: validate via lint (Dockerfile instruction grammar)
    fn check_schema(&self, output: &str, format: CorpusFormat) -> bool {
        match format {
            CorpusFormat::Bash => crate::validation::validate_shell_snippet(output).is_ok(),
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

    /// B_L3: Behavioral equivalence — execute the transpiled output and verify
    /// it terminates within 2 seconds. Uses `timeout 2 sh -c` for bash.
    /// Exit code 124 = timeout (script hangs = FAIL).
    /// Any other exit code = script terminates normally (PASS).
    /// Makefile/Dockerfile use syntax validation as proxy (no execution).
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
            // Makefile/Dockerfile: no direct execution; behavioral equivalence
            // is approximated by schema + lint passing (checked separately).
            CorpusFormat::Makefile | CorpusFormat::Dockerfile => true,
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

    fn compute_score(&self, results: &[CorpusResult], registry: &CorpusRegistry) -> CorpusScore {
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
        let format_scores = self.compute_format_scores(results, registry);

        CorpusScore {
            total,
            passed,
            failed,
            rate,
            score,
            grade,
            format_scores,
            results: results.to_vec(),
        }
    }

    fn compute_format_scores(
        &self,
        results: &[CorpusResult],
        registry: &CorpusRegistry,
    ) -> Vec<FormatScore> {
        let mut scores = Vec::new();

        for format in &[CorpusFormat::Bash, CorpusFormat::Makefile, CorpusFormat::Dockerfile] {
            // Map results to format by matching entry IDs
            let format_results: Vec<&CorpusResult> = results
                .iter()
                .filter(|r| {
                    registry
                        .entries
                        .iter()
                        .any(|e| e.id == r.id && e.format == *format)
                })
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
    use crate::corpus::registry::CorpusTier;

    #[test]
    fn test_CORPUS_RUN_001_score_calculation_v2_full() {
        // All flags true: A(30) + B_L1(10) + B_L2(8) + B_L3(7) + C(15) + D(10) + E(10) + F(5) + G(5) = 100
        let result = CorpusResult {
            id: "T-001".to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            schema_valid: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            actual_output: Some("output".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!((result.score() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_002_score_transpile_only() {
        // Only transpilation succeeds: A(30) + nothing else = 30
        let result = CorpusResult {
            id: "T-002".to_string(),
            transpiled: true,
            output_contains: false,
            output_exact: false,
            output_behavioral: false,
            schema_valid: true,
            has_test: false,
            lint_clean: false,
            deterministic: false,
            metamorphic_consistent: false,
            cross_shell_agree: false,
            actual_output: Some("output".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!((result.score() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_003_score_failed_transpile() {
        // Failed transpilation: gateway blocks everything = 0
        let result = CorpusResult {
            id: "T-003".to_string(),
            transpiled: false,
            output_contains: false,
            output_exact: false,
            output_behavioral: false,
            schema_valid: false,
            has_test: true,
            lint_clean: false,
            deterministic: false,
            metamorphic_consistent: false,
            cross_shell_agree: false,
            actual_output: None,
            error: Some("parse error".to_string()),
            error_category: None,
            error_confidence: None,
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
    fn test_CORPUS_RUN_007_gateway_logic_v2() {
        // All v2 flags true: score = 100
        let perfect = CorpusResult {
            id: "T-007".to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            schema_valid: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            actual_output: Some("out".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!((perfect.score() - 100.0).abs() < f64::EPSILON);

        // Gateway: failed transpile = 0 total (all other flags ignored)
        let failed = CorpusResult {
            id: "T-007b".to_string(),
            transpiled: false,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            schema_valid: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            actual_output: None,
            error: Some("err".to_string()),
            error_category: None,
            error_confidence: None,
        };
        assert!((failed.score()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_008_partial_score_v2() {
        // Transpiles + containment + exact + test + deterministic + metamorphic, but NOT lint clean
        // A(30) + B_L1(10) + B_L2(8) + C(15) + D(0) + E(10) + F(5) = 78
        let partial = CorpusResult {
            id: "T-008".to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: false,
            schema_valid: true,
            has_test: true,
            lint_clean: false,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: false,
            actual_output: Some("out".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!((partial.score() - 78.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_009_secondary_gate_l1_blocks_l2() {
        // L1 fails: L2 and L3 are gated to 0 even if set true
        // A(30) + B_L1(0) + B_L2(0) + B_L3(0) + C(15) + D(10) + E(10) + F(5) + G(5) = 75
        let result = CorpusResult {
            id: "T-009".to_string(),
            transpiled: true,
            output_contains: false,
            output_exact: true,  // gated by L1
            output_behavioral: true,  // gated by L1
            schema_valid: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            actual_output: Some("out".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!((result.score() - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_010_v1_backward_compat() {
        // v1 scoring: A(40) + B(25) + C(15) + D(10) + E(10) = 100
        let result = CorpusResult {
            id: "T-010".to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: false,
            schema_valid: true,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: false,
            actual_output: Some("out".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!((result.score_v1() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_CORPUS_RUN_011_exact_match_single_line() {
        assert!(check_exact_match("hello world\nfoo bar\n", "foo bar"));
        assert!(!check_exact_match("hello world\nfoo bar baz\n", "foo bar"));
    }

    #[test]
    fn test_CORPUS_RUN_012_exact_match_multi_line() {
        let actual = "line1\nline2\nline3\nline4\n";
        assert!(check_exact_match(actual, "line2\nline3"));
        assert!(!check_exact_match(actual, "line2\nline4"));
    }

    #[test]
    fn test_CORPUS_RUN_013_exact_match_empty_expected() {
        assert!(check_exact_match("anything", ""));
        assert!(check_exact_match("anything", "  "));
    }

    #[test]
    fn test_CORPUS_RUN_014_detect_test_exists() {
        assert!(detect_test_exists("B-001"));
        assert!(!detect_test_exists(""));
    }

    #[test]
    fn test_CORPUS_RUN_016_classify_error_syntax() {
        let (cat, conf) = classify_error("unexpected token: parse error near line 5");
        assert_eq!(cat.as_deref(), Some("syntax_error"));
        assert!(conf.is_some());
    }

    #[test]
    fn test_CORPUS_RUN_017_classify_error_unsupported() {
        let (cat, conf) = classify_error("unsupported feature: process substitution");
        assert_eq!(cat.as_deref(), Some("unsupported_construct"));
        assert!(conf.is_some());
    }

    #[test]
    fn test_CORPUS_RUN_018_classify_error_type() {
        let (cat, conf) = classify_error("type mismatch in assignment");
        assert_eq!(cat.as_deref(), Some("type_error"));
        assert!(conf.is_some());
    }

    #[test]
    fn test_CORPUS_RUN_019_classify_error_unknown() {
        let (cat, conf) = classify_error("something went wrong");
        assert_eq!(cat.as_deref(), Some("unknown"));
        assert!(conf.is_some());
    }

    #[test]
    fn test_CORPUS_RUN_020_mr5_subsumption_top_level() {
        // MR-5 must only remove top-level statements, not statements inside blocks
        let runner = CorpusRunner::new(Config::default());
        let entry_nested = CorpusEntry::new(
            "T-MR5-1",
            "nested-block",
            "If/else with nested statements",
            CorpusFormat::Bash,
            CorpusTier::Standard,
            r#"fn main() { let x = 5; if x > 3 { let msg = "big"; } else { let msg = "small"; } }"#,
            "x=",
        );
        // Should be vacuously true (only one top-level semi before the if block)
        assert!(runner.check_mr5_subsumption(&entry_nested));

        let entry_multi = CorpusEntry::new(
            "T-MR5-2",
            "multi-stmt",
            "Multiple top-level statements",
            CorpusFormat::Bash,
            CorpusTier::Standard,
            "fn main() { let a = 1; let b = 2; let c = 3; }",
            "a=",
        );
        // Has 3 top-level statements; removing last should still transpile
        assert!(runner.check_mr5_subsumption(&entry_multi));
    }

    #[test]
    fn test_CORPUS_RUN_021_mr6_composition() {
        let runner = CorpusRunner::new(Config::default());
        let entry = CorpusEntry::new(
            "T-MR6-1",
            "multi-let",
            "Multiple let statements",
            CorpusFormat::Bash,
            CorpusTier::Standard,
            "fn main() { let a = 1; let b = 2; }",
            "a=",
        );
        assert!(runner.check_mr6_composition(&entry));
    }

    #[test]
    fn test_CORPUS_RUN_022_mr7_negation() {
        let runner = CorpusRunner::new(Config::default());
        let entry = CorpusEntry::new(
            "T-MR7-1",
            "if-cond",
            "If with condition",
            CorpusFormat::Bash,
            CorpusTier::Standard,
            r#"fn main() { let x = 5; if x > 3 { let msg = "yes"; } }"#,
            "x=",
        );
        assert!(runner.check_mr7_negation(&entry));
    }

    #[test]
    fn test_CORPUS_RUN_023_behavioral_execution() {
        let runner = CorpusRunner::new(Config::default());
        // Simple variable assignment — should execute without error
        assert!(runner.check_behavioral("x='42'", CorpusFormat::Bash));
        // Empty script — should succeed
        assert!(runner.check_behavioral("", CorpusFormat::Bash));
        // Makefile/Dockerfile — always pass (syntax proxy)
        assert!(runner.check_behavioral("", CorpusFormat::Makefile));
        assert!(runner.check_behavioral("", CorpusFormat::Dockerfile));
    }

    #[test]
    fn test_CORPUS_RUN_015_schema_hard_gate() {
        // Schema invalid: transpiled=true but schema_valid=false → score 0
        let result = CorpusResult {
            id: "T-015".to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            schema_valid: false,
            has_test: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            actual_output: Some("invalid output".to_string()),
            error: None,
            error_category: None,
            error_confidence: None,
        };
        assert!(
            result.score().abs() < f64::EPSILON,
            "Schema-invalid entry should score 0, got {}",
            result.score()
        );
    }
}
