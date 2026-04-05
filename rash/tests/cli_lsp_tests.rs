#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for `bashrs lsp` (PMAT-211)
//!
//! Uses assert_cmd (MANDATORY per CLAUDE.md).
//! LSP is feature-gated behind `lsp` — these tests only run when built with that feature.

use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

// ---------------------------------------------------------------------------
// Help and CLI arg tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT211_lsp_in_main_help() {
    bashrs_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("lsp"));
}

#[test]
fn test_PMAT211_lsp_help() {
    bashrs_cmd()
        .arg("lsp")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Language Server Protocol"))
        .stdout(predicate::str::contains("--stdio"));
}

// ---------------------------------------------------------------------------
// LSP initialize smoke test (send request via stdin, check response)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT211_lsp_initialize_response() {
    let init_request =
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
    let content = format!(
        "Content-Length: {}\r\n\r\n{}",
        init_request.len(),
        init_request
    );

    let output = bashrs_cmd()
        .arg("lsp")
        .write_stdin(content)
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .expect("Failed to run bashrs lsp");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain a valid JSON-RPC response
    assert!(
        stdout.contains("\"jsonrpc\":\"2.0\""),
        "Expected JSON-RPC response, got: {stdout}"
    );
    assert!(
        stdout.contains("bashrs-lsp"),
        "Expected server name bashrs-lsp, got: {stdout}"
    );
    assert!(
        stdout.contains("textDocumentSync"),
        "Expected textDocumentSync capability, got: {stdout}"
    );
}

#[test]
fn test_PMAT211_lsp_version_in_response() {
    let init_request =
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
    let content = format!(
        "Content-Length: {}\r\n\r\n{}",
        init_request.len(),
        init_request
    );

    let output = bashrs_cmd()
        .arg("lsp")
        .write_stdin(content)
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .expect("Failed to run bashrs lsp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Version should be present in server info
    assert!(
        stdout.contains("version"),
        "Expected version in response, got: {stdout}"
    );
}

#[test]
fn test_PMAT211_lsp_diagnostic_provider() {
    let init_request =
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
    let content = format!(
        "Content-Length: {}\r\n\r\n{}",
        init_request.len(),
        init_request
    );

    let output = bashrs_cmd()
        .arg("lsp")
        .write_stdin(content)
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .expect("Failed to run bashrs lsp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("diagnosticProvider"),
        "Expected diagnosticProvider capability, got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Code Action capability tests (PMAT-214)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT214_lsp_code_action_capability() {
    let init_request =
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
    let content = format!(
        "Content-Length: {}\r\n\r\n{}",
        init_request.len(),
        init_request
    );

    let output = bashrs_cmd()
        .arg("lsp")
        .write_stdin(content)
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .expect("Failed to run bashrs lsp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("codeActionProvider"),
        "Expected codeActionProvider capability, got: {stdout}"
    );
}

#[test]
fn test_PMAT214_lsp_quickfix_kind() {
    let init_request =
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
    let content = format!(
        "Content-Length: {}\r\n\r\n{}",
        init_request.len(),
        init_request
    );

    let output = bashrs_cmd()
        .arg("lsp")
        .write_stdin(content)
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .expect("Failed to run bashrs lsp");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("quickfix"),
        "Expected quickfix code action kind, got: {stdout}"
    );
}
