//! Weak Supervision & Error Deduplication (§11.10.4)
//!
//! Hash-based error deduplication with programmatic labeling rules (weak supervision
//! à la Snorkel). Prevents the same shellcheck warning from inflating the fix backlog.
//! Classifies each unique error by risk level using 5 labeling functions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::pattern_store::classify_failure_signals;
use super::registry::CorpusRegistry;
use super::runner::{CorpusResult, CorpusRunner};

/// Risk severity for a deduplicated error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::Low => write!(f, "LOW"),
        }
    }
}

/// A deduplicated error with count, risk label, and affected entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellTrainingError {
    /// Error code (e.g. "B3_behavioral_fail", "D_lint_fail", "G_cross_shell_fail")
    pub error_code: String,
    /// Normalized error message
    pub message: String,
    /// FNV-1a hash of (error_code, message) for dedup key
    pub hash: u64,
    /// How many entries hit this exact error
    pub count: usize,
    /// Risk level from weak supervision labeling
    pub risk: RiskLevel,
    /// Which entries are affected
    pub entry_ids: Vec<String>,
}

/// Triage summary after deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTriage {
    /// Deduplicated errors sorted by risk (HIGH first)
    pub errors: Vec<ShellTrainingError>,
    /// Total error occurrences before dedup
    pub total_raw: usize,
    /// Unique errors after dedup
    pub total_unique: usize,
    /// Count of HIGH-risk unique errors
    pub high_count: usize,
    /// Count of MEDIUM-risk unique errors
    pub medium_count: usize,
    /// Count of LOW-risk unique errors
    pub low_count: usize,
}

/// A programmatic labeling rule (Snorkel-style weak supervision).
#[derive(Debug, Clone)]
pub struct LabelingRule {
    /// Rule name
    pub name: &'static str,
    /// Human-readable condition
    pub condition: &'static str,
    /// Risk label this rule assigns
    pub risk: RiskLevel,
}

/// Normalize an error message by stripping paths, line numbers, and entry IDs.
pub fn normalize_message(msg: &str) -> String {
    use regex::Regex;
    let mut normalized = msg.to_string();
    // Strip file paths (e.g. /tmp/bashrs_xxx/foo.sh)
    if let Ok(re) = Regex::new(r"/[^\s]+\.(sh|bash|mk|Makefile|Dockerfile)") {
        normalized = re.replace_all(&normalized, "<path>").to_string();
    }
    // Strip line:col references (e.g. "line 3", "3:5")
    if let Ok(re) = Regex::new(r"\bline\s+\d+") {
        normalized = re.replace_all(&normalized, "line N").to_string();
    }
    if let Ok(re) = Regex::new(r"\b\d+:\d+\b") {
        normalized = re.replace_all(&normalized, "N:N").to_string();
    }
    // Strip entry IDs (e.g. B-001, M-042, D-100)
    if let Ok(re) = Regex::new(r"\b[BMD]-\d{3}\b") {
        normalized = re.replace_all(&normalized, "<id>").to_string();
    }
    // Collapse whitespace
    if let Ok(re) = Regex::new(r"\s+") {
        normalized = re.replace_all(&normalized, " ").to_string();
    }
    normalized.trim().to_string()
}

/// FNV-1a hash of (error_code, normalized_message) for dedup key.
pub fn hash_error(code: &str, message: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    code.hash(&mut hasher);
    message.hash(&mut hasher);
    hasher.finish()
}

/// Return the 5 Snorkel-style labeling rules.
pub fn labeling_rules() -> Vec<LabelingRule> {
    vec![
        LabelingRule {
            name: "SEC_RULE",
            condition: "error matches SEC001-SEC008",
            risk: RiskLevel::High,
        },
        LabelingRule {
            name: "B3_FAIL",
            condition: "B3 behavioral failure",
            risk: RiskLevel::High,
        },
        LabelingRule {
            name: "G_FAIL",
            condition: "cross-shell disagreement",
            risk: RiskLevel::Medium,
        },
        LabelingRule {
            name: "QUOTING",
            condition: "SC2086 unquoted variable",
            risk: RiskLevel::Medium,
        },
        LabelingRule {
            name: "LINT_ONLY",
            condition: "lint-only (B3 passes)",
            risk: RiskLevel::Low,
        },
    ]
}

/// Check if error message contains any SEC001-SEC008 security linter code.
fn has_security_code(msg: &str) -> bool {
    const SEC_CODES: [&str; 8] = [
        "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
    ];
    SEC_CODES.iter().any(|code| msg.contains(code))
}

/// Check if a specific signal is present in the signals list.
fn has_signal(signals: &[String], target: &str) -> bool {
    signals.iter().any(|s| s == target)
}

/// Default risk from signal prefix when no labeling rule matches.
fn default_signal_risk(signals: &[String]) -> RiskLevel {
    if signals.iter().any(|s| s.starts_with("A_")) {
        return RiskLevel::High;
    }
    if signals
        .iter()
        .any(|s| s.starts_with("B1_") || s.starts_with("B2_"))
    {
        return RiskLevel::Low;
    }
    RiskLevel::Medium
}

/// Classify risk from failure signals using the 5 labeling rules.
/// Signals come from `classify_failure_signals()` (e.g. "B3_behavioral_fail", "D_lint_fail").
/// Additional context: error message for SEC/QUOTING detection.
pub fn classify_risk(signals: &[String], error_msg: &str) -> RiskLevel {
    if has_security_code(error_msg) || has_signal(signals, "B3_behavioral_fail") {
        return RiskLevel::High;
    }
    if has_signal(signals, "G_cross_shell_fail") || error_msg.contains("SC2086") {
        return RiskLevel::Medium;
    }
    if has_signal(signals, "D_lint_fail") && !has_signal(signals, "B3_behavioral_fail") {
        return RiskLevel::Low;
    }
    default_signal_risk(signals)
}

/// Main entry point: run all entries, deduplicate errors by hash, classify risk.
pub fn deduplicate_errors(registry: &CorpusRegistry, runner: &CorpusRunner) -> ErrorTriage {
    let mut error_map: HashMap<u64, ShellTrainingError> = HashMap::new();
    let mut total_raw: usize = 0;

    for entry in &registry.entries {
        let result: CorpusResult = runner.run_entry_with_trace(entry);
        let signals = classify_failure_signals(&result);

        if signals.is_empty() {
            continue;
        }

        let error_msg = result.error.as_deref().unwrap_or("");

        for signal in &signals {
            let normalized = normalize_message(&format!("{signal} {error_msg}"));
            let h = hash_error(signal, &normalized);
            total_raw += 1;

            error_map
                .entry(h)
                .and_modify(|e| {
                    e.count += 1;
                    if !e.entry_ids.contains(&entry.id) {
                        e.entry_ids.push(entry.id.clone());
                    }
                })
                .or_insert_with(|| ShellTrainingError {
                    error_code: signal.clone(),
                    message: normalized.clone(),
                    hash: h,
                    count: 1,
                    risk: classify_risk(&signals, error_msg),
                    entry_ids: vec![entry.id.clone()],
                });
        }
    }

    let mut errors: Vec<ShellTrainingError> = error_map.into_values().collect();
    // Sort: HIGH first, then MEDIUM, then LOW; within same risk, by count descending
    errors.sort_by(|a, b| a.risk.cmp(&b.risk).then_with(|| b.count.cmp(&a.count)));

    let high_count = errors.iter().filter(|e| e.risk == RiskLevel::High).count();
    let medium_count = errors
        .iter()
        .filter(|e| e.risk == RiskLevel::Medium)
        .count();
    let low_count = errors.iter().filter(|e| e.risk == RiskLevel::Low).count();
    let total_unique = errors.len();

    ErrorTriage {
        errors,
        total_raw,
        total_unique,
        high_count,
        medium_count,
        low_count,
    }
}

/// Evaluate which of the 5 labeling rules match for a given entry's signals.
/// Returns a [bool; 5] for SEC_RULE, B3_FAIL, G_FAIL, QUOTING, LINT_ONLY.
fn evaluate_rules(signals: &[String], error_msg: &str) -> [bool; 5] {
    let has_b3 = has_signal(signals, "B3_behavioral_fail");
    let has_lint = has_signal(signals, "D_lint_fail");
    [
        has_security_code(error_msg),
        has_b3,
        has_signal(signals, "G_cross_shell_fail"),
        error_msg.contains("SC2086"),
        has_lint && !has_b3,
    ]
}

/// Count how many entries each labeling rule matches.
pub fn count_rule_matches(
    registry: &CorpusRegistry,
    runner: &CorpusRunner,
) -> Vec<(LabelingRule, usize)> {
    let rules = labeling_rules();
    let mut counts = [0usize; 5];

    for entry in &registry.entries {
        let result = runner.run_entry_with_trace(entry);
        let signals = classify_failure_signals(&result);
        if signals.is_empty() {
            continue;
        }
        let error_msg = result.error.as_deref().unwrap_or("");
        let matched = evaluate_rules(&signals, error_msg);
        for (i, &hit) in matched.iter().enumerate() {
            if hit {
                counts[i] += 1;
            }
        }
    }

    rules.into_iter().zip(counts).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_message_strips_paths() {
        let msg = "Error in /tmp/bashrs_abc123/test.sh at line 42";
        let norm = normalize_message(msg);
        assert!(!norm.contains("/tmp/bashrs_abc123/test.sh"));
        assert!(norm.contains("<path>"));
        assert!(norm.contains("line N"));
    }

    #[test]
    fn test_normalize_message_strips_entry_ids() {
        let msg = "Entry B-001 failed with D-042 error";
        let norm = normalize_message(msg);
        assert!(!norm.contains("B-001"));
        assert!(!norm.contains("D-042"));
        assert!(norm.contains("<id>"));
    }

    #[test]
    fn test_normalize_message_strips_line_col() {
        let msg = "error at 3:5 in script";
        let norm = normalize_message(msg);
        assert!(!norm.contains("3:5"));
        assert!(norm.contains("N:N"));
    }

    #[test]
    fn test_normalize_message_collapses_whitespace() {
        let msg = "error   at   multiple   spaces";
        let norm = normalize_message(msg);
        assert_eq!(norm, "error at multiple spaces");
    }

    #[test]
    fn test_hash_error_deterministic() {
        let h1 = hash_error("B3_behavioral_fail", "execution timeout");
        let h2 = hash_error("B3_behavioral_fail", "execution timeout");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_error_different_codes() {
        let h1 = hash_error("B3_behavioral_fail", "execution timeout");
        let h2 = hash_error("D_lint_fail", "execution timeout");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_error_different_messages() {
        let h1 = hash_error("D_lint_fail", "SC2086 warning");
        let h2 = hash_error("D_lint_fail", "SC2034 warning");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_classify_risk_sec_rule() {
        let signals = vec!["D_lint_fail".to_string()];
        assert_eq!(classify_risk(&signals, "SEC001 detected"), RiskLevel::High);
    }

    #[test]
    fn test_classify_risk_b3_fail() {
        let signals = vec!["B3_behavioral_fail".to_string()];
        assert_eq!(classify_risk(&signals, ""), RiskLevel::High);
    }

    #[test]
    fn test_classify_risk_g_fail() {
        let signals = vec!["G_cross_shell_fail".to_string()];
        assert_eq!(classify_risk(&signals, ""), RiskLevel::Medium);
    }

    #[test]
    fn test_classify_risk_quoting() {
        let signals = vec!["D_lint_fail".to_string()];
        assert_eq!(
            classify_risk(&signals, "SC2086 unquoted var"),
            RiskLevel::Medium
        );
    }

    #[test]
    fn test_classify_risk_lint_only() {
        let signals = vec!["D_lint_fail".to_string()];
        assert_eq!(classify_risk(&signals, "some lint warning"), RiskLevel::Low);
    }

    #[test]
    fn test_classify_risk_transpile_fail() {
        let signals = vec!["A_transpile_fail".to_string()];
        assert_eq!(classify_risk(&signals, ""), RiskLevel::High);
    }

    #[test]
    fn test_classify_risk_containment() {
        let signals = vec!["B1_containment_fail".to_string()];
        assert_eq!(classify_risk(&signals, ""), RiskLevel::Low);
    }

    #[test]
    fn test_labeling_rules_count() {
        let rules = labeling_rules();
        assert_eq!(rules.len(), 5);
    }

    #[test]
    fn test_labeling_rules_names() {
        let rules = labeling_rules();
        let names: Vec<&str> = rules.iter().map(|r| r.name).collect();
        assert_eq!(
            names,
            vec!["SEC_RULE", "B3_FAIL", "G_FAIL", "QUOTING", "LINT_ONLY"]
        );
    }

    #[test]
    fn test_labeling_rules_risk_levels() {
        let rules = labeling_rules();
        assert_eq!(rules[0].risk, RiskLevel::High); // SEC_RULE
        assert_eq!(rules[1].risk, RiskLevel::High); // B3_FAIL
        assert_eq!(rules[2].risk, RiskLevel::Medium); // G_FAIL
        assert_eq!(rules[3].risk, RiskLevel::Medium); // QUOTING
        assert_eq!(rules[4].risk, RiskLevel::Low); // LINT_ONLY
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(format!("{}", RiskLevel::High), "HIGH");
        assert_eq!(format!("{}", RiskLevel::Medium), "MEDIUM");
        assert_eq!(format!("{}", RiskLevel::Low), "LOW");
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::High < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::Low);
    }

    #[test]
    fn test_shell_training_error_serialization() {
        let err = ShellTrainingError {
            error_code: "B3_behavioral_fail".to_string(),
            message: "execution timeout".to_string(),
            hash: 12345,
            count: 1,
            risk: RiskLevel::High,
            entry_ids: vec!["B-143".to_string()],
        };
        let json = serde_json::to_string(&err).expect("serialize failed");
        assert!(json.contains("B3_behavioral_fail"));
        assert!(json.contains("\"risk\":\"High\""));
    }

    #[test]
    fn test_error_triage_serialization() {
        let triage = ErrorTriage {
            errors: vec![],
            total_raw: 0,
            total_unique: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
        };
        let json = serde_json::to_string(&triage).expect("serialize failed");
        assert!(json.contains("total_raw"));
        assert!(json.contains("total_unique"));
    }
}
