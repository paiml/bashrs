//! Tests for DOCKER004: Invalid COPY --from reference
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::docker004;

#[test]
fn test_docker004_valid_copy_from_stage() {
    let source = "FROM golang:1.21 AS builder\nRUN go build\nFROM alpine:3.18\nCOPY --from=builder /app /app\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty(), "Valid stage reference should pass");
}

#[test]
fn test_docker004_invalid_copy_from_stage() {
    let source = "FROM golang:1.21 AS builder\nFROM alpine:3.18\nCOPY --from=nonexistent /app /app\n";
    let result = docker004::check(source);
    assert!(!result.diagnostics.is_empty(), "Invalid stage reference should fail");
}

#[test]
fn test_docker004_numeric_stage_reference() {
    let source = "FROM golang:1.21\nFROM alpine:3.18\nCOPY --from=0 /app /app\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty(), "Numeric stage reference is always valid");
}

#[test]
fn test_docker004_no_copy_from() {
    let source = "FROM ubuntu:22.04\nCOPY app.py /app/\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty(), "Regular COPY without --from is fine");
}

#[test]
fn test_docker004_empty_dockerfile() {
    let source = "";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_docker004_multiple_stages() {
    let source = "FROM golang:1.21 AS builder\nFROM node:18 AS frontend\nFROM alpine:3.18\nCOPY --from=builder /app /app\nCOPY --from=frontend /dist /dist\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty(), "Both stages are defined");
}

#[test]
fn test_docker004_mixed_valid_invalid() {
    let source = "FROM golang:1.21 AS builder\nFROM alpine:3.18\nCOPY --from=builder /app /app\nCOPY --from=missing /other /other\n";
    let result = docker004::check(source);
    assert_eq!(result.diagnostics.len(), 1, "Only the invalid reference should fail");
}

#[test]
fn test_docker004_case_sensitive_stage_name() {
    let source = "FROM golang:1.21 AS Builder\nFROM alpine:3.18\nCOPY --from=builder /app /app\n";
    let result = docker004::check(source);
    // Stage names are case-sensitive: Builder != builder
    assert!(!result.diagnostics.is_empty(), "Case mismatch should be flagged");
}

#[test]
fn test_docker004_diagnostic_message() {
    let source = "FROM alpine:3.18\nCOPY --from=myapp /app /app\n";
    let result = docker004::check(source);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("myapp"));
}

#[test]
fn test_docker004_no_from_directive() {
    let source = "COPY /local /container\nRUN echo hi\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_docker004_numeric_stage_1() {
    let source = "FROM golang:1.21\nFROM node:18\nFROM alpine:3.18\nCOPY --from=1 /dist /dist\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty(), "Numeric index 1 is always valid");
}

#[test]
fn test_docker004_stage_name_with_hyphens() {
    let source = "FROM golang:1.21 AS my-builder\nFROM alpine:3.18\nCOPY --from=my-builder /app /app\n";
    let result = docker004::check(source);
    assert!(result.diagnostics.is_empty(), "Hyphenated stage name should work");
}
