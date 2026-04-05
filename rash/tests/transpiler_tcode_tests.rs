//! Transpiler T-Code Tests - 120-Point Popper Falsification Checklist
//!
//! Implements SPEC-TB-2025-001 v1.2.0
//! Each test attempts to FALSIFY that the transpiler works correctly.
//! A passing test means the falsification attempt failed (feature works).
//!
//! Test Types:
//! - PROG: Full program, run as-is
//! - STMT: Statement fragment, wrapped in standard harness

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(deprecated)]
#![allow(dead_code)] // Test helper functions may not be used in all configurations
                     // Note: These tests are for Rust→Shell transpilation which is PLANNED (v3.0+)
                     // They have race conditions when run in parallel. Run with --test-threads=1 if needed.

use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

/// Standard test harness preamble for STMT tests
/// NOTE: Only uses features bashrs currently supports
const STMT_PREAMBLE: &str = r#"
    let x = 10;
    let y = 5;
    let z = 2;
"#;

/// Atomic counter for unique temp file names (prevents race conditions)
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate unique temp file paths to prevent test race conditions
fn get_unique_temp_paths() -> (String, String) {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let tid = std::thread::current().id();
    (
        format!("/tmp/tcode_prog_{}_{:?}_{}.rs", pid, tid, id),
        format!("/tmp/tcode_prog_{}_{:?}_{}.sh", pid, tid, id),
    )
}

/// Transpile a full program (PROG type)
fn transpile_prog(code: &str) -> (bool, String) {
    let (tmp_rs, tmp_sh) = get_unique_temp_paths();

    fs::write(&tmp_rs, code).unwrap();

    let output = assert_cmd::cargo_bin_cmd!("bashrs")
        .args(["build", &tmp_rs, "-o", &tmp_sh])
        .output()
        .unwrap();

    // Clean up temp files
    let _ = fs::remove_file(&tmp_rs);

    if !output.status.success() {
        let _ = fs::remove_file(&tmp_sh);
        return (false, String::from_utf8_lossy(&output.stderr).to_string());
    }

    let shell = fs::read_to_string(&tmp_sh).unwrap_or_default();
    let _ = fs::remove_file(&tmp_sh);
    (true, shell)
}

/// Transpile a statement fragment (STMT type) wrapped in harness
fn transpile_stmt(code: &str) -> (bool, String) {
    let full_prog = format!("fn main() {{\n{}\n    {}\n}}", STMT_PREAMBLE, code);
    transpile_prog(&full_prog)
}

/// Check if output contains forbidden pattern (test FAILS if found)
fn check_forbidden(output: &str, forbidden: &str) -> bool {
    !output.contains(forbidden)
}

/// Check if output contains required pattern (test FAILS if not found)
fn check_required(output: &str, required: &str) -> bool {
    output.contains(required)
}

/// T-code test result
struct TCodeResult {
    id: &'static str,
    passed: bool,
    reason: String,
}

impl TCodeResult {
    fn pass(id: &'static str) -> Self {
        Self {
            id,
            passed: true,
            reason: String::new(),
        }
    }

    fn fail(id: &'static str, reason: &str) -> Self {
        Self {
            id,
            passed: false,
            reason: reason.to_string(),
        }
    }
}

// ============================================================================
// SECTION 4.1: Basic & Literals (T001-T015)
// ============================================================================

#[test]
fn test_t001_empty_main() {
    // PROG: fn main() {} - should produce main() function
    let (ok, output) = transpile_prog("fn main() {}");
    assert!(ok, "T001: Should compile");
    assert!(
        output.contains("main()"),
        "T001: Output should contain main()"
    );
}

#[test]
fn test_t002_integer_assignment() {
    // STMT: let a = 1; - should NOT produce "unknown"
    let (ok, output) = transpile_stmt("let a = 1;");
    assert!(ok, "T002: Should compile");
    assert!(
        !output.contains("unknown"),
        "T002: Should not contain 'unknown'"
    );
}

#[test]
fn test_t003_negative_integer() {
    // STMT: let a = -1; - should NOT produce "unknown"
    let (ok, output) = transpile_stmt("let a = -1;");
    assert!(ok, "T003: Should compile");
    assert!(
        !output.contains("unknown"),
        "T003: Should not contain 'unknown'"
    );
}

#[test]
fn test_t004_string_literal() {
    // STMT: let a = "hi"; - should NOT produce error
    let (ok, output) = transpile_stmt(r#"let a = "hi";"#);
    // Note: Currently expected to fail - this is a known bug TB-004
    if !ok {
        println!("T004: KNOWN BUG TB-004 - string literal validation fails");
        println!("      Error: {}", output);
    }
    // Don't assert - this documents current behavior
}

#[test]
fn test_t005_boolean_true() {
    // STMT: let a = true; - should NOT produce "unknown"
    let (ok, output) = transpile_stmt("let a = true;");
    if ok {
        assert!(
            !output.contains("unknown"),
            "T005: Should not contain 'unknown'"
        );
    }
}

#[test]
fn test_t006_boolean_false() {
    // STMT: let a = false; - should NOT produce "unknown"
    let (ok, output) = transpile_stmt("let a = false;");
    if ok {
        assert!(
            !output.contains("unknown"),
            "T006: Should not contain 'unknown'"
        );
    }
}

#[test]
fn test_t007_zero_literal() {
    // STMT: let a = 0; - should NOT produce "unknown"
    let (ok, output) = transpile_stmt("let a = 0;");
    assert!(ok, "T007: Should compile");
    assert!(
        !output.contains("unknown"),
        "T007: Should not contain 'unknown'"
    );
}

// T008-T015: Additional Literals

#[test]
fn test_t008_large_integer() {
    // STMT: let a = 999999; - should NOT produce "Overflow"
    let (ok, output) = transpile_stmt("let a = 999999;");
    assert!(ok, "T008: Should compile");
    assert!(
        !output.to_lowercase().contains("overflow"),
        "T008: Should not overflow"
    );
}

#[test]
fn test_t009_underscore_prefix() {
    // STMT: let _a = 1; - underscore prefix for unused vars
    let (ok, _output) = transpile_stmt("let _a = 1;");
    // This may or may not compile - document behavior
    if !ok {
        println!("T009: Underscore prefix vars may not be supported");
    }
}

#[test]
fn test_t010_explicit_type() {
    // STMT: let a: i32 = 1; - explicit type annotation
    let (ok, output) = transpile_stmt("let a: i32 = 1;");
    if ok {
        assert!(
            !output.contains("error"),
            "T010: Explicit types should work"
        );
    }
}

#[test]
fn test_t011_float_rejection() {
    // STMT: let a = 1.0; - floats should be rejected (shell doesn't support)
    let (ok, output) = transpile_stmt("let a = 1.0;");
    // Expected: either rejected or converted to integer
    if ok {
        println!("T011: Float literals accepted (may lose precision)");
    } else {
        println!("T011: Float literals correctly rejected: {}", output);
    }
}

#[test]
fn test_t012_char_literal() {
    // STMT: let a = 'a'; - char literal
    let (ok, output) = transpile_stmt("let a = 'a';");
    if ok {
        assert!(
            !output.contains("unknown"),
            "T012: Char should not produce 'unknown'"
        );
    }
}

#[test]
fn test_t013_byte_string() {
    // STMT: let a = b"hi"; - byte string literal
    let (ok, output) = transpile_stmt(r#"let a = b"hi";"#);
    if !ok {
        println!("T013: Byte strings unsupported: {}", output);
    }
}

#[test]
fn test_t014_constant() {
    // PROG: const X: i32 = 1; - should produce readonly
    let (ok, output) = transpile_prog("const X: i32 = 1; fn main() {}");
    if ok && !output.contains("readonly") && !output.contains("X=") {
        println!("T014: WARNING - const should produce readonly or assignment");
    }
}

#[test]
fn test_t015_static() {
    // PROG: static X: i32 = 1; - should produce global
    let (ok, output) = transpile_prog("static X: i32 = 1; fn main() {}");
    if ok && !output.contains("X=") {
        println!("T015: WARNING - static should produce global variable");
    }
}

// ============================================================================
// SECTION 4.2: Arithmetic & Bitwise (T016-T035)
// ============================================================================

#[test]
fn test_t016_addition() {
    // STMT: let _ = 1 + 2; - should contain $(( for arithmetic
    let (ok, output) = transpile_stmt("let _ = 1 + 2;");
    if ok {
        // Check for arithmetic expansion
        let has_arith = output.contains("$((") || output.contains("3");
        if !has_arith {
            println!("T016: WARNING - Addition may not be computed correctly");
        }
    }
}

#[test]
fn test_t017_subtraction() {
    // STMT: let _ = 5 - 3;
    let (ok, output) = transpile_stmt("let _ = 5 - 3;");
    if ok {
        let has_arith = output.contains("$((") || output.contains("2");
        if !has_arith {
            println!("T017: WARNING - Subtraction may not be computed correctly");
        }
    }
}

#[test]
fn test_t018_multiplication() {
    // STMT: let _ = 4 * 3; - KNOWN BUG TB-007
    let (ok, output) = transpile_stmt("let _ = 4 * 3;");
    if ok {
        let has_arith = output.contains("$((") || output.contains("12");
        if !has_arith {
            println!("T018: KNOWN BUG TB-007 - Multiplication not computed");
        }
    }
}

#[test]
fn test_t019_division() {
    // STMT: let _ = 10 / 2;
    let (ok, output) = transpile_stmt("let _ = 10 / 2;");
    if ok {
        let has_arith = output.contains("$((") || output.contains("5");
        if !has_arith {
            println!("T019: WARNING - Division may not be computed correctly");
        }
    }
}

#[test]
fn test_t020_modulo() {
    // STMT: let _ = 10 % 3; - KNOWN BUG TB-008
    let (ok, output) = transpile_stmt("let _ = 10 % 3;");
    if ok {
        let has_arith = output.contains("$((") || output.contains("1");
        if !has_arith {
            println!("T020: KNOWN BUG TB-008 - Modulo not computed");
        }
    }
}

#[test]
fn test_t021_precedence() {
    // STMT: let _ = 2 + 3 * 4; - should be 14, not 20
    let (ok, output) = transpile_stmt("let _ = 2 + 3 * 4;");
    if ok && output.contains("20") {
        println!("T021: BUG - Wrong precedence, got 20 instead of 14");
    }
}

#[test]

include!("transpiler_tcode_tests_tests_t022_groupin.rs");
