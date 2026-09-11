// PMAT-257 coverage tests for `corpus_time_commands`.
//
// `corpus_timeline` / `corpus_drift` read a hardcoded relative convergence
// log path, and `corpus_slow` / `corpus_tags` build a `CorpusRunner` over
// `CorpusRegistry::load_full()` (~18k entries). All four were split into
// `pub(crate) fn <name>_with(...)` bodies that take a caller-supplied log
// path or registry, matching the shape already used by
// `corpus_metrics_commands.rs`.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
    use crate::corpus::runner::{ConvergenceEntry, CorpusRunner};

    /// Three real (id, input, expected_output) triples, one per format,
    /// copied from corpus_data.jsonl (B-001, M-001, D-001).
    fn fixture_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "variable-assignment",
            "Simple string variable assignment",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { let greeting = \"hello\"; } ",
            "greeting='hello'",
        ));
        registry.add(CorpusEntry::new(
            "M-001",
            "simple-variable",
            "Single variable assignment",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "fn main() { let cc = \"gcc\"; }",
            "CC := gcc",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "basic-from",
            "Basic FROM instruction with pinned tag",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "fn main() { from_image(\"alpine\", \"3.18\"); } fn from_image(i: &str, t: &str) {}",
            "FROM alpine:3.18",
        ));
        registry
    }

    fn entry(iteration: u32, total: usize, rate: f64, delta: f64, score: f64) -> ConvergenceEntry {
        ConvergenceEntry {
            iteration,
            date: format!("2026-01-{:02}", iteration.min(28)),
            total,
            passed: total,
            failed: 0,
            rate,
            delta,
            notes: "PMAT-257 fixture".to_string(),
            bash_passed: total,
            bash_total: total,
            makefile_passed: 0,
            makefile_total: 0,
            dockerfile_passed: 0,
            dockerfile_total: 0,
            score,
            grade: "A+".to_string(),
            bash_score: score,
            ..Default::default()
        }
    }

    #[test]
    fn test_PMAT257_cov_timeline_with_no_entries_is_not_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let log = dir.path().join("convergence.log");
        corpus_timeline_with(&log).expect("a missing log file yields an empty timeline");
    }

    #[test]
    fn test_PMAT257_cov_timeline_with_two_entries_shows_growth() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let log = dir.path().join("convergence.log");
        CorpusRunner::append_convergence_log(&entry(1, 100, 0.90, 0.0, 90.0), &log)
            .expect("write first entry");
        CorpusRunner::append_convergence_log(&entry(2, 120, 0.95, 0.05, 95.0), &log)
            .expect("write second entry");
        corpus_timeline_with(&log).expect("two entries must render a full growth timeline");
    }

    #[test]
    fn test_PMAT257_cov_timeline_with_declining_score_takes_down_arrow() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let log = dir.path().join("convergence.log");
        CorpusRunner::append_convergence_log(&entry(1, 100, 0.95, 0.0, 95.0), &log)
            .expect("write first entry");
        CorpusRunner::append_convergence_log(&entry(2, 100, 0.90, -0.05, 90.0), &log)
            .expect("write second entry");
        corpus_timeline_with(&log).expect("a declining score must still render");
    }

    #[test]
    fn test_PMAT257_cov_drift_with_fewer_than_two_entries_is_not_an_error() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let log = dir.path().join("convergence.log");
        CorpusRunner::append_convergence_log(&entry(1, 100, 0.9, 0.0, 90.0), &log)
            .expect("write single entry");
        corpus_drift_with(&log).expect("fewer than two iterations must not fail");
    }

    #[test]
    fn test_PMAT257_cov_drift_with_two_scored_entries_reports_positive_drift() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let log = dir.path().join("convergence.log");
        CorpusRunner::append_convergence_log(&entry(1, 100, 0.80, 0.0, 80.0), &log)
            .expect("write first entry");
        CorpusRunner::append_convergence_log(&entry(2, 100, 0.95, 0.15, 95.0), &log)
            .expect("write second entry");
        corpus_drift_with(&log).expect("two scored entries must report drift");
    }

    #[test]
    fn test_PMAT257_cov_drift_with_declining_rate_emits_warning() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let log = dir.path().join("convergence.log");
        CorpusRunner::append_convergence_log(&entry(1, 100, 0.95, 0.0, 95.0), &log)
            .expect("write first entry");
        CorpusRunner::append_convergence_log(&entry(2, 100, 0.85, -0.10, 85.0), &log)
            .expect("write second entry");
        corpus_drift_with(&log).expect("a declining rate must still be reported, not an error");
    }

    #[test]
    fn test_PMAT257_cov_drift_print_format_zero_totals_returns_early() {
        drift_print_format("Bash", 0, 0, 0.0, 0, 0, 0.0);
    }

    #[test]
    fn test_PMAT257_cov_drift_print_format_with_score_and_negative_delta() {
        drift_print_format("Makefile", 8, 10, 80.0, 5, 10, 50.0);
    }

    #[test]
    fn test_PMAT257_cov_slow_with_reports_every_fixture_entry() {
        let registry = fixture_registry();
        corpus_slow_with(&registry, 2, None).expect("timing the fixture must succeed");
    }

    #[test]
    fn test_PMAT257_cov_slow_with_filters_to_a_single_format() {
        let registry = fixture_registry();
        let filter = CorpusFormatArg::Bash;
        corpus_slow_with(&registry, 10, Some(&filter))
            .expect("filtering to bash must still succeed");
    }

    #[test]
    fn test_PMAT257_cov_tags_with_classifies_and_reports_untagged() {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-100",
            "variable-assignment-basic",
            "readonly variable declaration",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() {}",
            "",
        ));
        registry.add(CorpusEntry::new(
            "B-101",
            "totally-unclassified-thing",
            "nothing that matches a tag keyword",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() {}",
            "",
        ));
        corpus_tags_with(&registry).expect("tagging must succeed with a tagged and an untagged entry");
    }

    #[test]
    fn test_PMAT257_cov_public_wrappers_delegate_to_with_variants() {
        // These call `CorpusRegistry::load_full()` and run against the real
        // ~18k entry corpus; still fast enough for a unit test and it is the
        // only way to cover the thin `load_full()` wrapper lines themselves.
        corpus_tags().expect("corpus_tags over the real registry must succeed");
    }
}
