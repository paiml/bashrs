//! PMAT-257 coverage tests for `corpus_config_commands_corpus.rs`
//! (`include!`d into the `corpus_config_commands` module via
//! `corpus_config_commands_corpus_2.rs` -> `corpus_config_commands_corpus_3.rs`
//! -> `corpus_config_commands.rs`).
//!
//! `corpus_ssc_report`, `corpus_model_card`, `corpus_training_config`,
//! `corpus_publish_dataset`, and `corpus_publish_conversations` all build
//! their payload from `CorpusRegistry::load_full()` (directly, or via
//! `corpus_baseline_entries()` / `generate_model_card()` /
//! `generate_training_config()` / `generate_ssc_report()`, all of which
//! score the full ~18k-entry corpus end-to-end). Each was split into a
//! twin that takes the already-computed payload (or a small fixture
//! registry), matching `corpus_diag_commands.rs`.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_config_commands::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
    use crate::corpus::ssc_report::{SscMetric, SscSection, SscStatus, SscStatusReport};

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

    fn tiny_report(overall_ready: bool, section_status: SscStatus) -> SscStatusReport {
        SscStatusReport {
            spec_version: "v11-test".to_string(),
            corpus_size: 3,
            overall_ready,
            sections: vec![SscSection {
                name: "fixture-section".to_string(),
                spec_ref: "S0".to_string(),
                status: section_status,
                metrics: vec![SscMetric {
                    name: "metric".to_string(),
                    value: "1".to_string(),
                    target: "1".to_string(),
                    passed: true,
                }],
            }],
        }
    }

    #[test]
    fn test_PMAT257_cov_ssc_report_with_human_no_gate() {
        let report = tiny_report(true, SscStatus::Pass);
        corpus_ssc_report_with(&report, false, false).expect("human report without gate");
    }

    #[test]
    fn test_PMAT257_cov_ssc_report_with_json_gate_passes() {
        let report = tiny_report(true, SscStatus::Pass);
        corpus_ssc_report_with(&report, true, true).expect("json report with passing gate");
    }

    #[test]
    fn test_PMAT257_cov_ssc_report_with_gate_fails_on_fail_section() {
        let report = tiny_report(false, SscStatus::Fail);
        let err = corpus_ssc_report_with(&report, false, true)
            .expect_err("gate must fail when a section is Fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("fixture-section"),
            "error names the section: {msg}"
        );
    }

    #[test]
    fn test_PMAT257_cov_model_card_with_writes_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("README.md");
        corpus_model_card_with("# fixture card\n", Some(out.clone()))
            .expect("model card write must succeed");
        let contents = std::fs::read_to_string(&out).expect("read model card");
        assert_eq!(contents, "# fixture card\n");
    }

    #[test]
    fn test_PMAT257_cov_model_card_with_no_output_prints_to_stdout() {
        corpus_model_card_with("# fixture card\n", None)
            .expect("model card without output path must still succeed");
    }

    #[test]
    fn test_PMAT257_cov_training_config_with_json_to_file() {
        use crate::corpus::training_config::generate_training_config_from;
        let config = generate_training_config_from(&tiny_registry());
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("training_config.json");
        corpus_training_config_with(&config, Some(out.clone()), true)
            .expect("training config json write must succeed");
        assert!(out.exists());
    }

    #[test]
    fn test_PMAT257_cov_training_config_with_yaml_to_stdout() {
        use crate::corpus::training_config::generate_training_config_from;
        let config = generate_training_config_from(&tiny_registry());
        corpus_training_config_with(&config, None, false)
            .expect("training config yaml to stdout must succeed");
    }

    #[test]
    fn test_PMAT257_cov_publish_dataset_with_tiny_registry() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("dataset");
        corpus_publish_dataset_with(&tiny_registry(), out.clone())
            .expect("publish dataset runs over a tiny registry");
        assert!(out.join("train.jsonl").exists());
        assert!(out.join("val.jsonl").exists());
        assert!(out.join("test.jsonl").exists());
        assert!(out.join("README.md").exists());
        assert!(out.join("training_config.yaml").exists());
    }

    #[test]
    fn test_PMAT257_cov_publish_conversations_with_tiny_registry() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("conversations");
        corpus_publish_conversations_with(&tiny_registry(), out.clone(), 11)
            .expect("publish conversations runs over a tiny registry");
        assert!(out.join("conversations.jsonl").exists());
        assert!(out.join("README.md").exists());
    }
}
