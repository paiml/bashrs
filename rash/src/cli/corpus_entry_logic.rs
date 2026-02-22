//! Corpus entry validation, classification, and metadata logic.
//!
//! This module contains pure logic functions for validating individual corpus
//! entries, counting entries by format, and classifying entries for B2
//! diagnosis. All functions are stateless and free of I/O side effects.

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry};

/// Validate a single corpus entry and return a list of issue descriptions.
///
/// Checks for:
/// - Duplicate IDs (tracked via `seen_ids`)
/// - ID prefix consistency with format (`B-`, `M-`, `D-`)
/// - Non-empty name, description, input, and expected_output
/// - Bash entries must contain `fn main()`
pub fn validate_corpus_entry(
    entry: &CorpusEntry,
    seen_ids: &mut std::collections::HashSet<String>,
) -> Vec<String> {
    let mut issues = Vec::new();

    if !seen_ids.insert(entry.id.clone()) {
        issues.push("Duplicate ID".to_string());
    }

    let valid_prefix = match entry.format {
        CorpusFormat::Bash => entry.id.starts_with("B-"),
        CorpusFormat::Makefile => entry.id.starts_with("M-"),
        CorpusFormat::Dockerfile => entry.id.starts_with("D-"),
    };
    if !valid_prefix {
        issues.push(format!("ID prefix doesn't match format {:?}", entry.format));
    }

    if entry.name.is_empty() {
        issues.push("Empty name".to_string());
    }
    if entry.description.is_empty() {
        issues.push("Empty description".to_string());
    }
    if entry.input.is_empty() {
        issues.push("Empty input".to_string());
    }
    if entry.expected_output.is_empty() {
        issues.push("Empty expected_output".to_string());
    }
    if entry.format == CorpusFormat::Bash && !entry.input.contains("fn main()") {
        issues.push("Bash entry missing fn main()".to_string());
    }

    issues
}

/// Count entries in a registry that match the given format.
#[must_use]
pub fn count_format(registry: &CorpusRegistry, format: &CorpusFormat) -> usize {
    registry
        .entries
        .iter()
        .filter(|e| &e.format == format)
        .count()
}

/// Count entries in a registry that match all given formats, returning `(bash, makefile, dockerfile)`.
#[must_use]
pub fn count_all_formats(registry: &CorpusRegistry) -> (usize, usize, usize) {
    let bash = count_format(registry, &CorpusFormat::Bash);
    let makefile = count_format(registry, &CorpusFormat::Makefile);
    let dockerfile = count_format(registry, &CorpusFormat::Dockerfile);
    (bash, makefile, dockerfile)
}

/// Check whether an entry ID has the correct prefix for its declared format.
#[must_use]
pub fn id_has_valid_prefix(id: &str, format: &CorpusFormat) -> bool {
    match format {
        CorpusFormat::Bash => id.starts_with("B-"),
        CorpusFormat::Makefile => id.starts_with("M-"),
        CorpusFormat::Dockerfile => id.starts_with("D-"),
    }
}

/// Extract the numeric part from a corpus entry ID (e.g. `"B-042"` → `Some(42)`).
///
/// Returns `None` if the ID does not contain a `-` separator or the numeric
/// part cannot be parsed.
#[must_use]
pub fn id_numeric(id: &str) -> Option<u32> {
    id.find('-')
        .and_then(|pos| id[pos + 1..].parse::<u32>().ok())
}

/// Return the format prefix character for a `CorpusFormat`.
///
/// - `Bash` → `'B'`
/// - `Makefile` → `'M'`
/// - `Dockerfile` → `'D'`
#[must_use]
pub fn format_prefix(format: &CorpusFormat) -> char {
    match format {
        CorpusFormat::Bash => 'B',
        CorpusFormat::Makefile => 'M',
        CorpusFormat::Dockerfile => 'D',
    }
}

/// Determine whether a corpus entry is a "milestone" entry (spec §11.11).
///
/// Milestone entries have names containing the word `"milestone"` (case-insensitive).
#[must_use]
pub fn is_milestone_entry(entry: &CorpusEntry) -> bool {
    entry.name.to_lowercase().contains("milestone")
}

/// Compute entry metadata summary: (id, format_prefix, name_length, input_lines, has_main).
#[must_use]
pub fn entry_metadata_summary(entry: &CorpusEntry) -> (String, char, usize, usize, bool) {
    let prefix = format_prefix(&entry.format);
    let name_len = entry.name.len();
    let input_lines = entry.input.lines().count();
    let has_main = entry.input.contains("fn main()");
    (entry.id.clone(), prefix, name_len, input_lines, has_main)
}

/// Validate all entries in the registry and return grouped issues.
///
/// Returns a vector of `(entry_id, issue)` pairs for all entries with problems.
#[must_use]
pub fn validate_all_entries(registry: &CorpusRegistry) -> Vec<(String, String)> {
    let mut seen_ids = std::collections::HashSet::new();
    let mut all_issues = Vec::new();
    for entry in &registry.entries {
        for issue in validate_corpus_entry(entry, &mut seen_ids) {
            all_issues.push((entry.id.clone(), issue));
        }
    }
    all_issues
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::corpus::registry::CorpusTier;

    fn make_entry(id: &str, format: CorpusFormat, name: &str, input: &str) -> CorpusEntry {
        CorpusEntry::new(
            id,
            name,
            "test description",
            format,
            CorpusTier::Trivial,
            input,
            "expected output",
        )
    }

    fn valid_bash(id: &str) -> CorpusEntry {
        make_entry(id, CorpusFormat::Bash, "test entry", "fn main() {\n  println!(\"hi\");\n}")
    }

    fn valid_makefile(id: &str) -> CorpusEntry {
        make_entry(id, CorpusFormat::Makefile, "make entry", "all:\n\techo done")
    }

    fn valid_dockerfile(id: &str) -> CorpusEntry {
        make_entry(id, CorpusFormat::Dockerfile, "docker entry", "FROM alpine\nRUN echo hi")
    }

    // ===== validate_corpus_entry tests =====

    #[test]
    fn test_CORPUS_ENTRY_001_validate_entry_valid_bash_no_issues() {
        let entry = valid_bash("B-001");
        let mut seen = std::collections::HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(issues.is_empty(), "Expected no issues, got: {issues:?}");
    }

    #[test]
    fn test_CORPUS_ENTRY_002_validate_entry_duplicate_id_detected() {
        let entry1 = valid_bash("B-001");
        let entry2 = valid_bash("B-001");
        let mut seen = std::collections::HashSet::new();
        validate_corpus_entry(&entry1, &mut seen);
        let issues = validate_corpus_entry(&entry2, &mut seen);
        assert!(issues.contains(&"Duplicate ID".to_string()));
    }

    #[test]
    fn test_CORPUS_ENTRY_003_validate_entry_wrong_prefix_detected() {
        let entry = make_entry("M-001", CorpusFormat::Bash, "test", "fn main() {}");
        let mut seen = std::collections::HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(issues.iter().any(|i| i.contains("ID prefix")));
    }

    #[test]
    fn test_CORPUS_ENTRY_004_validate_entry_empty_name_detected() {
        let entry = make_entry("B-001", CorpusFormat::Bash, "", "fn main() {}");
        let mut seen = std::collections::HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(issues.contains(&"Empty name".to_string()));
    }

    #[test]
    fn test_CORPUS_ENTRY_005_validate_entry_bash_missing_main_detected() {
        let entry = make_entry("B-001", CorpusFormat::Bash, "test", "echo hello");
        let mut seen = std::collections::HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(issues.iter().any(|i| i.contains("fn main()")));
    }

    #[test]
    fn test_CORPUS_ENTRY_006_validate_entry_makefile_no_main_check() {
        let entry = valid_makefile("M-001");
        let mut seen = std::collections::HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        // Makefiles don't require fn main()
        assert!(!issues.iter().any(|i| i.contains("fn main()")));
    }

    #[test]
    fn test_CORPUS_ENTRY_007_validate_entry_dockerfile_valid() {
        let entry = valid_dockerfile("D-001");
        let mut seen = std::collections::HashSet::new();
        let issues = validate_corpus_entry(&entry, &mut seen);
        assert!(issues.is_empty(), "Expected no issues, got: {issues:?}");
    }

    // ===== count_format tests =====

    #[test]
    fn test_CORPUS_ENTRY_008_count_format_bash_only() {
        let registry = CorpusRegistry {
            entries: vec![
                valid_bash("B-001"),
                valid_bash("B-002"),
                valid_makefile("M-001"),
            ],
        };
        assert_eq!(count_format(&registry, &CorpusFormat::Bash), 2);
        assert_eq!(count_format(&registry, &CorpusFormat::Makefile), 1);
        assert_eq!(count_format(&registry, &CorpusFormat::Dockerfile), 0);
    }

    #[test]
    fn test_CORPUS_ENTRY_009_count_format_empty_registry() {
        let registry = CorpusRegistry { entries: vec![] };
        assert_eq!(count_format(&registry, &CorpusFormat::Bash), 0);
    }

    // ===== count_all_formats tests =====

    #[test]
    fn test_CORPUS_ENTRY_010_count_all_formats_mixed() {
        let registry = CorpusRegistry {
            entries: vec![
                valid_bash("B-001"),
                valid_bash("B-002"),
                valid_bash("B-003"),
                valid_makefile("M-001"),
                valid_makefile("M-002"),
                valid_dockerfile("D-001"),
            ],
        };
        let (bash, make, docker) = count_all_formats(&registry);
        assert_eq!(bash, 3);
        assert_eq!(make, 2);
        assert_eq!(docker, 1);
    }

    // ===== id_has_valid_prefix tests =====

    #[test]
    fn test_CORPUS_ENTRY_011_id_has_valid_prefix_bash_correct() {
        assert!(id_has_valid_prefix("B-001", &CorpusFormat::Bash));
        assert!(!id_has_valid_prefix("M-001", &CorpusFormat::Bash));
    }

    #[test]
    fn test_CORPUS_ENTRY_012_id_has_valid_prefix_makefile_correct() {
        assert!(id_has_valid_prefix("M-042", &CorpusFormat::Makefile));
        assert!(!id_has_valid_prefix("B-042", &CorpusFormat::Makefile));
    }

    #[test]
    fn test_CORPUS_ENTRY_013_id_has_valid_prefix_dockerfile_correct() {
        assert!(id_has_valid_prefix("D-007", &CorpusFormat::Dockerfile));
        assert!(!id_has_valid_prefix("B-007", &CorpusFormat::Dockerfile));
    }

    // ===== id_numeric tests =====

    #[test]
    fn test_CORPUS_ENTRY_014_id_numeric_parses_correctly() {
        assert_eq!(id_numeric("B-042"), Some(42));
        assert_eq!(id_numeric("M-001"), Some(1));
        assert_eq!(id_numeric("D-15769"), Some(15769));
    }

    #[test]
    fn test_CORPUS_ENTRY_015_id_numeric_invalid_returns_none() {
        assert_eq!(id_numeric("nohyphen"), None);
        assert_eq!(id_numeric("B-abc"), None);
        assert_eq!(id_numeric(""), None);
    }

    // ===== format_prefix tests =====

    #[test]
    fn test_CORPUS_ENTRY_016_format_prefix_correct_chars() {
        assert_eq!(format_prefix(&CorpusFormat::Bash), 'B');
        assert_eq!(format_prefix(&CorpusFormat::Makefile), 'M');
        assert_eq!(format_prefix(&CorpusFormat::Dockerfile), 'D');
    }

    // ===== is_milestone_entry tests =====

    #[test]
    fn test_CORPUS_ENTRY_017_is_milestone_entry_detects_keyword() {
        let entry = make_entry("B-100", CorpusFormat::Bash, "Milestone 100", "fn main() {}");
        assert!(is_milestone_entry(&entry));
    }

    #[test]
    fn test_CORPUS_ENTRY_018_is_milestone_entry_case_insensitive() {
        let entry = make_entry("B-100", CorpusFormat::Bash, "MILESTONE marker", "fn main() {}");
        assert!(is_milestone_entry(&entry));
    }

    #[test]
    fn test_CORPUS_ENTRY_019_is_milestone_entry_regular_entry_false() {
        let entry = valid_bash("B-001");
        assert!(!is_milestone_entry(&entry));
    }

    // ===== entry_metadata_summary tests =====

    #[test]
    fn test_CORPUS_ENTRY_020_entry_metadata_summary_bash() {
        let entry = valid_bash("B-007");
        let (id, prefix, _name_len, input_lines, has_main) = entry_metadata_summary(&entry);
        assert_eq!(id, "B-007");
        assert_eq!(prefix, 'B');
        assert!(input_lines >= 1);
        assert!(has_main);
    }

    #[test]
    fn test_CORPUS_ENTRY_021_entry_metadata_summary_makefile_no_main() {
        let entry = valid_makefile("M-003");
        let (id, prefix, _, _, has_main) = entry_metadata_summary(&entry);
        assert_eq!(id, "M-003");
        assert_eq!(prefix, 'M');
        assert!(!has_main);
    }

    // ===== validate_all_entries tests =====

    #[test]
    fn test_CORPUS_ENTRY_022_validate_all_entries_clean_registry() {
        let registry = CorpusRegistry {
            entries: vec![valid_bash("B-001"), valid_makefile("M-001"), valid_dockerfile("D-001")],
        };
        let issues = validate_all_entries(&registry);
        assert!(issues.is_empty(), "Expected no issues: {issues:?}");
    }

    #[test]
    fn test_CORPUS_ENTRY_023_validate_all_entries_catches_duplicate() {
        let registry = CorpusRegistry {
            entries: vec![valid_bash("B-001"), valid_bash("B-001")],
        };
        let issues = validate_all_entries(&registry);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|(_, i)| i.contains("Duplicate")));
    }
}
