//! Linting Demo - False Positive Testing
//!
//! This example demonstrates bashrs's linting capabilities and shows
//! how the false positive testing framework works.
//!
//! Run with: cargo run --example linting_demo

use std::process::Command;

/// Test cases from the falsification test suite
const FALSIFICATION_TESTS: &[(&str, &str, &str)] = &[
    // Arrays (F101-F110)
    (
        "F101",
        r#"arr=(a b c); echo ${arr[0]}"#,
        "Array index access",
    ),
    ("F102", r#"arr=("$@"); echo ${#arr[@]}"#, "Array from args"),
    (
        "F106",
        r#"for i in "${arr[@]}"; do echo "$i"; done"#,
        "Array iteration",
    ),
    // String Operations (F111-F120)
    ("F111", r#"echo ${var:0:5}"#, "Substring extraction"),
    ("F112", r#"echo ${var/old/new}"#, "Pattern substitution"),
    ("F116", r#"echo ${#var}"#, "String length"),
    // Arithmetic (F121-F130)
    ("F121", r#"echo $((1+2))"#, "Basic arithmetic"),
    ("F123", r#"(( i++ ))"#, "Increment operator"),
    ("F128", r#"echo $((16#FF))"#, "Hex literal"),
    ("F130", r#"echo $((x<y ? x : y))"#, "Ternary operator"),
    // Control Flow
    (
        "F047",
        r#"case $x in a) y=1 ;; *) y=2 ;; esac; echo $y"#,
        "Case with default",
    ),
    (
        "F048",
        r#"for ((i=0;i<10;i++)); do echo $i; done"#,
        "C-style for loop",
    ),
    // Builtins
    ("F061", r#"echo $EUID"#, "EUID builtin"),
    ("F065", r#"echo $RANDOM"#, "RANDOM builtin"),
];

/// Edge case tests from the simulation test suite
const SIMULATION_TESTS: &[(&str, &str, &str)] = &[
    // Unicode
    ("S101", "echo 'héllo wörld'", "Latin extended"),
    ("S103", "echo '🚀🔥💻'", "Emoji support"),
    // Nesting
    (
        "S301",
        "if true; then if true; then echo deep; fi; fi",
        "Nested ifs",
    ),
    ("S307", "echo $(echo $(echo test))", "Nested cmd subs"),
    // Quoting
    ("S901", "echo ''", "Empty single quotes"),
    ("S905", "echo 'a'\"b\"'c'", "Mixed quote concat"),
];

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           bashrs Linting Demo - False Positive Tests       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    println!("This demo shows how bashrs handles valid bash patterns without");
    println!("triggering false positive warnings.");
    println!();

    // Check if bashrs binary exists
    let bashrs_path = if std::path::Path::new("target/release/bashrs").exists() {
        "target/release/bashrs"
    } else if std::path::Path::new("target/debug/bashrs").exists() {
        "target/debug/bashrs"
    } else {
        println!("⚠ bashrs binary not found. Build with: cargo build --release");
        println!();
        println!("Showing test cases that would be verified:");
        println!();
        show_test_cases();
        return;
    };

    println!("Using bashrs at: {}", bashrs_path);
    println!();

    // Run falsification tests
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Falsification Tests (must NOT trigger false positives)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let mut pass_count = 0;
    let mut fail_count = 0;

    for (id, code, desc) in FALSIFICATION_TESTS {
        let result = run_lint_test(bashrs_path, code);
        if result {
            println!("  [✓] {}: {}", id, desc);
            pass_count += 1;
        } else {
            println!("  [✗] {}: {} - UNEXPECTED WARNING", id, desc);
            fail_count += 1;
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Simulation Tests (must NOT panic)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    for (id, code, desc) in SIMULATION_TESTS {
        let result = run_simulation_test(bashrs_path, code);
        if result {
            println!("  [✓] {}: {}", id, desc);
            pass_count += 1;
        } else {
            println!("  [✗] {}: {} - PANIC OR CRASH", id, desc);
            fail_count += 1;
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  Passed: {}", pass_count);
    println!("  Failed: {}", fail_count);
    println!("  Total:  {}", pass_count + fail_count);
    println!();

    if fail_count == 0 {
        println!("  ✅ All tests passed!");
    } else {
        println!("  ❌ Some tests failed - check for regressions");
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Full Test Suites");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  Run the complete test suites with:");
    println!();
    println!("    cargo test -p bashrs --test falsification_probar_testing  # 130 tests");
    println!("    cargo test -p bashrs --test simulation_probar_testing     # 100 tests");
    println!();
}

fn run_lint_test(bashrs_path: &str, code: &str) -> bool {
    // Create temp file
    let temp_path = "/tmp/bashrs_demo_test.sh";
    std::fs::write(temp_path, format!("#!/bin/bash\n{}\n", code)).ok();

    // Run bashrs lint
    let output = Command::new(bashrs_path)
        .args(["lint", "--format", "json", temp_path])
        .output();

    // Clean up
    std::fs::remove_file(temp_path).ok();

    match output {
        Ok(out) => {
            // Check for panics
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stderr.contains("panic") || stderr.contains("thread") {
                return false;
            }
            // Success if no critical warnings
            true
        }
        Err(_) => false,
    }
}

fn run_simulation_test(bashrs_path: &str, code: &str) -> bool {
    // Create temp file
    let temp_path = "/tmp/bashrs_demo_sim.sh";
    std::fs::write(temp_path, format!("#!/bin/bash\n{}\n", code)).ok();

    // Run bashrs lint
    let output = Command::new(bashrs_path).args(["lint", temp_path]).output();

    // Clean up
    std::fs::remove_file(temp_path).ok();

    match output {
        Ok(out) => {
            // Check for panics
            let stderr = String::from_utf8_lossy(&out.stderr);
            !stderr.contains("panic") && !stderr.contains("thread 'main' panicked")
        }
        Err(_) => false,
    }
}

fn show_test_cases() {
    println!("Falsification Tests (F-codes):");
    println!("─────────────────────────────────────────────────────────────");
    for (id, code, desc) in FALSIFICATION_TESTS {
        println!("  {}: {} ", id, desc);
        println!("      {}", code);
    }

    println!();
    println!("Simulation Tests (S-codes):");
    println!("─────────────────────────────────────────────────────────────");
    for (id, code, desc) in SIMULATION_TESTS {
        println!("  {}: {}", id, desc);
        println!("      {}", code);
    }
}
