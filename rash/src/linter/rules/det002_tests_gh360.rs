//! GH-360: DET002's message names remedies; every remedy it names must clear
//! the rule. One fixture per advertised form variant, and the message text is
//! pinned so a remedy cannot be advertised without a fixture.

#![allow(non_snake_case)]

use super::check;

fn count(script: &str) -> usize {
    check(script).diagnostics.len()
}

#[test]
fn test_GH360_advertised_append_redirect_clears() {
    assert_eq!(count("#!/bin/bash\ndate +%s >> \"$LOG_FILE\"\n"), 0);
    assert_eq!(count("#!/bin/bash\necho \"t=$(date +%s)\" >> \"$LOG_FILE\"\n"), 0);
}

#[test]
fn test_GH360_advertised_tee_a_clears() {
    assert_eq!(count("#!/bin/bash\necho \"t=$(date +%s)\" | tee -a \"$LOG_FILE\"\n"), 0);
}

#[test]
fn test_GH360_advertised_logger_clears() {
    assert_eq!(count("#!/bin/bash\nlogger \"t=$(date +%s)\"\n"), 0);
}

#[test]
fn test_GH360_message_only_advertises_sinks_that_work() {
    // An append to a target that does not name a log is NOT cleared (an
    // append-only `>> dist/checksums.txt` is still an artifact), so the
    // message must say the target has to be a log.
    assert_eq!(count("#!/bin/bash\ndate +%s >> \"$OUT\"\n"), 1);
    let msg = &check("#!/bin/bash\ndate +%s >> \"$OUT\"\n").diagnostics[0].message;
    assert!(msg.contains("log"), "{msg}");
    assert!(msg.contains("target whose name contains"), "{msg}");
    assert!(
        !msg.contains("append-only sink"),
        "unqualified 'append-only sink' advertises a remedy the rule rejects: {msg}"
    );
}
