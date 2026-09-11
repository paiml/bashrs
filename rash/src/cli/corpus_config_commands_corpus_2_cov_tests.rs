//! PMAT-257 coverage tests for `corpus_config_commands_corpus_2.rs`
//! (`include!`d into the `corpus_config_commands` module).
//!
//! `corpus_baselines`, `corpus_label_audit`, and `corpus_validate_contracts`
//! all delegate to library functions that walk the real ~18k-entry corpus
//! (`corpus_baseline_entries()`, `run_corpus_label_audit()`,
//! `run_all_contracts()` — the last is documented as taking "minutes"). Each
//! was split into a `pub(crate) fn <name>_with(...)` twin that takes an
//! already-built (small) input, so a test can exercise the printing logic in
//! milliseconds without touching the corpus. `corpus_generalization_tests`
//! and `corpus_tokenizer_validation` never touch the corpus registry at all
//! (static fixture data), so they're covered directly.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_config_commands::*;
    use crate::corpus::contract_validation::{ContractResult, ContractValidationReport};
    use crate::corpus::label_audit::run_label_audit;

    #[test]
    fn test_PMAT257_cov_baselines_with_mixed_labels() {
        let entries: Vec<(&str, u8)> = vec![("echo hello", 0), ("eval \"$CMD\"", 1), ("ls -la", 0)];
        corpus_baselines_with(&entries).expect("baselines over a tiny fixture must succeed");
    }

    #[test]
    fn test_PMAT257_cov_baselines_with_empty_is_not_an_error() {
        let entries: Vec<(&str, u8)> = vec![];
        corpus_baselines_with(&entries).expect("baselines over an empty fixture must not panic");
    }

    #[test]
    fn test_PMAT257_cov_label_audit_with_genuine_and_false_positive() {
        let entries: Vec<(&str, &str, u8)> = vec![
            ("B-1", "eval \"$CMD\"", 1),
            ("B-2", "echo hello", 1), // no unsafe signal -> false positive
        ];
        let report = run_label_audit(&entries);
        assert_eq!(report.total_audited, 2);
        assert_eq!(report.false_positives, 1);
        corpus_label_audit_with(&report)
            .expect("label audit report with a false positive must print without error");
    }

    #[test]
    fn test_PMAT257_cov_label_audit_with_all_genuine_no_false_positives_section() {
        let entries: Vec<(&str, &str, u8)> = vec![("B-1", "eval \"$CMD\"", 1)];
        let report = run_label_audit(&entries);
        assert_eq!(report.false_positives, 0);
        corpus_label_audit_with(&report)
            .expect("label audit report with no false positives must still print");
    }

    #[test]
    fn test_PMAT257_cov_validate_contracts_with_all_passed() {
        let report = ContractValidationReport {
            contracts: vec![ContractResult {
                id: "C-TOK-001".to_string(),
                name: "Tokenizer quality".to_string(),
                passed: true,
                value: 80.0,
                threshold: 70.0,
                detail: "fixture".to_string(),
            }],
            all_passed: true,
            passed_count: 1,
            failed_count: 0,
        };
        corpus_validate_contracts_with(&report)
            .expect("an all-passed contract report must print PASSED");
    }

    #[test]
    fn test_PMAT257_cov_validate_contracts_with_a_failure() {
        let report = ContractValidationReport {
            contracts: vec![
                ContractResult {
                    id: "C-TOK-001".to_string(),
                    name: "Tokenizer quality".to_string(),
                    passed: true,
                    value: 80.0,
                    threshold: 70.0,
                    detail: "fixture".to_string(),
                },
                ContractResult {
                    id: "C-LABEL-001".to_string(),
                    name: "Label accuracy".to_string(),
                    passed: false,
                    value: 50.0,
                    threshold: 90.0,
                    detail: "fixture failure".to_string(),
                },
            ],
            all_passed: false,
            passed_count: 1,
            failed_count: 1,
        };
        corpus_validate_contracts_with(&report)
            .expect("a report with a failing contract must still print, not error");
    }

    #[test]
    fn test_PMAT257_cov_generalization_tests_runs_over_static_fixtures() {
        // No `CorpusRegistry::load_full()` anywhere in this function -- just
        // a static fixture list of OOD scripts run through the linter.
        corpus_generalization_tests().expect("generalization tests must always succeed");
    }

    #[test]
    fn test_PMAT257_cov_tokenizer_validation_runs_over_static_fixtures() {
        corpus_tokenizer_validation().expect("tokenizer validation must always succeed");
    }

    #[test]
    fn test_PMAT257_cov_export_splits_from_input_jsonl_writes_output_dir() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("merged.jsonl");
        let mut lines = String::new();
        for i in 0..12 {
            let label = i % 3 == 0;
            lines.push_str(&format!(
                "{{\"input\":\"echo case-{i}\",\"label\":{}}}\n",
                u8::from(label)
            ));
        }
        std::fs::write(&input, lines).expect("write fixture jsonl");

        let output_dir = dir.path().join("splits");
        corpus_export_splits(Some(output_dir.clone()), Some(input))
            .expect("export-splits from a pre-merged jsonl must succeed");

        assert!(output_dir.join("train.jsonl").exists());
        assert!(output_dir.join("val.jsonl").exists());
        assert!(output_dir.join("test.jsonl").exists());
    }

    #[test]
    fn test_PMAT257_cov_export_splits_from_input_jsonl_no_output_dir() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("merged.jsonl");
        std::fs::write(
            &input,
            "{\"script\":\"echo hi\",\"label\":0}\n{\"unsafe_script\":\"eval $x\",\"label\":1}\n",
        )
        .expect("write fixture jsonl");

        corpus_export_splits(None, Some(input))
            .expect("export-splits without an output dir must still succeed");
    }
}
