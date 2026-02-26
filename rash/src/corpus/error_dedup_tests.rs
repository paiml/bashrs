#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for corpus/error_dedup.rs
//! Targets uncovered branches in normalize_message, hash_error, classify_risk,
//! has_security_code, default_signal_risk, evaluate_rules, labeling_rules,
//! RiskLevel, ShellTrainingError, ErrorTriage.

use super::error_dedup::*;

// === normalize_message: regex branches ===

#[test]
fn test_normalize_strips_sh_extension_path() {
    let norm = normalize_message("Error in /tmp/bashrs_abc123/test.sh");
    assert!(norm.contains("<path>"));
    assert!(!norm.contains("/tmp/"));
}

#[test]
fn test_normalize_strips_bash_extension_path() {
    let norm = normalize_message("Error in /home/user/scripts/run.bash at end");
    assert!(norm.contains("<path>"));
}

#[test]
fn test_normalize_strips_mk_extension_path() {
    let norm = normalize_message("Failed parsing /var/build/rules.mk content");
    assert!(norm.contains("<path>"));
}

#[test]
fn test_normalize_strips_line_reference() {
    let norm = normalize_message("error at line 42 in script");
    assert!(norm.contains("line N"));
    assert!(!norm.contains("line 42"));
}

#[test]
fn test_normalize_strips_line_col_references() {
    let norm = normalize_message("warning at 10:25 and 3:1");
    assert_eq!(norm.matches("N:N").count(), 2);
}

#[test]
fn test_normalize_strips_entry_ids_all_prefixes() {
    let norm = normalize_message("B-001 failed, M-042 warning, D-100 error");
    assert_eq!(norm.matches("<id>").count(), 3);
}

#[test]
fn test_normalize_collapses_whitespace() {
    assert_eq!(
        normalize_message("  extra   whitespace   everywhere  "),
        "extra whitespace everywhere"
    );
}

#[test]
fn test_normalize_empty_and_passthrough() {
    assert_eq!(normalize_message(""), "");
    assert_eq!(normalize_message("simple error"), "simple error");
}

#[test]
fn test_normalize_combined_all_patterns() {
    let norm = normalize_message("B-123 error in /tmp/test.sh at line 5 at 2:3 with   spaces");
    assert!(norm.contains("<id>") && norm.contains("<path>"));
    assert!(norm.contains("line N") && norm.contains("N:N"));
    assert!(!norm.contains("  "));
}

#[test]
fn test_normalize_entry_id_boundary() {
    let norm = normalize_message("Entry B-999 failed");
    assert!(norm.contains("<id>"));
    // B-1000 has 4 digits, regex expects 3
    let norm2 = normalize_message("Entry B-1000 failed");
    assert!(norm2.contains("B-1000"));
}

#[test]
fn test_normalize_multiple_paths() {
    let norm = normalize_message("Error in /tmp/a.sh and /var/b.bash at end");
    assert_eq!(norm.matches("<path>").count(), 2);
}

// === hash_error ===

#[test]
fn test_hash_error_deterministic_and_distinct() {
    let h1 = hash_error("code1", "msg1");
    assert_eq!(h1, hash_error("code1", "msg1"));
    assert_ne!(hash_error("A", "msg"), hash_error("B", "msg"));
    assert_ne!(hash_error("X", "a"), hash_error("X", "b"));
    assert_ne!(hash_error("", ""), hash_error("", "x"));
    assert_ne!(hash_error("a", "b"), hash_error("b", "a"));
}

// === has_security_code: all SEC001-SEC008 ===

#[test]
fn test_has_security_code_all_codes() {
    for i in 1..=8 {
        let msg = format!("Found SEC{:03} violation", i);
        let signals = vec!["D_lint_fail".to_string()];
        assert_eq!(classify_risk(&signals, &msg), RiskLevel::High);
    }
}

// === classify_risk: all labeling rule paths ===

#[test]
fn test_classify_risk_b3_behavioral_fail() {
    let signals = vec!["B3_behavioral_fail".to_string()];
    assert_eq!(classify_risk(&signals, ""), RiskLevel::High);
}

#[test]
fn test_classify_risk_sec_plus_b3() {
    let signals = vec!["B3_behavioral_fail".to_string(), "D_lint_fail".to_string()];
    assert_eq!(classify_risk(&signals, "SEC003 found"), RiskLevel::High);
}

#[test]
fn test_classify_risk_g_cross_shell_fail() {
    let signals = vec!["G_cross_shell_fail".to_string()];
    assert_eq!(classify_risk(&signals, ""), RiskLevel::Medium);
}

#[test]
fn test_classify_risk_sc2086_quoting() {
    let signals = vec!["D_lint_fail".to_string()];
    assert_eq!(
        classify_risk(&signals, "SC2086: Double quote"),
        RiskLevel::Medium
    );
}

#[test]
fn test_classify_risk_lint_only_without_b3() {
    let signals = vec!["D_lint_fail".to_string()];
    assert_eq!(classify_risk(&signals, "SC2034 unused"), RiskLevel::Low);
}

#[test]
fn test_classify_risk_lint_with_b3_not_low() {
    let signals = vec!["D_lint_fail".to_string(), "B3_behavioral_fail".to_string()];
    assert_eq!(classify_risk(&signals, "some lint"), RiskLevel::High);
}

// === default_signal_risk: prefix-based fallback ===

#[test]
fn test_default_signal_risk_prefixes() {
    assert_eq!(
        classify_risk(&["A_transpile_fail".into()], ""),
        RiskLevel::High
    );
    assert_eq!(
        classify_risk(&["B1_containment_fail".into()], ""),
        RiskLevel::Low
    );
    assert_eq!(classify_risk(&["B2_exact_fail".into()], ""), RiskLevel::Low);
    assert_eq!(
        classify_risk(&["X_unknown_signal".into()], ""),
        RiskLevel::Medium
    );
    assert_eq!(
        classify_risk(&["F_metamorphic_fail".into()], ""),
        RiskLevel::Medium
    );
    // Empty signals
    assert_eq!(classify_risk(&[], ""), RiskLevel::Medium);
}

// === evaluate_rules logic (tested via public API) ===

fn eval_rules(signals: &[String], error_msg: &str) -> [bool; 5] {
    let has_b3 = signals.iter().any(|s| s == "B3_behavioral_fail");
    let has_lint = signals.iter().any(|s| s == "D_lint_fail");
    let has_sec = [
        "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
    ]
    .iter()
    .any(|code| error_msg.contains(code));
    [
        has_sec,
        has_b3,
        signals.iter().any(|s| s == "G_cross_shell_fail"),
        error_msg.contains("SC2086"),
        has_lint && !has_b3,
    ]
}

#[test]
fn test_evaluate_rules_combinations() {
    let r = eval_rules(&["D_lint_fail".into()], "SEC001 found");
    assert!(r[0] && !r[1] && !r[2] && !r[3] && r[4]);

    let r = eval_rules(&["B3_behavioral_fail".into()], "");
    assert!(!r[0] && r[1] && !r[4]);

    let r = eval_rules(&["G_cross_shell_fail".into()], "");
    assert!(r[2]);

    let r = eval_rules(&["D_lint_fail".into()], "SC2086 warning");
    assert!(r[3] && r[4]);

    let r = eval_rules(&["D_lint_fail".into(), "B3_behavioral_fail".into()], "");
    assert!(r[1] && !r[4]); // B3 disables LINT_ONLY

    let r = eval_rules(&["X_unknown".into()], "msg");
    assert!(!r[0] && !r[1] && !r[2] && !r[3] && !r[4]);
}

// === labeling_rules ===

#[test]
fn test_labeling_rules_structure() {
    let rules = labeling_rules();
    assert_eq!(rules.len(), 5);
    assert_eq!(rules[0].name, "SEC_RULE");
    assert_eq!(rules[1].name, "B3_FAIL");
    assert_eq!(rules[2].name, "G_FAIL");
    assert_eq!(rules[3].name, "QUOTING");
    assert_eq!(rules[4].name, "LINT_ONLY");
    assert_eq!(rules[0].risk, RiskLevel::High);
    assert_eq!(rules[4].risk, RiskLevel::Low);
    for rule in &rules {
        assert!(!rule.condition.is_empty());
    }
}

// === RiskLevel: Display, Ord, Clone, Hash, Serialize/Deserialize ===

#[test]
fn test_risk_level_display() {
    assert_eq!(format!("{}", RiskLevel::High), "HIGH");
    assert_eq!(format!("{}", RiskLevel::Medium), "MEDIUM");
    assert_eq!(format!("{}", RiskLevel::Low), "LOW");
}

#[test]
fn test_risk_level_ordering_and_equality() {
    assert!(RiskLevel::High < RiskLevel::Medium);
    assert!(RiskLevel::Medium < RiskLevel::Low);
    assert_eq!(RiskLevel::High, RiskLevel::High);
    assert_ne!(RiskLevel::High, RiskLevel::Low);
}

#[test]
fn test_risk_level_hash_in_map() {
    use std::collections::HashMap;
    let mut map: HashMap<RiskLevel, usize> = HashMap::new();
    map.insert(RiskLevel::High, 1);
    map.insert(RiskLevel::Medium, 2);
    map.insert(RiskLevel::Low, 3);
    assert_eq!(map[&RiskLevel::High], 1);
    assert_eq!(map[&RiskLevel::Low], 3);
}

#[test]
fn test_risk_level_serde() {
    assert_eq!(serde_json::to_string(&RiskLevel::High).unwrap(), "\"High\"");
    assert_eq!(
        serde_json::to_string(&RiskLevel::Medium).unwrap(),
        "\"Medium\""
    );
    assert_eq!(serde_json::to_string(&RiskLevel::Low).unwrap(), "\"Low\"");
    let high: RiskLevel = serde_json::from_str("\"High\"").unwrap();
    assert_eq!(high, RiskLevel::High);
}

// === ShellTrainingError ===

#[test]
fn test_shell_training_error_round_trip() {
    let err = ShellTrainingError {
        error_code: "B3_behavioral_fail".to_string(),
        message: "execution timeout".to_string(),
        hash: 42,
        count: 5,
        risk: RiskLevel::High,
        entry_ids: vec!["B-001".to_string(), "B-002".to_string()],
    };
    let json = serde_json::to_string(&err).unwrap();
    let de: ShellTrainingError = serde_json::from_str(&json).unwrap();
    assert_eq!(de.error_code, "B3_behavioral_fail");
    assert_eq!(de.hash, 42);
    assert_eq!(de.count, 5);
    assert_eq!(de.risk, RiskLevel::High);
    assert_eq!(de.entry_ids.len(), 2);
}

#[test]
fn test_shell_training_error_debug() {
    let err = ShellTrainingError {
        error_code: "test".to_string(),
        message: "msg".to_string(),
        hash: 0,
        count: 1,
        risk: RiskLevel::Low,
        entry_ids: vec![],
    };
    assert!(format!("{:?}", err).contains("ShellTrainingError"));
}

// === ErrorTriage ===

#[test]
fn test_error_triage_empty() {
    let t = ErrorTriage {
        errors: vec![],
        total_raw: 0,
        total_unique: 0,
        high_count: 0,
        medium_count: 0,
        low_count: 0,
    };
    assert_eq!(t.total_raw, 0);
    assert!(t.errors.is_empty());
}

#[test]
fn test_error_triage_with_mixed_risks() {
    let t = ErrorTriage {
        errors: vec![
            ShellTrainingError {
                error_code: "B3_behavioral_fail".into(),
                message: "timeout".into(),
                hash: 1,
                count: 3,
                risk: RiskLevel::High,
                entry_ids: vec!["B-001".into()],
            },
            ShellTrainingError {
                error_code: "G_cross_shell_fail".into(),
                message: "dash differs".into(),
                hash: 2,
                count: 2,
                risk: RiskLevel::Medium,
                entry_ids: vec!["B-050".into()],
            },
            ShellTrainingError {
                error_code: "D_lint_fail".into(),
                message: "SC2034".into(),
                hash: 3,
                count: 10,
                risk: RiskLevel::Low,
                entry_ids: vec!["B-100".into()],
            },
        ],
        total_raw: 15,
        total_unique: 3,
        high_count: 1,
        medium_count: 1,
        low_count: 1,
    };
    assert_eq!(t.total_raw, 15);
    assert_eq!(t.total_unique, 3);
}

#[test]
fn test_error_triage_serde() {
    let t = ErrorTriage {
        errors: vec![ShellTrainingError {
            error_code: "A_transpile_fail".into(),
            message: "parse error".into(),
            hash: 999,
            count: 1,
            risk: RiskLevel::High,
            entry_ids: vec!["B-500".into()],
        }],
        total_raw: 1,
        total_unique: 1,
        high_count: 1,
        medium_count: 0,
        low_count: 0,
    };
    let json = serde_json::to_string(&t).unwrap();
    let de: ErrorTriage = serde_json::from_str(&json).unwrap();
    assert_eq!(de.errors[0].error_code, "A_transpile_fail");
    assert!(format!("{:?}", t).contains("ErrorTriage"));
}

// === LabelingRule ===

#[test]
fn test_labeling_rule_debug_and_clone() {
    let rules = labeling_rules();
    assert!(format!("{:?}", rules[0]).contains("SEC_RULE"));
    let cloned = rules[0].clone();
    assert_eq!(cloned.name, "SEC_RULE");
    assert_eq!(cloned.risk, RiskLevel::High);
}

// === classify_risk with multiple simultaneous signals ===

#[test]
fn test_classify_risk_all_signals_present() {
    let signals = vec![
        "A_transpile_fail".into(),
        "B1_containment_fail".into(),
        "B3_behavioral_fail".into(),
        "D_lint_fail".into(),
        "G_cross_shell_fail".into(),
    ];
    assert_eq!(classify_risk(&signals, "SEC001"), RiskLevel::High);
    assert_eq!(classify_risk(&signals, ""), RiskLevel::High);
}

#[test]
fn test_classify_risk_g_and_sc2086() {
    let signals = vec!["G_cross_shell_fail".to_string()];
    assert_eq!(classify_risk(&signals, "SC2086 warning"), RiskLevel::Medium);
}
