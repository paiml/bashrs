#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Dockerfile Transpilation Integration Tests
//!
//! Tests the Rust DSL -> Dockerfile transpilation pipeline.
//! Uses assert_cmd for CLI testing (MANDATORY per CLAUDE.md).

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// Create a bashrs command (MANDATORY pattern per CLAUDE.md)
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Create a temporary Rust file with given content
fn create_temp_rust_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file
}

#[test]
fn test_DOCKER_BUILD_001_basic_generation() {
    let rust_code = r#"
        fn main() {
            from_image("rust", "1.75-alpine");
            workdir("/app");
            copy(".", ".");
            user("65534");
        }
        fn from_image(image: &str, tag: &str) {}
        fn workdir(path: &str) {}
        fn copy(src: &str, dst: &str) {}
        fn user(u: &str) {}
    "#;

    let result = bashrs::transpile_dockerfile(rust_code, bashrs::Config::default()).unwrap();
    assert!(
        result.contains("FROM rust:1.75-alpine"),
        "Expected FROM in: {}",
        result
    );
    assert!(result.contains("WORKDIR /app"));
    assert!(result.contains("COPY . ."));
    assert!(result.contains("USER 65534"));
}

#[test]
fn test_DOCKER_BUILD_002_multi_stage() {
    let rust_code = r#"
        fn main() {
            from_image_as("rust", "1.75-alpine", "builder");
            workdir("/app");
            from_image("alpine", "3.18");
            copy_from("builder", "/app/bin", "/usr/local/bin/");
        }
        fn from_image_as(image: &str, tag: &str, alias: &str) {}
        fn from_image(image: &str, tag: &str) {}
        fn workdir(path: &str) {}
        fn copy_from(stage: &str, src: &str, dst: &str) {}
    "#;

    let result = bashrs::transpile_dockerfile(rust_code, bashrs::Config::default()).unwrap();
    assert!(
        result.contains("FROM rust:1.75-alpine AS builder"),
        "Expected multi-stage build in: {}",
        result
    );
    assert!(result.contains("FROM alpine:3.18"));
    assert!(result.contains("COPY --from=builder"));
}

#[test]
fn test_DOCKER_BUILD_003_cli_dockerfile_build() {
    let rust_code = r#"
        fn main() {
            from_image("alpine", "3.18");
            user("65534");
        }
        fn from_image(image: &str, tag: &str) {}
        fn user(u: &str) {}
    "#;

    let input_file = create_temp_rust_file(rust_code);
    let output_file = NamedTempFile::new().unwrap();

    bashrs_cmd()
        .arg("dockerfile")
        .arg("build")
        .arg(input_file.path())
        .arg("-o")
        .arg(output_file.path())
        .assert()
        .success();

    let output = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(
        output.contains("FROM alpine:3.18"),
        "CLI should produce valid Dockerfile: {}",
        output
    );
}

#[test]
fn test_DOCKER_BUILD_004_user_directive() {
    let rust_code = r#"
        fn main() {
            from_image("alpine", "3.18");
            user("65534");
        }
        fn from_image(image: &str, tag: &str) {}
        fn user(u: &str) {}
    "#;

    let result = bashrs::transpile_dockerfile(rust_code, bashrs::Config::default()).unwrap();
    assert!(
        result.contains("USER 65534"),
        "USER directive must be present for DOCKER003 compliance"
    );
}

#[test]
fn test_DOCKER_BUILD_005_no_latest_tag() {
    let rust_code = r#"
        fn main() {
            from_image("rust", "1.75-alpine");
        }
        fn from_image(image: &str, tag: &str) {}
    "#;

    let result = bashrs::transpile_dockerfile(rust_code, bashrs::Config::default()).unwrap();
    assert!(
        !result.contains(":latest"),
        "Should use pinned versions, not latest (DOCKER002)"
    );
}

#[test]
fn test_DOCKER_BUILD_006_determinism() {
    let rust_code = r#"
        fn main() {
            from_image("alpine", "3.18");
            workdir("/app");
            user("65534");
        }
        fn from_image(image: &str, tag: &str) {}
        fn workdir(path: &str) {}
        fn user(u: &str) {}
    "#;

    let config = bashrs::Config::default();
    let result1 = bashrs::transpile_dockerfile(rust_code, config.clone()).unwrap();
    let result2 = bashrs::transpile_dockerfile(rust_code, config).unwrap();
    assert_eq!(
        result1, result2,
        "Dockerfile transpilation must be deterministic"
    );
}
