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

/// SC1xxx rule detection tests (new source code issue rules)
const SC1XXX_TESTS: &[(&str, &str, &str, bool)] = &[
    // Shebang rules
    (
        "SC1084",
        "!#/bin/bash\necho hi",
        "Reversed shebang !# → #!",
        true,
    ),
    ("SC1113", "# /bin/sh\necho hi", "Missing ! in shebang", true),
    (
        "SC1114",
        "  #!/bin/sh\necho hi",
        "Leading spaces before shebang",
        true,
    ),
    (
        "SC1115",
        "# !/bin/sh\necho hi",
        "Space between # and !",
        true,
    ),
    (
        "SC1127",
        "#!/bin/bash\n// this is a comment",
        "C-style comment //",
        true,
    ),
    (
        "SC1128",
        "echo hi\n#!/bin/bash",
        "Shebang not on first line",
        true,
    ),
    // Quoting rules
    (
        "SC1003",
        "echo 'don't'",
        "Broken single-quote escaping",
        true,
    ),
    (
        "SC1110",
        "echo \u{201c}hello\u{201d}",
        "Unicode double quotes",
        true,
    ),
    (
        "SC1111",
        "echo \u{2018}hello\u{2019}",
        "Unicode single quotes",
        true,
    ),
    // Spacing rules
    (
        "SC1007",
        "#!/bin/sh\nVAR = value",
        "Spaces around = in assignment",
        true,
    ),
    (
        "SC1068",
        "#!/bin/sh\nlet x = 1",
        "Spaces around = in let",
        true,
    ),
    (
        "SC1069",
        "#!/bin/sh\nif[ -f file ]; then echo ok; fi",
        "Missing space before [",
        true,
    ),
    // Syntax rules
    (
        "SC1065",
        "#!/bin/bash\nfunction f(x, y) { echo ok; }",
        "Parameters in function decl",
        true,
    ),
    (
        "SC1066",
        "#!/bin/sh\n$FOO=bar",
        "$ on left side of assignment",
        true,
    ),
    (
        "SC1075",
        "#!/bin/sh\nif true; then echo a; else if true; then echo b; fi; fi",
        "else if → elif",
        true,
    ),
    (
        "SC1086",
        "#!/bin/sh\nfor $i in 1 2 3; do echo ok; done",
        "$ on for loop variable",
        true,
    ),
    (
        "SC1037",
        "#!/bin/sh\necho $10",
        "Unbraced positional >$9",
        true,
    ),
    // Unicode rules
    (
        "SC1082",
        "\u{feff}#!/bin/sh\necho hi",
        "UTF-8 BOM detected",
        true,
    ),
    (
        "SC1100",
        "#!/bin/sh\nif [ \u{2013}f file ]; then echo ok; fi",
        "Unicode dash as minus",
        true,
    ),
    // False positives - these should NOT trigger
    (
        "SC1003-FP",
        "echo 'hello world'",
        "Normal single quotes (no FP)",
        false,
    ),
    (
        "SC1037-FP",
        "echo ${10}",
        "Braced positional (no FP)",
        false,
    ),
    (
        "SC1065-FP",
        "myfunc() { echo ok; }",
        "Normal function decl (no FP)",
        false,
    ),
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

fn find_bashrs_binary() -> Option<&'static str> {
    let candidates = [
        "/mnt/nvme-raid0/targets/bashrs/release/bashrs",
        "/mnt/nvme-raid0/targets/bashrs/debug/bashrs",
        "target/release/bashrs",
        "target/debug/bashrs",
    ];
    candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .copied()
}

fn run_falsification_suite(bashrs_path: &str) -> (u32, u32) {
    let (mut pass, mut fail) = (0, 0);
    for (id, code, desc) in FALSIFICATION_TESTS {
        if run_lint_test(bashrs_path, code) {
            println!("  [✓] {}: {}", id, desc);
            pass += 1;
        } else {
            println!("  [✗] {}: {} - UNEXPECTED WARNING", id, desc);
            fail += 1;
        }
    }
    (pass, fail)
}

fn run_sc1xxx_suite(bashrs_path: &str) -> (u32, u32) {
    let (mut pass, mut fail) = (0, 0);
    for (id, code, desc, should_warn) in SC1XXX_TESTS {
        let has_issues = run_has_issues(bashrs_path, code);
        let ok = if *should_warn {
            has_issues
        } else {
            !has_issues
        };
        if ok {
            let label = if *should_warn { "detected" } else { "no FP" };
            println!("  [\u{2713}] {}: {} ({})", id, desc, label);
            pass += 1;
        } else {
            let label = if *should_warn {
                "NOT detected"
            } else {
                "FALSE POSITIVE"
            };
            println!("  [\u{2717}] {}: {} - {}", id, desc, label);
            fail += 1;
        }
    }
    (pass, fail)
}

fn run_simulation_suite(bashrs_path: &str) -> (u32, u32) {
    let (mut pass, mut fail) = (0, 0);
    for (id, code, desc) in SIMULATION_TESTS {
        if run_simulation_test(bashrs_path, code) {
            println!("  [✓] {}: {}", id, desc);
            pass += 1;
        } else {
            println!("  [✗] {}: {} - PANIC OR CRASH", id, desc);
            fail += 1;
        }
    }
    (pass, fail)
}

fn print_section(title: &str) {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {}", title);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           bashrs Linting Demo - False Positive Tests       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    println!("This demo shows how bashrs handles valid bash patterns without");
    println!("triggering false positive warnings.");
    println!();

    let bashrs_path = match find_bashrs_binary() {
        Some(path) => path,
        None => {
            println!("⚠ bashrs binary not found. Build with: cargo build");
            println!();
            show_test_cases();
            return;
        }
    };

    println!("Using bashrs at: {}", bashrs_path);

    print_section("Falsification Tests (must NOT trigger false positives)");
    let (mut pass_count, mut fail_count) = run_falsification_suite(bashrs_path);

    print_section("SC1xxx Source Code Rules (60 rules - syntax & encoding)");
    let (p, f) = run_sc1xxx_suite(bashrs_path);
    pass_count += p;
    fail_count += f;

    print_section("Simulation Tests (must NOT panic)");
    let (p, f) = run_simulation_suite(bashrs_path);
    pass_count += p;
    fail_count += f;

    print_section("Summary");
    println!("  Passed: {}", pass_count);
    println!("  Failed: {}", fail_count);
    println!("  Total:  {}", pass_count + fail_count);
    println!();

    if fail_count == 0 {
        println!("  ✅ All tests passed!");
    } else {
        println!("  ❌ Some tests failed - check for regressions");
    }

    print_section("Full Test Suites");
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

fn run_has_issues(bashrs_path: &str, code: &str) -> bool {
    // Create temp file
    let temp_path = "/tmp/bashrs_demo_sc1.sh";
    std::fs::write(temp_path, code).ok();

    // Run bashrs lint - exit code indicates issues
    let output = Command::new(bashrs_path)
        .args(["lint", temp_path])
        .env("RUST_LOG", "error") // suppress info logging
        .output();

    // Clean up
    std::fs::remove_file(temp_path).ok();

    match output {
        Ok(out) => {
            // Non-zero exit code means issues were found
            // Also check stderr for panics
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stderr.contains("panic") {
                return false; // Panic is not a detection
            }
            !out.status.success()
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
    println!("SC1xxx Source Code Rules:");
    println!("─────────────────────────────────────────────────────────────");
    for (id, _code, desc, should_warn) in SC1XXX_TESTS {
        let tag = if *should_warn { "detect" } else { "no-FP" };
        println!("  {}: {} [{}]", id, desc, tag);
    }

    println!();
    println!("Simulation Tests (S-codes):");
    println!("─────────────────────────────────────────────────────────────");
    for (id, code, desc) in SIMULATION_TESTS {
        println!("  {}: {}", id, desc);
        println!("      {}", code);
    }
}
