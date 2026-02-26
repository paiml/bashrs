//! Coverage tests for corpus/pattern_store.rs — targets classify_failure_signals,
//! mine_patterns, suggest_fixes, ShellFixPattern/PatternStore construction and serde.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::pattern_store::{classify_failure_signals, mine_patterns, suggest_fixes};
use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
use crate::corpus::runner::{CorpusResult, CorpusRunner};
use crate::corpus::{PatternStore, ShellFixPattern};
use crate::models::Config;

// --- classify_failure_signals: exhaustive combinations ---

#[test]
fn test_PSTORE_001_classify_all_pass() {
    let r = CorpusResult {
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        lint_clean: true,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert!(classify_failure_signals(&r).is_empty());
}

#[test]
fn test_PSTORE_002_classify_transpile_fail_gates() {
    let r = CorpusResult {
        transpiled: false,
        ..Default::default()
    };
    let signals = classify_failure_signals(&r);
    assert_eq!(signals, vec!["A_transpile_fail"]);
}

#[test]
fn test_PSTORE_003_classify_transpile_fail_ignores_true_flags() {
    let r = CorpusResult {
        transpiled: false,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        lint_clean: true,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&r), vec!["A_transpile_fail"]);
}

#[test]
fn test_PSTORE_004_classify_individual_signals() {
    // B1 only
    let b1 = CorpusResult {
        transpiled: true,
        output_contains: false,
        output_exact: true,
        output_behavioral: true,
        lint_clean: true,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert!(classify_failure_signals(&b1).contains(&"B1_containment_fail".into()));

    // B2 only
    let b2 = CorpusResult {
        transpiled: true,
        output_contains: true,
        output_exact: false,
        output_behavioral: true,
        lint_clean: true,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&b2), vec!["B2_exact_fail"]);

    // B3 only
    let b3 = CorpusResult {
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: false,
        lint_clean: true,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&b3), vec!["B3_behavioral_fail"]);

    // D only
    let d = CorpusResult {
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        lint_clean: false,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&d), vec!["D_lint_fail"]);

    // G only
    let g = CorpusResult {
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        lint_clean: true,
        cross_shell_agree: false,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&g), vec!["G_cross_shell_fail"]);
}

#[test]
fn test_PSTORE_005_classify_all_fail_after_transpile() {
    let r = CorpusResult {
        transpiled: true,
        output_contains: false,
        output_exact: false,
        output_behavioral: false,
        lint_clean: false,
        cross_shell_agree: false,
        ..Default::default()
    };
    let signals = classify_failure_signals(&r);
    assert_eq!(signals.len(), 5);
    assert!(signals.contains(&"B1_containment_fail".into()));
    assert!(signals.contains(&"B2_exact_fail".into()));
    assert!(signals.contains(&"B3_behavioral_fail".into()));
    assert!(signals.contains(&"D_lint_fail".into()));
    assert!(signals.contains(&"G_cross_shell_fail".into()));
}

#[test]
fn test_PSTORE_006_classify_combinations() {
    // B1 + B2
    let r1 = CorpusResult {
        transpiled: true,
        output_contains: false,
        output_exact: false,
        output_behavioral: true,
        lint_clean: true,
        cross_shell_agree: true,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&r1).len(), 2);
    // D + G
    let r2 = CorpusResult {
        transpiled: true,
        output_contains: true,
        output_exact: true,
        output_behavioral: true,
        lint_clean: false,
        cross_shell_agree: false,
        ..Default::default()
    };
    assert_eq!(classify_failure_signals(&r2).len(), 2);
}

// --- mine_patterns ---

fn bash_entry(id: &str, input: &str, expected: &str) -> CorpusEntry {
    CorpusEntry::new(
        id,
        id,
        id,
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        input,
        expected,
    )
}

#[test]
fn test_PSTORE_007_mine_patterns_all_pass() {
    let runner = CorpusRunner::new(Config::default());
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry(
        "T-PS-007",
        r#"fn main() { println!("hello"); }"#,
        "echo",
    ));
    let store = mine_patterns(&reg, &runner);
    assert_eq!(store.total_entries, 1);
    assert_eq!(store.version, "1.0.0");
}

#[test]
fn test_PSTORE_008_mine_patterns_with_failure() {
    let runner = CorpusRunner::new(Config::default());
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry(
        "T-PS-008a",
        r#"fn main() { println!("hi"); }"#,
        "echo",
    ));
    reg.add(bash_entry("T-PS-008b", "not valid rust!!!", "anything"));
    let store = mine_patterns(&reg, &runner);
    assert_eq!(store.total_entries, 2);
    assert!(store.total_failures >= 1);
}

#[test]
fn test_PSTORE_009_mine_patterns_empty_registry() {
    let store = mine_patterns(
        &CorpusRegistry::new(),
        &CorpusRunner::new(Config::default()),
    );
    assert_eq!(store.total_entries, 0);
    assert_eq!(store.total_failures, 0);
    assert!(store.patterns.is_empty());
}

#[test]
fn test_PSTORE_010_mine_patterns_all_failures() {
    let runner = CorpusRunner::new(Config::default());
    let mut reg = CorpusRegistry::new();
    for i in 0..3 {
        reg.add(bash_entry(
            &format!("T-PS-010-{i}"),
            &format!("broken {i}!!!"),
            "x",
        ));
    }
    let store = mine_patterns(&reg, &runner);
    assert_eq!(store.total_entries, 3);
    assert!(store.total_failures >= 1);
}

#[test]
fn test_PSTORE_011_mine_patterns_mixed_formats() {
    let runner = CorpusRunner::new(Config::default());
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry("T-PS-011a", "fn main() { let x = 1; }", "x="));
    reg.add(CorpusEntry::new(
        "T-PS-011b",
        "m",
        "m",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        r#"fn main() { let cc = "gcc"; }"#,
        "CC",
    ));
    assert_eq!(mine_patterns(&reg, &runner).total_entries, 2);
}

// --- suggest_fixes ---

#[test]
fn test_PSTORE_012_suggest_entry_not_found() {
    let store = PatternStore {
        patterns: vec![],
        total_entries: 0,
        total_failures: 0,
        version: "1.0.0".into(),
    };
    assert!(suggest_fixes(
        "nonexistent",
        &CorpusRegistry::new(),
        &CorpusRunner::new(Config::default()),
        &store
    )
    .is_empty());
}

#[test]
fn test_PSTORE_013_suggest_passing_entry() {
    let runner = CorpusRunner::new(Config::default());
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry(
        "T-PS-013",
        r#"fn main() { println!("hi"); }"#,
        "echo",
    ));
    let store = PatternStore {
        patterns: vec![ShellFixPattern {
            error_signal: "B1_containment_fail".into(),
            causal_decision: "assignment_value:single_quote".into(),
            fix_type: "quoting_strategy".into(),
            confidence: 0.8,
            evidence_ids: vec!["B-999".into()],
        }],
        total_entries: 100,
        total_failures: 5,
        version: "1.0.0".into(),
    };
    let _ = suggest_fixes("T-PS-013", &reg, &runner, &store);
}

#[test]
fn test_PSTORE_014_suggest_failing_entry() {
    let runner = CorpusRunner::new(Config::default());
    let mut reg = CorpusRegistry::new();
    reg.add(bash_entry("T-PS-014", "not valid!!!", "anything"));
    let store = PatternStore {
        patterns: vec![ShellFixPattern {
            error_signal: "A_transpile_fail".into(),
            causal_decision: "ir_dispatch:Unknown".into(),
            fix_type: "ir_node_handling".into(),
            confidence: 0.9,
            evidence_ids: vec!["B-999".into()],
        }],
        total_entries: 100,
        total_failures: 5,
        version: "1.0.0".into(),
    };
    let _ = suggest_fixes("T-PS-014", &reg, &runner, &store);
}

// --- ShellFixPattern fields and serde ---

#[test]
fn test_PSTORE_015_pattern_fields_and_serde() {
    let pattern = ShellFixPattern {
        error_signal: "D_lint_fail".into(),
        causal_decision: "string_emit:unquoted".into(),
        fix_type: "string_handling".into(),
        confidence: 0.72,
        evidence_ids: vec!["B-100".into(), "B-200".into()],
    };
    assert_eq!(pattern.error_signal, "D_lint_fail");
    assert_eq!(pattern.evidence_ids.len(), 2);

    let json = serde_json::to_string(&pattern).unwrap();
    let loaded: ShellFixPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.confidence, 0.72);
    assert_eq!(loaded.evidence_ids, vec!["B-100", "B-200"]);
}

#[test]
fn test_PSTORE_016_pattern_multiple_evidence_serde() {
    let pattern = ShellFixPattern {
        error_signal: "B2_exact_fail".into(),
        causal_decision: "variable_expansion:unbraced".into(),
        fix_type: "expansion_strategy".into(),
        confidence: 0.65,
        evidence_ids: vec![
            "B-100".into(),
            "B-200".into(),
            "B-300".into(),
            "B-400".into(),
        ],
    };
    let json = serde_json::to_string(&pattern).unwrap();
    assert!(json.contains("B-300"));
    assert_eq!(pattern.evidence_ids.len(), 4);
}

// --- PatternStore sorting and serde ---

#[test]
fn test_PSTORE_017_store_sorting() {
    let mut store = PatternStore {
        patterns: vec![
            ShellFixPattern {
                error_signal: "B1".into(),
                causal_decision: "a:b".into(),
                fix_type: "a".into(),
                confidence: 0.5,
                evidence_ids: vec!["1".into()],
            },
            ShellFixPattern {
                error_signal: "D".into(),
                causal_decision: "c:d".into(),
                fix_type: "c".into(),
                confidence: 0.9,
                evidence_ids: vec!["2".into()],
            },
            ShellFixPattern {
                error_signal: "G".into(),
                causal_decision: "e:f".into(),
                fix_type: "e".into(),
                confidence: 0.7,
                evidence_ids: vec!["3".into()],
            },
        ],
        total_entries: 100,
        total_failures: 10,
        version: "1.0.0".into(),
    };
    store
        .patterns
        .sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    assert_eq!(store.patterns[0].confidence, 0.9);
    assert_eq!(store.patterns[1].confidence, 0.7);
    assert_eq!(store.patterns[2].confidence, 0.5);
}

#[test]
fn test_PSTORE_018_store_serde_roundtrip() {
    let store = PatternStore {
        patterns: vec![
            ShellFixPattern {
                error_signal: "B3_behavioral_fail".into(),
                causal_decision: "assignment_value:single_quote".into(),
                fix_type: "quoting_strategy".into(),
                confidence: 0.85,
                evidence_ids: vec!["B-143".into()],
            },
            ShellFixPattern {
                error_signal: "D_lint_fail".into(),
                causal_decision: "string_emit:unquoted".into(),
                fix_type: "string_handling".into(),
                confidence: 0.72,
                evidence_ids: vec!["B-100".into(), "B-200".into()],
            },
        ],
        total_entries: 900,
        total_failures: 5,
        version: "1.0.0".into(),
    };
    let loaded: PatternStore =
        serde_json::from_str(&serde_json::to_string_pretty(&store).unwrap()).unwrap();
    assert_eq!(loaded.patterns.len(), 2);
    assert_eq!(loaded.total_entries, 900);
    assert_eq!(loaded.patterns[0].error_signal, "B3_behavioral_fail");
}

#[test]
fn test_PSTORE_019_empty_store_serde() {
    let store = PatternStore {
        patterns: vec![],
        total_entries: 0,
        total_failures: 0,
        version: "1.0.0".into(),
    };
    let loaded: PatternStore =
        serde_json::from_str(&serde_json::to_string(&store).unwrap()).unwrap();
    assert!(loaded.patterns.is_empty());
}

// --- Debug and Clone trait coverage ---

#[test]
fn test_PSTORE_020_debug_and_clone() {
    let pattern = ShellFixPattern {
        error_signal: "B1_containment_fail".into(),
        causal_decision: "ir_dispatch:Let".into(),
        fix_type: "ir_node_handling".into(),
        confidence: 0.75,
        evidence_ids: vec!["B-001".into()],
    };
    assert!(format!("{pattern:?}").contains("B1_containment_fail"));
    let cloned = pattern.clone();
    assert_eq!(cloned.error_signal, pattern.error_signal);

    let store = PatternStore {
        patterns: vec![pattern],
        total_entries: 10,
        total_failures: 1,
        version: "1.0.0".into(),
    };
    assert!(format!("{store:?}").contains("PatternStore"));
    let cs = store.clone();
    assert_eq!(cs.patterns.len(), 1);
    assert_eq!(cs.total_entries, 10);
}

#[test]
fn test_PSTORE_021_store_version() {
    let store = PatternStore {
        patterns: vec![],
        total_entries: 50,
        total_failures: 3,
        version: "2.0.0".into(),
    };
    assert_eq!(store.version, "2.0.0");
}
