# Rash Transpiler Testing Infrastructure: Final Specification

**Version:** 1.2 (Production-Ready)  
**Status:** Incorporates formal methods, semantic equivalence, and static analysis  
**Last Updated:** 2025-10-03

---

## Changelog

**v1.2 (2025-10-03):**
- Added Section 1.6: Negative case testing for CLI error handling
- Added Section 9: Static analysis of transpiler codebase
- Integrated CODEOWNERS file for critical code protection
- Enhanced CI/CD with static analysis gates
- Added error message quality metrics

**v1.1 (2025-10-03):**
- Added formal verification considerations (Section 2)
- Enhanced semantic equivalence testing beyond arithmetic
- Added corpus-based fuzzing with real-world Rust code
- Integrated cryptographic hash verification for determinism

**v1.0 (2025-10-02):**
- Initial comprehensive specification

---

## Executive Summary

This document specifies a production-grade, multi-layered testing infrastructure for the Rash transpiler, which converts a safe subset of Rust to POSIX-compliant shell scripts. The architecture combines six complementary validation strategies—unit, integration, execution, property-based, fuzzing, and static analysis—with formal verification considerations to achieve zero shell injection vulnerabilities, deterministic output, semantic equivalence with source Rust programs, and graceful degradation for unsupported features.

**Core Requirements:**
- **Safety:** 100% shell injection immunity (property-tested + formally verified)
- **Correctness:** Behavioral equivalence with `rustc` execution (differential testing)
- **Compliance:** All output passes `shellcheck --shell=sh` with zero warnings
- **Determinism:** Identical output across runs (BLAKE3 hash verification)
- **Usability:** Clear error messages for unsupported features (negative testing)
- **Performance:** <100ms transpilation for 90th percentile programs

---

## 1. Architecture: Six-Layer Testing Pyramid

The testing strategy implements a pyramid structure where test count and execution speed are inversely proportional to test scope and fidelity. This approach, validated in compiler testing research [^1], ensures rapid feedback during development while maintaining comprehensive validation.

```
           /\
          /  \       Layer 6: Static Analysis (continuous, pre-merge)
         /    \      Layer 5: Fuzzing (continuous, coverage-guided)
        /      \     Layer 4: Property-Based (1000+ cases, invariant validation)
       /        \    Layer 3: Execution (30+ multi-shell, equivalence testing)
      /          \   Layer 2: Integration (50+ snapshots, regression prevention)
     /            \  Layer 1: Unit Tests (100+, <2s total, AST/IR validation)
    /______________\
```

### 1.1 Layer 1: Unit Tests - Foundational Correctness

**Purpose:** Validate individual components (parser, IR lowering, codegen primitives) in isolation with millisecond-level feedback.

**Methodology:** Traditional unit testing with edge case enumeration and boundary condition validation. Coverage target: >95% line coverage for all non-integration code.

```rust
// tests/unit/parser_tests.rs
use rash::parser::{parse, RashAst, Type};
use rash::errors::ParseError;

#[test]
fn parse_function_with_references() {
    let input = r#"
        fn greet(name: &str) -> String {
            format!("Hello, {}", name)
        }
    "#;
    
    let ast = parse(input).expect("parse failed");
    
    assert_eq!(ast.functions.len(), 1);
    let func = &ast.functions[0];
    assert_eq!(func.name, "greet");
    assert_eq!(func.params[0].ty, Type::StrRef);
    assert_eq!(func.return_ty, Type::String);
}

#[test]
fn reject_unsupported_async_syntax() {
    let input = "async fn fetch_data() -> Result<String, Error> {}";
    
    match parse(input) {
        Err(ParseError::UnsupportedFeature { feature, span }) => {
            assert_eq!(feature, "async functions");
            assert!(span.start > 0); // Verify span tracking
        }
        _ => panic!("Should reject async syntax"),
    }
}

#[test]
fn reject_trait_definitions() {
    let input = "trait Drawable { fn draw(&self); }";
    
    match parse(input) {
        Err(ParseError::UnsupportedFeature { feature, .. }) => {
            assert_eq!(feature, "trait definitions");
        }
        _ => panic!("Should reject traits"),
    }
}

#[test]
fn variable_name_mangling_avoids_shell_keywords() {
    use rash::codegen::mangle_identifier;
    
    // Shell reserved words must be mangled
    assert_eq!(mangle_identifier("if"), "_rash_if");
    assert_eq!(mangle_identifier("while"), "_rash_while");
    assert_eq!(mangle_identifier("case"), "_rash_case");
    assert_eq!(mangle_identifier("do"), "_rash_do");
    
    // POSIX special variables must be mangled
    assert_eq!(mangle_identifier("IFS"), "_rash_IFS");
    assert_eq!(mangle_identifier("PATH"), "_rash_PATH");
    
    // Safe identifiers pass through
    assert_eq!(mangle_identifier("user_count"), "user_count");
    assert_eq!(mangle_identifier("MAX_SIZE"), "MAX_SIZE");
}

// Property test: All valid Rust subset programs parse
proptest! {
    #[test]
    fn all_valid_integer_literals_parse(n in any::<i64>()) {
        let code = format!("fn main() {{ let x = {}; }}", n);
        assert!(parse(&code).is_ok());
    }
    
    #[test]
    fn valid_identifiers_parse(
        name in "[a-z_][a-z0-9_]{0,30}"
    ) {
        // Ensure not a keyword
        if !RUST_KEYWORDS.contains(&name.as_str()) {
            let code = format!("fn {}() {{}}", name);
            assert!(parse(&code).is_ok());
        }
    }
}
```

**Performance Target:** <100ms for complete unit test suite (enables sub-second TDD cycles).

---

### 1.2 Layer 2: Integration Tests - Snapshot-Based Regression Prevention

**Purpose:** Verify end-to-end transpilation produces expected shell scripts. Detect unintended changes in codegen output.

**Methodology:** Snapshot testing with `insta` [^2]. Each test fixture (`.rs` file) generates a golden file (`.sh.snap`) that is version-controlled and reviewed during changes.

```rust
// tests/integration/transpile_snapshots.rs
use insta::{assert_snapshot, glob};
use rash::transpile;

#[test]
fn test_all_fixtures() {
    glob!("fixtures/*.rs", |path| {
        let rust_code = std::fs::read_to_string(path).unwrap();
        let shell_script = transpile(&rust_code).unwrap();
        
        let test_name = path.file_stem().unwrap().to_str().unwrap();
        assert_snapshot!(test_name, shell_script);
    });
}

#[test]
fn verify_deterministic_output() {
    let rust_code = include_str!("fixtures/complex_control_flow.rs");
    
    // Generate 10 times, compute cryptographic hashes
    let outputs: Vec<_> = (0..10)
        .map(|_| transpile(rust_code).unwrap())
        .collect();
    
    let hashes: Vec<_> = outputs.iter()
        .map(|s| blake3::hash(s.as_bytes()))
        .collect();
    
    // All hashes must be identical
    assert!(hashes.windows(2).all(|w| w[0] == w[1]),
        "Non-deterministic codegen detected");
}

#[test]
fn snapshot_includes_version_and_flags() {
    let code = "fn main() { println!(\"test\"); }";
    let output = transpile(code).unwrap();
    
    // Header must include version for debugging
    assert!(output.starts_with("#!/bin/sh\n"));
    assert!(output.contains("# Generated by Rash"));
    
    // Must set strict error handling
    assert!(output.contains("set -euf"));
}
```

**Example Snapshot:**
```sh
# tests/snapshots/string_interpolation.sh.snap
#!/bin/sh
# Generated by Rash v0.1.0 (commit: a1b2c3d)
# Source: tests/fixtures/string_interpolation.rs
# DO NOT EDIT - This file is auto-generated

set -euf

main() {
    local name="World"
    local message
    message="Hello, ${name}!"
    printf '%s\n' "$message"
}

main "$@"
```

**Workflow:**
```bash
# Update snapshots after intentional changes
$ cargo insta review

# Fail CI if snapshots drift
$ cargo insta test --check --unreferenced=reject
```

---

### 1.3 Layer 3: Execution Tests - Multi-Shell Equivalence Validation

**Purpose:** Verify generated scripts execute correctly across POSIX shells and produce output semantically equivalent to native Rust execution.

**Methodology:** Differential testing against `rustc` [^3]. For each test case, compile and execute the Rust source, then execute the transpiled shell script in multiple shells. Assert stdout, stderr, and exit codes match.

```rust
// tests/execution/multi_shell_tests.rs
use std::process::{Command, Output};
use tempfile::NamedTempFile;

const POSIX_SHELLS: &[&str] = &["dash", "bash", "ash", "ksh"];

struct ExecutionTest {
    name: &'static str,
    rust_code: &'static str,
    expected_stdout: &'static str,
    expected_stderr: &'static str,
    expected_exit_code: i32,
}

impl ExecutionTest {
    fn run(&self) {
        // Baseline: Native Rust execution
        let rust_output = self.compile_and_run_rust();
        assert_eq!(rust_output.stdout, self.expected_stdout,
            "Test '{}': Rust stdout mismatch", self.name);
        assert_eq!(rust_output.exit_code, self.expected_exit_code,
            "Test '{}': Rust exit code mismatch", self.name);
        
        // Transpile to shell
        let shell_script = transpile(self.rust_code)
            .expect(&format!("Transpilation failed for '{}'", self.name));
        
        // Test in each POSIX shell
        for shell in POSIX_SHELLS {
            if !is_shell_available(shell) {
                eprintln!("Skipping {}: not installed", shell);
                continue;
            }
            
            let shell_output = self.run_shell_script(&shell_script, shell);
            
            assert_eq!(shell_output.stdout, self.expected_stdout,
                "Test '{}' failed in {}: stdout mismatch", self.name, shell);
            assert_eq!(shell_output.exit_code, self.expected_exit_code,
                "Test '{}' failed in {}: exit code mismatch", self.name, shell);
        }
    }
    
    fn compile_and_run_rust(&self) -> TestOutput {
        let mut src_file = NamedTempFile::new().unwrap();
        write!(src_file, "{}", self.rust_code).unwrap();
        
        let exe_path = "/tmp/rash_test_binary";
        
        // Compile with rustc
        let compile_status = Command::new("rustc")
            .args(&[
                src_file.path().to_str().unwrap(),
                "-o", exe_path,
                "--edition", "2021"
            ])
            .status()
            .expect("Failed to invoke rustc");
        
        assert!(compile_status.success(), "Rust compilation failed");
        
        // Execute
        let output = Command::new(exe_path)
            .output()
            .expect("Failed to execute compiled binary");
        
        TestOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
    
    fn run_shell_script(&self, script: &str, shell: &str) -> TestOutput {
        let mut script_file = NamedTempFile::new().unwrap();
        write!(script_file, "{}", script).unwrap();
        
        let output = Command::new(shell)
            .arg(script_file.path())
            .output()
            .expect(&format!("Failed to execute {} script", shell));
        
        TestOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}

#[test]
fn test_arithmetic_evaluation() {
    ExecutionTest {
        name: "arithmetic_evaluation",
        rust_code: r#"
            fn main() {
                let x = (10 + 5) * 2 - 8 / 2;
                println!("{}", x);
            }
        "#,
        expected_stdout: "26\n",
        expected_stderr: "",
        expected_exit_code: 0,
    }.run();
}

#[test]
fn test_shell_injection_prevention() {
    ExecutionTest {
        name: "shell_injection_prevention",
        rust_code: r#"
            fn main() {
                let malicious = "'; rm -rf /; echo '";
                println!("{}", malicious);
            }
        "#,
        expected_stdout: "'; rm -rf /; echo '\n",
        expected_stderr: "",
        expected_exit_code: 0,
    }.run();
}

#[test]
fn test_exit_code_propagation() {
    ExecutionTest {
        name: "exit_code_propagation",
        rust_code: r#"
            use std::process::exit;
            fn main() {
                exit(42);
            }
        "#,
        expected_stdout: "",
        expected_stderr: "",
        expected_exit_code: 42,
    }.run();
}
```

**Docker-Based Isolation Testing:**

For comprehensive POSIX compliance verification, execute tests in isolated containers with minimal POSIX implementations (e.g., Alpine's BusyBox).

```rust
#[test]
#[ignore] // Slow, run in CI only
fn test_alpine_busybox_compatibility() {
    let rust_code = r#"
        fn main() {
            for i in 1..=5 {
                println!("Line {}", i);
            }
        }
    "#;
    
    let shell_script = transpile(rust_code).unwrap();
    
    let output = Command::new("docker")
        .args(&[
            "run", "--rm", "-i",
            "alpine:3.19",
            "sh", "-c", &shell_script
        ])
        .output()
        .expect("Docker execution failed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n");
}
```

---

### 1.4 Layer 4: Property-Based Testing - Invariant Validation

**Purpose:** Discover edge cases through automated input generation. Verify critical safety and correctness properties hold for all inputs.

**Methodology:** Property-based testing with `proptest` [^4]. Define properties as invariants that must hold for all generated inputs. Focus on security-critical properties (injection immunity) and semantic equivalence.

```rust
// tests/property/safety_properties.rs
use proptest::prelude::*;
use rash::{transpile, safety};

proptest! {
    /// CRITICAL PROPERTY: No shell metacharacters can trigger code execution
    #[test]
    fn no_command_injection_via_string_interpolation(
        user_input in r#"[\x20-\x7E&&[^"]]{0,100}"# // Printable ASCII except quotes
    ) {
        let rust_code = format!(r#"
            fn main() {{
                let data = "{}";
                println!("{{}}", data);
            }}
        "#, user_input.replace('\\', "\\\\").replace('"', "\\\""));
        
        let shell_script = transpile(&rust_code)
            .expect("Transpilation failed");
        
        // Static analysis: All variable references must be quoted
        let unquoted_vars = safety::find_unquoted_variables(&shell_script);
        prop_assert!(unquoted_vars.is_empty(),
            "Found unquoted variables: {:?}", unquoted_vars);
        
        // Dynamic validation: Execute and verify output
        let output = run_in_dash(&shell_script);
        prop_assert_eq!(output.stdout.trim(), user_input);
    }
    
    /// SEMANTIC PROPERTY: Arithmetic operations preserve Rust semantics
    #[test]
    fn arithmetic_preserves_semantics(
        a in -1000i64..1000,
        b in 1i64..1000, // Avoid division by zero
        c in -1000i64..1000,
    ) {
        let rust_code = format!(r#"
            fn main() {{
                let result = ({} + {}) * {} / {};
                println!("{{}}", result);
            }}
        "#, a, b, c, b);
        
        // Baseline: Rust execution
        let rust_result = compile_and_run_rust(&rust_code)
            .expect("Rust compilation failed");
        
        // Test: Shell execution
        let shell_script = transpile(&rust_code).unwrap();
        let shell_result = run_in_dash(&shell_script);
        
        prop_assert_eq!(rust_result.stdout, shell_result.stdout,
            "Arithmetic divergence: ({} + {}) * {} / {}", a, b, c, b);
    }
    
    /// COMPLIANCE PROPERTY: Output must be POSIX-compliant
    #[test]
    fn output_is_posix_compliant(
        program in arb_valid_rash_program()
    ) {
        let shell_script = transpile(&program)
            .expect("Transpilation failed");
        
        // No bashisms allowed
        prop_assert!(!contains_bashisms(&shell_script),
            "Bashism detected in output");
        
        // Must pass shellcheck in POSIX mode
        let shellcheck_result = run_shellcheck_posix(&shell_script);
        prop_assert!(shellcheck_result.is_ok(),
            "ShellCheck failed: {:?}", shellcheck_result);
        
        // Must execute in dash (strict POSIX shell)
        let dash_result = run_in_dash(&shell_script);
        prop_assert!(dash_result.exit_code == 0 || 
                     program_expects_failure(&program),
            "Failed to execute in dash");
    }
}

// Custom strategy: Generate structurally valid Rash programs
fn arb_valid_rash_program() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_simple_expression().boxed(),
        arb_function_call().boxed(),
        arb_control_flow().boxed(),
        arb_string_operations().boxed(),
    ]
}

fn arb_simple_expression() -> impl Strategy<Value = String> {
    (any::<i64>(), "[a-z_][a-z0-9_]{0,10}").prop_map(|(value, name)| {
        format!(r#"
            fn main() {{
                let {} = {};
                println!("{{}}", {});
            }}
        "#, name, value, name)
    })
}

fn arb_control_flow() -> impl Strategy<Value = String> {
    (any::<i64>(), any::<i64>()).prop_map(|(threshold, value)| {
        format!(r#"
            fn main() {{
                let x = {};
                if x > {} {{
                    println!("high");
                }} else {{
                    println!("low");
                }}
            }}
        "#, value, threshold)
    })
}
```

**Enhanced Semantic Equivalence Testing:**

Following the review feedback, we expand property testing to cover semantic equivalence beyond arithmetic:

```rust
proptest! {
    /// SEMANTIC PROPERTY: String concatenation matches Rust
    #[test]
    fn string_concat_preserves_semantics(
        s1 in "[a-z]{1,20}",
        s2 in "[a-z]{1,20}",
    ) {
        let rust_code = format!(r#"
            fn main() {{
                let a = "{}";
                let b = "{}";
                let c = format!("{{}}{{}}", a, b);
                println!("{{}}", c);
            }}
        "#, s1, s2);
        
        let rust_output = compile_and_run_rust(&rust_code).unwrap();
        let shell_script = transpile(&rust_code).unwrap();
        let shell_output = run_in_dash(&shell_script);
        
        prop_assert_eq!(rust_output.stdout, shell_output.stdout);
    }
    
    /// SEMANTIC PROPERTY: Loop iteration matches Rust
    #[test]
    fn loop_iteration_preserves_semantics(
        start in 0i64..10,
        end in 0i64..20,
    ) {
        // Only test if valid range
        if start <= end {
            let rust_code = format!(r#"
                fn main() {{
                    for i in {}..{} {{
                        println!("{{}}", i);
                    }}
                }}
            "#, start, end);
            
            let rust_output = compile_and_run_rust(&rust_code).unwrap();
            let shell_script = transpile(&rust_code).unwrap();
            let shell_output = run_in_dash(&shell_script);
            
            prop_assert_eq!(rust_output.stdout, shell_output.stdout);
        }
    }
    
    /// SEMANTIC PROPERTY: Error handling matches Rust
    #[test]
    fn error_handling_preserves_semantics(
        should_fail in prop::bool::ANY,
        error_code in 1i32..100,
    ) {
        let rust_code = format!(r#"
            use std::process::exit;
            fn main() {{
                if {} {{
                    eprintln!("Error occurred");
                    exit({});
                }}
                println!("Success");
            }}
        "#, should_fail, error_code);
        
        let rust_output = compile_and_run_rust(&rust_code).unwrap();
        let shell_script = transpile(&rust_code).unwrap();
        let shell_output = run_in_dash(&shell_script);
        
        prop_assert_eq!(rust_output.exit_code, shell_output.exit_code);
        prop_assert_eq!(rust_output.stderr, shell_output.stderr);
    }
}
```

---

### 1.5 Layer 5: Fuzzing - Automated Edge Case Discovery

**Purpose:** Uncover deep bugs, panics, and correctness violations through coverage-guided mutation of inputs.

**Methodology:** Differential fuzzing [^5] using `cargo-fuzz` and optional AFL.rs. Generate structurally valid Rust ASTs, transpile, and compare execution against `rustc`.

```rust
// fuzz/fuzz_targets/transpile_robustness.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use rash::transpile;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        // Transpilation must never panic
        let _ = transpile(source);
    }
});

// fuzz/fuzz_targets/differential_execution.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use rash::{transpile, test_utils};
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzProgram {
    functions: Vec<FuzzFunction>,
    main_body: Vec<Statement>,
}

impl FuzzProgram {
    fn to_rust_code(&self) -> String {
        let mut code = String::new();
        
        // Generate function definitions
        for func in &self.functions {
            code.push_str(&func.to_rust());
        }
        
        // Generate main
        code.push_str("fn main() {\n");
        for stmt in &self.main_body {
            code.push_str(&format!("    {}\n", stmt.to_rust()));
        }
        code.push_str("}\n");
        
        code
    }
}

fuzz_target!(|program: FuzzProgram| {
    let rust_code = program.to_rust_code();
    
    // Only test if valid Rust (passes parsing)
    if let Ok(_) = syn::parse_file(&rust_code) {
        // Try to compile with rustc
        if let Ok(rust_output) = test_utils::compile_and_run(&rust_code, Duration::from_secs(1)) {
            // Transpile to shell
            if let Ok(shell_script) = transpile(&rust_code) {
                // Execute in dash
                if let Ok(shell_output) = test_utils::run_shell("dash", &shell_script, Duration::from_secs(1)) {
                    // CRITICAL: Outputs must match
                    assert_eq!(
                        rust_output.stdout, shell_output.stdout,
                        "Differential bug found:\n{}",
                        rust_code
                    );
                    assert_eq!(
                        rust_output.exit_code, shell_output.exit_code,
                        "Exit code mismatch:\n{}",
                        rust_code
                    );
                }
            }
        }
    }
});
```

**Corpus-Based Fuzzing (Addressing Review Feedback):**

To improve bug discovery beyond random generation, seed the fuzzer with real-world Rust code:

```bash
# Create corpus from real Rust projects
fuzz/corpus/differential_execution/
├── 001_cargo_command.rs          # From cargo source
├── 002_ripgrep_search.rs         # From ripgrep source  
├── 003_tokio_runtime.rs          # From tokio (simplified)
├── 004_serde_derive.rs           # From serde
└── ...

# Run fuzzer with corpus
$ cargo fuzz run differential_execution \
    --corpus=fuzz/corpus/differential_execution \
    -- -max_total_time=3600 \
       -max_len=8192 \
       -dict=fuzz/rust_keywords.dict
```

**Fuzzing Dictionary (Rust-specific tokens):**

```
# fuzz/rust_keywords.dict
"fn"
"let"
"mut"
"if"
"else"
"for"
"while"
"loop"
"match"
"return"
"println!"
"format!"
"Vec"
"String"
"Option"
"Result"
```

**Continuous Fuzzing Infrastructure:**

```yaml
# .github/workflows/continuous-fuzz.yml
name: Continuous Fuzzing

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [transpile_robustness, differential_execution]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust nightly
        run: rustup toolchain install nightly
      
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      
      - name: Run fuzzer
        run: |
          cargo +nightly fuzz run ${{ matrix.target }} \
            -- -max_total_time=21600 \  # 6 hours
               -rss_limit_mb=4096 \
               -timeout=60
      
      - name: Upload artifacts on crash
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-crash-${{ matrix.target }}
          path: fuzz/artifacts/${{ matrix.target }}/
```

---

### 1.6 Layer 6: Negative Testing - CLI Error Handling Validation

**Purpose:** Verify the transpiler CLI provides clear, actionable error messages when encountering unsupported Rust features or invalid input. This addresses the critical user experience aspect: graceful degradation when the tool cannot transpile a program.

**Methodology:** Integration testing of the CLI binary itself, asserting that:
1. Unsupported features produce non-zero exit codes
2. Error messages are clear, actionable, and include source location
3. Error output follows consistent formatting
4. Suggestions for alternatives are provided when applicable

**Rationale:** While unit tests verify the parser rejects unsupported syntax, negative testing validates the end-to-end user experience. Poor error messages are a primary cause of tool abandonment in developer tooling [^9].

```rust
// tests/execution/cli_error_handling_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;

/// Test that unsupported features produce clear error messages
#[test]
fn test_async_syntax_error_message() {
    let rust_code = r#"
        async fn fetch_data() -> Result<String, Error> {
            Ok("data".to_string())
        }
        
        fn main() {
            let data = fetch_data().await;
        }
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: unsupported feature: async functions"))
        .stderr(predicate::str::contains("async fn fetch_data"))
        .stderr(predicate::str::contains("note: Rash does not support async/await"))
        .stderr(predicate::str::contains("help: consider using synchronous alternatives"));
}

#[test]
fn test_trait_definition_error_message() {
    let rust_code = r#"
        trait Drawable {
            fn draw(&self);
        }
        
        fn main() {}
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: unsupported feature: trait definitions"))
        .stderr(predicate::str::contains("trait Drawable"))
        .stderr(predicate::str::contains("note: Rash supports only a limited subset of Rust"));
}

#[test]
fn test_generic_type_error_message() {
    let rust_code = r#"
        fn sort<T: Ord>(items: Vec<T>) -> Vec<T> {
            items.sort();
            items
        }
        
        fn main() {}
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: unsupported feature: generic type parameters"))
        .stderr(predicate::str::contains("fn sort<T: Ord>"))
        .stderr(predicate::str::contains("note: supported generic types: Vec<T>, Option<T>, Result<T, E>"));
}

#[test]
fn test_lifetime_annotation_error_message() {
    let rust_code = r#"
        fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
            if x.len() > y.len() { x } else { y }
        }
        
        fn main() {}
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: unsupported feature: explicit lifetime annotations"))
        .stderr(predicate::str::contains("<'a>"))
        .stderr(predicate::str::contains("note: Rash uses simple lifetime elision rules"));
}

#[test]
fn test_unsafe_block_error_message() {
    let rust_code = r#"
        fn main() {
            unsafe {
                let ptr = std::ptr::null::<i32>();
            }
        }
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: unsupported feature: unsafe code"))
        .stderr(predicate::str::contains("unsafe {"))
        .stderr(predicate::str::contains("note: Rash prioritizes safety and does not support unsafe blocks"));
}

#[test]
fn test_macro_definition_error_message() {
    let rust_code = r#"
        macro_rules! my_macro {
            () => { println!("hello"); }
        }
        
        fn main() {
            my_macro!();
        }
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: unsupported feature: macro definitions"))
        .stderr(predicate::str::contains("macro_rules! my_macro"));
}

/// Test that syntax errors provide helpful diagnostics
#[test]
fn test_syntax_error_with_caret_diagnostic() {
    let rust_code = r#"
        fn main() {
            let x = 10 +;
        }
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: expected expression"))
        .stderr(predicate::str::contains("let x = 10 +;"))
        .stderr(predicate::str::contains("^")); // Caret pointing to error
}

/// Test that multiple errors are reported
#[test]
fn test_multiple_errors_reported() {
    let rust_code = r#"
        async fn first() {}
        
        trait MyTrait {}
        
        fn main() {
            unsafe { }
        }
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg(file.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("async functions"))
        .stderr(predicate::str::contains("trait definitions"))
        .stderr(predicate::str::contains("unsafe code"))
        .stderr(predicate::str::contains("error: aborting due to 3 previous errors"));
}

/// Test that --help provides clear usage information
#[test]
fn test_help_flag() {
    Command::cargo_bin("rash")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("rash [OPTIONS] <INPUT>"))
        .stdout(predicate::str::contains("OPTIONS:"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--check"))
        .stdout(predicate::str::contains("EXAMPLES:"));
}

/// Test that --version provides version information
#[test]
fn test_version_flag() {
    Command::cargo_bin("rash")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rash"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Test that missing input file produces clear error
#[test]
fn test_missing_input_file_error() {
    Command::cargo_bin("rash")
        .unwrap()
        .arg("nonexistent_file.rs")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: file not found"))
        .stderr(predicate::str::contains("nonexistent_file.rs"));
}

/// Test that --check flag validates without writing output
#[test]
fn test_check_flag_validates_only() {
    let rust_code = r#"
        fn main() {
            println!("Hello, world!");
        }
    "#;
    
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    
    Command::cargo_bin("rash")
        .unwrap()
        .arg("--check")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Transpilation successful"))
        .stdout(predicate::str::is_empty().not()); // Output not generated
}
```

**Error Message Quality Metrics:**

To ensure error messages meet quality standards, we define measurable criteria:

```rust
// tests/execution/error_message_quality_tests.rs

#[derive(Debug)]
struct ErrorMessageQuality {
    has_error_prefix: bool,           // "error:" prefix present
    has_source_location: bool,        // Line/column information
    has_code_snippet: bool,           // Shows problematic code
    has_caret_indicator: bool,        // ^ pointing to issue
    has_explanation: bool,            // "note:" with context
    has_suggestion: bool,             // "help:" with alternative
    message_length: usize,
}

impl ErrorMessageQuality {
    fn from_stderr(stderr: &str) -> Self {
        Self {
            has_error_prefix: stderr.contains("error:"),
            has_source_location: stderr.contains(':') && stderr.chars().filter(|c| c.is_numeric()).count() > 0,
            has_code_snippet: stderr.lines().any(|l| !l.starts_with("error:") && !l.starts_with("note:") && !l.starts_with("help:")),
            has_caret_indicator: stderr.contains('^'),
            has_explanation: stderr.contains("note:"),
            has_suggestion: stderr.contains("help:"),
            message_length: stderr.len(),
        }
    }
    
    fn score(&self) -> f32 {
        let mut score = 0.0;
        if self.has_error_prefix { score += 1.0; }
        if self.has_source_location { score += 1.5; }
        if self.has_code_snippet { score += 1.5; }
        if self.has_caret_indicator { score += 1.0; }
        if self.has_explanation { score += 2.0; }
        if self.has_suggestion { score += 2.0; }
        
        // Penalize excessively long messages (>500 chars)
        if self.message_length > 500 {
            score -= 1.0;
        }
        
        score / 9.0 // Normalize to 0-1
    }
}

#[test]
fn test_error_message_quality_meets_threshold() {
    let unsupported_features = vec![
        ("async fn", "async fn test() {}"),
        ("trait", "trait Test {}"),
        ("impl", "impl Test {}"),
        ("unsafe", "unsafe { }"),
    ];
    
    for (feature, code) in unsupported_features {
        let full_code = format!("fn main() {{ {} }}", code);
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(full_code.as_bytes()).unwrap();
        
        let output = Command::cargo_bin("rash")
            .unwrap()
            .arg(file.path())
            .output()
            .unwrap();
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        let quality = ErrorMessageQuality::from_stderr(&stderr);
        
        assert!(
            quality.score() >= 0.7,
            "Error message quality too low for '{}': score={:.2}, quality={:?}",
            feature,
            quality.score(),
            quality
        );
    }
}
```

**Success Criteria for Negative Testing:**

- [ ] All unsupported features produce exit code 1
- [ ] Error messages include source location (file:line:column)
- [ ] Error messages show code snippet with caret indicator
- [ ] Error messages provide explanation ("note:") and suggestion ("help:")
- [ ] Error message quality score ≥0.7 for all test cases
- [ ] Multiple errors reported (up to 10), not just first error
- [ ] `--help` and `--version` flags work correctly
- [ ] File not found errors are clear and actionable

---

## 2. Formal Verification Considerations

**Addressing Review Feedback:** While the testing suite is comprehensive, certain critical properties benefit from formal verification to provide mathematical guarantees beyond testing.

### 2.1 Properties Amenable to Formal Verification

**2.1.1 Shell Injection Immunity**

The most critical safety property—that no user input can trigger code execution—can be formally verified using symbolic execution or abstract interpretation [^6].

**Approach:** Model the transpiler's string escaping logic as a state machine and prove that all paths through the machine produce quoted output.

```rust
// Formal specification (pseudocode)
forall input: String,
  let output = escape_for_shell(input);
  assert!(is_safely_quoted(output));
  assert!(execute_shell(output) == literal_string(input));
```

**Tool Candidates:**
- **KLEE:** Symbolic execution engine for C/C++/Rust (via LLVM)
- **Prusti:** Rust verifier based on Viper (supports pre/post-conditions)
- **RustHorn:** CHC-based verification for Rust

**Example with Prusti annotations:**

```rust
use prusti_contracts::*;

#[pure]
#[ensures(result.starts_with('"') && result.ends_with('"'))]
fn escape_string(s: &str) -> String {
    let mut escaped = String::from('"');
    for c in s.chars() {
        match c {
            '"' => escaped.push_str(r#"\""#),
            '\\' => escaped.push_str(r"\\"),
            '$' => escaped.push_str(r"\$"),
            '`' => escaped.push_str(r"\`"),
            _ => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}

#[requires(!s.is_empty())]
#[ensures(execute_shell(&result) == s)]
fn transpile_println(s: &str) -> String {
    format!("printf '%s\\n' {}", escape_string(s))
}
```

**2.1.2 AST → IR Transformation Correctness**

The lowering from AST to intermediate representation (IR) must preserve program semantics. This can be verified using bisimulation [^7].

**Approach:** Define operational semantics for both AST and IR, then prove that for every AST execution trace, there exists a corresponding IR trace with identical observable behavior.

```
∀ ast : RashAst,
  let ir = lower_to_ir(ast);
  ∀ trace_ast ∈ traces(ast),
    ∃ trace_ir ∈ traces(ir),
      observable(trace_ast) = observable(trace_ir)
```

**Tool Candidates:**
- **Coq:** Interactive theorem prover (used in CompCert C compiler verification)
- **Isabelle/HOL:** Higher-order logic prover
- **Lean 4:** Dependent type theory prover with Rust-like syntax

**2.1.3 POSIX Compliance**

Verify that generated shell code only uses constructs from the POSIX.1-2017 standard.

**Approach:** Define a formal grammar for POSIX shell, then prove that all codegen output parses under this grammar.

```rust
// Codegen produces AST nodes that are POSIX-compliant by construction
enum PosixCommand {
    SimpleCommand { args: Vec<PosixWord> },
    Pipeline { commands: Vec<PosixCommand> },
    IfStatement { condition: PosixTest, then_branch: Box<PosixCommand>, else_branch: Option<Box<PosixCommand>> },
    // ... only POSIX constructs
}

// Illegal construct at type level
// BashCommand::ProcessSubstitution { ... }  // Does not exist
```

### 2.2 Verification Roadmap

**Phase 1 (v0.2):** Add Prusti contracts to safety-critical functions (string escaping, variable quoting).

**Phase 2 (v0.3):** Formally verify AST→IR lowering preserves control flow structure using Coq.

**Phase 3 (v1.0):** Full verification of core transpiler pipeline, generating machine-checkable proof.

---

## 3. Mutation Testing Strategy

Mutation testing provides a measure of test suite quality by making small modifications to the source code and ensuring the test suite detects the change [^8].

### 3.1 Configuration

```toml
# mutants.toml
[test]
timeout = "60s"
filter = "not(integration or slow)"

[[mutants]]
name = "safety-critical"
files = [
    "src/codegen/string_escaping.rs",
    "src/codegen/variable_quoting.rs",
    "src/safety/injection_check.rs",
]
# Zero tolerance for missed mutants
minimum_tested = 100

[[mutants]]
name = "core-logic"
files = [
    "src/parser.rs",
    "src/ir.rs",
    "src/codegen.rs",
]
minimum_tested = 90

[[mutants]]
name = "utilities"
files = [
    "src/utils/*.rs",
]
minimum_tested = 75
```

### 3.2 High-Value Mutation Patterns

```rust
// Original: Proper escaping
fn escape_shell_word(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
    //              ^^^ MUTATE: Remove replacement
    //              Should be caught by property tests
}

// Original: Correct arithmetic precedence
fn codegen_binary_op(lhs: &Expr, op: BinOp, rhs: &Expr) -> String {
    format!("$(( {} {} {} ))", 
        codegen_expr(lhs), 
        op.to_shell(), 
        codegen_expr(rhs))
    //  ^^ MUTATE: Swap lhs/rhs
    //     Should be caught by arithmetic equivalence tests
}

// Original: Safe variable expansion
fn codegen_variable(name: &str) -> String {
    format!("\"${}\"", name)
    //      ^      ^ MUTATE: Remove quotes
    //              Should be caught by injection property tests
}
```

### 3.3 Mutation Test Execution

```bash
# Run mutation testing on changed files (pre-commit)
$ git diff --name-only | grep '\.rs$' | while read file; do
    cargo mutants --file "$file" --timeout 60
done

# Run full mutation analysis (nightly)
$ cargo mutants --workspace --timeout 300 --output mutants.json

# Generate report
$ cargo mutants --list-files mutants.json --missed > missed_mutants.txt
```

---

## 4. Static Analysis of Transpiler Codebase

**Addressing Review Feedback:** While the testing strategy focuses on validating transpiler *output*, we must also apply static analysis to the transpiler's own Rust codebase to proactively identify issues before runtime.

### 4.1 Static Analysis Tools Integration

**4.1.1 Clippy - Lint Enforcement**

Clippy provides 500+ lints for common mistakes, performance issues, and idiomatic code violations [^10].

```toml
# .cargo/config.toml
[target.'cfg(all())']
rustflags = [
    "-D", "warnings",                    # Deny all warnings
    "-D", "clippy::all",                 # Enable all Clippy lints
    "-D", "clippy::pedantic",            # Pedantic lints
    "-D", "clippy::nursery",             # Experimental lints
    "-W", "clippy::cargo",               # Cargo-related lints
]
```

**Critical Clippy lints for transpiler safety:**

```rust
// Enforce these specific lints with deny level
#![deny(clippy::unwrap_used)]              // No .unwrap() in production code
#![deny(clippy::expect_used)]              // No .expect() without justification
#![deny(clippy::panic)]                    // No panic! calls
#![deny(clippy::indexing_slicing)]         // No unchecked array indexing
#![deny(clippy::integer_arithmetic)]       // Check arithmetic overflow
#![deny(clippy::shadow_reuse)]             // No variable shadowing
#![deny(clippy::todo)]                     // No TODO markers in main branch
#![deny(clippy::unimplemented)]            // No unimplemented!()

// Security-related lints
#![deny(clippy::dbg_macro)]                // No dbg!() in production
#![deny(clippy::print_stdout)]             // No println! except in CLI layer
#![deny(clippy::use_debug)]                // Limit Debug usage
```

**4.1.2 Cargo-Audit - Dependency Vulnerability Scanning**

Scan dependencies for known security vulnerabilities using the RustSec Advisory Database.

```bash
# Install cargo-audit
$ cargo install cargo-audit

# Check for vulnerabilities
$ cargo audit

# Check and deny warnings in CI
$ cargo audit --deny warnings
```

**4.1.3 Cargo-Deny - License and Security Policy Enforcement**

```toml
# deny.toml
[advisories]
vulnerability = "deny"          # Deny known vulnerabilities
unmaintained = "warn"           # Warn on unmaintained crates
unsound = "deny"                # Deny unsound crates
yanked = "deny"                 # Deny yanked crates

[licenses]
unlicensed = "deny"
# Only allow permissive licenses
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
]
deny = [
    "GPL-3.0",                  # Copyleft license
    "AGPL-3.0",
]

[bans]
multiple-versions = "warn"      # Warn on duplicate dependencies
wildcards = "deny"              # No wildcard dependencies
allow-wildcard-paths = false

# Deny specific crates known to be problematic
[[bans.deny]]
name = "openssl"
# Prefer rustls over OpenSSL
```

**4.1.4 Miri - Undefined Behavior Detection**

Miri is an interpreter for Rust's mid-level intermediate representation (MIR) that can detect undefined behavior.

```bash
# Install Miri
$ rustup +nightly component add miri

# Run unit tests under Miri
$ cargo +nightly miri test --lib

# Run specific test
$ cargo +nightly miri test test_string_escaping
```

**Example: Detect out-of-bounds access**

```rust
// This would be caught by Miri
#[test]
fn test_buffer_overflow() {
    let buf = vec![0u8; 10];
    // Miri detects: index out of bounds
    let _ = buf[10];
}
```

**4.1.5 Cargo-Semver-Checks - API Stability**

Ensure public API changes are semver-compliant.

```bash
$ cargo install cargo-semver-checks

# Check for breaking changes
$ cargo semver-checks check-release

# Fail CI on breaking changes in minor/patch releases
$ cargo semver-checks check-release --baseline-rev v0.1.0
```

**4.1.6 Rudra - Memory Safety Analyzer**

Rudra is a static analyzer specifically designed to find memory safety bugs in Rust using unsafe code patterns.

```bash
$ cargo install cargo-rudra

# Analyze codebase for unsafe patterns
$ cargo rudra
```

### 4.2 CI Integration for Static Analysis

```yaml
# .github/workflows/static-analysis.yml
name: Static Analysis

on: [push, pull_request]

jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      
      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
  
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-audit
        run: cargo install cargo-audit
      
      - name: Security audit
        run: cargo audit --deny warnings
  
  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-deny
        run: cargo install cargo-deny
      
      - name: Check licenses and bans
        run: cargo deny check
  
  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: miri
      
      - name: Run Miri
        run: cargo +nightly miri test --lib
  
  semver:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-semver-checks
        run: cargo install cargo-semver-checks
      
      - name: Check API compatibility
        run: |
          BASE_REF="${{ github.base_ref }}"
          cargo semver-checks check-release --baseline-rev "origin/$BASE_REF"
```

### 4.3 Pre-Commit Static Analysis

```bash
#!/bin/bash
# .git/hooks/pre-commit (enhanced)

set -e

echo "🔍 Running static analysis..."

# Clippy (fast)
echo "📎 Clippy..."
cargo clippy --all-targets -- -D warnings || {
    echo "❌ Clippy warnings found"
    exit 1
}

# Format check
echo "🎨 Format check..."
cargo fmt --check || {
    echo "❌ Code not formatted. Run: cargo fmt"
    exit 1
}

# Security audit (medium)
echo "🔒 Security audit..."
cargo audit --deny warnings || {
    echo "❌ Security vulnerabilities found"
    exit 1
}

# Run Miri on changed files (slow, only if unsafe code modified)
CHANGED_UNSAFE=$(git diff --cached --name-only | xargs grep -l "unsafe" || true)
if [ -n "$CHANGED_UNSAFE" ]; then
    echo "⚠️  Unsafe code changed, running Miri..."
    cargo +nightly miri test --lib || {
        echo "❌ Miri detected undefined behavior"
        exit 1
    }
fi

echo "✅ Static analysis passed"
```

### 4.4 Static Analysis Metrics

**Quality Gate Requirements:**

- [ ] Zero Clippy warnings on deny-level lints
- [ ] Zero known security vulnerabilities (cargo-audit)
- [ ] All dependencies use approved licenses (cargo-deny)
- [ ] No duplicate dependency versions (cargo-deny)
- [ ] Zero undefined behavior detected by Miri
- [ ] API changes are semver-compliant
- [ ] No unsafe code except in designated safety-critical modules

**Tracking Dashboard:**

```bash
# Generate comprehensive static analysis report
$ mkdir -p reports

# Clippy report
$ cargo clippy --all-targets --message-format=json > reports/clippy.json

# Audit report
$ cargo audit --json > reports/audit.json

# Deny report
$ cargo deny check --format json > reports/deny.json

# Parse and generate markdown summary
$ python3 scripts/generate_static_analysis_report.py
```

---

## 5. Test Organization and Infrastructure

### 5.1 Directory Structure

```
rash/
├── Cargo.toml
├── mutants.toml
├── deny.toml                         # Dependency policy
├── CODEOWNERS                        # Code review requirements
├── .github/
│   └── workflows/
│       ├── test.yml                  # Fast CI (unit + integration)
│       ├── comprehensive.yml         # Full suite (execution + property)
│       ├── continuous-fuzz.yml       # 6-hour fuzzing runs
│       ├── mutation-test.yml         # Nightly mutation testing
│       └── static-analysis.yml       # Clippy, audit, deny, Miri
│
├── src/
│   ├── lib.rs
│   ├── parser.rs                     # Rust → AST
│   ├── ir.rs                         # AST → IR lowering
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── string_escaping.rs        # CRITICAL: Injection prevention
│   │   ├── variable_quoting.rs       # CRITICAL: Safe expansion
│   │   ├── arithmetic.rs             # POSIX arithmetic expressions
│   │   └── control_flow.rs           # if/while/for translation
│   ├── safety/
│   │   ├── injection_check.rs        # Static analysis
│   │   ├── bashism_detector.rs       # POSIX compliance checker
│   │   └── shellcheck_runner.rs      # Integration with shellcheck
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── error_reporting.rs        # User-facing error messages
│   │   └── diagnostics.rs            # Source location tracking
│   └── bin/
│       └── rash.rs                   # CLI entry point
│
├── tests/
│   ├── unit/
│   │   ├── parser_tests.rs           # Parse correctness
│   │   ├── ir_lowering_tests.rs      # AST → IR validation
│   │   └── codegen_tests.rs          # Individual codegen functions
│   │
│   ├── integration/
│   │   ├── transpile_snapshots.rs    # insta snapshot tests
│   │   ├── determinism_tests.rs      # Cryptographic hash validation
│   │   └── fixtures/                 # Test cases (.rs files)
│   │       ├── arithmetic.rs
│   │       ├── string_ops.rs
│   │       ├── control_flow.rs
│   │       └── ...
│   │
│   ├── execution/
│   │   ├── multi_shell_tests.rs      # dash, bash, ash, ksh
│   │   ├── docker_isolation_tests.rs # Alpine, Debian, Fedora
│   │   ├── equivalence_tests.rs      # Rust vs Shell output
│   │   └── cli_error_handling_tests.rs # Negative testing
│   │
│   ├── property/
│   │   ├── safety_properties.rs      # Injection immunity
│   │   ├── arithmetic_properties.rs  # Semantic equivalence
│   │   ├── string_properties.rs      # String operation correctness
│   │   └── posix_properties.rs       # Compliance checks
│   │
│   ├── snapshots/                    # Generated by insta
│   │   ├── arithmetic.sh.snap
│   │   └── ...
│   │
│   └── common/
│       ├── mod.rs                    # Shared test utilities
│       ├── shell_runner.rs           # Shell execution helpers
│       └── fixtures.rs               # Test data generators
│
├── fuzz/
│   ├── Cargo.toml
│   ├── fuzz_targets/
│   │   ├── transpile_robustness.rs   # Crash detection
│   │   └── differential_execution.rs # Behavioral equivalence
│   ├── corpus/                       # Seed inputs
│   │   └── differential_execution/
│   │       ├── cargo_snippet.rs
│   │       ├── ripgrep_snippet.rs
│   │       └── ...
│   └── rust_keywords.dict            # Fuzzing dictionary
│
├── benches/
│   ├── transpile_throughput.rs       # Ops/sec measurement
│   ├── codegen_size.rs               # Output size optimization
│   └── parser_performance.rs         # Parse speed
│
├── formal/                           # Formal verification artifacts
│   ├── coq/
│   │   └── ir_correctness.v          # AST→IR proofs
│   └── prusti/
│       └── safety_contracts.rs       # Annotated safety code
│
├── scripts/
│   ├── pre-commit-checks.sh          # Local pre-commit validation
│   ├── generate_static_analysis_report.py
│   └── extract_real_world_corpus.sh  # Build fuzzing corpus
│
└── docs/
    ├── testing-strategy.md           # This document
    ├── formal-verification.md        # Verification roadmap
    └── contributing.md               # Developer guide
```

### 5.2 CODEOWNERS File

**Purpose:** Ensure safety-critical code receives mandatory review from designated experts before merging.

```
# CODEOWNERS
# This file defines mandatory reviewers for specific paths

# Safety-critical code requires review from security team
/src/codegen/string_escaping.rs    @alice @security-team
/src/codegen/variable_quoting.rs   @alice @security-team
/src/safety/                       @alice @security-team

# Core transpiler logic requires review from compiler team
/src/parser.rs                     @bob @compiler-team
/src/ir.rs                         @bob @compiler-team
/src/codegen/                      @bob @compiler-team

# Formal verification artifacts require review from verification team
/formal/                           @carol @verification-team

# Test infrastructure can be reviewed by any maintainer
/tests/                            @maintainers
/fuzz/                             @maintainers

# CI/CD configuration requires devops review
/.github/workflows/                @devops-team

# Dependency changes require security audit
/Cargo.toml                        @security-team
/Cargo.lock                        @security-team
/deny.toml                         @security-team
```

**Integration with GitHub:**
- Place `CODEOWNERS` in repository root
- Enable "Require review from Code Owners" in branch protection
- Configure required number of approvals per team

### 5.3 CI/CD Pipeline Configuration

```yaml
# .github/workflows/test.yml (Fast feedback, runs on every push)
name: Fast Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Run unit tests
        run: cargo test --lib --bins --all-features
      
      - name: Run doctests
        run: cargo test --doc
  
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Install insta CLI
        run: cargo install cargo-insta
      
      - name: Run integration tests
        run: cargo test --test '*_snapshots'
      
      - name: Check snapshots are committed
        run: cargo insta test --check --unreferenced=reject
  
  negative-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Build CLI binary
        run: cargo build --bin rash
      
      - name: Run CLI error handling tests
        run: cargo test --test cli_error_handling_tests
```

```yaml
# .github/workflows/comprehensive.yml (Runs on main branch merges)
name: Comprehensive Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  multi-shell-tests:
    strategy:
      matrix:
        os: [ubuntu-22.04, macos-13]
        shell: [dash, bash, ash]
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install shells
        run: |
          if [ "$RUNNER_OS" == "Linux" ]; then
            sudo apt-get update
            sudo apt-get install -y dash bash busybox-static shellcheck
            ln -s /bin/busybox $(pwd)/ash
          elif [ "$RUNNER_OS" == "macOS" ]; then
            brew install bash dash shellcheck
          fi
      
      - name: Run execution tests
        run: cargo test --test execution_tests
        env:
          TEST_SHELL: ${{ matrix.shell }}
  
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run property-based tests
        run: |
          cargo test --test property_tests --release -- \
            --test-threads=1 \
            --nocapture
        env:
          PROPTEST_CASES: 10000  # More cases in CI
  
  docker-isolation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run Docker-based tests
        run: cargo test --test docker_tests -- --ignored
  
  shellcheck-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install ShellCheck
        run: sudo apt-get install -y shellcheck
      
      - name: Run ShellCheck compliance tests
        run: cargo test shellcheck_compliance
```

```yaml
# .github/workflows/mutation-test.yml (Nightly)
name: Mutation Testing

on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM UTC daily
  workflow_dispatch:

jobs:
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install cargo-mutants
        run: cargo install cargo-mutants
      
      - name: Run mutation tests
        run: |
          cargo mutants --workspace \
            --timeout 300 \
            --output mutants.json \
            --no-shuffle  # Deterministic order
      
      - name: Check mutation score
        run: |
          SCORE=$(cargo mutants --list mutants.json | \
                  grep "mutation score" | \
                  cut -d':' -f2 | \
                  tr -d ' %')
          
          if [ "$SCORE" -lt 85 ]; then
            echo "Mutation score $SCORE% below threshold (85%)"
            exit 1
          fi
      
      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: mutants.json
```

### 5.4 Pre-Commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit (production version)

set -e

echo "🔍 Running pre-commit checks..."

# Fast unit tests (<2s)
echo "🧪 Unit tests..."
cargo test --lib --bins --quiet || {
    echo "❌ Unit tests failed"
    exit 1
}

# Format check (<1s)
echo "🎨 Format check..."
cargo fmt --check || {
    echo "❌ Code not formatted. Run: cargo fmt"
    exit 1
}

# Clippy lints (<5s)
echo "📎 Clippy..."
cargo clippy --all-targets -- -D warnings || {
    echo "❌ Clippy warnings found"
    exit 1
}

# Security audit (<2s)
echo "🔒 Security audit..."
cargo audit --deny warnings 2>/dev/null || {
    echo "❌ Security vulnerabilities found"
    exit 1
}

# Safety-critical property tests (<10s)
echo "🛡️  Safety checks..."
cargo test safety_properties --quiet || {
    echo "❌ Safety property tests failed"
    exit 1
}

# Snapshot consistency (<3s)
echo "📸 Snapshot check..."
cargo insta test --check --quiet 2>/dev/null || {
    echo "❌ Snapshots need review. Run: cargo insta review"
    exit 1
}

# If safety-critical files changed, run mutation tests
CHANGED=$(git diff --cached --name-only --diff-filter=ACM | \
          grep -E "(string_escaping|variable_quoting|injection_check)\.rs$" || true)

if [ -n "$CHANGED" ]; then
    echo "⚠️  Safety-critical files changed, running mutation tests..."
    for file in $CHANGED; do
        cargo mutants --file "src/$file" --timeout 60 --quiet || {
            echo "❌ Mutation tests failed for $file"
            exit 1
        }
    done
fi

# If CLI error handling changed, run negative tests
if git diff --cached --name-only | grep -q "cli/error_reporting.rs"; then
    echo "⚙️  CLI error handling changed, running negative tests..."
    cargo test --test cli_error_handling_tests --quiet || {
        echo "❌ CLI error handling tests failed"
        exit 1
    }
fi

echo "✅ All pre-commit checks passed ($(date +%T))"
```

---

## 6. Performance Targets and Benchmarks

### 6.1 Test Suite Performance Requirements

| Test Layer | Count | Max Runtime | Feedback Time |
|------------|-------|-------------|---------------|
| Unit tests | 100+ | 2s | <10s (TDD cycle) |
| Integration tests | 50+ | 5s | <30s (snapshot review) |
| Execution tests (local shells) | 30+ | 10s | <1min |
| Execution tests (Docker) | 20+ | 60s | CI only |
| Negative tests (CLI errors) | 15+ | 3s | <30s |
| Property tests (1000 cases each) | 10+ | 30s | <2min |
| Fuzzing (continuous) | ∞ | Unbounded | 6h CI runs |
| Static analysis (Clippy + audit) | - | 10s | <30s |
| Mutation tests (critical files) | - | 5min | Pre-commit |
| Mutation tests (full workspace) | - | 30min | Nightly CI |

### 6.2 Transpiler Performance Benchmarks

```rust
// benches/transpile_throughput.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rash::transpile;

fn bench_transpile_simple(c: &mut Criterion) {
    let hello_world = r#"
        fn main() {
            println!("Hello, world!");
        }
    "#;
    
    c.bench_function("hello_world", |b| {
        b.iter(|| transpile(hello_world))
    });
}

fn bench_transpile_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("program_size");
    
    for lines in [10, 50, 100, 500, 1000] {
        let code = generate_program_with_lines(lines);
        
        group.bench_with_input(
            BenchmarkId::from_parameter(lines),
            &code,
            |b, code| {
                b.iter(|| transpile(code))
            }
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_transpile_simple, bench_transpile_complex);
criterion_main!(benches);
```

**Target Metrics:**
- **Hello World:** <10ms transpilation (90th percentile: <5ms)
- **100 lines:** <50ms transpilation
- **1000 lines:** <500ms transpilation
- **Memory:** <100MB peak for 1000-line programs
- **Output size:** <2x input size (average compression ratio)

---

## 7. Success Metrics and Quality Gates

### 7.1 Test Coverage Requirements

```bash
# Install coverage tool
$ cargo install cargo-tarpaulin

# Generate coverage report
$ cargo tarpaulin \
    --engine llvm \
    --out Html \
    --output-dir coverage \
    --exclude-files 'tests/*' \
    --fail-under 90
```

**Minimum Coverage Targets:**
- **Safety-critical code:** 100% line + branch coverage
- **Core transpiler:** 95% line coverage, 90% branch coverage
- **CLI/Error handling:** 90% line coverage
- **Utilities:** 85% line coverage
- **Overall project:** 90% line coverage

### 7.2 Quality Gate Checklist (v0.1 Release)

- [ ] **Safety (Zero Tolerance)**
  - [ ] 100% of injection property tests pass (10,000 cases)
  - [ ] All variable expansions quoted (static analysis)
  - [ ] Zero ShellCheck warnings on all test outputs
  - [ ] Mutation score >95% on `string_escaping.rs`
  - [ ] Zero vulnerabilities detected by cargo-audit
  - [ ] Zero undefined behavior detected by Miri

- [ ] **Correctness**
  - [ ] 100% of execution tests match Rust behavior
  - [ ] Arithmetic equivalence: 10,000 property test cases
  - [ ] String operations: 5,000 property test cases
  - [ ] Control flow: All test cases pass in 4+ shells

- [ ] **POSIX Compliance**
  - [ ] No bashisms detected (regex scanner)
  - [ ] Works in dash 0.5.12+ (Debian default)
  - [ ] Works in busybox ash 1.36+ (Alpine default)
  - [ ] ShellCheck score: 10/10 on strict mode

- [ ] **Determinism**
  - [ ] 100 runs produce identical output (BLAKE3 hash)
  - [ ] No timestamp or PRNG calls in codegen
  - [ ] Variable mangling is deterministic

- [ ] **Usability**
  - [ ] All unsupported features produce clear errors
  - [ ] Error message quality score ≥0.7
  - [ ] Error messages include source location + suggestions
  - [ ] `--help` and `--version` work correctly

- [ ] **Performance**
  - [ ] Hello World: <10ms (99th percentile)
  - [ ] 90th percentile programs: <100ms
  - [ ] Peak memory: <200MB for 1000-line programs

- [ ] **Test Quality**
  - [ ] Mutation score: >85% overall, >95% critical code
  - [ ] Code coverage: >90% lines, >85% branches
  - [ ] Fuzzing: 100K execs without crashes
  - [ ] Property tests: 10K+ cases per property

- [ ] **Static Analysis**
  - [ ] Zero Clippy warnings (deny-level)
  - [ ] Zero known vulnerabilities
  - [ ] All dependencies use approved licenses
  - [ ] API is semver-compliant

### 7.3 Continuous Monitoring

```yaml
# .github/workflows/metrics.yml
name: Quality Metrics

on:
  push:
    branches: [main]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v3
  
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo bench --bench transpile_throughput
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/results.json
  
  mutation-score:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-mutants
      - run: cargo mutants --workspace --output metrics.json
      - run: |
          echo "MUTATION_SCORE=$(jq '.score' metrics.json)" >> $GITHUB_ENV
      - uses: schneegans/dynamic-badges-action@v1.7.0
        with:
          auth: ${{ secrets.GIST_SECRET }}
          gistID: <gist-id>
          filename: mutation-score.json
          label: Mutation Score
          message: ${{ env.MUTATION_SCORE }}%
          color: ${{ env.MUTATION_SCORE > 85 && 'green' || 'red' }}
  
  static-analysis-summary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run all static analysis
        run: |
          cargo clippy --all-targets --message-format=json > clippy.json
          cargo audit --json > audit.json
          cargo deny check --format json > deny.json
      - name: Generate report
        run: python3 scripts/generate_static_analysis_report.py
      - uses: actions/upload-artifact@v3
        with:
          name: static-analysis-report
          path: reports/
```

---

## 8. Anti-Patterns and Common Pitfalls

### 8.1 Testing Anti-Patterns to Avoid

❌ **Manual shell script review instead of automated execution tests**  
✅ Always use multi-shell execution with assertion checking

❌ **Assuming quote escaping is correct without property testing**  
✅ Property test ALL string operations with malicious inputs

❌ **Skipping ShellCheck integration**  
✅ Run ShellCheck on every generated script, fail on warnings

❌ **Trusting snapshot diffs without manual inspection**  
✅ Review every snapshot change for unintended codegen modifications

❌ **Tolerating flaky tests in execution layer**  
✅ Execution tests must be 100% deterministic; fix or delete flakes

❌ **Ignoring mutation test failures**  
✅ Every uncaught mutant represents a missing test case

❌ **Testing only in bash**  
✅ POSIX compliance requires dash and ash testing at minimum

❌ **Mocking shell execution in tests**  
✅ Always test against real shell interpreters

❌ **Fuzzing with pure random input**  
✅ Seed corpus with real-world Rust code for better coverage

❌ **Shipping without mutation testing critical code**  
✅ Safety-critical modules must have >95% mutation score

❌ **Poor error messages for unsupported features**  
✅ Every error must include location, explanation, and suggestion

❌ **Skipping static analysis on transpiler code**  
✅ Run Clippy, audit, and deny checks on every commit

### 8.2 Development Anti-Patterns

❌ **Implementing features before tests**  
✅ Write failing test → implement feature → verify test passes

❌ **Optimizing codegen before correctness is proven**  
✅ Correct first, fast second (use benchmarks to guide optimization)

❌ **Adding dependencies without justification**  
✅ Every dependency increases attack surface; minimize dependencies

❌ **Using unstable Rust features**  
✅ Stick to stable Rust to ensure long-term maintainability

❌ **Ignoring ShellCheck warnings**  
✅ Every warning represents a potential bug or portability issue

❌ **Using `.unwrap()` in production code**  
✅ Enforce `#![deny(clippy::unwrap_used)]` and use proper error handling

❌ **Merging without code owner approval**  
✅ Ensure CODEOWNERS is enforced in branch protection

---

## 9. References and Further Reading

[^1]: **Compiler Testing Strategies**  
    Le, V., Afshari, M., & Su, Z. (2014). "Compiler validation via equivalence modulo inputs." *ACM SIGPLAN Notices*, 49(6), 216-226.  
    https://doi.org/10.1145/2666356.2594334

[^2]: **Snapshot Testing Methodology**  
    Greif, S., Benitte, R., & Terrill, M. (2020). "The State of JavaScript Testing." *Testing Library Documentation*.  
    https://testing-library.com/docs/

[^3]: **Differential Testing in Compilers**  
    McKeeman, W. M. (1998). "Differential testing for software." *Digital Technical Journal*, 10(1), 100-107.  
    https://doi.org/10.1109/52.687949

[^4]: **Property-Based Testing**  
    Claessen, K., & Hughes, J. (2000). "QuickCheck: a lightweight tool for random testing of Haskell programs." *ACM SIGPLAN Notices*, 35(9), 268-279.  
    https://doi.org/10.1145/357766.351266

[^5]: **Coverage-Guided Fuzzing**  
    Böhme, M., Pham, V. T., & Roychoudhury, A. (2016). "Coverage-based greybox fuzzing as Markov chain." *ACM SIGSAC Conference on Computer and Communications Security*, 1032-1043.  
    https://doi.org/10.1145/2976749.2978428

[^6]: **Symbolic Execution for Security**  
    Cadar, C., Dunbar, D., & Engler, D. R. (2008). "KLEE: Unassisted and automatic generation of high-coverage tests for complex systems programs." *OSDI*, 8, 209-224.  
    https://www.usenix.org/legacy/event/osdi08/tech/full_papers/cadar/cadar.pdf

[^7]: **Bisimulation in Compiler Verification**  
    Leroy, X. (2009). "Formal verification of a realistic compiler." *Communications of the ACM*, 52(7), 107-115.  
    https://doi.org/10.1145/1538788.1538814

[^8]: **Mutation Testing Effectiveness**  
    Jia, Y., & Harman, M. (2011). "An analysis and survey of the development of mutation testing." *IEEE Transactions on Software Engineering*, 37(5), 649-678.  
    https://doi.org/10.1109/TSE.2010.62

[^9]: **Error Message Quality in Developer Tools**  
    Becker, B. A., et al. (2019). "Compiler error messages considered unhelpful: The landscape of text-based programming error message research." *ITiCSE Working Group Reports*, 177-210.  
    https://doi.org/10.1145/3344429.3372508

[^10]: **Clippy: Rust Linter**  
    Rust Team. (2023). "Clippy: A collection of lints to catch common mistakes and improve your Rust code."  
    https://github.com/rust-lang/rust-clippy

**Additional Resources:**

- **POSIX.1-2017 Standard:** IEEE Std 1003.1-2017 Shell Command Language  
  https://pubs.opengroup.org/onlinepubs/9699919799/

- **ShellCheck Wiki:** Common shell scripting pitfalls  
  https://www.shellcheck.net/wiki/

- **Rust Fuzzing Book:** Guide to fuzzing Rust programs  
  https://rust-fuzz.github.io/book/

- **cargo-mutants Documentation:** Mutation testing for Rust  
  https://mutants.rs/

- **proptest Book:** Property-based testing in Rust  
  https://proptest-rs.github.io/proptest/

- **CompCert:** Verified C compiler (formal verification reference)  
  https://compcert.org/

- **RustSec Advisory Database:** Security vulnerability tracking  
  https://rustsec.org/

- **Prusti:** Rust verification tool based on Viper  
  https://www.pm.inf.ethz.ch/research/prusti.html

---

## Appendix A: Example Test Suite Execution

```bash
# Complete test suite execution (developer workflow)

# 1. Fast feedback: Unit tests (<2s)
$ cargo test --lib --bins
Running unittests src/lib.rs (target/debug/deps/rash-...)
test parser::tests::parse_basic_function ... ok
test codegen::tests::variable_name_mangling ... ok
...
test result: ok. 127 passed; 0 failed; finished in 1.83s

# 2. Integration: Snapshot tests (<5s)
$ cargo test --test transpile_snapshots
Running tests/integration/transpile_snapshots.rs
test test_all_fixtures ... ok
test verify_deterministic_output ... ok
test result: ok. 52 passed; 0 failed; finished in 4.21s

# 3. Execution: Multi-shell tests (<10s)
$ cargo test --test execution_tests
Running tests/execution/multi_shell_tests.rs
test test_arithmetic_evaluation ... ok (dash: ✓, bash: ✓, ash: ✓)
test test_shell_injection_prevention ... ok (dash: ✓, bash: ✓, ash: ✓)
test result: ok. 34 passed; 0 failed; finished in 8.92s

# 4. Negative: CLI error handling (<3s)
$ cargo test --test cli_error_handling_tests
Running tests/execution/cli_error_handling_tests.rs
test test_async_syntax_error_message ... ok
test test_trait_definition_error_message ... ok
test test_error_message_quality_meets_threshold ... ok
test result: ok. 15 passed; 0 failed; finished in 2.67s

# 5. Properties: Safety invariants (<30s)
$ cargo test --test property_tests -- --test-threads=1
Running tests/property/safety_properties.rs
test no_command_injection ... ok [1000/1000 cases passed]
test arithmetic_preserves_semantics ... ok [1000/1000 cases passed]
test result: ok. 12 passed; 0 failed; finished in 27.45s

# 6. Static Analysis: Clippy + audit (<10s)
$ cargo clippy --all-targets -- -D warnings
Checking rash v0.1.0...
Finished dev [unoptimized] target(s) in 5.23s

$ cargo audit
    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 524 security advisories (from advisory-db)
    Scanning Cargo.lock for vulnerabilities (42 crate dependencies)
Crate:     Security Vulnerabilities
rash:      0 found

# 7. Mutation: Critical files only (<5min, pre-commit)
$ cargo mutants --file src/codegen/string_escaping.rs
Analyzing mutations in src/codegen/string_escaping.rs...
Found 23 mutants
Running tests...
23/23 mutants caught by tests
Mutation score: 100.0%
Finished in 4m 12s

# 8. Coverage: Generate report
$ cargo tarpaulin --engine llvm --out Html
|| Tested/Total Lines:
|| src/codegen/string_escaping.rs: 100.0% (42/42)
|| src/cli/error_reporting.rs: 95.2% (60/63)
|| src/parser.rs: 96.8% (215/222)
|| ...
|| Overall: 92.3% (1847/2001)

# 9. Fuzzing: Run for 1 hour
$ cargo +nightly fuzz run differential_execution -- -max_total_time=3600
#1      INITED cov: 1234 ft: 567 corp: 1/1b exec/s: 0 rss: 45Mb
#2      NEW    cov: 1256 ft: 589 corp: 2/15b exec/s: 1234 rss: 52Mb
...
#125478 pulse  cov: 2341 ft: 1823 corp: 342/12Kb exec/s: 34 rss: 128Mb
Done: 125478 execs, 0 crashes, 0 timeouts, 342 corpus entries
```

---

## Appendix B: Quick Reference

### Test Commands Cheat Sheet

```bash
# Development (fast feedback)
cargo test --lib --bins              # Unit tests only (<2s)
cargo test --test snapshots          # Integration tests (<5s)
cargo test --test cli_error_*        # Negative tests (<3s)
cargo insta review                   # Review snapshot changes

# Pre-commit (comprehensive local testing)
./scripts/pre-commit-checks.sh       # All fast tests + safety checks
cargo test                           # Full test suite (<30s)
cargo clippy --all-targets           # Lint checks
cargo audit                          # Security audit
cargo fmt --check                    # Format check

# CI (comprehensive validation)
cargo test --workspace               # All tests
cargo test -- --ignored              # Docker-based tests
cargo tarpaulin --out Html           # Coverage report
cargo mutants --workspace            # Mutation testing
cargo deny check                     # License/dependency policy

# Fuzzing
cargo +nightly fuzz run transpile_robustness
cargo +nightly fuzz run differential_execution

# Benchmarking
cargo bench                          # All benchmarks
cargo bench -- hello_world           # Specific benchmark

# Quality checks
shellcheck scripts/*.sh              # Validate our scripts
cargo +nightly miri test --lib       # Undefined behavior detection
cargo semver-checks                  # API stability
```

### Property Test Templates

```rust
// Template: Safety property (injection immunity)
proptest! {
    #[test]
    fn no_injection_in_FEATURE(input in "arbitrary_string_pattern") {
        let code = generate_code_using(input);
        let output = transpile(&code).unwrap();
        assert!(safety::is_safe(&output));
        let result = run_shell(&output);
        assert_eq!(result, expected_literal(input));
    }
}

// Template: Semantic equivalence
proptest! {
    #[test]
    fn FEATURE_matches_rust(params in arb_valid_params()) {
        let code = generate_code_from(params);
        let rust_output = compile_and_run_rust(&code).unwrap();
        let shell_output = run_shell(&transpile(&code).unwrap());
        assert_eq!(rust_output, shell_output);
    }
}

// Template: POSIX compliance
proptest! {
    #[test]
    fn FEATURE_is_posix_compliant(prog in arb_program()) {
        let output = transpile(&prog).unwrap();
        assert!(!contains_bashisms(&output));
        assert!(shellcheck_passes(&output));
        assert!(runs_in_dash(&output).is_ok());
    }
}

// Template: Negative testing (error messages)
proptest! {
    #[test]
    fn UNSUPPORTED_FEATURE_produces_clear_error(
        prog in arb_program_with_unsupported_feature()
    ) {
        let result = transpile(&prog);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("error: unsupported feature"));
        assert!(error_msg.contains("note:"));
        assert!(error_msg.contains("help:"));
    }
}
```

---

**Document Status:** Production-Ready (v1.2)  
**Next Review:** After v0.2 milestone (formal verification integration)  
**Maintainer:** Rash Core Team  
**Contributors:** See CODEOWNERS  
**Last Updated:** 2025-10-03
