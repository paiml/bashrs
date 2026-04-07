//! Coverage tests for corpus/registry.rs loading functions.
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier, Grade};

#[test]
fn test_REG_COV_031_by_format_and_tier_makefile_standard() {
    let r = CorpusRegistry::load_tier1_and_tier2();
    for e in r.by_format_and_tier(CorpusFormat::Makefile, CorpusTier::Standard) {
        assert_eq!(e.format, CorpusFormat::Makefile);
        assert_eq!(e.tier, CorpusTier::Standard);
    }
}

#[test]
fn test_REG_COV_032_count_by_format_matches() {
    let r = CorpusRegistry::load_tier1();
    assert_eq!(
        r.count_by_format(CorpusFormat::Bash),
        r.by_format(CorpusFormat::Bash).len()
    );
    assert_eq!(
        r.count_by_format(CorpusFormat::Makefile),
        r.by_format(CorpusFormat::Makefile).len()
    );
    assert_eq!(
        r.count_by_format(CorpusFormat::Dockerfile),
        r.by_format(CorpusFormat::Dockerfile).len()
    );
}

#[test]
fn test_REG_COV_033_tier_ordering() {
    assert!(CorpusTier::Trivial < CorpusTier::Standard);
    assert!(CorpusTier::Standard < CorpusTier::Complex);
    assert!(CorpusTier::Complex < CorpusTier::Adversarial);
    assert!(CorpusTier::Adversarial < CorpusTier::Production);
}

#[test]
fn test_REG_COV_034_entry_debug_clone() {
    let e = CorpusEntry::new(
        "B-X",
        "t",
        "d",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        "i",
        "o",
    );
    assert_eq!(e.id, e.clone().id);
    let _ = format!("{:?}", e);
}

#[test]
fn test_REG_COV_035_registry_debug_clone() {
    let r = CorpusRegistry::load_tier1();
    assert_eq!(r.len(), r.clone().len());
    let _ = format!("{:?}", r);
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_036_m001_correct() {
    let r = CorpusRegistry::load_tier1();
    let m = r.entries.iter().find(|e| e.id == "M-001").unwrap();
    assert_eq!(m.format, CorpusFormat::Makefile);
    assert_eq!(m.tier, CorpusTier::Trivial);
    assert!(!m.shellcheck);
    assert!(m.expected_output.contains("CC"));
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_037_d001_correct() {
    let r = CorpusRegistry::load_tier1();
    let d = r.entries.iter().find(|e| e.id == "D-001").unwrap();
    assert_eq!(d.format, CorpusFormat::Dockerfile);
    assert!(d.expected_output.contains("FROM"));
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_038_b001_correct() {
    let r = CorpusRegistry::load_tier1();
    let b = r.entries.iter().find(|e| e.id == "B-001").unwrap();
    assert_eq!(b.format, CorpusFormat::Bash);
    assert!(b.shellcheck);
    assert!(b.deterministic);
    assert!(b.idempotent);
}

#[test]
fn test_REG_COV_039_entry_serde_roundtrip() {
    let e = CorpusEntry::new(
        "B-S",
        "s",
        "d",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "i",
        "o",
    );
    let j = serde_json::to_string(&e).unwrap();
    let d: CorpusEntry = serde_json::from_str(&j).unwrap();
    assert_eq!(d.id, e.id);
    assert_eq!(d.format, e.format);
    assert_eq!(d.shellcheck, e.shellcheck);
}

#[test]
fn test_REG_COV_040_registry_serde_roundtrip() {
    let r = CorpusRegistry::load_tier1();
    let j = serde_json::to_string(&r).unwrap();
    let d: CorpusRegistry = serde_json::from_str(&j).unwrap();
    assert_eq!(d.len(), r.len());
}
