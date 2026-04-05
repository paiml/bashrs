//! Tests for `convert_binary_to_value` and `convert_expr` via the public
//! `transpile()` API.
//!
//! Covers all 18 BinaryOp variants, both UnaryOp variants, and the major
//! `convert_expr` branches (FunctionCall with various names, Variable,
//! Literal, Binary, Unary, Index, MethodCall, IfExpr).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::Config;

/// Helper: transpile a snippet wrapped in `fn main() { ... }` and return the
/// shell output. Panics on transpilation failure.
#[test]
fn test_IR_EXPR_032_format_concat_via_println() {
    let out = transpile_full(
        r#"
fn main() {
    let name = "World";
    println!("Hello {}", name);
}
"#,
    );
    // println! with format args produces echo with concatenation
    assert!(
        out.contains("echo") || out.contains("printf"),
        "Expected echo or printf for println! in:\n{out}"
    );
    assert!(
        out.contains("Hello") || out.contains("hello"),
        "Expected Hello in output:\n{out}"
    );
}

// ====================================================================
// Compound: nested arithmetic
// ====================================================================

#[test]
fn test_IR_EXPR_033_nested_arithmetic() {
    // Use variables to avoid constant folding
    let out = transpile_full("fn main() { let a = 1; let b = 2; let c = 3; let x = (a + b) * c; }");
    assert!(
        out.contains("$(("),
        "Expected arithmetic expansion in:\n{out}"
    );
}

// ====================================================================
// Compound: multiple binary ops in sequence
// ====================================================================

#[test]
fn test_IR_EXPR_034_multiple_binary_ops() {
    let out = transpile_full(
        r#"
fn main() {
    let x = 10;
    let y = 5;
    let a = x + y;
    let p = 20;
    let q = 3;
    let b = p - q;
    let c = a * b;
}
"#,
    );
    assert!(
        out.contains("$(("),
        "Expected arithmetic expansion in:\n{out}"
    );
}

// ====================================================================
// analyze_command_effects
// ====================================================================

#[test]
fn test_IR_EXPR_035_exec_effects_curl() {
    // Ensure transpile succeeds for exec("curl ...")
    let out = transpile_main(r#"exec("curl http://example.com");"#);
    assert!(out.contains("eval"), "Expected eval for exec() in:\n{out}");
}

// ====================================================================
// convert_expr: method call (falls through to convert_method_call_to_value)
// ====================================================================

#[test]
fn test_IR_EXPR_036_method_call_expr() {
    // Method calls on variables produce "unknown" in value context,
    // but the transpiler should not fail
    let result = crate::transpile(
        r#"
fn main() {
    let s = "hello";
    let t = s.len();
}
"#,
        &Config::default(),
    );
    // The transpiler should handle this without error
    assert!(
        result.is_ok(),
        "Method call should not cause transpile failure"
    );
}

// ====================================================================
// Edge case: division by literal zero still transpiles
// ====================================================================

#[test]
fn test_IR_EXPR_037_div_by_zero_literal() {
    // The transpiler generates shell code; runtime division by zero is a shell concern.
    // Constant folding may catch this or pass it through
    let result = crate::transpile(
        "fn main() { let a = 10; let b = 0; let x = a / b; }",
        &Config::default(),
    );
    assert!(result.is_ok(), "Division by zero should still transpile");
}

// ====================================================================
// Edge case: chained shifts
// ====================================================================

#[test]
fn test_IR_EXPR_038_chained_shift() {
    let out = transpile_full(
        "fn main() { let a = 1; let b = 2; let c = 3; let x = a << b; let y = x << c; }",
    );
    assert!(out.contains("<<"), "Expected << operator in:\n{out}");
}

// ====================================================================
// Combination: comparison used in while loop
// ====================================================================

#[test]
fn test_IR_EXPR_039_comparison_in_while() {
    let out = transpile_full(
        r#"
fn main() {
    let mut i = 0;
    while i < 10 {
        i = i + 1;
    }
}
"#,
    );
    assert!(out.contains("while"), "Expected while in:\n{out}");
    assert!(
        out.contains("-lt"),
        "Expected -lt in while condition:\n{out}"
    );
}

// ====================================================================
// Edge case: boolean false literal
// ====================================================================

#[test]
fn test_IR_EXPR_040_literal_bool_false() {
    let out = transpile_main("let flag = false;");
    // false is emitted as 0 in shell
    assert!(
        out.contains("0") || out.contains("false"),
        "Expected boolean false representation in:\n{out}"
    );
}
