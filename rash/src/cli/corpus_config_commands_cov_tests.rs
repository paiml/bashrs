//! PMAT-257 coverage tests for `corpus_config_commands`, kept in a sibling
//! file per the ticket instructions: the source file mixes handlers that
//! build a `CorpusRunner` over the real ~18k-entry registry with ones that
//! only touch static tables, and every `*_with(registry, ...)` twin needs a
//! small fixture registry to stay a millisecond-scale unit test.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_config_commands::*;
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
    fn test_PMAT257_cov_cwe_mapping_human_and_json() {
        corpus_cwe_mapping(false).expect("human-readable CWE table must print");
        corpus_cwe_mapping(true).expect("JSON CWE mapping must serialize");
    }

    #[test]
    fn test_PMAT257_cov_label_lints_safe_and_unsafe_scripts() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("in.jsonl");
        let output = dir.path().join("out.jsonl");
        std::fs::write(
            &input,
            concat!(
                "{\"script\":\"echo hello\"}\n",
                "{\"text\":\"eval \\\"$CMD\\\"\"}\n",
            ),
        )
        .expect("write input jsonl");

        corpus_label(input, Some(output.clone())).expect("labeling well-formed input must succeed");

        let contents = std::fs::read_to_string(&output).expect("read labeled output");
        assert!(contents.contains("\"label\""), "{contents}");
        assert!(contents.contains("\"unsafe\""), "{contents}");
        assert!(contents.contains("\"safe\""), "{contents}");
    }

    #[test]
    fn test_PMAT257_cov_label_to_stdout_when_no_output_path() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("in.jsonl");
        std::fs::write(&input, "{\"input\":\"true\"}\n\n").expect("write input jsonl");

        corpus_label(input, None).expect("labeling without an output path prints to stdout");
    }

    #[test]
    fn test_PMAT257_cov_label_missing_script_field_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("in.jsonl");
        std::fs::write(&input, "{\"other\":\"nope\"}\n").expect("write input jsonl");

        let err = corpus_label(input, None).expect_err("a line with no script field must fail");
        assert!(format!("{err}").contains("missing"), "{err}");
    }

    #[test]
    fn test_PMAT257_cov_label_invalid_json_is_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let input = dir.path().join("in.jsonl");
        std::fs::write(&input, "not json at all\n").expect("write input jsonl");

        let err = corpus_label(input, None).expect_err("a malformed JSON line must fail");
        assert!(format!("{err}").contains("Invalid JSON"), "{err}");
    }

    #[test]
    fn test_PMAT257_cov_export_benchmark_writes_file_with_limit() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let output = dir.path().join("bench.jsonl");
        // `limit` bounds how many of the real ~18k corpus entries are
        // transpiled and linted, so a small limit keeps this a fast unit test
        // without needing a `_with(registry, ...)` twin.
        corpus_export_benchmark(Some(output.clone()), Some(3))
            .expect("export with a small limit must succeed");
        assert!(output.exists());
    }

    #[test]
    fn test_PMAT257_cov_export_benchmark_to_stdout() {
        corpus_export_benchmark(None, Some(1)).expect("export to stdout must succeed");
    }

    #[test]
    fn test_PMAT257_cov_domain_categories_with_tiny_registry() {
        corpus_domain_categories_with(&tiny_registry())
            .expect("domain categories over a tiny registry must succeed");
    }

    #[test]
    fn test_PMAT257_cov_domain_coverage_with_tiny_registry() {
        corpus_domain_coverage_with(&tiny_registry())
            .expect("domain coverage over a tiny registry must succeed");
    }

    #[test]
    fn test_PMAT257_cov_domain_matrix_with_tiny_registry() {
        corpus_domain_matrix_with(&tiny_registry())
            .expect("domain matrix over a tiny registry must succeed");
    }

    #[test]
    fn test_PMAT257_cov_tier_weights_with_tiny_registry() {
        corpus_tier_weights_with(&tiny_registry())
            .expect("tier weights over a tiny registry must succeed");
    }
}
