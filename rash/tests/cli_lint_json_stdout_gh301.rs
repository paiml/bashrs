//! GH-301: `bashrs lint --format json` prints exactly one JSON document on
//! stdout. Tracing output belongs on stderr, where it cannot corrupt the
//! document every machine consumer parses.
#![allow(clippy::unwrap_used)]
#![allow(non_snake_case)] // ticket-mandated test name: test_<TICKET>_<scenario>

use assert_cmd::Command;

#[test]
fn test_GH301_lint_json_stdout_is_one_json_document() {
    let dir = std::env::temp_dir().join(format!("bashrs-gh301-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("t.sh");
    std::fs::write(&script, "#!/bin/sh\necho \"$(date)\" > VERSION\n").unwrap();
    let out = Command::cargo_bin("bashrs")
        .unwrap()
        .args(["lint", "--format", "json"])
        .arg(&script)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let doc: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not one JSON document ({e}):\n{stdout}"));
    assert!(
        doc.get("diagnostics").is_some(),
        "no diagnostics key: {doc}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
