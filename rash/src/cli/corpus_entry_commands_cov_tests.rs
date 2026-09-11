//! PMAT-257 coverage tests for `corpus_entry_commands.rs`.
//!
//! `corpus_check_entry` builds its report from `CorpusRegistry::load_full()`
//! (18,000+ entries) run through a `CorpusRunner` -- too slow for a unit
//! test as written. It was split into `corpus_check_entry_with(registry, ...)`
//! which takes the registry as a parameter, matching `corpus_diag_commands.rs`.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_entry_commands::*;
    use crate::cli::args::CorpusOutputFormat;
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
    fn test_PMAT257_cov_check_entry_human_format() {
        corpus_check_entry_with(&tiny_registry(), "B-001", &CorpusOutputFormat::Human)
            .expect("human-format check runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_check_entry_json_format() {
        corpus_check_entry_with(&tiny_registry(), "M-001", &CorpusOutputFormat::Json)
            .expect("json-format check runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_check_entry_dockerfile_entry() {
        corpus_check_entry_with(&tiny_registry(), "D-001", &CorpusOutputFormat::Human)
            .expect("dockerfile entry check runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_check_entry_missing_id_is_an_error() {
        let err = corpus_check_entry_with(&tiny_registry(), "NOPE-999", &CorpusOutputFormat::Human)
            .expect_err("unknown id must fail");
        assert!(format!("{err}").contains("NOPE-999"));
    }

    #[test]
    fn test_PMAT257_cov_truncate_line_short_string_unchanged() {
        assert_eq!(truncate_line("hello", 60), "hello");
    }

    #[test]
    fn test_PMAT257_cov_truncate_line_long_string_is_truncated() {
        let long = "a".repeat(100);
        let truncated = truncate_line(&long, 10);
        assert_eq!(truncated, format!("{}...", "a".repeat(10)));
    }

    #[test]
    fn test_PMAT257_cov_truncate_line_uses_first_line_only() {
        let multi = "first line\nsecond line";
        assert_eq!(truncate_line(multi, 60), "first line");
    }
}
