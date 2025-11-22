#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
//! Test for Issue #10: Dockerfile-specific quality scoring
//!
//! RED PHASE: These tests should FAIL initially

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn rash_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

// Example Dockerfiles for testing

const GOOD_DOCKERFILE: &str = r#"FROM alpine:3.18

# Install dependencies with version pinning
RUN set -euo pipefail && \
    apk add --no-cache \
        curl=8.2.1-r0 \
        bash=5.2.15-r5 && \
    rm -rf /var/cache/apk/*

WORKDIR /app

COPY app.sh /app/

RUN chmod +x /app/app.sh

USER nobody

CMD ["/app/app.sh"]
"#;

const BAD_DOCKERFILE: &str = r#"FROM alpine

RUN apk update
RUN apk upgrade
RUN apk add curl
RUN apk add bash

WORKDIR /app

COPY app.sh /app/

RUN chmod 777 /app/app.sh

CMD /app/app.sh
"#;

const EXCELLENT_DOCKERFILE: &str = r#"# Multi-stage build example
# Purpose: Build optimized production container
FROM rust:1.73.0-alpine AS builder

RUN set -euo pipefail && \
    apk add --no-cache \
        musl-dev=1.2.4-r1 \
        openssl-dev=3.1.2-r0 && \
    rm -rf /var/cache/apk/*

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY src ./src
RUN set -euo pipefail && \
    cargo build --release && \
    strip target/release/app

FROM alpine:3.18

RUN set -euo pipefail && \
    apk add --no-cache \
        ca-certificates=20230506-r0 \
        libgcc=12.2.1_git20220924-r10 && \
    rm -rf /var/cache/apk/* && \
    addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

WORKDIR /app

COPY --from=builder --chown=appuser:appuser /build/target/release/app /app/

USER appuser

HEALTHCHECK --interval=30s --timeout=3s \
    CMD pgrep app || exit 1

CMD ["/app/app"]
"#;

#[test]
fn test_issue_010_dockerfile_flag_exists() {
    // RED: Test that --dockerfile flag is recognized
    rash_cmd()
        .arg("score")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--dockerfile"));
}

#[test]
fn test_issue_010_score_dockerfile_mode_good() {
    let temp_file = "/tmp/test_dockerfile_good.dockerfile";
    fs::write(temp_file, GOOD_DOCKERFILE).unwrap();

    // RED: Test that --dockerfile flag produces different output
    rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_file)
        .arg("--detailed")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dockerfile Quality Score"))
        .stdout(predicate::str::contains("Safety:"))
        .stdout(predicate::str::contains("Layer Optimization:"))
        .stdout(predicate::str::contains("Determinism:"));

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_issue_010_score_dockerfile_mode_bad() {
    let temp_file = "/tmp/test_dockerfile_bad.dockerfile";
    fs::write(temp_file, BAD_DOCKERFILE).unwrap();

    // RED: Bad Dockerfile should score poorly (D or F grade)
    let output = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_file)
        .arg("--detailed")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention grade and likely D or F
    assert!(stdout.contains("Overall Grade:"));
    assert!(
        stdout.contains("Grade: D") || stdout.contains("Grade: F"),
        "Expected D or F grade for bad Dockerfile"
    );

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_issue_010_score_dockerfile_mode_excellent() {
    let temp_file = "/tmp/test_dockerfile_excellent.dockerfile";
    fs::write(temp_file, EXCELLENT_DOCKERFILE).unwrap();

    // RED: Excellent Dockerfile should score highly (A or B grade)
    let output = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_file)
        .arg("--detailed")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should mention grade and likely A or B
    assert!(stdout.contains("Overall Grade:"));
    assert!(
        stdout.contains("Grade: A") || stdout.contains("Grade: B"),
        "Expected A or B grade for excellent Dockerfile"
    );

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_issue_010_dockerfile_detects_set_pipefail() {
    let dockerfile_with_pipefail = r#"FROM alpine:3.18
RUN set -euo pipefail && apk add curl=8.2.1-r0
"#;

    let dockerfile_without_pipefail = r#"FROM alpine:3.18
RUN apk add curl
"#;

    let temp_with = "/tmp/test_dockerfile_with_pipefail.dockerfile";
    let temp_without = "/tmp/test_dockerfile_without_pipefail.dockerfile";

    fs::write(temp_with, dockerfile_with_pipefail).unwrap();
    fs::write(temp_without, dockerfile_without_pipefail).unwrap();

    // RED: Dockerfile with pipefail should score higher in safety
    let output_with = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_with)
        .arg("--detailed")
        .output()
        .unwrap();

    let output_without = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_without)
        .arg("--detailed")
        .output()
        .unwrap();

    // Both should succeed
    assert!(output_with.status.success());
    assert!(output_without.status.success());

    // With pipefail should mention it as good practice
    let _stdout_with = String::from_utf8_lossy(&output_with.stdout);
    let stdout_without = String::from_utf8_lossy(&output_without.stdout);

    // Without pipefail should suggest adding it
    assert!(
        stdout_without.contains("set -euo pipefail") || stdout_without.contains("error handling")
    );

    let _ = fs::remove_file(temp_with);
    let _ = fs::remove_file(temp_without);
}

#[test]
fn test_issue_010_dockerfile_detects_version_pinning() {
    let dockerfile_pinned = r#"FROM alpine:3.18
RUN apk add --no-cache curl=8.2.1-r0
"#;

    let dockerfile_unpinned = r#"FROM alpine:latest
RUN apk add curl
"#;

    let temp_pinned = "/tmp/test_dockerfile_pinned.dockerfile";
    let temp_unpinned = "/tmp/test_dockerfile_unpinned.dockerfile";

    fs::write(temp_pinned, dockerfile_pinned).unwrap();
    fs::write(temp_unpinned, dockerfile_unpinned).unwrap();

    // RED: Pinned versions should score higher in determinism
    let output_pinned = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_pinned)
        .arg("--detailed")
        .output()
        .unwrap();

    let output_unpinned = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_unpinned)
        .arg("--detailed")
        .output()
        .unwrap();

    assert!(output_pinned.status.success());
    assert!(output_unpinned.status.success());

    let stdout_unpinned = String::from_utf8_lossy(&output_unpinned.stdout);

    // Unpinned should suggest version pinning
    assert!(stdout_unpinned.contains("version") || stdout_unpinned.contains("pin"));

    let _ = fs::remove_file(temp_pinned);
    let _ = fs::remove_file(temp_unpinned);
}

#[test]
fn test_issue_010_dockerfile_detects_cache_cleanup() {
    let dockerfile_with_cleanup = r#"FROM alpine:3.18
RUN set -euo pipefail && \
    apk add --no-cache curl=8.2.1-r0 && \
    rm -rf /var/cache/apk/*
"#;

    let dockerfile_without_cleanup = r#"FROM alpine:3.18
RUN apk add curl
"#;

    let temp_with = "/tmp/test_dockerfile_with_cleanup.dockerfile";
    let temp_without = "/tmp/test_dockerfile_without_cleanup.dockerfile";

    fs::write(temp_with, dockerfile_with_cleanup).unwrap();
    fs::write(temp_without, dockerfile_without_cleanup).unwrap();

    // RED: Dockerfile with cleanup should score higher in layer optimization
    let output_with = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_with)
        .arg("--detailed")
        .output()
        .unwrap();

    let output_without = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_without)
        .arg("--detailed")
        .output()
        .unwrap();

    assert!(output_with.status.success());
    assert!(output_without.status.success());

    let stdout_without = String::from_utf8_lossy(&output_without.stdout);

    // Without cleanup should suggest it
    assert!(stdout_without.contains("cache") || stdout_without.contains("cleanup"));

    let _ = fs::remove_file(temp_with);
    let _ = fs::remove_file(temp_without);
}

#[test]
fn test_issue_010_dockerfile_json_output() {
    let temp_file = "/tmp/test_dockerfile_json.dockerfile";
    fs::write(temp_file, GOOD_DOCKERFILE).unwrap();

    // RED: JSON output should work with --dockerfile
    rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_file)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"grade\""))
        .stdout(predicate::str::contains("\"score\""))
        .stdout(predicate::str::contains("\"safety\""))
        .stdout(predicate::str::contains("\"layer_optimization\""))
        .stdout(predicate::str::contains("\"determinism\""));

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_issue_010_normal_bash_script_still_works() {
    let bash_script = r#"#!/bin/bash
set -euo pipefail

function main() {
    echo "test"
}

main "$@"
"#;

    let temp_file = "/tmp/test_normal_bash.sh";
    fs::write(temp_file, bash_script).unwrap();

    // RED: Normal bash scripts without --dockerfile should still work
    rash_cmd()
        .arg("score")
        .arg(temp_file)
        .arg("--detailed")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bash Script Quality Score"))
        .stdout(predicate::str::contains("Complexity:"))
        .stdout(predicate::str::contains("Safety:"))
        .stdout(predicate::str::contains("Maintainability:"));

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_issue_010_dockerfile_multi_stage_detected() {
    let temp_file = "/tmp/test_dockerfile_multistage.dockerfile";
    fs::write(temp_file, EXCELLENT_DOCKERFILE).unwrap();

    // RED: Multi-stage builds should be recognized and scored positively
    rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_file)
        .arg("--detailed")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("multi-stage")
                .or(predicate::str::contains("Multi-stage"))
                .or(predicate::str::contains("optimization")),
        );

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_issue_010_dockerfile_user_directive_security() {
    let dockerfile_with_user = r#"FROM alpine:3.18
RUN adduser -D appuser
USER appuser
CMD ["./app"]
"#;

    let dockerfile_without_user = r#"FROM alpine:3.18
CMD ["./app"]
"#;

    let temp_with = "/tmp/test_dockerfile_with_user.dockerfile";
    let temp_without = "/tmp/test_dockerfile_without_user.dockerfile";

    fs::write(temp_with, dockerfile_with_user).unwrap();
    fs::write(temp_without, dockerfile_without_user).unwrap();

    // RED: Dockerfile with USER directive should score higher in security
    let output_without = rash_cmd()
        .arg("score")
        .arg("--dockerfile")
        .arg(temp_without)
        .arg("--detailed")
        .output()
        .unwrap();

    assert!(output_without.status.success());

    let stdout_without = String::from_utf8_lossy(&output_without.stdout);

    // Should suggest adding USER directive
    assert!(stdout_without.contains("USER") || stdout_without.contains("non-root"));

    let _ = fs::remove_file(temp_with);
    let _ = fs::remove_file(temp_without);
}
