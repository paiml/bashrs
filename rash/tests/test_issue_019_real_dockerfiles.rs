#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Test Issue #19 Dockerfile linting on real ruchy-docker Dockerfiles

use bashrs::linter::rules::lint_dockerfile;
use std::fs;
use std::path::Path;

#[test]
#[ignore] // Run with --ignored to test against real files
fn test_real_python_dockerfile() {
    let path = "/home/noah/src/ruchy-docker/docker/python/fibonacci.Dockerfile";
    if !Path::new(path).exists() {
        println!("Skipping: {} not found", path);
        return;
    }

    let dockerfile = fs::read_to_string(path).expect("Failed to read Dockerfile");
    let result = lint_dockerfile(&dockerfile);

    println!("\n=== {} ===", path);
    println!("Total diagnostics: {}", result.diagnostics.len());
    for diag in &result.diagnostics {
        println!(
            "  Line {}: {} - {}",
            diag.span.start_line, diag.code, diag.message
        );
    }

    // Python Dockerfile has USER directive, so should NOT warn DOCKER001
    let docker001_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER001")
        .count();
    assert_eq!(docker001_count, 0, "Python Dockerfile has USER directive");

    // Should warn about unpinned base images
    let docker002_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER002")
        .count();
    assert!(
        docker002_count > 0,
        "Should warn about unpinned base images"
    );
}

#[test]
#[ignore]
fn test_real_rust_dockerfile() {
    let path = "/home/noah/src/ruchy-docker/docker/rust/fibonacci.Dockerfile";
    if !Path::new(path).exists() {
        println!("Skipping: {} not found", path);
        return;
    }

    let dockerfile = fs::read_to_string(path).expect("Failed to read Dockerfile");
    let result = lint_dockerfile(&dockerfile);

    println!("\n=== {} ===", path);
    println!("Total diagnostics: {}", result.diagnostics.len());
    for diag in &result.diagnostics {
        println!(
            "  Line {}: {} - {}",
            diag.span.start_line, diag.code, diag.message
        );
    }

    // Rust Dockerfile uses scratch, so should NOT warn DOCKER001
    let docker001_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER001")
        .count();
    assert_eq!(
        docker001_count, 0,
        "Rust Dockerfile uses scratch (no USER needed)"
    );

    // Should warn about unpinned base images in builder stage
    let docker002_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "DOCKER002")
        .count();
    assert!(
        docker002_count > 0,
        "Should warn about unpinned base images"
    );
}
