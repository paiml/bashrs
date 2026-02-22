//! Corpus helper logic: trending, formatting, date utilities, and cache I/O.
//!
//! This module contains pure logic functions used by the corpus CLI commands
//! for formatting output, computing trends, generating dates, and managing
//! the corpus run cache.

use std::path::Path;

/// Compute the failing dimension labels for a corpus result.
///
/// Returns a comma-separated string of dimension codes that failed
/// (e.g. `"A, B1, B2"`).
#[must_use]
pub fn corpus_failing_dims(r: &crate::corpus::runner::CorpusResult) -> String {
    let mut dims = Vec::new();
    if !r.transpiled {
        dims.push("A");
    }
    if !r.output_contains {
        dims.push("B1");
    }
    if !r.output_exact {
        dims.push("B2");
    }
    if !r.output_behavioral {
        dims.push("B3");
    }
    if !r.lint_clean {
        dims.push("D");
    }
    if !r.deterministic {
        dims.push("E");
    }
    if !r.metamorphic_consistent {
        dims.push("F");
    }
    if !r.cross_shell_agree {
        dims.push("G");
    }
    if !r.schema_valid {
        dims.push("Schema");
    }
    dims.join(", ")
}

/// Format a per-format pass/total column (e.g. `"499/500"` or `"-"` if no data).
#[must_use]
pub fn fmt_pass_total(passed: usize, total: usize) -> String {
    if total > 0 {
        format!("{passed}/{total}")
    } else {
        "-".to_string()
    }
}

/// Compute a trend arrow by comparing two values.
///
/// Returns `"\\u{2191}"` (up arrow) if current > previous,
/// `"\\u{2193}"` (down arrow) if current < previous,
/// `"\\u{2192}"` (right arrow) if equal.
#[must_use]
pub fn trend_arrow(current: usize, previous: usize) -> &'static str {
    if current > previous {
        "\u{2191}"
    } else if current < previous {
        "\u{2193}"
    } else {
        "\u{2192}"
    }
}

/// Generate ISO 8601 date string without chrono dependency.
///
/// Shells out to the `date` command. Returns `"unknown"` on failure.
#[must_use]
pub fn chrono_free_date() -> String {
    use std::process::Command;
    Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Path for the corpus cache file.
pub const CORPUS_CACHE_PATH: &str = ".quality/last-corpus-run.json";

/// Save corpus run results to cache file for instant diagnosis.
pub fn corpus_save_last_run(score: &crate::corpus::runner::CorpusScore) {
    let path = Path::new(CORPUS_CACHE_PATH);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(score) {
        let _ = std::fs::write(path, json);
    }
}

/// Load cached corpus results. Returns `None` if no cache exists or parse fails.
#[must_use]
pub fn corpus_load_last_run() -> Option<crate::corpus::runner::CorpusScore> {
    let data = std::fs::read_to_string(CORPUS_CACHE_PATH).ok()?;
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ===== fmt_pass_total tests =====

    #[test]
    fn test_corpus_fmt_pass_total_with_data() {
        assert_eq!(fmt_pass_total(499, 500), "499/500");
    }

    #[test]
    fn test_corpus_fmt_pass_total_zero_total() {
        assert_eq!(fmt_pass_total(0, 0), "-");
    }

    #[test]
    fn test_corpus_fmt_pass_total_all_passed() {
        assert_eq!(fmt_pass_total(100, 100), "100/100");
    }

    #[test]
    fn test_corpus_fmt_pass_total_none_passed() {
        assert_eq!(fmt_pass_total(0, 50), "0/50");
    }

    // ===== trend_arrow tests =====

    #[test]
    fn test_corpus_trend_arrow_up() {
        assert_eq!(trend_arrow(10, 5), "\u{2191}");
    }

    #[test]
    fn test_corpus_trend_arrow_down() {
        assert_eq!(trend_arrow(3, 7), "\u{2193}");
    }

    #[test]
    fn test_corpus_trend_arrow_equal() {
        assert_eq!(trend_arrow(5, 5), "\u{2192}");
    }

    #[test]
    fn test_corpus_trend_arrow_zero() {
        assert_eq!(trend_arrow(0, 0), "\u{2192}");
    }

    // ===== chrono_free_date tests =====

    #[test]
    fn test_corpus_chrono_free_date_format() {
        let date = chrono_free_date();
        // Should be YYYY-MM-DD format or "unknown"
        if date != "unknown" {
            assert_eq!(date.len(), 10, "Date should be 10 chars: {date}");
            assert_eq!(&date[4..5], "-", "Should have dash at position 4");
            assert_eq!(&date[7..8], "-", "Should have dash at position 7");
            // Year should be numeric
            assert!(date[0..4].chars().all(|c| c.is_ascii_digit()));
            // Month should be numeric
            assert!(date[5..7].chars().all(|c| c.is_ascii_digit()));
            // Day should be numeric
            assert!(date[8..10].chars().all(|c| c.is_ascii_digit()));
        }
    }

    // ===== corpus_save_last_run / corpus_load_last_run tests =====

    #[test]
    fn test_corpus_load_last_run_missing_file() {
        // If cache doesn't exist at the default path, should return None
        // (This test only verifies the None path; the actual file may or may not exist.)
        // We test the function signature and behavior rather than file system state.
        let result = corpus_load_last_run();
        // Either Some or None is valid depending on environment; just verify no panic
        let _ = result;
    }

    // ===== corpus_failing_dims tests =====

    /// Helper to create a test CorpusResult with all dimensions passing.
    fn make_result_all_pass(id: &str) -> crate::corpus::runner::CorpusResult {
        crate::corpus::runner::CorpusResult {
            id: id.to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            has_test: false,
            coverage_ratio: 0.0,
            schema_valid: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            expected_output: None,
            actual_output: None,
            error: None,
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    #[test]
    fn test_corpus_failing_dims_all_pass() {
        let r = make_result_all_pass("B-001");
        assert_eq!(corpus_failing_dims(&r), "");
    }

    #[test]
    fn test_corpus_failing_dims_some_failures() {
        let mut r = make_result_all_pass("B-002");
        r.output_contains = false;
        r.output_exact = false;
        assert_eq!(corpus_failing_dims(&r), "B1, B2");
    }

    #[test]
    fn test_corpus_failing_dims_all_fail() {
        let mut r = make_result_all_pass("B-003");
        r.transpiled = false;
        r.output_contains = false;
        r.output_exact = false;
        r.output_behavioral = false;
        r.lint_clean = false;
        r.deterministic = false;
        r.metamorphic_consistent = false;
        r.cross_shell_agree = false;
        r.schema_valid = false;
        assert_eq!(
            corpus_failing_dims(&r),
            "A, B1, B2, B3, D, E, F, G, Schema"
        );
    }
}
