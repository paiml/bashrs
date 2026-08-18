#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Test bashrs Dockerfile linting on all ruchy-docker Dockerfiles

use bashrs::linter::rules::lint_dockerfile;
use std::fs;
use std::path::Path;

fn test_dockerfile(path: &str, name: &str) {
    if !Path::new(path).exists() {
        println!("Skipping: {} (not found)", name);
        return;
    }

    let dockerfile = fs::read_to_string(path).expect("Failed to read");
    let result = lint_dockerfile(&dockerfile);

    println!("\n===================================================================");
    println!("{}", name);
    println!("===================================================================");

    if result.diagnostics.is_empty() {
        println!("✅ No issues found!");
    } else {
        println!("Total issues: {}", result.diagnostics.len());
        for diag in &result.diagnostics {
            println!(
                "  Line {:3}: {} - {}",
                diag.span.start_line, diag.code, diag.message
            );
        }
    }
}

#[test]
#[ignore]
fn test_all_ruchy_docker_dockerfiles() {
    let base = "/home/noah/src/ruchy-docker/docker";

    test_dockerfile(&format!("{}/c/fibonacci.Dockerfile", base), "C Language");
    test_dockerfile(&format!("{}/deno/fibonacci.Dockerfile", base), "Deno");
    test_dockerfile(&format!("{}/go/fibonacci.Dockerfile", base), "Go");
    test_dockerfile(&format!("{}/julia/fibonacci.Dockerfile", base), "Julia");
    test_dockerfile(&format!("{}/python/fibonacci.Dockerfile", base), "Python");
    test_dockerfile(
        &format!("{}/ruchy-compiled/fibonacci.Dockerfile", base),
        "Ruchy (Compiled)",
    );
    test_dockerfile(
        &format!("{}/ruchy-transpiled/fibonacci.Dockerfile", base),
        "Ruchy (Transpiled)",
    );
    test_dockerfile(&format!("{}/rust/fibonacci.Dockerfile", base), "Rust");

    println!("\n===================================================================");
    println!("SUMMARY: Tested 8 Dockerfiles from ruchy-docker project");
    println!("===================================================================");
}
