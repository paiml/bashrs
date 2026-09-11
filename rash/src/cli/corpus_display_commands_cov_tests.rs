//! PMAT-257 coverage tests for `corpus_display_commands.rs`.
//!
//! `corpus_heatmap`, `corpus_dashboard`, and `corpus_search` all build a
//! `CorpusRunner` and score entries out of the real
//! `CorpusRegistry::load_full()` (18,000+ entries) -- too slow for a unit
//! test as written. Each was split into a `*_with(registry, ...)` twin that
//! takes the registry as a parameter, matching `corpus_diag_commands.rs`.
//!
//! Wired from `commands.rs`.

#[cfg(test)]
mod pmat257_cov_tests {
    use super::super::corpus_display_commands::*;
    use crate::cli::args::{CorpusFormatArg, CorpusOutputFormat};
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};

    fn tiny_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "hello-bash",
            "PMAT-257 fixture for the bash format, used to exercise the corpus display commands and their search matching against a long description that exceeds seventy two characters",
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
            "",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18\nWORKDIR /app\n",
            "FROM alpine:3.18",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_heatmap_with_no_filter() {
        corpus_heatmap_with(&tiny_registry(), 10, None).expect("heatmap runs over tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_heatmap_with_each_format_filter() {
        let registry = tiny_registry();
        corpus_heatmap_with(&registry, 10, Some(&CorpusFormatArg::Bash)).expect("bash filter runs");
        corpus_heatmap_with(&registry, 10, Some(&CorpusFormatArg::Makefile))
            .expect("makefile filter runs");
        corpus_heatmap_with(&registry, 10, Some(&CorpusFormatArg::Dockerfile))
            .expect("dockerfile filter runs");
    }

    #[test]
    fn test_PMAT257_cov_heatmap_with_small_limit() {
        corpus_heatmap_with(&tiny_registry(), 1, None).expect("heatmap respects a small limit");
    }

    #[test]
    fn test_PMAT257_cov_dashboard_with_tiny_registry() {
        corpus_dashboard_with(&tiny_registry()).expect("dashboard runs over tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_dashboard_print_formats_and_history() {
        use crate::corpus::runner::{ConvergenceEntry, CorpusRunner};
        use crate::models::Config;

        let registry = tiny_registry();
        let runner = CorpusRunner::new(Config::default());
        let score = runner.run(&registry);
        dashboard_print_formats(&score);

        let entries = vec![
            ConvergenceEntry {
                iteration: 1,
                score: 90.0,
                delta: 1.0,
                notes: "first".to_string(),
                ..Default::default()
            },
            ConvergenceEntry {
                iteration: 2,
                score: 95.0,
                delta: -0.5,
                notes: "second".to_string(),
                ..Default::default()
            },
        ];
        dashboard_print_history(&entries);
    }

    #[test]
    fn test_PMAT257_cov_search_with_human_no_matches() {
        corpus_search_with(
            &tiny_registry(),
            "no-such-pattern-xyz",
            &CorpusOutputFormat::Human,
            None,
        )
        .expect("search with no matches still succeeds");
    }

    #[test]
    fn test_PMAT257_cov_search_with_human_matches_long_description() {
        corpus_search_with(&tiny_registry(), "bash", &CorpusOutputFormat::Human, None)
            .expect("search finds the bash fixture and prints its truncated description");
    }

    #[test]
    fn test_PMAT257_cov_search_with_human_matches_empty_description() {
        corpus_search_with(
            &tiny_registry(),
            "dockerfile",
            &CorpusOutputFormat::Human,
            None,
        )
        .expect("search finds the dockerfile fixture with an empty description");
    }

    #[test]
    fn test_PMAT257_cov_search_with_json_and_format_filter() {
        corpus_search_with(
            &tiny_registry(),
            "hello",
            &CorpusOutputFormat::Json,
            Some(&CorpusFormatArg::Makefile),
        )
        .expect("json search with a format filter succeeds");
    }
}
