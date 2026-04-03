#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: linter-docker-make-v1.yaml
//!
//! Each test pair attempts to FALSIFY a Dockerfile or Makefile linter contract.
//! SOUND: known-bad input MUST be flagged.
//! PRECISE: known-safe input must NOT be flagged.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

fn docker_has(source: &str, code: &str) -> bool {
    let result = bashrs::linter::rules::lint_dockerfile(source);
    result.diagnostics.iter().any(|d| d.code == code)
}

fn make_has(source: &str, code: &str) -> bool {
    let result = bashrs::linter::lint_makefile(source);
    result.diagnostics.iter().any(|d| d.code == code)
}

// ============================================================================
// DOCKER001: missing USER directive
// ============================================================================

#[test]
fn falsify_DOCKER001_SOUND_no_user() {
    assert!(
        docker_has("FROM ubuntu:latest\nRUN apt-get update", "DOCKER001"),
        "F-DOCKER001-SOUND: Dockerfile without USER MUST be flagged"
    );
}

#[test]
fn falsify_DOCKER001_PRECISE_has_user() {
    assert!(
        !docker_has("FROM ubuntu:22.04\nUSER appuser\nRUN echo hi", "DOCKER001"),
        "F-DOCKER001-PRECISE: Dockerfile with USER must NOT be flagged"
    );
}

// ============================================================================
// DOCKER002: unpinned base image
// ============================================================================

#[test]
fn falsify_DOCKER002_SOUND_latest_tag() {
    assert!(
        docker_has("FROM ubuntu:latest\nRUN echo hi", "DOCKER002"),
        "F-DOCKER002-SOUND: :latest tag MUST be flagged"
    );
}

#[test]
fn falsify_DOCKER002_PRECISE_sha256_pinned() {
    // SHA256-pinned images should not trigger DOCKER002
    assert!(
        !docker_has(
            "FROM ubuntu@sha256:abcdef1234567890abcdef1234567890\nRUN echo hi",
            "DOCKER002"
        ),
        "F-DOCKER002-PRECISE: sha256-pinned image must NOT be flagged"
    );
}

// ============================================================================
// DOCKER006: ADD vs COPY
// ============================================================================

#[test]
fn falsify_DOCKER006_SOUND_add_local() {
    assert!(
        docker_has("FROM ubuntu\nADD . /app", "DOCKER006"),
        "F-DOCKER006-SOUND: ADD for local files MUST be flagged"
    );
}

#[test]
fn falsify_DOCKER006_PRECISE_copy() {
    assert!(
        !docker_has("FROM ubuntu\nCOPY . /app", "DOCKER006"),
        "F-DOCKER006-PRECISE: COPY must NOT be flagged"
    );
}

// ============================================================================
// MAKE001: unsorted wildcard
// ============================================================================

#[test]
fn falsify_MAKE001_SOUND_bare_wildcard() {
    assert!(
        make_has("SRCS = $(wildcard *.c)", "MAKE001"),
        "F-MAKE001-SOUND: $(wildcard) without $(sort) MUST be flagged"
    );
}

#[test]
fn falsify_MAKE001_PRECISE_sorted_wildcard() {
    assert!(
        !make_has("SRCS := $(sort $(wildcard *.c))", "MAKE001"),
        "F-MAKE001-PRECISE: $(sort $(wildcard)) must NOT be flagged"
    );
}

// ============================================================================
// MAKE002: mkdir without -p
// ============================================================================

#[test]
fn falsify_MAKE002_SOUND_mkdir_bare() {
    assert!(
        make_has("all:\n\tmkdir /tmp/foo", "MAKE002"),
        "F-MAKE002-SOUND: mkdir without -p in recipe MUST be flagged"
    );
}

#[test]
fn falsify_MAKE002_PRECISE_mkdir_p() {
    assert!(
        !make_has("all:\n\tmkdir -p /tmp/foo", "MAKE002"),
        "F-MAKE002-PRECISE: mkdir -p must NOT be flagged"
    );
}

// ============================================================================
// MAKE004: missing .PHONY
// ============================================================================

#[test]
fn falsify_MAKE004_SOUND_no_phony() {
    assert!(
        make_has("all:\n\techo done", "MAKE004"),
        "F-MAKE004-SOUND: target without .PHONY MUST be flagged"
    );
}

#[test]
fn falsify_MAKE004_PRECISE_has_phony() {
    assert!(
        !make_has(".PHONY: all\nall:\n\techo done", "MAKE004"),
        "F-MAKE004-PRECISE: .PHONY target must NOT be flagged"
    );
}
