//! PMAT-257 coverage tests for `corpus_report_commands.rs`.
//!
//! `corpus_show_entry`, `corpus_export`, and `corpus_show_failures` all build
//! a `CorpusRunner` over the real `CorpusRegistry::load_full()` (18,000+
//! entries) -- too slow for a unit test as written. Each was split into a
//! `*_with(registry, ...)` twin. `corpus_show_history` reads a fixed
//! `.quality/convergence.log` path; it was split into a `*_with(log_path,
//! ...)` twin, matching `corpus_converge_table_with`.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_report_commands::*;
    use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
    use crate::corpus::runner::{ConvergenceEntry, CorpusResult};
    use std::io::Write;

    fn tiny_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "hello-bash",
            "PMAT-257 fixture",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            r#"fn main() { let greeting = "hello"; }"#,
            "greeting='hello'",
        ));
        registry.add(CorpusEntry::new(
            "M-001",
            "hello-makefile",
            "PMAT-257 fixture",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "all:\n\techo hello\n",
            "all:",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "hello-dockerfile",
            "PMAT-257 fixture",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18\nWORKDIR /app\n",
            "FROM alpine:3.18",
        ));
        registry
    }

    /// A registry with one entry deliberately expecting content that will
    /// never appear in the transpiled output, so `corpus_show_failures_with`
    /// has something real to print.
    fn registry_with_failure() -> CorpusRegistry {
        let mut registry = tiny_registry();
        registry.add(CorpusEntry::new(
            "B-002",
            "always-fails",
            "PMAT-257 fixture that never matches",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            r#"fn main() { let x = 1; }"#,
            "this-string-will-never-appear-in-output",
        ));
        registry
    }

    // ---- corpus_show_entry_with ----

    #[test]
    fn test_PMAT257_cov_show_entry_with_human_found() {
        corpus_show_entry_with(&tiny_registry(), "B-001", &CorpusOutputFormat::Human)
            .expect("entry exists in tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_show_entry_with_json_found() {
        corpus_show_entry_with(&tiny_registry(), "M-001", &CorpusOutputFormat::Json)
            .expect("json format succeeds");
    }

    #[test]
    fn test_PMAT257_cov_show_entry_with_not_found_is_error() {
        let result = corpus_show_entry_with(&tiny_registry(), "Z-999", &CorpusOutputFormat::Human);
        assert!(result.is_err(), "unknown id must return an error");
    }

    // ---- corpus_export_with ----

    #[test]
    fn test_PMAT257_cov_export_with_stdout_no_filter() {
        corpus_export_with(&tiny_registry(), None, None).expect("export to stdout succeeds");
    }

    #[test]
    fn test_PMAT257_cov_export_with_each_format_filter() {
        let registry = tiny_registry();
        corpus_export_with(&registry, None, Some(&CorpusFormatArg::Bash)).expect("bash filter");
        corpus_export_with(&registry, None, Some(&CorpusFormatArg::Makefile))
            .expect("makefile filter");
        corpus_export_with(&registry, None, Some(&CorpusFormatArg::Dockerfile))
            .expect("dockerfile filter");
    }

    #[test]
    fn test_PMAT257_cov_export_with_writes_to_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("export.json");
        let path_str = path.to_str().expect("utf8 path");
        corpus_export_with(&tiny_registry(), Some(path_str), None).expect("export to file");
        let contents = std::fs::read_to_string(&path).expect("exported file exists");
        assert!(contents.contains("aggregate_score"));
    }

    // ---- corpus_show_history_with ----

    fn write_log(dir: &std::path::Path, lines: &[String]) -> std::path::PathBuf {
        let path = dir.join("convergence.log");
        let mut f = std::fs::File::create(&path).expect("create log");
        for line in lines {
            writeln!(f, "{line}").expect("write log line");
        }
        path
    }

    fn entry_json(
        iteration: u32,
        passed: usize,
        total: usize,
        bash_total: usize,
        score: f64,
    ) -> String {
        let entry = ConvergenceEntry {
            iteration,
            date: "2026-01-01".to_string(),
            total,
            passed,
            failed: total - passed,
            rate: passed as f64 / total as f64,
            delta: 0.1,
            notes: "note".to_string(),
            bash_total,
            bash_passed: bash_total,
            score,
            ..Default::default()
        };
        serde_json::to_string(&entry).expect("serialize convergence entry")
    }

    #[test]
    fn test_PMAT257_cov_show_history_with_missing_file_is_empty_message() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("does-not-exist.log");
        corpus_show_history_with(&path, &CorpusOutputFormat::Human, None)
            .expect("missing log file yields a friendly message, not an error");
    }

    #[test]
    fn test_PMAT257_cov_show_history_with_human_format_data_and_score() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let lines = vec![
            entry_json(1, 8, 10, 10, 80.0),
            entry_json(2, 9, 10, 10, 90.0),
        ];
        let path = write_log(dir.path(), &lines);
        corpus_show_history_with(&path, &CorpusOutputFormat::Human, None)
            .expect("human format with per-format and score data");
    }

    #[test]
    fn test_PMAT257_cov_show_history_with_human_no_format_no_score() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let lines = vec![entry_json(1, 5, 10, 0, 0.0)];
        let path = write_log(dir.path(), &lines);
        corpus_show_history_with(&path, &CorpusOutputFormat::Human, None)
            .expect("human format without per-format or score data");
    }

    #[test]
    fn test_PMAT257_cov_show_history_with_last_truncates() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let lines = vec![
            entry_json(1, 1, 10, 0, 0.0),
            entry_json(2, 2, 10, 0, 0.0),
            entry_json(3, 3, 10, 0, 0.0),
        ];
        let path = write_log(dir.path(), &lines);
        corpus_show_history_with(&path, &CorpusOutputFormat::Human, Some(1))
            .expect("last=1 truncates to the final entry");
    }

    #[test]
    fn test_PMAT257_cov_show_history_with_json_format() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let lines = vec![entry_json(1, 5, 10, 0, 0.0)];
        let path = write_log(dir.path(), &lines);
        corpus_show_history_with(&path, &CorpusOutputFormat::Json, None)
            .expect("json output succeeds");
    }

    // ---- corpus_show_failures_with ----

    #[test]
    fn test_PMAT257_cov_show_failures_with_no_filter_no_dimension() {
        corpus_show_failures_with(
            &registry_with_failure(),
            &CorpusOutputFormat::Human,
            None,
            None,
        )
        .expect("show failures over tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_show_failures_with_each_format_filter() {
        let registry = registry_with_failure();
        for filter in [
            CorpusFormatArg::Bash,
            CorpusFormatArg::Makefile,
            CorpusFormatArg::Dockerfile,
        ] {
            corpus_show_failures_with(&registry, &CorpusOutputFormat::Human, Some(&filter), None)
                .expect("format filter succeeds");
        }
    }

    #[test]
    fn test_PMAT257_cov_show_failures_with_each_dimension() {
        let registry = registry_with_failure();
        for dim in ["a", "b1", "b2", "b3", "d", "e", "f", "g", "schema", "other"] {
            corpus_show_failures_with(&registry, &CorpusOutputFormat::Json, None, Some(dim))
                .expect("dimension filter succeeds");
        }
    }

    // ---- corpus_print_failures (pure) ----

    #[test]
    fn test_PMAT257_cov_print_failures_empty_human() {
        corpus_print_failures(&[], &CorpusOutputFormat::Human).expect("empty failures ok");
    }

    #[test]
    fn test_PMAT257_cov_print_failures_nonempty_human_and_json() {
        let r = CorpusResult {
            id: "X-001".to_string(),
            transpiled: true,
            output_contains: false,
            lint_clean: false,
            ..Default::default()
        };
        let failures = vec![&r];
        corpus_print_failures(&failures, &CorpusOutputFormat::Human).expect("human failures");
        corpus_print_failures(&failures, &CorpusOutputFormat::Json).expect("json failures");
    }

    // ---- pure helpers ----

    #[test]
    fn test_PMAT257_cov_fmt_pass_total_both_branches() {
        assert_eq!(fmt_pass_total(4, 5), "4/5");
        assert_eq!(fmt_pass_total(0, 0), "-");
    }

    #[test]
    fn test_PMAT257_cov_trend_arrow_all_branches() {
        assert_eq!(trend_arrow(5, 3), "\u{2191}");
        assert_eq!(trend_arrow(1, 3), "\u{2193}");
        assert_eq!(trend_arrow(3, 3), "\u{2192}");
    }

    #[test]
    fn test_PMAT257_cov_corpus_failing_dims_all_flags() {
        let r = CorpusResult {
            id: "X".to_string(),
            transpiled: false,
            output_contains: false,
            output_exact: false,
            output_behavioral: false,
            lint_clean: false,
            deterministic: false,
            metamorphic_consistent: false,
            cross_shell_agree: false,
            schema_valid: false,
            ..Default::default()
        };
        let dims = corpus_failing_dims(&r);
        for tag in ["A", "B1", "B2", "B3", "D", "E", "F", "G", "Schema"] {
            assert!(dims.contains(tag), "expected {tag} in {dims}");
        }
    }

    #[test]
    fn test_PMAT257_cov_corpus_failing_dims_none() {
        let r = CorpusResult {
            id: "X".to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            schema_valid: true,
            ..Default::default()
        };
        assert_eq!(corpus_failing_dims(&r), "");
    }

    #[test]
    fn test_PMAT257_cov_print_history_row_with_and_without_prev() {
        let e1 = ConvergenceEntry {
            iteration: 1,
            date: "2026-01-01".to_string(),
            total: 10,
            passed: 8,
            failed: 2,
            rate: 0.8,
            delta: 0.0,
            notes: "first".to_string(),
            bash_total: 5,
            bash_passed: 4,
            score: 80.0,
            grade: "B".to_string(),
            ..Default::default()
        };
        let e2 = ConvergenceEntry {
            iteration: 2,
            passed: 9,
            rate: 0.9,
            delta: 1.0,
            score: 90.0,
            grade: String::new(),
            bash_total: 5,
            bash_passed: 5,
            ..e1.clone()
        };
        corpus_print_history_row(&e1, None, true, true);
        corpus_print_history_row(&e2, Some(&e1), true, true);
        corpus_print_history_row(&e1, None, false, false);
    }
}
