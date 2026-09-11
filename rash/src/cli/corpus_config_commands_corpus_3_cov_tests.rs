//! PMAT-257 coverage tests for `corpus_config_commands_corpus_3.rs`
//! (`include!`d into the `corpus_config_commands` module, which is itself
//! `include!`d from `corpus_config_commands.rs`).
//!
//! `corpus_tier_analysis`, `corpus_tier_targets`, `corpus_quality_gates`,
//! `corpus_metrics_check`, `corpus_gate_status_cmd`, `corpus_dataset_info`,
//! `corpus_publish_check`, and `corpus_generate_conversations` all build a
//! `CorpusRunner` (or transpile) over the real ~18k-entry
//! `CorpusRegistry::load_full()`. Each was split into a
//! `pub(crate) fn <name>_with(registry: &CorpusRegistry, ...)` twin that
//! takes a small fixture registry, matching `corpus_diag_commands.rs`.
//! `corpus_export_dataset` delegates to `dataset::run_and_export`, which is
//! out of scope for this ticket (lives in `corpus/dataset_export.rs`) and
//! itself scores the full corpus, so it was split into a pure
//! `corpus_export_dataset_format` mapper plus a `corpus_export_dataset_with`
//! twin that handles only the write/print logic on already-exported data.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_config_commands::*;
    use crate::cli::args::DatasetExportFormat;
    use crate::corpus::dataset::ExportFormat;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};

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

    #[test]
    fn test_PMAT257_cov_tier_analysis_with_tiny_registry() {
        corpus_tier_analysis_with(&tiny_registry())
            .expect("tier analysis runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_tier_targets_with_tiny_registry() {
        corpus_tier_targets_with(&tiny_registry()).expect("tier targets runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_quality_gates_with_tiny_registry() {
        corpus_quality_gates_with(&tiny_registry())
            .expect("quality gates runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_metrics_check_with_tiny_registry() {
        corpus_metrics_check_with(&tiny_registry())
            .expect("metrics check runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_gate_status_cmd_with_tiny_registry() {
        corpus_gate_status_cmd_with(&tiny_registry())
            .expect("gate status runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_dataset_info_with_tiny_registry() {
        corpus_dataset_info_with(&tiny_registry()).expect("dataset info runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_publish_check_with_tiny_registry() {
        corpus_publish_check_with(&tiny_registry())
            .expect("publish check runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_generate_conversations_with_tiny_registry_to_stdout() {
        corpus_generate_conversations_with(&tiny_registry(), None, 42, None, false)
            .expect("generate conversations runs over a tiny registry (chatml, stdout)");
    }

    #[test]
    fn test_PMAT257_cov_generate_conversations_with_tiny_registry_entrenar_format_to_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("conversations.jsonl");
        corpus_generate_conversations_with(&tiny_registry(), Some(out.clone()), 7, Some(2), true)
            .expect("generate conversations runs over a tiny registry (entrenar, file)");
        assert!(out.exists(), "conversations file must be written");
    }

    #[test]
    fn test_PMAT257_cov_export_dataset_format_maps_every_variant() {
        assert_eq!(
            corpus_export_dataset_format(DatasetExportFormat::Json),
            ExportFormat::Json
        );
        assert_eq!(
            corpus_export_dataset_format(DatasetExportFormat::Jsonl),
            ExportFormat::JsonLines
        );
        assert_eq!(
            corpus_export_dataset_format(DatasetExportFormat::Csv),
            ExportFormat::Csv
        );
        assert_eq!(
            corpus_export_dataset_format(DatasetExportFormat::Classification),
            ExportFormat::Classification
        );
        assert_eq!(
            corpus_export_dataset_format(DatasetExportFormat::MultiLabelClassification),
            ExportFormat::MultiLabelClassification
        );
    }

    #[test]
    fn test_PMAT257_cov_export_dataset_with_writes_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("dataset.jsonl");
        corpus_export_dataset_with(ExportFormat::JsonLines, 3, "{\"a\":1}\n", Some(out.clone()))
            .expect("export dataset write must succeed");
        assert!(out.exists());
        let contents = std::fs::read_to_string(&out).expect("read exported dataset");
        assert_eq!(contents, "{\"a\":1}\n");
    }

    #[test]
    fn test_PMAT257_cov_export_dataset_with_no_output_prints_to_stdout() {
        corpus_export_dataset_with(ExportFormat::Csv, 0, "a,b\n1,2\n", None)
            .expect("export dataset without output path must still succeed");
    }
}
