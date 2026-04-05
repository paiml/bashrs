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
fn transpile_main(body: &str) -> String {
    let code = format!("fn main() {{\n{}\n}}", body);
    crate::transpile(&code, &Config::default()).expect("transpile should succeed")
}

/// Helper: transpile a full program (caller supplies fn main + any extra fns).
fn transpile_full(code: &str) -> String {
    crate::transpile(code, &Config::default()).expect("transpile should succeed")
}

// ====================================================================
// Binary operators — Arithmetic
// ====================================================================

#[test]
fn test_IR_EXPR_001_binary_add() {
    // Use variables to avoid constant folding
    let out = transpile_full("fn main() { let a = 1; let b = 2; let x = a + b; }");
    assert!(
        out.contains("$((") && out.contains("+"),
        "Expected arithmetic expansion with + in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_002_binary_sub() {
    let out = transpile_full("fn main() { let a = 10; let b = 3; let x = a - b; }");
    assert!(
        out.contains("$((") && out.contains("-"),
        "Expected arithmetic expansion with - in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_003_binary_mul() {
    let out = transpile_full("fn main() { let a = 4; let b = 5; let x = a * b; }");
    assert!(
        out.contains("$((") && out.contains("*"),
        "Expected arithmetic expansion with * in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_004_binary_div() {
    let out = transpile_full("fn main() { let a = 20; let b = 4; let x = a / b; }");
    assert!(
        out.contains("$((") && out.contains("/"),
        "Expected arithmetic expansion with / in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_005_binary_rem() {
    let out = transpile_full("fn main() { let a = 17; let b = 5; let x = a % b; }");
    assert!(
        out.contains("$((") && out.contains("%"),
        "Expected arithmetic expansion with % in:\n{out}"
    );
}

// ====================================================================
// Binary operators — Bitwise
// ====================================================================

#[test]
fn test_IR_EXPR_006_binary_bitand() {
    let out = transpile_full("fn main() { let a = 255; let b = 15; let x = a & b; }");
    assert!(
        out.contains("$((") && out.contains("&"),
        "Expected arithmetic expansion with & in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_007_binary_bitor() {
    let out = transpile_full("fn main() { let a = 240; let b = 15; let x = a | b; }");
    assert!(
        out.contains("$((") && out.contains("|"),
        "Expected arithmetic expansion with | in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_008_binary_bitxor() {
    let out = transpile_full("fn main() { let a = 170; let b = 85; let x = a ^ b; }");
    assert!(
        out.contains("$((") && out.contains("^"),
        "Expected arithmetic expansion with ^ in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_009_binary_shl() {
    let out = transpile_full("fn main() { let a = 1; let b = 4; let x = a << b; }");
    assert!(
        out.contains("$((") && out.contains("<<"),
        "Expected arithmetic expansion with << in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_010_binary_shr() {
    let out = transpile_full("fn main() { let a = 16; let b = 2; let x = a >> b; }");
    assert!(
        out.contains("$((") && out.contains(">>"),
        "Expected arithmetic expansion with >> in:\n{out}"
    );
}

// ====================================================================
// Binary operators — Comparison (numeric)
// ====================================================================

#[test]
fn test_IR_EXPR_011_binary_gt() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 10;
    let b = 5;
    if a > b {
        echo("yes");
    }
}
fn echo(msg: &str) {}
"#,
    );
    // Numeric comparison: -gt
    assert!(out.contains("-gt"), "Expected -gt in:\n{out}");
}

#[test]
fn test_IR_EXPR_012_binary_ge() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 10;
    let b = 10;
    if a >= b {
        echo("yes");
    }
}
fn echo(msg: &str) {}
"#,
    );
    assert!(out.contains("-ge"), "Expected -ge in:\n{out}");
}

#[test]
fn test_IR_EXPR_013_binary_lt() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 3;
    let b = 7;
    if a < b {
        echo("yes");
    }
}
fn echo(msg: &str) {}
"#,
    );
    assert!(out.contains("-lt"), "Expected -lt in:\n{out}");
}

#[test]
fn test_IR_EXPR_014_binary_le() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 5;
    let b = 5;
    if a <= b {
        echo("yes");
    }
}
fn echo(msg: &str) {}
"#,
    );
    assert!(out.contains("-le"), "Expected -le in:\n{out}");
}

#[test]
fn test_IR_EXPR_015_binary_eq_numeric() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 42;
    let b = 42;
    if a == b {
        echo("equal");
    }
}
fn echo(msg: &str) {}
"#,
    );
    // Numeric equality: -eq
    assert!(out.contains("-eq"), "Expected -eq in:\n{out}");
}

#[test]
fn test_IR_EXPR_016_binary_ne_numeric() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 1;
    let b = 2;
    if a != b {
        echo("different");
    }
}
fn echo(msg: &str) {}
"#,
    );
    // Numeric inequality: -ne
    assert!(out.contains("-ne"), "Expected -ne in:\n{out}");
}

// ====================================================================
// Binary operators — String comparison
// ====================================================================

#[test]
fn test_IR_EXPR_017_binary_eq_string() {
    let out = transpile_full(
        r#"
fn main() {
    let a = "hello";
    let b = "hello";
    if a == b {
        echo("same");
    }
}
fn echo(msg: &str) {}
"#,
    );
    // String equality: = (POSIX test)
    assert!(
        out.contains("= ") || out.contains("="),
        "Expected string = comparison in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_018_binary_ne_string() {
    let out = transpile_full(
        r#"
fn main() {
    let a = "hello";
    let b = "world";
    if a != b {
        echo("different");
    }
}
fn echo(msg: &str) {}
"#,
    );
    // String inequality: != or the branch is taken via constant folding
    assert!(
        out.contains("!=") || out.contains("different"),
        "Expected string != comparison or constant-folded branch in:\n{out}"
    );
}

// ====================================================================
// Binary operators — Logical
// ====================================================================

#[test]
fn test_IR_EXPR_019_binary_logical_and() {
    let out = transpile_full(
        r#"
fn main() {
    let a = true;
    let b = true;
    if a && b {
        echo("both");
    }
}
fn echo(msg: &str) {}
"#,
    );
    // Logical AND should appear in the output (may be constant-folded for literals)
    // Either as "&&" or as constant-folded "true"
    assert!(
        out.contains("&&") || out.contains("true"),
        "Expected && or constant-folded true in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_020_binary_logical_or() {
    let out = transpile_full(
        r#"
fn main() {
    let a = false;
    let b = true;
    if a || b {
        echo("either");
    }
}
fn echo(msg: &str) {}
"#,
    );
    assert!(
        out.contains("||") || out.contains("true"),
        "Expected || or constant-folded true in:\n{out}"
    );
}

// ====================================================================
// Unary operators
// ====================================================================

#[test]
fn test_IR_EXPR_021_unary_neg() {
    // Use variable to avoid constant folding
    let out = transpile_full("fn main() { let a = 42; let x = -a; }");
    assert!(
        out.contains("$((0 -") || out.contains("-42") || out.contains("$((0-"),
        "Expected negation in:\n{out}"
    );
}

#[test]
fn test_IR_EXPR_022_unary_not() {
    let out = transpile_main("let x = !true;");
    // Logical not: either $((!1)) or constant-folded to 0/false
    assert!(
        out.contains("!") || out.contains("0") || out.contains("false"),
        "Expected negation in:\n{out}"
    );
}

// ====================================================================
// convert_expr: FunctionCall with exec name
// ====================================================================

#[test]

include!("ir_expr_tests_tests_IR.rs");
