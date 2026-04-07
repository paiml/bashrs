fn test_REG_COV_047_tier5_bash_via_full() {
    let r = CorpusRegistry::load_full();
    let t5: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Bash && e.tier == CorpusTier::Production)
        .cloned()
        .collect();
    assert!(t5.len() >= 5, "tier5 bash: {}", t5.len());
    assert_entries_valid(&t5, "tier5_bash");
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_048_tier5_makefile_via_full() {
    let r = CorpusRegistry::load_full();
    let t5m: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Makefile && e.tier == CorpusTier::Production)
        .cloned()
        .collect();
    assert!(t5m.len() >= 5, "tier5 makefile: {}", t5m.len());
    assert_entries_valid(&t5m, "tier5_makefile");
    for e in &t5m {
        assert!(e.id.starts_with("M-"));
        assert!(!e.shellcheck);
    }
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_049_tier5_dockerfile_via_full() {
    let r = CorpusRegistry::load_full();
    let t5d: Vec<_> = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Dockerfile && e.tier == CorpusTier::Production)
        .cloned()
        .collect();
    assert!(t5d.len() >= 5, "tier5 dockerfile: {}", t5d.len());
    assert_entries_valid(&t5d, "tier5_dockerfile");
    for e in &t5d {
        assert!(e.id.starts_with("D-"));
        assert!(!e.shellcheck);
    }
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_050_load_full_all_entries_valid() {
    assert_entries_valid(&CorpusRegistry::load_full().entries, "load_full");
}

#[test]
fn test_REG_COV_051_load_full_no_duplicate_ids() {
    let r = CorpusRegistry::load_full();
    let mut ids: Vec<&str> = r.entries.iter().map(|e| e.id.as_str()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), total, "found {} duplicates", total - ids.len());
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_052_expansion_entries_over_1000() {
    let adv_len = CorpusRegistry::load_all_with_adversarial().len();
    let full_len = CorpusRegistry::load_full().len();
    assert!(
        full_len - adv_len > 1000,
        "expansion: {}",
        full_len - adv_len
    );
}

#[test]
fn test_REG_COV_053_bash_shellcheck_consistency() {
    for e in &CorpusRegistry::load_full().entries {
        match e.format {
            CorpusFormat::Bash => assert!(e.shellcheck, "{} missing shellcheck", e.id),
            _ => assert!(!e.shellcheck, "{} has unexpected shellcheck", e.id),
        }
    }
}

#[test]
fn test_REG_COV_054_deterministic_flag_always_true() {
    for e in &CorpusRegistry::load_full().entries {
        assert!(e.deterministic, "{} not deterministic", e.id);
    }
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_055_expansion_makefile_count() {
    let r = CorpusRegistry::load_full();
    let mfp = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Makefile && e.tier == CorpusTier::Production)
        .count();
    assert!(mfp >= 50, "production makefile: {mfp}");
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_056_expansion_dockerfile_count() {
    let r = CorpusRegistry::load_full();
    let dfp = r
        .entries
        .iter()
        .filter(|e| e.format == CorpusFormat::Dockerfile && e.tier == CorpusTier::Production)
        .count();
    assert!(dfp >= 50, "production dockerfile: {dfp}");
}

#[test]
fn test_REG_COV_057_idempotent_flag_always_true() {
    for e in &CorpusRegistry::load_full().entries {
        assert!(e.idempotent, "{} not idempotent", e.id);
    }
}

#[test]
fn test_REG_COV_027_by_format_only_matching() {
    let r = CorpusRegistry::load_tier1();
    for e in r.by_format(CorpusFormat::Bash) {
        assert_eq!(e.format, CorpusFormat::Bash);
    }
    for e in r.by_format(CorpusFormat::Makefile) {
        assert_eq!(e.format, CorpusFormat::Makefile);
    }
    for e in r.by_format(CorpusFormat::Dockerfile) {
        assert_eq!(e.format, CorpusFormat::Dockerfile);
    }
}

#[test]
fn test_REG_COV_028_by_format_sums_to_total() {
    let r = CorpusRegistry::load_tier1();
    let s = r.by_format(CorpusFormat::Bash).len()
        + r.by_format(CorpusFormat::Makefile).len()
        + r.by_format(CorpusFormat::Dockerfile).len();
    assert_eq!(s, r.len());
}

#[test]
fn test_REG_COV_029_by_tier_only_matching() {
    let r = CorpusRegistry::load_all();
    for e in r.by_tier(CorpusTier::Trivial) {
        assert_eq!(e.tier, CorpusTier::Trivial);
    }
    for e in r.by_tier(CorpusTier::Complex) {
        assert_eq!(e.tier, CorpusTier::Complex);
    }
}

#[test]
    #[ignore = "requires runtime corpus data (externalized from builtin)"]
fn test_REG_COV_030_by_format_and_tier() {
    let r = CorpusRegistry::load_all();
    let bt = r.by_format_and_tier(CorpusFormat::Bash, CorpusTier::Trivial);
    assert!(!bt.is_empty());
    for e in &bt {
        assert_eq!(e.format, CorpusFormat::Bash);
        assert_eq!(e.tier, CorpusTier::Trivial);
    }
}

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
