#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: corpus-registry-v1.yaml (PMAT-245 / #284)
//!
//! The corpus registry is the project's regression corpus. From 2026-03-25
//! through v7.0.1 it silently held zero entries, and `bashrs corpus run`
//! reported 0 failures / 0 entries as a clean run. Every test here is a way
//! for an empty, truncated, or filler-populated registry to FAIL loudly.
//!
//! Reference: docs/audit/quality-report09-2026.md §2, GH-284, PR #285.

use std::collections::HashSet;

use bashrs::corpus::{CorpusFormat, CorpusRegistry};

/// The data file the registry is compiled from, read here independently of
/// the loader so a truncated parse cannot agree with itself.
const CORPUS_DATA: &str = include_str!("../src/corpus/registry/corpus_data.jsonl");

/// Release bar (CLAUDE.md, Release Schedule): `bashrs corpus run` must report
/// at least this many entries before a release may ship.
const RELEASE_BAR: usize = 17_942;

fn ids(registry: &CorpusRegistry) -> Vec<&str> {
    registry.entries.iter().map(|e| e.id.as_str()).collect()
}

fn id_set(registry: &CorpusRegistry) -> HashSet<&str> {
    ids(registry).into_iter().collect()
}

// ============================================================================
// F-CORPUS-001: load_full() meets the release bar
// ============================================================================

#[test]
fn F_CORPUS_001_load_full_meets_release_bar() {
    let n = CorpusRegistry::load_full().len();
    assert!(
        n >= RELEASE_BAR,
        "load_full() returned {n} entries, release bar is {RELEASE_BAR} — \
         the corpus has been stubbed or truncated again (GH-284)"
    );
}

// ============================================================================
// F-CORPUS-002: a known entry is present and is not filler
// ============================================================================

#[test]
fn F_CORPUS_002_b001_is_pinned_and_not_filler() {
    let registry = CorpusRegistry::load_full();
    let b001 = registry
        .entries
        .iter()
        .find(|e| e.id == "B-001")
        .expect("entry B-001 missing — the registry is populated but not with the corpus");
    assert!(!b001.input.is_empty(), "B-001 has an empty input");
    assert!(
        !b001.expected_output.is_empty(),
        "B-001 has an empty expected_output"
    );
    assert_eq!(b001.format, CorpusFormat::Bash, "B-001 is a Bash entry");
    assert_eq!(
        b001.name, "variable-assignment",
        "B-001 is the variable-assignment entry"
    );
}

// ============================================================================
// F-CORPUS-003: entry ids are unique
// ============================================================================

#[test]
fn F_CORPUS_003_entry_ids_are_unique() {
    let registry = CorpusRegistry::load_full();
    let mut seen = HashSet::new();
    let dups: Vec<&str> = ids(&registry)
        .into_iter()
        .filter(|id| !seen.insert(*id))
        .collect();
    assert!(
        dups.is_empty(),
        "{} duplicate entry ids inflate the count without adding coverage; first: {:?}",
        dups.len(),
        dups.first()
    );
}

// ============================================================================
// F-CORPUS-004: no entry is empty
// ============================================================================

#[test]
fn F_CORPUS_004_no_entry_is_empty() {
    let registry = CorpusRegistry::load_full();
    let empty: Vec<String> = registry
        .entries
        .iter()
        .filter(|e| {
            e.id.is_empty()
                || e.name.is_empty()
                || e.input.is_empty()
                || e.expected_output.is_empty()
        })
        .map(|e| e.id.clone())
        .collect();
    assert!(
        empty.is_empty(),
        "{} entries have an empty id, name, input or expected_output and would pass every \
         dimension vacuously; first: {:?}",
        empty.len(),
        empty.first()
    );
}

// ============================================================================
// F-CORPUS-005: all three formats are represented at their measured scale
// ============================================================================

#[test]
fn F_CORPUS_005_all_three_formats_present() {
    let registry = CorpusRegistry::load_full();
    let count = |f: CorpusFormat| registry.by_format(f).len();
    let (bash, make, docker) = (
        count(CorpusFormat::Bash),
        count(CorpusFormat::Makefile),
        count(CorpusFormat::Dockerfile),
    );
    // Measured at the restore (PR #285): 16,431 / 804 / 707.
    assert!(bash >= 16_000, "Bash entries: {bash}, expected >= 16000");
    assert!(make >= 800, "Makefile entries: {make}, expected >= 800");
    assert!(
        docker >= 700,
        "Dockerfile entries: {docker}, expected >= 700"
    );
    assert_eq!(
        bash + make + docker,
        registry.len(),
        "every entry has one of the three formats"
    );
}

// ============================================================================
// F-CORPUS-006: load_full() is deterministic
// ============================================================================

#[test]
fn F_CORPUS_006_load_full_is_deterministic() {
    let first = CorpusRegistry::load_full();
    let second = CorpusRegistry::load_full();
    assert_eq!(first.len(), second.len(), "two loads disagree on the count");
    assert_eq!(
        ids(&first),
        ids(&second),
        "two loads return different ids or a different order"
    );
}

// ============================================================================
// F-CORPUS-007: the registry agrees with the data file
// ============================================================================

#[test]
fn F_CORPUS_007_registry_agrees_with_data_file() {
    let lines = CORPUS_DATA.lines().filter(|l| !l.trim().is_empty()).count();
    let loaded = CorpusRegistry::load_full().len();
    assert_eq!(
        loaded, lines,
        "load_full() has {loaded} entries but corpus_data.jsonl has {lines} non-empty lines — \
         the parser dropped lines, so the count the runner reports is not the corpus that ships"
    );
    assert!(
        lines >= RELEASE_BAR,
        "the shipped data file itself is below the release bar: {lines}"
    );
}

// ============================================================================
// F-CORPUS-008: tier loaders are non-empty subsets of load_full()
// ============================================================================

#[test]
fn F_CORPUS_008_tier_loads_are_subsets_of_full() {
    let full = CorpusRegistry::load_full();
    let full_ids = id_set(&full);
    let chain = [
        ("load_tier1", CorpusRegistry::load_tier1()),
        (
            "load_tier1_and_tier2",
            CorpusRegistry::load_tier1_and_tier2(),
        ),
        ("load_all", CorpusRegistry::load_all()),
        (
            "load_all_with_adversarial",
            CorpusRegistry::load_all_with_adversarial(),
        ),
    ];
    let mut prev: Option<(&str, HashSet<&str>)> = None;
    for (name, registry) in &chain {
        let this = id_set(registry);
        assert!(!this.is_empty(), "{name}() returned no entries");
        assert!(
            this.is_subset(&full_ids),
            "{name}() selects entries load_full() does not contain"
        );
        if let Some((prev_name, prev_ids)) = &prev {
            assert!(
                prev_ids.is_subset(&this),
                "{prev_name}() is not a subset of {name}() — the tier chain is not monotone"
            );
        }
        prev = Some((name, this));
    }
    assert!(
        chain[3].1.len() < full.len(),
        "load_full() must be strictly larger than the adversarial subset, else the bitmask is unused"
    );
}
