//! Coverage tests for corpus/registry.rs loading functions.
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier, Grade};

fn assert_entries_valid(entries: &[CorpusEntry], ctx: &str) {
    assert!(!entries.is_empty(), "{ctx}: expected non-empty list");
    for e in entries {
        assert!(!e.id.is_empty(), "{ctx}: empty id");
        assert!(!e.name.is_empty(), "{ctx}: {} empty name", e.id);
        assert!(!e.input.is_empty(), "{ctx}: {} empty input", e.id);
        assert!(
            !e.expected_output.is_empty(),
            "{ctx}: {} empty expected_output",
            e.id
        );
    }
}

#[test]
fn test_REG_COV_001_new_returns_empty_registry() {
    let reg = CorpusRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

#[test]
fn test_REG_COV_002_default_matches_new() {
    assert!(CorpusRegistry::default().is_empty());
}

#[test]
fn test_REG_COV_003_add_entry_increases_len() {
    let mut reg = CorpusRegistry::new();
    reg.add(CorpusEntry::new(
        "T-001",
        "t",
        "d",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        "in",
        "out",
    ));
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());
}

#[test]
fn test_REG_COV_004_entry_new_bash_fields() {
    let e = CorpusEntry::new(
        "B-T",
        "n",
        "d",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "i",
        "o",
    );
    assert!(e.shellcheck && e.deterministic && e.idempotent);
    assert_eq!(
        (e.id.as_str(), e.name.as_str(), e.description.as_str()),
        ("B-T", "n", "d")
    );
    assert_eq!(e.format, CorpusFormat::Bash);
    assert_eq!(e.tier, CorpusTier::Standard);
    assert_eq!((e.input.as_str(), e.expected_output.as_str()), ("i", "o"));
}

#[test]
fn test_REG_COV_005_entry_new_makefile_no_shellcheck() {
    assert!(
        !CorpusEntry::new(
            "M-T",
            "n",
            "d",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "i",
            "o"
        )
        .shellcheck
    );
}

#[test]
fn test_REG_COV_006_entry_new_dockerfile_no_shellcheck() {
    assert!(
        !CorpusEntry::new(
            "D-T",
            "n",
            "d",
            CorpusFormat::Dockerfile,
            CorpusTier::Complex,
            "i",
            "o"
        )
        .shellcheck
    );
}

#[test]
fn test_REG_COV_007_tier_weights() {
    assert_eq!(CorpusTier::Trivial.weight(), 1.0);
    assert_eq!(CorpusTier::Standard.weight(), 1.5);
    assert_eq!(CorpusTier::Complex.weight(), 2.0);
    assert_eq!(CorpusTier::Adversarial.weight(), 2.5);
    assert_eq!(CorpusTier::Production.weight(), 3.0);
}

#[test]
fn test_REG_COV_008_tier_target_rates() {
    assert_eq!(CorpusTier::Trivial.target_rate(), 1.0);
    assert_eq!(CorpusTier::Standard.target_rate(), 0.99);
    assert_eq!(CorpusTier::Complex.target_rate(), 0.98);
    assert_eq!(CorpusTier::Adversarial.target_rate(), 0.95);
    assert_eq!(CorpusTier::Production.target_rate(), 0.95);
}

#[test]
fn test_REG_COV_009_grade_from_score_all_branches() {
    assert_eq!(Grade::from_score(100.0), Grade::APlus);
    assert_eq!(Grade::from_score(97.0), Grade::APlus);
    assert_eq!(Grade::from_score(96.9), Grade::A);
    assert_eq!(Grade::from_score(90.0), Grade::A);
    assert_eq!(Grade::from_score(89.9), Grade::B);
    assert_eq!(Grade::from_score(80.0), Grade::B);
    assert_eq!(Grade::from_score(79.9), Grade::C);
    assert_eq!(Grade::from_score(70.0), Grade::C);
    assert_eq!(Grade::from_score(69.9), Grade::D);
    assert_eq!(Grade::from_score(60.0), Grade::D);
    assert_eq!(Grade::from_score(59.9), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
}

#[test]
fn test_REG_COV_010_grade_display() {
    assert_eq!(format!("{}", Grade::APlus), "A+");
    assert_eq!(format!("{}", Grade::A), "A");
    assert_eq!(format!("{}", Grade::B), "B");
    assert_eq!(format!("{}", Grade::C), "C");
    assert_eq!(format!("{}", Grade::D), "D");
    assert_eq!(format!("{}", Grade::F), "F");
}

#[test]
fn test_REG_COV_011_format_display() {
    assert_eq!(format!("{}", CorpusFormat::Bash), "bash");
    assert_eq!(format!("{}", CorpusFormat::Makefile), "makefile");
    assert_eq!(format!("{}", CorpusFormat::Dockerfile), "dockerfile");
}

#[test]
fn test_REG_COV_012_load_tier1_has_all_formats() {
    let r = CorpusRegistry::load_tier1();
    assert!(!r.is_empty());
    assert!(r.count_by_format(CorpusFormat::Bash) > 0);
    assert!(r.count_by_format(CorpusFormat::Makefile) > 0);
    assert!(r.count_by_format(CorpusFormat::Dockerfile) > 0);
}

#[test]
fn test_REG_COV_013_load_tier1_all_trivial() {
    let r = CorpusRegistry::load_tier1();
    for e in &r.entries {
        assert_eq!(e.tier, CorpusTier::Trivial, "entry {} not Trivial", e.id);
    }
}

#[test]
fn test_REG_COV_014_load_tier1_known_ids() {
    let r = CorpusRegistry::load_tier1();
    let ids: Vec<&str> = r.entries.iter().map(|e| e.id.as_str()).collect();
    for id in &["B-001", "B-002", "M-001", "D-001"] {
        assert!(ids.contains(id), "{id} missing");
    }
}

#[test]
fn test_REG_COV_041_load_tier1_bash_entries_valid() {
    let r = CorpusRegistry::load_tier1();
    let bash: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Bash)
        .cloned()
        .collect();
    assert!(bash.len() >= 5, "tier1 bash: {}", bash.len());
    assert_entries_valid(&bash, "tier1_bash");
    for e in &bash {
        assert!(e.id.starts_with("B-"), "bad id {}", e.id);
        assert!(e.shellcheck);
    }
}

#[test]
fn test_REG_COV_042_load_tier1_makefile_entries_valid() {
    let r = CorpusRegistry::load_tier1();
    let mf: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Makefile)
        .cloned()
        .collect();
    assert!(mf.len() >= 5, "tier1 makefile: {}", mf.len());
    assert_entries_valid(&mf, "tier1_makefile");
    for e in &mf {
        assert!(e.id.starts_with("M-"));
        assert!(!e.shellcheck);
    }
}

#[test]
fn test_REG_COV_043_load_tier1_dockerfile_entries_valid() {
    let r = CorpusRegistry::load_tier1();
    let df: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Dockerfile)
        .cloned()
        .collect();
    assert!(df.len() >= 5, "tier1 dockerfile: {}", df.len());
    assert_entries_valid(&df, "tier1_dockerfile");
    for e in &df {
        assert!(e.id.starts_with("D-"));
        assert!(!e.shellcheck);
    }
}

#[test]
fn test_REG_COV_015_tier12_larger_than_tier1() {
    assert!(CorpusRegistry::load_tier1_and_tier2().len() > CorpusRegistry::load_tier1().len());
}

#[test]
fn test_REG_COV_016_tier12_has_standard() {
    assert!(!CorpusRegistry::load_tier1_and_tier2()
        .by_tier(CorpusTier::Standard)
        .is_empty());
}

#[test]
fn test_REG_COV_044_tier2_bash_entries_valid() {
    let r = CorpusRegistry::load_tier1_and_tier2();
    let t2: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Bash && e.tier == CorpusTier::Standard)
        .cloned()
        .collect();
    assert!(t2.len() >= 5, "tier2 bash: {}", t2.len());
    assert_entries_valid(&t2, "tier2_bash");
}

#[test]
fn test_REG_COV_017_load_all_larger_than_tier12() {
    assert!(CorpusRegistry::load_all().len() > CorpusRegistry::load_tier1_and_tier2().len());
}

#[test]
fn test_REG_COV_018_load_all_has_complex() {
    assert!(!CorpusRegistry::load_all()
        .by_tier(CorpusTier::Complex)
        .is_empty());
}

#[test]
fn test_REG_COV_045_tier3_bash_entries_valid() {
    let r = CorpusRegistry::load_all();
    let t3: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Bash && e.tier == CorpusTier::Complex)
        .cloned()
        .collect();
    assert!(t3.len() >= 5, "tier3 bash: {}", t3.len());
    assert_entries_valid(&t3, "tier3_bash");
}

#[test]
fn test_REG_COV_019_adversarial_larger_than_all() {
    assert!(CorpusRegistry::load_all_with_adversarial().len() > CorpusRegistry::load_all().len());
}

#[test]
fn test_REG_COV_020_adversarial_has_tier4() {
    assert!(!CorpusRegistry::load_all_with_adversarial()
        .by_tier(CorpusTier::Adversarial)
        .is_empty());
}

#[test]
fn test_REG_COV_046_tier4_bash_entries_valid() {
    let r = CorpusRegistry::load_all_with_adversarial();
    let t4: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Bash && e.tier == CorpusTier::Adversarial)
        .cloned()
        .collect();
    assert!(t4.len() >= 5, "tier4 bash: {}", t4.len());
    assert_entries_valid(&t4, "tier4_bash");
}

#[test]
fn test_REG_COV_021_load_full_at_least_15000() {
    let r = CorpusRegistry::load_full();
    assert!(r.len() >= 15_000, "got {}", r.len());
}

#[test]
fn test_REG_COV_022_load_full_all_formats() {
    let r = CorpusRegistry::load_full();
    assert!(r.count_by_format(CorpusFormat::Bash) > 0);
    assert!(r.count_by_format(CorpusFormat::Makefile) > 0);
    assert!(r.count_by_format(CorpusFormat::Dockerfile) > 0);
}

#[test]
fn test_REG_COV_023_load_full_all_tiers() {
    let r = CorpusRegistry::load_full();
    assert!(!r.by_tier(CorpusTier::Trivial).is_empty());
    assert!(!r.by_tier(CorpusTier::Standard).is_empty());
    assert!(!r.by_tier(CorpusTier::Complex).is_empty());
    assert!(!r.by_tier(CorpusTier::Adversarial).is_empty());
    assert!(!r.by_tier(CorpusTier::Production).is_empty());
}

#[test]
fn test_REG_COV_024_load_full_bash_dominates() {
    let r = CorpusRegistry::load_full();
    let b = r.count_by_format(CorpusFormat::Bash);
    assert!(b > r.count_by_format(CorpusFormat::Makefile));
    assert!(b > r.count_by_format(CorpusFormat::Dockerfile));
}

#[test]
fn test_REG_COV_025_load_full_known_ids() {
    let r = CorpusRegistry::load_full();
    let ids: Vec<&str> = r.entries.iter().map(|e| e.id.as_str()).collect();
    for id in &["B-001", "B-002", "M-001", "M-002", "D-001", "D-002"] {
        assert!(ids.contains(id), "{id} missing");
    }
}

#[test]
fn test_REG_COV_026_load_full_larger_than_adversarial() {
    assert!(CorpusRegistry::load_full().len() > CorpusRegistry::load_all_with_adversarial().len());
}

#[test]

include!("registry_coverage_tests_incl2.rs");
