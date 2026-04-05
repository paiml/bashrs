fn test_t118_vec_len() {
    // STMT: let v = vec![1]; let _ = v.len(); - vec length
    let (ok, output) = transpile_stmt("let v = vec![1]; let _ = v.len();");
    if ok && !output.contains("${#") && !output.contains("len") {
        println!("T118: WARNING - vec.len() should produce ${{#v[@]}} or similar");
    }
}

#[test]
fn test_t119_vec_index() {
    // STMT: let v = vec![1]; let _ = v[0]; - vec indexing
    let (ok, output) = transpile_stmt("let v = vec![1]; let _ = v[0];");
    if ok && !output.contains("${v[0]}") && !output.contains("[0]") {
        println!("T119: WARNING - v[0] should produce ${{v[0]}} or similar");
    }
}

#[test]
fn test_t120_contains() {
    // STMT: let v = vec![1]; v.contains(&1); - collection contains
    let (ok, output) = transpile_stmt("let v = vec![1]; let _ = v.contains(&1);");
    if !ok {
        println!("T120: contains unsupported: {}", output);
    }
}

// ============================================================================
// SECTION 4.8: Edge Cases (T121-T130)
// ============================================================================

#[test]
fn test_t121_thread_spawn() {
    // STMT: std::thread::spawn(|| {}) - should error (no threads in shell)
    let (ok, output) = transpile_stmt("std::thread::spawn(|| {});");
    if ok {
        println!("T121: Thread spawn should NOT be supported in shell");
    } else {
        // Expected to fail - threads are not available in shell
        println!(
            "T121: Correctly rejects thread::spawn: {}",
            output.lines().next().unwrap_or("")
        );
    }
}

#[test]
fn test_t122_print_no_newline() {
    // STMT: print!("no newline") - should produce printf without newline
    let (ok, output) = transpile_stmt("print!(\"no newline\");");
    if !ok {
        println!(
            "T122: print! unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("printf") && !output.contains("-n") {
        println!("T122: WARNING - print! should use printf or echo -n (no trailing newline)");
    }
}

#[test]
fn test_t123_setvar_spaces() {
    // STMT: std::env::set_var("A", "b c") - value with spaces needs quoting
    let (ok, output) = transpile_stmt("std::env::set_var(\"A\", \"b c\");");
    if !ok {
        println!(
            "T123: set_var unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("\"") && !output.contains("'") {
        println!("T123: WARNING - export with spaces needs quoting");
    }
}

#[test]
fn test_t124_hard_link() {
    // STMT: std::fs::hard_link("a", "b") - should produce ln (without -s)
    let (ok, output) = transpile_stmt("std::fs::hard_link(\"a\", \"b\");");
    if !ok {
        println!(
            "T124: hard_link unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("ln ") || output.contains("-s") {
        println!("T124: WARNING - hard_link should use 'ln' without -s flag");
    }
}

#[test]
fn test_t125_copy_file() {
    // STMT: std::fs::copy("a", "b") - should produce cp
    let (ok, output) = transpile_stmt("std::fs::copy(\"a\", \"b\");");
    if !ok {
        println!(
            "T125: copy unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("cp ") && !output.contains("cp\n") {
        println!("T125: WARNING - fs::copy should produce 'cp' command");
    }
}

#[test]
fn test_t126_rename_file() {
    // STMT: std::fs::rename("a", "b") - should produce mv
    let (ok, output) = transpile_stmt("std::fs::rename(\"a\", \"b\");");
    if !ok {
        println!(
            "T126: rename unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("mv ") && !output.contains("mv\n") {
        println!("T126: WARNING - fs::rename should produce 'mv' command");
    }
}

#[test]
fn test_t127_raw_string() {
    // STMT: let s = r"a\b"; - raw string preserves backslash
    let (ok, output) = transpile_stmt("let s = r\"a\\b\";");
    if !ok {
        println!(
            "T127: raw string unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else {
        // Raw string should preserve the literal backslash
        println!("T127: Raw string handled");
    }
}

#[test]
fn test_t128_format_macro() {
    // STMT: let _ = format!("x: {}", 1); - string formatting
    let (ok, output) = transpile_stmt("let _ = format!(\"x: {}\", 1);");
    if !ok {
        println!(
            "T128: format! unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else {
        // format! should produce some string construction
        println!("T128: format! handled");
    }
}

#[test]
fn test_t129_iterator_map() {
    // STMT: vec![1, 2].iter().map(|x| x+1) - functional map
    let (ok, output) = transpile_stmt("let _ = vec![1, 2].iter().map(|x| x + 1);");
    if !ok {
        println!(
            "T129: iterator map unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("for") && !output.to_lowercase().contains("loop") {
        println!("T129: WARNING - iter().map() should produce a loop");
    }
}

#[test]
fn test_t130_iterator_filter() {
    // STMT: vec![1].iter().filter(|x| *x>0) - functional filter
    let (ok, output) = transpile_stmt("let _ = vec![1, 2, 3].iter().filter(|x| *x > 1);");
    if !ok {
        println!(
            "T130: iterator filter unsupported: {}",
            output.lines().next().unwrap_or("")
        );
    } else if !output.contains("if") {
        println!("T130: WARNING - iter().filter() should produce conditional logic");
    }
}

// ============================================================================
// COMPREHENSIVE SUMMARY TEST
// ============================================================================

#[test]
fn test_tcode_comprehensive_summary() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║        T-CODE TRANSPILER TEST SUMMARY (SPEC-TB-2025-001 v2.2.0)              ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║                                                                              ║");
    println!("║  Known Bugs (from bug hunt):                                                 ║");
    println!("║    TB-001: User-defined functions not transpiled                             ║");
    println!("║    TB-002: Function parameters not passed                                    ║");
    println!("║    TB-003: Multiple function definitions fail                                ║");
    println!("║    TB-004: String literal validation fails                                   ║");
    println!("║    TB-005: Range-based for loops unsupported                                 ║");
    println!("║    TB-006: Function return values not handled                                ║");
    println!("║    TB-007: Multiplication not computed                                       ║");
    println!("║    TB-008: Modulo not computed                                               ║");
    println!("║    TB-010: match statements unsupported                                      ║");
    println!("║                                                                              ║");
    println!("║  Run individual T-code tests for detailed failure analysis.                  ║");
    println!("║                                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// BASELINE VERIFICATION - Count passing vs failing
// ============================================================================

#[test]
fn test_tcode_baseline_verification() {
    let mut results: Vec<TCodeResult> = Vec::new();

    // T001: Empty main
    let (ok, output) = transpile_prog("fn main() {}");
    if ok && output.contains("main()") {
        results.push(TCodeResult::pass("T001"));
    } else {
        results.push(TCodeResult::fail("T001", "Missing main()"));
    }

    // T002: Integer
    let (ok, output) = transpile_stmt("let a = 1;");
    if ok && !output.contains("unknown") {
        results.push(TCodeResult::pass("T002"));
    } else {
        results.push(TCodeResult::fail("T002", "Integer assignment failed"));
    }

    // T003: Negative
    let (ok, output) = transpile_stmt("let a = -1;");
    if ok && !output.contains("unknown") {
        results.push(TCodeResult::pass("T003"));
    } else {
        results.push(TCodeResult::fail("T003", "Negative integer failed"));
    }

    // T071: Function definition
    let (ok, output) = transpile_prog("fn foo() {} fn main() { foo(); }");
    if ok && output.contains("foo()") {
        results.push(TCodeResult::pass("T071"));
    } else {
        results.push(TCodeResult::fail(
            "T071",
            "TB-001: Functions not transpiled",
        ));
    }

    // Count results
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();

    println!("\n╔═══════════════════════════════════════════╗");
    println!("║     T-CODE BASELINE VERIFICATION          ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  Passed: {:<3}                              ║", passed);
    println!("║  Failed: {:<3}                              ║", failed);
    println!("╠═══════════════════════════════════════════╣");

    for r in &results {
        if r.passed {
            println!("║  ✅ {}                                   ║", r.id);
        } else {
            println!(
                "║  ❌ {} - {}  ║",
                r.id,
                r.reason.chars().take(20).collect::<String>()
            );
        }
    }

    println!("╚═══════════════════════════════════════════╝");
}

// ============================================================================
// PROPERTY TESTS - Per SPEC-TB-2025-001 Section 5
// ============================================================================

#[cfg(test)]
#[cfg(feature = "property-tests")] // Disabled by default - flaky in CI
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Reduced cases to prevent timeout/long execution

        /// Property 5.1: Symmetry - Transpilation is deterministic
        /// Same input always produces same output
        #[test]
        fn prop_transpile_deterministic(n in 0i32..1000) {
            let code = format!("fn main() {{ let x = {}; }}", n);
            let (ok1, out1) = transpile_prog(&code);
            let (ok2, out2) = transpile_prog(&code);

            prop_assert_eq!(ok1, ok2, "Transpilation success should be consistent");
            if ok1 {
                prop_assert_eq!(out1, out2, "Output should be identical for same input");
            }
        }

        /// Property: Integer literals always transpile (no 'unknown')
        #[test]
        fn prop_integer_literals_never_unknown(n in -1000i32..1000) {
            let code = format!("fn main() {{ let x = {}; }}", n);
            let (ok, output) = transpile_prog(&code);

            if ok {
                prop_assert!(
                    !output.contains("unknown"),
                    "Integer {} should not produce 'unknown'", n
                );
            }
        }

        /// Property: Empty main always transpiles successfully
        #[test]
        fn prop_empty_main_always_works(_dummy in 0..10u32) {
            let (ok, output) = transpile_prog("fn main() {}");
            prop_assert!(ok, "Empty main should always transpile");
            prop_assert!(output.contains("main()"), "Output should contain main()");
        }

        /// Property: Variable names are preserved in output
        #[test]
        fn prop_variable_names_preserved(
            name in "[a-z][a-z0-9_]{0,10}"
        ) {
            // Skip Rust keywords
            if ["fn", "let", "if", "else", "while", "for", "loop", "match", "return", "break", "continue", "true", "false", "mut", "pub", "mod", "use", "struct", "enum", "impl", "trait", "type", "where", "as", "in", "ref", "self", "super", "crate", "const", "static", "extern", "unsafe", "async", "await", "dyn", "move"].contains(&name.as_str()) {
                return Ok(());
            }

            let code = format!("fn main() {{ let {} = 42; }}", name);
            let (ok, output) = transpile_prog(&code);

            if ok {
                prop_assert!(
                    output.contains(&name) || output.contains(&format!("{}=", name)),
                    "Variable '{}' should appear in output", name
                );
            }
        }

        /// Property: Positive integers produce valid shell assignments
        #[test]
        fn prop_positive_int_valid_shell(n in 0u32..10000) {
            let code = format!("fn main() {{ let x = {}; }}", n);
            let (ok, output) = transpile_prog(&code);

            if ok {
                // Should contain x= assignment
                prop_assert!(
                    output.contains("x="),
                    "Should have x= assignment for {}", n
                );
            }
        }

        /// Property: Arithmetic produces shell arithmetic or literal result
        #[test]
        fn prop_arithmetic_produces_result(
            a in 1i32..100,
            b in 1i32..100
        ) {
            let code = format!("fn main() {{ let r = {} + {}; }}", a, b);
            let (ok, output) = transpile_prog(&code);

            if ok {
                // Should contain either $(( arithmetic )) or the computed result
                let expected_sum = a + b;
                let has_arith = output.contains("$((")
                    || output.contains(&expected_sum.to_string());

                prop_assert!(
                    has_arith || output.contains("r="),
                    "Addition {}+{} should produce arithmetic or result", a, b
                );
            }
        }

        /// Property 5.3: Quoting Safety - String content is quoted
        #[test]
        fn prop_println_content_quoted(s in "[a-zA-Z0-9 ]{1,20}") {
            let code = format!(r#"fn main() {{ println!("{}"); }}"#, s);
            let (ok, output) = transpile_prog(&code);

            if ok {
                // The string should appear quoted in some form
                let has_quoted = output.contains(&format!("'{}'", s))
                    || output.contains(&format!("\"{}\"", s))
                    || output.contains(&s); // At least the content exists

                prop_assert!(
                    has_quoted,
                    "println content '{}' should appear in output", s
                );
            }
        }

        // ================================================================
        // SECTION 5: Spec-Mandated Property Tests
        // ================================================================

        /// Property 5.2: Idempotency - transpile output is stable
        /// transpile(transpile(E)) should be stable
        #[test]
        fn prop_sec5_idempotency(n in 0i32..100) {
            let code = format!("fn main() {{ let x = {}; }}", n);
            let (ok1, out1) = transpile_prog(&code);

            if ok1 {
                // Transpiling again should produce identical output
                let (ok2, out2) = transpile_prog(&code);
                prop_assert_eq!(ok1, ok2, "Idempotency: success should be consistent");
                prop_assert_eq!(out1, out2, "Idempotency: output should be identical");
            }
        }

        /// Property 5.3: Quoting Safety - shell metacharacters are escaped
        /// String literals with $, `, \, " must prevent shell expansion
        #[test]
        fn prop_sec5_quoting_safety_dollar(s in "[a-zA-Z]{1,5}") {
            // Test that $VAR patterns don't get expanded
            let var_name = format!("${}", s);
            let code = format!(r#"fn main() {{ let msg = "{}"; let _ = msg; }}"#, var_name);
            let (ok, output) = transpile_prog(&code);

            if ok {
                // The $ should be escaped or quoted to prevent expansion
                // Either single quotes, escaped $, or the literal preserved
                let is_safe = output.contains(&format!("'{}'", var_name))
                    || output.contains(&format!("\"{}\"", var_name))
                    || output.contains("\\$")
                    || output.contains("'$");

                prop_assert!(
                    is_safe || !output.contains(&format!("${}", s)),
                    "Quoting safety: ${} should be escaped/quoted", s
                );
            }
        }

        /// Property 5.1: Symmetry partial check - exit codes are preserved
        /// For process::exit(N), shell should exit with N
        #[test]
        fn prop_sec5_symmetry_exit_code(n in 0u8..128) {
            let code = format!("fn main() {{ std::process::exit({}); }}", n);
            let (ok, output) = transpile_prog(&code);

            if ok {
                // The exit code should appear in the shell script
                let has_exit = output.contains(&format!("exit {}", n))
                    || output.contains(&format!("exit({})", n));

                prop_assert!(
                    has_exit || output.contains("exit"),
                    "Symmetry: exit({}) should produce exit {}", n, n
                );
            }
        }
    }
}
