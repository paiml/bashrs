#[cfg(test)]
mod analysis_validate_corpus_entry {
    use crate::cli::commands::corpus_analysis_commands::validate_corpus_entry;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    use std::collections::HashSet;

    #[test]
    fn test_valid_bash_entry_no_issues() {
        let entry = CorpusEntry::new(
            "B-001",
            "test-entry",
            "A test entry",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.is_empty(),
            "Valid entry should have no issues: {issues:?}"
        );
    }

    #[test]
    fn test_duplicate_id_is_reported() {
        let entry = CorpusEntry::new(
            "B-001",
            "test-entry",
            "A test entry",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        seen.insert("B-001".to_string()); // Pre-insert to simulate duplicate
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("Duplicate")),
            "Should report duplicate ID: {issues:?}"
        );
    }

    #[test]
    fn test_wrong_prefix_bash_reported() {
        let entry = CorpusEntry::new(
            "M-001", // Wrong prefix for Bash format
            "test",
            "description",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("prefix")),
            "Should report prefix mismatch: {issues:?}"
        );
    }

    #[test]
    fn test_wrong_prefix_makefile_reported() {
        let entry = CorpusEntry::new(
            "B-001", // Wrong prefix for Makefile format
            "make-test",
            "description",
            CorpusFormat::Makefile,
            CorpusTier::Standard,
            "all:\n\techo hello",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("prefix")),
            "Should report prefix mismatch: {issues:?}"
        );
    }

    #[test]
    fn test_seen_ids_updated_after_validation() {
        let entry = CorpusEntry::new(
            "B-042",
            "test",
            "description",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "fn main() { println!(\"hello\"); }",
            "hello",
        );
        let mut seen = HashSet::new();
        let _ = validate_corpus_entry(&entry, &mut seen);
        assert!(seen.contains("B-042"), "Seen IDs should contain B-042");
    }

    #[test]
    fn test_makefile_no_fn_main_requirement() {
        let entry = CorpusEntry::new(
            "M-001",
            "make-test",
            "A Makefile entry",
            CorpusFormat::Makefile,
            CorpusTier::Standard,
            "all:\n\techo hello",
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        let has_main_issue = issues.iter().any(|i| i.contains("fn main"));
        assert!(
            !has_main_issue,
            "Makefile should not require fn main(): {issues:?}"
        );
    }

    #[test]
    fn test_bash_missing_fn_main_reported() {
        let entry = CorpusEntry::new(
            "B-999",
            "no-main",
            "Entry without fn main",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            "echo hello", // No fn main()
            "hello",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(
            issues.iter().any(|i| i.contains("fn main")),
            "Bash entry missing fn main() should be reported: {issues:?}"
        );
    }

    #[test]
    fn test_dockerfile_prefix_d_is_valid() {
        let entry = CorpusEntry::new(
            "D-001",
            "docker-test",
            "A Dockerfile entry",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18",
            "FROM alpine",
        );
        let mut seen = HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        let prefix_issue = issues.iter().any(|i| i.contains("prefix"));
        assert!(
            !prefix_issue,
            "D- prefix for Dockerfile should be valid: {issues:?}"
        );
    }
}
