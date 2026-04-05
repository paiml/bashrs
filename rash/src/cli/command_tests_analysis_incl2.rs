fn test_corpus_entry_new_makefile_no_shellcheck() {
    let entry = CorpusEntry::new(
        "M-001",
        "make-test",
        "desc",
        CorpusFormat::Makefile,
        CorpusTier::Standard,
        "let x = 5;",
        "X := 5",
    );
    assert!(!entry.shellcheck); // non-bash entries get shellcheck=false
    assert!(entry.deterministic);
}

// ── CorpusRegistry tests ────────────────────────────────────────────────────

#[test]
fn test_corpus_registry_new_empty() {
    let registry = crate::corpus::registry::CorpusRegistry::new();
    assert!(registry.entries.is_empty());
}

#[test]
fn test_corpus_registry_add_and_by_format() {
    let mut registry = crate::corpus::registry::CorpusRegistry::new();
    registry.add(mock_entry(
        "B-001",
        "t1",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    ));
    registry.add(mock_entry(
        "M-001",
        "t2",
        CorpusFormat::Makefile,
        CorpusTier::Standard,
    ));
    registry.add(mock_entry(
        "B-002",
        "t3",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
    ));

    assert_eq!(registry.by_format(CorpusFormat::Bash).len(), 2);
    assert_eq!(registry.by_format(CorpusFormat::Makefile).len(), 1);
    assert_eq!(registry.by_format(CorpusFormat::Dockerfile).len(), 0);
}

#[test]
fn test_corpus_registry_by_tier() {
    let mut registry = crate::corpus::registry::CorpusRegistry::new();
    registry.add(mock_entry(
        "B-001",
        "t1",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    ));
    registry.add(mock_entry(
        "B-002",
        "t2",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
    ));
    registry.add(mock_entry(
        "B-003",
        "t3",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    ));

    assert_eq!(registry.by_tier(CorpusTier::Standard).len(), 2);
    assert_eq!(registry.by_tier(CorpusTier::Trivial).len(), 1);
    assert_eq!(registry.by_tier(CorpusTier::Complex).len(), 0);
}

// ── ConvergenceEntry tests ──────────────────────────────────────────────────

#[test]
fn test_convergence_entry_default() {
    let e = ConvergenceEntry::default();
    assert_eq!(e.iteration, 0);
    assert_eq!(e.total, 0);
    assert!((e.score - 0.0).abs() < 0.01);
    assert!(e.grade.is_empty());
}

#[test]
fn test_convergence_entry_serialization_roundtrip() {
    let e = ConvergenceEntry {
        iteration: 42,
        date: "2025-06-15".to_string(),
        total: 1000,
        passed: 999,
        failed: 1,
        rate: 0.999,
        delta: 0.001,
        notes: "test run".to_string(),
        bash_passed: 500,
        bash_total: 500,
        makefile_passed: 300,
        makefile_total: 300,
        dockerfile_passed: 199,
        dockerfile_total: 200,
        score: 99.2,
        grade: "A+".to_string(),
        bash_score: 99.5,
        makefile_score: 100.0,
        dockerfile_score: 98.0,
        lint_passed: 998,
        lint_rate: 0.998,
    };
    let json = serde_json::to_string(&e).unwrap();
    let parsed: ConvergenceEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.iteration, 42);
    assert_eq!(parsed.total, 1000);
    assert!((parsed.score - 99.2).abs() < 0.01);
    assert_eq!(parsed.grade, "A+");
}

// ── CorpusResult score edge cases ───────────────────────────────────────────

#[test]
fn test_corpus_result_score_v1_all_pass() {
    let r = mock_result("B-001", true);
    let v1 = r.score_v1();
    // A(40) + B(25) + C(0.95*15=14.25) + D(10) + E(10) = 99.25
    assert!((v1 - 99.25).abs() < 0.01, "V1 expected 99.25, got {v1}");
}

#[test]
fn test_corpus_result_score_v1_fail() {
    let r = mock_result("B-001", false);
    assert!((r.score_v1() - 0.0).abs() < 0.01);
}

#[test]
fn test_corpus_result_score_transpiled_but_not_contains() {
    let mut r = mock_result("B-001", true);
    r.output_contains = false;
    let s = r.score();
    // A(30) + B1(0) + B2(0, gated by B1) + B3(0, gated by B1) + C(14.25) + D(10) + E(10) + F(5) + G(5) = 74.25
    assert!((s - 74.25).abs() < 0.01, "Expected 74.25, got {s}");
}

#[test]
fn test_corpus_result_score_contains_but_not_exact() {
    let mut r = mock_result("B-001", true);
    r.output_exact = false;
    let s = r.score();
    // A(30) + B1(10) + B2(0) + B3(7) + C(14.25) + D(10) + E(10) + F(5) + G(5) = 91.25
    assert!((s - 91.25).abs() < 0.01, "Expected 91.25, got {s}");
}

#[test]
fn test_corpus_result_default() {
    let r = CorpusResult::default();
    assert!(!r.transpiled);
    assert!((r.score() - 0.0).abs() < 0.01);
    assert!(r.id.is_empty());
}
