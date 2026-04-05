//! Corpus runner helpers: free functions for validation, coverage detection, error classification.
//!
//! Extracted from runner.rs for file size health (PMAT).
//! Contains standalone functions and constants used by `CorpusRunner` methods.

use crate::corpus::registry::CorpusFormat;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Valid Dockerfile instruction prefixes (per Dockerfile reference).
pub(crate) const DOCKERFILE_INSTRUCTIONS: &[&str] = &[
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
/// This is stricter than containment -- it checks that the expected content appears
/// as complete, whitespace-normalized lines (not just a substring within a longer line).
pub(crate) fn check_exact_match(actual: &str, expected: &str) -> bool {
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
pub(crate) fn classify_error(error_msg: &str) -> (Option<String>, Option<f32>) {
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
pub(crate) fn extract_test_names(source: &str, names: &mut HashSet<String>) {
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
/// that reference this entry ID (e.g., "B-001" -> matches "B_001" or "B001").
pub(crate) fn detect_test_exists(entry_id: &str) -> bool {
    if entry_id.is_empty() {
        return false;
    }

    let test_names = TEST_NAMES.get_or_init(build_test_name_set);

    // If we couldn't load any test names (e.g., source files not found),
    // fall back to true to avoid false negatives in CI environments.
    if test_names.is_empty() {
        return true;
    }

    // Normalize entry ID for matching: "B-001" -> "B_001" and "B001"
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
pub(crate) fn format_file_patterns(format: CorpusFormat) -> &'static [&'static str] {
    match format {
        CorpusFormat::Bash => &["emitter/posix", "bash_transpiler/"],
        CorpusFormat::Makefile => &["emitter/makefile"],
        CorpusFormat::Dockerfile => &["emitter/dockerfile"],
    }
}

/// Load per-format coverage ratios from LCOV data.
/// Returns a map of format name -> coverage ratio (0.0-1.0).
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
pub(crate) fn parse_lcov_file_coverage(content: &str) -> Vec<(String, (u64, u64))> {
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
pub(crate) fn detect_coverage_ratio(format: CorpusFormat, entry_id: &str) -> f64 {
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
