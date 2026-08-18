#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! GH-230 end-to-end CLI tests.
//!
//! DET002 used to fire identically on a genuine reproducibility defect, on a
//! log line, and on `SOURCE_DATE_EPOCH` - its own remedy. These drive the real
//! binary over the three reproducers from the ticket.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper: the bashrs binary under test.
fn rash_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

/// Write a script to a temp file and hand back the handle (keeps it alive).
fn script(body: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".sh").expect("temp file");
    f.write_all(body.as_bytes()).expect("write script");
    f.flush().expect("flush script");
    f
}

/// The genuine defect: a timestamp that names a build artifact.
const ARTIFACT: &str =
    "#!/bin/sh\nTIMESTAMP=\"$(date +%Y%m%d_%H%M%S)\"\ncp build.log \"out/report_$TIMESTAMP.log\"\n";
/// The log line: append-only, so the timestamp IS the point.
const LOG_LINE: &str = "#!/bin/sh\nLOG_FILE=/var/log/app.log\necho \"[$(date '+%Y-%m-%d %H:%M:%S')] started\" | tee -a \"$LOG_FILE\"\n";
/// The remedy: `SOURCE_DATE_EPOCH` with a `date` fallback.
const SDE: &str = "#!/bin/sh\nTIMESTAMP=\"$(date -u -d \"@${SOURCE_DATE_EPOCH:-$(date +%s)}\" +%Y%m%d 2>/dev/null || date +%Y%m%d)\"\ncp build.log \"out/report_$TIMESTAMP.log\"\n";

#[test]
fn test_GH230_cli_log_script_reports_no_det002() {
    let f = script(LOG_LINE);
    rash_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("DET002").not());
}

#[test]
fn test_GH230_cli_source_date_epoch_script_is_clean() {
    let f = script(SDE);
    rash_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("DET002").not());
}

#[test]
fn test_GH230_cli_artifact_script_names_the_sink_line() {
    let f = script(ARTIFACT);
    rash_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("DET002"))
        .stdout(predicate::str::contains("line 3"))
        .stdout(predicate::str::contains("SOURCE_DATE_EPOCH"));
}

#[test]
fn test_GH230_cli_artifact_span_stays_on_the_date_line() {
    // Suppressions and .bashrsignore line scopes key on span.start_line, so the
    // anchor must not move to the sink.
    let f = script(ARTIFACT);
    rash_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("2:12-18"));
}

#[test]
fn test_GH230_cli_json_output_carries_the_sink_message() {
    let f = script(ARTIFACT);
    let out = rash_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg("--format")
        .arg("json")
        .arg(f.path())
        .output()
        .expect("run bashrs lint --format json");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // `bashrs lint` writes a tracing INFO line to stdout ahead of the JSON
    // document, so start parsing at the first `{` (same as the corpus tooling).
    let start = stdout.find('{').expect("JSON document in stdout");
    let v: serde_json::Value =
        serde_json::from_str(&stdout[start..]).expect("valid JSON lint output");
    let diags = v["diagnostics"].as_array().expect("diagnostics array");
    let det002: Vec<&serde_json::Value> = diags
        .iter()
        .filter(|d| d["code"] == "DET002")
        .collect::<Vec<_>>();
    assert_eq!(det002.len(), 1, "exactly one DET002 in {stdout}");
    let msg = det002[0]["message"].as_str().expect("message string");
    assert!(msg.contains("line 3"), "message must name the sink: {msg}");
    assert!(msg.contains("SOURCE_DATE_EPOCH"), "remedy missing: {msg}");
}

#[test]
fn test_GH230_cli_suppression_on_the_date_line_still_works() {
    // The span stayed put, so an existing `disable-line` keeps working.
    let body = "#!/bin/sh\nTIMESTAMP=\"$(date +%Y%m%d)\" # bashrs disable-line=DET002\ncp build.log \"out/report_$TIMESTAMP.log\"\n";
    let f = script(body);
    rash_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("DET002").not());
}
