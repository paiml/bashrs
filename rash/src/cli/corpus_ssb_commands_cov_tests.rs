//! PMAT-257 coverage tests for `corpus_ssb_commands`, kept in a sibling file:
//! the source file holds legacy functions above the pre-commit complexity gate
//! (corpus_pipeline_check, cognitive 39), so staging it fails the hook even for
//! a tests-only change. This file is wired from `commands.rs`.

// PMAT-257: coverage for the SSB pipeline handlers. This file has no
// CorpusRunner references, but `corpus_shellcheck_validate` falls back to
// `corpus_baseline_entries()` (transpiles the full ~18k-entry corpus,
// documented in its own comment as "~120s for full corpus") whenever the
// pre-built splits file at the hardcoded relative path
// "training/shellsafetybench/splits/test.jsonl" is missing -- and under
// `cargo test` the cwd is the crate root (`rash/`), where that file does not
// exist. So that one handler is left uncalled here; `corpus_eval_benchmark`
// and `corpus_pipeline_check` take no such path and are cheap to call.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_ssb_commands::*;
    use crate::models::Result;

    #[test]
    fn test_PMAT257_cov_eval_benchmark_reads_predictions_and_prints_text() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("preds.jsonl");
        std::fs::write(
            &path,
            concat!(
                "{\"id\":\"B-1\",\"classification\":\"unsafe\",\"label\":1,",
                "\"cited_rules\":[\"SEC001\"],\"cited_cwes\":[\"CWE-78\"],",
                "\"explanation\":\"eval is dangerous\",\"script\":\"eval $x\",",
                "\"ground_truth_rules\":[\"SEC001\"],\"ground_truth_cwes\":[\"CWE-78\"]}\n",
                "{\"id\":\"B-2\",\"classification\":\"safe\",\"label\":0}\n",
            ),
        )
        .expect("write predictions");

        corpus_eval_benchmark(path.clone(), false)
            .expect("well-formed predictions file must evaluate");
        corpus_eval_benchmark(path, true)
            .expect("the json branch must also succeed on the same file");
    }

    #[test]
    fn test_PMAT257_cov_eval_benchmark_missing_file_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let missing = dir.path().join("does-not-exist.jsonl");
        let err = corpus_eval_benchmark(missing, false)
            .expect_err("a missing predictions file must fail to read");
        assert!(format!("{err}").contains("Failed to read"), "{err}");
    }

    #[test]
    fn test_PMAT257_cov_eval_benchmark_empty_file_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("blank.jsonl");
        std::fs::write(&path, "\n   \n").expect("write blank file");
        let err = corpus_eval_benchmark(path, false)
            .expect_err("a file with no valid prediction lines must fail");
        assert!(format!("{err}").contains("No valid predictions"), "{err}");
    }

    #[test]
    fn test_PMAT257_cov_pipeline_check_both_json_values() {
        // verificar/alimentar are required tools that are not installed in
        // this sandbox, so the preflight check is expected to fail overall;
        // both branches (text and json) still exercise every print path.
        let json_result = corpus_pipeline_check(true);
        let text_result = corpus_pipeline_check(false);
        // Whichever way it comes out, it must not panic, and the two calls
        // must agree since they inspect the same environment.
        assert_eq!(json_result.is_ok(), text_result.is_ok());
    }
}
