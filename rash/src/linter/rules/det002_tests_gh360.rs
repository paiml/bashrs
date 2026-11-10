//! GH-360: DET002 advertised `>>`, `tee -a` and `logger` as remedies while
//! honouring an append only when its DESTINATION is a log (name contains
//! log/journal/history/audit/trace, or /dev/*). That condition is deliberate
//! (an append to `dist/checksums.txt` is an artifact), so the message now states
//! it and every advertised form is pinned below.

#![allow(non_snake_case)]

use super::check;

fn fires(src: &str) -> bool {
    check(src).diagnostics.iter().any(|d| d.code == "DET002")
}

fn message(src: &str) -> String {
    check(src).diagnostics[0].message.clone()
}

#[test]
fn test_GH360_append_redirect_to_log_destination_clears_det002() {
    assert!(!fires("L_LOG=/tmp/x.jsonl\ndate +%s.%3N >> \"$L_LOG\"\n"));
    assert!(!fires("echo \"t=$(date +%s)\" >> /tmp/latency.log\n"));
}

#[test]
fn test_GH360_tee_a_to_log_destination_clears_det002() {
    assert!(!fires("LOG_FILE=/tmp/x\necho \"t=$(date +%s)\" | tee -a \"$LOG_FILE\"\n"));
}

#[test]
fn test_GH360_logger_clears_det002() {
    assert!(!fires("logger \"t=$(date +%s)\"\n"));
}

#[test]
fn test_GH360_append_to_unnamed_destination_still_fires() {
    assert!(fires("L=/tmp/x.jsonl\necho \"t=$(date +%s)\" >> \"$L\"\n"));
    assert!(fires("L=/tmp/x\necho \"t=$(date +%s)\" | tee -a \"$L\"\n"));
}

#[test]
fn test_GH360_message_states_the_destination_condition() {
    let m = message("L=/tmp/x\necho \"t=$(date +%s)\" >> \"$L\"\n");
    assert!(m.contains("destination") && m.contains("log"), "{m}");
    assert!(!m.contains("(>>, tee -a, logger)"), "{m}");
}
