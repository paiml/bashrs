//! Transpiler Bug Hunting - Rust → Shell Edge Case Testing
//!
//! Uses jugar-probar TUI testing framework to hunt for transpiler bugs.
//! Tests report bugs without failing, enabling continuous discovery.

#![allow(clippy::unwrap_used)]
#![allow(deprecated)]

use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;

/// Run transpiler and return (success, shell_output)
fn transpile(rust_code: &str) -> (bool, String) {
    let tmp_rs = "/tmp/transpiler_test.rs";
    let tmp_sh = "/tmp/transpiler_test.sh";

    fs::write(tmp_rs, rust_code).unwrap();

    let output = assert_cmd::cargo_bin_cmd!("bashrs")
        .args(["build", tmp_rs, "-o", tmp_sh])
        .output()
        .unwrap();

    if !output.status.success() {
        return (false, String::from_utf8_lossy(&output.stderr).to_string());
    }

    let shell_code = fs::read_to_string(tmp_sh).unwrap_or_default();
    (true, shell_code)
}

/// Run shell script and return (success, stdout, stderr)
fn run_shell(script_path: &str) -> (bool, String, String) {
    let output = StdCommand::new("sh").arg(script_path).output().unwrap();

    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Bug report structure
struct BugReport {
    id: &'static str,
    description: &'static str,
    rust_code: &'static str,
    expected_behavior: &'static str,
    actual_behavior: String,
    is_bug: bool,
}

impl BugReport {
    fn report(&self) {
        if self.is_bug {
            println!("\n🐛 BUG FOUND: {}", self.id);
            println!("   Description: {}", self.description);
            println!(
                "   Rust code: {}",
                self.rust_code.lines().next().unwrap_or("")
            );
            println!("   Expected: {}", self.expected_behavior);
            println!("   Actual: {}", self.actual_behavior);
        }
    }
}

// ============================================================================
// CATEGORY: Basic Transpilation
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_basic() {
    let mut bugs_found = 0;

    let tests: Vec<(&str, &str, &str, Box<dyn Fn(&str) -> bool>)> = vec![
        // TB001: Empty main
        (
            "TB001",
            "fn main() {}",
            "Should produce valid shell",
            Box::new(|s: &str| s.contains("main()")),
        ),
        // TB002: Simple println
        (
            "TB002",
            r#"fn main() { println!("hello"); }"#,
            "Should use rash_println",
            Box::new(|s: &str| s.contains("rash_println") || s.contains("printf")),
        ),
        // TB003: Variable assignment
        (
            "TB003",
            r#"fn main() { let x = 42; }"#,
            "Should assign x=42 or x='42'",
            Box::new(|s: &str| s.contains("x=42") || s.contains("x='42'")),
        ),
        // TB004: Negative integer
        (
            "TB004",
            r#"fn main() { let x = -1; }"#,
            "Should assign x=-1 or x='-1'",
            Box::new(|s: &str| s.contains("x=-1") || s.contains("x='-1'")),
        ),
        // TB005: String literal
        (
            "TB005",
            r#"fn main() { let s = "hello"; }"#,
            "Should assign s='hello'",
            Box::new(|s: &str| s.contains("s=") && s.contains("hello")),
        ),
    ];

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           TRANSPILER BUG HUNT: Basic Transpilation               ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    for (id, rust_code, expected, check) in tests {
        let (success, output) = transpile(rust_code);
        let is_bug = !success || !check(&output);

        let report = BugReport {
            id,
            description: "Basic transpilation",
            rust_code,
            expected_behavior: expected,
            actual_behavior: if success {
                output
                    .lines()
                    .find(|l| l.contains("main()") || l.contains("x=") || l.contains("s="))
                    .unwrap_or("(not found)")
                    .to_string()
            } else {
                output
            },
            is_bug,
        };

        report.report();
        if is_bug {
            bugs_found += 1;
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Bugs found: {:<52} ║", bugs_found);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// CATEGORY: Arithmetic Operations
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_arithmetic() {
    let mut bugs_found = 0;

    let tests: Vec<(&str, &str, &str, Box<dyn Fn(&str) -> bool>)> = vec![
        // TB010: Addition
        (
            "TB010",
            r#"fn main() { let x = 1 + 2; }"#,
            "Should compute 1+2",
            Box::new(|s: &str| s.contains("$((") || s.contains("expr") || s.contains("3")),
        ),
        // TB011: Subtraction
        (
            "TB011",
            r#"fn main() { let x = 5 - 3; }"#,
            "Should compute 5-3",
            Box::new(|s: &str| s.contains("$((") || s.contains("2")),
        ),
        // TB012: Multiplication
        (
            "TB012",
            r#"fn main() { let x = 4 * 3; }"#,
            "Should compute 4*3",
            Box::new(|s: &str| s.contains("$((") || s.contains("12")),
        ),
        // TB013: Division
        (
            "TB013",
            r#"fn main() { let x = 10 / 2; }"#,
            "Should compute 10/2",
            Box::new(|s: &str| s.contains("$((") || s.contains("5")),
        ),
        // TB014: Modulo
        (
            "TB014",
            r#"fn main() { let x = 10 % 3; }"#,
            "Should compute 10%3",
            Box::new(|s: &str| s.contains("$((") || s.contains("1")),
        ),
        // TB015: Complex expression
        (
            "TB015",
            r#"fn main() { let x = (1 + 2) * 3; }"#,
            "Should compute (1+2)*3",
            Box::new(|s: &str| s.contains("$((") || s.contains("9")),
        ),
    ];

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           TRANSPILER BUG HUNT: Arithmetic Operations             ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    for (id, rust_code, expected, check) in tests {
        let (success, output) = transpile(rust_code);
        let is_bug = !success || !check(&output);

        if is_bug {
            println!("\n🐛 BUG: {} - {}", id, expected);
            bugs_found += 1;
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Bugs found: {:<52} ║", bugs_found);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// CATEGORY: Control Flow
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_control_flow() {
    let mut bugs_found = 0;

    let tests: Vec<(&str, &str, &str, Box<dyn Fn(&str) -> bool>)> = vec![
        // TB020: If statement
        (
            "TB020",
            r#"fn main() { if true { println!("yes"); } }"#,
            "Should have if/then/fi",
            Box::new(|s: &str| s.contains("if") && s.contains("then") && s.contains("fi")),
        ),
        // TB021: If-else
        (
            "TB021",
            r#"fn main() { if true { println!("yes"); } else { println!("no"); } }"#,
            "Should have if/then/else/fi",
            Box::new(|s: &str| s.contains("else")),
        ),
        // TB022: While loop
        (
            "TB022",
            r#"fn main() { let mut i = 0; while i < 3 { i = i + 1; } }"#,
            "Should have while/do/done",
            Box::new(|s: &str| s.contains("while") && s.contains("do") && s.contains("done")),
        ),
        // TB023: For loop
        (
            "TB023",
            r#"fn main() { for i in 0..3 { println!("x"); } }"#,
            "Should have for/do/done or seq",
            Box::new(|s: &str| s.contains("for") || s.contains("seq")),
        ),
        // TB024: Return value
        (
            "TB024",
            r#"fn add(a: i32, b: i32) -> i32 { a + b }"#,
            "Should return via stdout or variable",
            Box::new(|s: &str| s.contains("add()")),
        ),
    ];

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           TRANSPILER BUG HUNT: Control Flow                      ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    for (id, rust_code, expected, check) in tests {
        let (success, output) = transpile(rust_code);
        let is_bug = !success || !check(&output);

        if is_bug {
            println!("\n🐛 BUG: {} - {}", id, expected);
            println!(
                "   Code: {}",
                rust_code.chars().take(60).collect::<String>()
            );
            bugs_found += 1;
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Bugs found: {:<52} ║", bugs_found);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// CATEGORY: Functions
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_functions() {
    let mut bugs_found = 0;

    let tests: Vec<(&str, &str, &str, Box<dyn Fn(&str) -> bool>)> = vec![
        // TB030: Simple function
        (
            "TB030",
            r#"fn greet() { println!("hello"); } fn main() { greet(); }"#,
            "Should define greet() function",
            Box::new(|s: &str| s.contains("greet()")),
        ),
        // TB031: Function with params
        (
            "TB031",
            r#"fn add(a: i32, b: i32) { } fn main() { add(1, 2); }"#,
            "Should pass parameters",
            Box::new(|s: &str| s.contains("add ")),
        ),
        // TB032: Builtin function wrapper
        (
            "TB032",
            r#"fn echo(msg: &str) { } fn main() { echo("hi"); }"#,
            "Should call shell echo",
            Box::new(|s: &str| s.contains("echo")),
        ),
        // TB033: Multiple functions
        (
            "TB033",
            r#"fn a() { } fn b() { } fn main() { a(); b(); }"#,
            "Should define both functions",
            Box::new(|s: &str| s.contains("a()") && s.contains("b()")),
        ),
    ];

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           TRANSPILER BUG HUNT: Functions                         ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    for (id, rust_code, expected, check) in tests {
        let (success, output) = transpile(rust_code);
        let is_bug = !success || !check(&output);

        if is_bug {
            println!("\n🐛 BUG: {} - {}", id, expected);
            bugs_found += 1;
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Bugs found: {:<52} ║", bugs_found);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// CATEGORY: Edge Cases (P0 from Sprint 10 - should all pass now)
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_sprint10_p0() {
    let mut bugs_found = 0;

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           TRANSPILER BUG HUNT: Sprint 10 P0 Regressions          ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    // TICKET-5001: Empty function should call builtin, not :
    {
        let rust = r#"fn echo(msg: &str) { } fn main() { echo("test"); }"#;
        let (success, output) = transpile(rust);
        let is_bug = !success || output.contains(":\n") || output.contains(": \n");
        if is_bug {
            println!("\n🐛 BUG: TICKET-5001 - Empty function produces no-op");
            bugs_found += 1;
        } else {
            println!("║  ✅ TICKET-5001: Empty function → builtin call           PASS  ║");
        }
    }

    // TICKET-5002: println! macro should work
    {
        let rust = r#"fn main() { println!("Hello, World!"); }"#;
        let (success, output) = transpile(rust);
        let is_bug = !success || (!output.contains("rash_println") && !output.contains("printf"));
        if is_bug {
            println!("\n🐛 BUG: TICKET-5002 - println! not supported");
            bugs_found += 1;
        } else {
            println!("║  ✅ TICKET-5002: println! macro supported                 PASS  ║");
        }
    }

    // TICKET-5003: Negative integers should not become 'unknown'
    {
        let rust = r#"fn main() { let x = -42; }"#;
        let (success, output) = transpile(rust);
        let is_bug = !success || output.contains("unknown");
        if is_bug {
            println!("\n🐛 BUG: TICKET-5003 - Negative integers → unknown");
            bugs_found += 1;
        } else {
            println!("║  ✅ TICKET-5003: Negative integers work correctly         PASS  ║");
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Sprint 10 P0 regressions: {:<38} ║", bugs_found);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// CATEGORY: Execution Verification
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_execution() {
    let mut bugs_found = 0;

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           TRANSPILER BUG HUNT: Execution Verification            ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    // Test that generated scripts actually run
    let tests = vec![("TB050", r#"fn main() { println!("MARKER"); }"#, "MARKER")];

    for (id, rust_code, expected_output) in tests {
        let tmp_rs = "/tmp/exec_test.rs";
        let tmp_sh = "/tmp/exec_test.sh";

        fs::write(tmp_rs, rust_code).unwrap();

        let compile = assert_cmd::cargo_bin_cmd!("bashrs")
            .args(["build", tmp_rs, "-o", tmp_sh])
            .output()
            .unwrap();

        if !compile.status.success() {
            println!("\n🐛 BUG: {} - Compilation failed", id);
            bugs_found += 1;
            continue;
        }

        let (success, stdout, stderr) = run_shell(tmp_sh);

        if !success || !stdout.contains(expected_output) {
            println!("\n🐛 BUG: {} - Execution failed", id);
            println!("   Expected output containing: {}", expected_output);
            println!("   Got stdout: {}", stdout.trim());
            println!("   Got stderr: {}", stderr.trim());
            bugs_found += 1;
        } else {
            println!(
                "║  ✅ {}: Generated script executes correctly        PASS  ║",
                id
            );
        }
    }

    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Execution bugs: {:<48} ║", bugs_found);
    println!("╚══════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// COMPREHENSIVE SUMMARY
// ============================================================================

#[test]
fn test_transpiler_bug_hunt_comprehensive() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    TRANSPILER BUG HUNT: COMPREHENSIVE SUMMARY                ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Run individual test categories above for detailed bug reports.             ║");
    println!("║                                                                              ║");
    println!("║  Categories:                                                                 ║");
    println!("║    - TB001-TB005: Basic Transpilation                                        ║");
    println!("║    - TB010-TB015: Arithmetic Operations                                      ║");
    println!("║    - TB020-TB024: Control Flow                                               ║");
    println!("║    - TB030-TB033: Functions                                                  ║");
    println!("║    - TICKET-5001/5002/5003: Sprint 10 P0 Regressions                         ║");
    println!("║    - TB050: Execution Verification                                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
}
