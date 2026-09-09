//! Coverage tests for corpus registry load functions.
//! Calling load_full() exercises ALL internal load_tier* and load_expansion* methods,
//! covering ~500+ lines of corpus data construction.
#![allow(clippy::unwrap_used)]

use crate::corpus::registry::{CorpusFormat, CorpusRegistry};

#[test]
#[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_coverage_load_full_all_formats() {
    let registry = CorpusRegistry::load_full();

    assert!(
        registry.entries.len() > 10000,
        "Full registry should have >10k entries, got {}",
        registry.entries.len()
    );

    let bash_count = registry
        .entries
        .iter()
        .filter(|e| matches!(e.format, CorpusFormat::Bash))
        .count();
    let make_count = registry
        .entries
        .iter()
        .filter(|e| matches!(e.format, CorpusFormat::Makefile))
        .count();
    let docker_count = registry
        .entries
        .iter()
        .filter(|e| matches!(e.format, CorpusFormat::Dockerfile))
        .count();

    assert!(
        bash_count > 5000,
        "Should have >5000 bash entries, got {bash_count}"
    );
    assert!(
        make_count > 100,
        "Should have >100 makefile entries, got {make_count}"
    );
    assert!(
        docker_count > 100,
        "Should have >100 dockerfile entries, got {docker_count}"
    );
}

#[test]
fn test_coverage_load_full_ids_valid() {
    let registry = CorpusRegistry::load_full();
    for entry in &registry.entries {
        assert!(!entry.id.is_empty(), "Entry should have non-empty id");
        assert!(
            !entry.name.is_empty(),
            "Entry {} should have non-empty name",
            entry.id
        );
        assert!(
            !entry.input.is_empty(),
            "Entry {} should have non-empty input",
            entry.id
        );
    }
}

#[test]
#[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_coverage_load_full_by_format() {
    let registry = CorpusRegistry::load_full();
    let bash_entries = registry.by_format(CorpusFormat::Bash);
    let make_entries = registry.by_format(CorpusFormat::Makefile);
    let docker_entries = registry.by_format(CorpusFormat::Dockerfile);

    assert!(bash_entries.len() > make_entries.len());
    assert!(bash_entries.len() > docker_entries.len());
    assert_eq!(
        bash_entries.len() + make_entries.len() + docker_entries.len(),
        registry.entries.len()
    );
}

/// PMAT-245 / #284: the corpus registry must actually contain the corpus.
///
/// `corpus_data.rs` was replaced by no-op stubs whose header claimed "the actual
/// corpus data is loaded at runtime via `rash corpus load`". No such subcommand
/// exists and no corpus data file ships, so `load_full()` silently returned an
/// empty registry and `bashrs corpus run` scored 0 entries as a clean run for
/// five months. A 0-entry corpus cannot fail; this test is the gate that makes
/// it fail.
#[test]
fn test_PMAT245_corpus_registry_not_empty() {
    let registry = CorpusRegistry::load_full();
    let n = registry.len();
    assert!(
        n >= 17_000,
        "corpus registry loaded {n} entries, expected >= 17000 — \
         corpus_data.rs has been stubbed out again (see #284)"
    );

    // A count alone is satisfiable by synthetic filler, so pin a known entry:
    // B-001 is the first corpus entry and has been stable since the corpus existed.
    let b001 =
        registry.entries.iter().find(|e| e.id == "B-001").expect(
            "corpus entry B-001 missing — the registry is populated but not with the corpus",
        );
    assert!(
        !b001.input.is_empty() && !b001.expected_output.is_empty(),
        "corpus entry B-001 has empty input or expected_output — filler, not corpus data"
    );
}
