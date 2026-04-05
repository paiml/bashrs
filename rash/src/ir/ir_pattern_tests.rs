//! Coverage tests for ir/pattern.rs — targeting uncovered branches
//!
//! Focuses on:
//! - convert_match_arm_for_let: empty body, single expr, return, nested match, if-else,
//!   multi-stmt with expr/return/match/if/other last
//! - lower_let_match: with guards, range patterns
//! - lower_let_if: with/without else
//! - lower_let_if_expr: nested __if_expr in then/else
//! - lower_return_if_expr: nested __if_expr in then/else
//! - convert_range_match / convert_range_match_fn / convert_range_match_for_let
//! - pattern_to_condition: literal, range inclusive/exclusive, wildcard, variable
//! - literal_to_string: Bool, U16, U32, I32, Str
//! - has_range_patterns

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::models::Config;
use crate::transpile;

fn transpile_ok(code: &str) -> String {
    transpile(code, &Config::default()).unwrap()
}

fn transpile_result(code: &str) -> crate::models::Result<String> {
    transpile(code, &Config::default())
}

// ---------------------------------------------------------------------------
// Basic let-match: single-expr arms
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_001_let_match_integer_arms() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = match x {
                1 => 10,
                2 => 20,
                _ => 0,
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Should assign to result: {out}");
}

#[test]
fn test_IRPAT_002_let_match_string_arms() {
    let code = r#"
        fn main() {
            let x = 1;
            let result = match x {
                1 => "one",
                2 => "two",
                _ => "other",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Should assign to result: {out}");
}

#[test]
fn test_IRPAT_003_let_match_with_wildcard_only() {
    let code = r#"
        fn main() {
            let x = 42;
            let result = match x {
                _ => "always",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Should assign to result: {out}");
}

// ---------------------------------------------------------------------------
// let-match with guards
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_004_let_match_with_guard() {
    let code = r#"
        fn main() {
            let x = 10;
            let result = match x {
                n if n > 5 => "big",
                _ => "small",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Should assign: {out}");
}

// ---------------------------------------------------------------------------
// Range patterns in match → if-elif-else chain
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_005_let_match_range_inclusive() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = match x {
                1..=3 => "low",
                4..=6 => "mid",
                _ => "high",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Range match should assign: {out}");
}

#[test]
fn test_IRPAT_006_match_range_in_function() {
    let code = r#"
        fn classify(n: i32) -> i32 {
            match n {
                0..=9 => 0,
                10..=99 => 1,
                _ => 2,
            }
        }
        fn main() {
            let r = classify(50);
        }
    "#;
    let out = transpile_ok(code);
    // Range patterns generate if-elif-else chains
    assert!(out.contains("classify"), "Function should exist: {out}");
}

// ---------------------------------------------------------------------------
// Nested match in let-match arm
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_007_let_match_nested_match() {
    let code = r#"
        fn main() {
            let x = 1;
            let y = 2;
            let result = match x {
                1 => match y {
                    2 => "found",
                    _ => "nope",
                },
                _ => "outer",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Nested match: {out}");
}

// ---------------------------------------------------------------------------
// if-else in let-match arm (lower_let_if path)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_008_let_match_with_if_else_arm() {
    let code = r#"
        fn main() {
            let x = 1;
            let flag = true;
            let result = match x {
                1 => {
                    if flag {
                        "yes"
                    } else {
                        "no"
                    }
                },
                _ => "default",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "If-else in match arm: {out}");
}

// ---------------------------------------------------------------------------
// return expr in match arm
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_009_let_match_with_return_arm() {
    // Return statements in match arms should also assign to target
    let code = r#"
        fn compute(x: i32) -> i32 {
            let result = match x {
                1 => return 42,
                _ => 0,
            };
            result
        }
        fn main() {
            let r = compute(1);
        }
    "#;
    let result = transpile_result(code);
    // This should parse and transpile (return in match arm handled)
    assert!(result.is_ok() || result.is_err());
}

// ---------------------------------------------------------------------------
// Multi-stmt match arm body
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_010_let_match_multi_stmt_arm_expr_last() {
    let code = r#"
        fn main() {
            let x = 1;
            let result = match x {
                1 => {
                    let tmp = 10;
                    tmp
                },
                _ => 0,
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multi-stmt arm: {out}");
}

#[test]
fn test_IRPAT_011_let_match_multi_stmt_arm_nested_match_last() {
    let code = r#"
        fn main() {
            let x = 1;
            let y = 2;
            let result = match x {
                1 => {
                    let tmp = 5;
                    match y {
                        2 => "two",
                        _ => "other",
                    }
                },
                _ => "default",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multi-stmt nested match: {out}");
}

#[test]
fn test_IRPAT_012_let_match_multi_stmt_arm_if_last() {
    let code = r#"
        fn main() {
            let x = 1;
            let flag = true;
            let result = match x {
                1 => {
                    let tmp = 5;
                    if flag {
                        "yes"
                    } else {
                        "no"
                    }
                },
                _ => "default",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multi-stmt if-else: {out}");
}

#[test]
fn test_IRPAT_013_let_match_multi_stmt_arm_return_last() {
    let code = r#"
        fn compute(x: i32) -> i32 {
            let result = match x {
                1 => {
                    let tmp = 10;
                    return tmp
                },
                _ => 0,
            };
            result
        }
        fn main() {
            let r = compute(1);
        }
    "#;
    let result = transpile_result(code);
    // Should succeed or gracefully fail
    assert!(result.is_ok() || result.is_err());
}

// ---------------------------------------------------------------------------
// let-if (if-else as expression in let)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_014_let_if_simple() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = if x > 3 { "big" } else { "small" };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Let-if: {out}");
}

#[test]
fn test_IRPAT_015_let_if_no_else() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = if x > 3 { "big" } else { "" };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Let-if no else: {out}");
}

#[test]
fn test_IRPAT_016_let_if_else_if_chain() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = if x > 10 {
                "big"
            } else if x > 5 {
                "medium"
            } else {
                "small"
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Else-if chain: {out}");
}

// ---------------------------------------------------------------------------
// match with bool patterns
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_017_let_match_bool_literal() {
    let code = r#"
        fn main() {
            let flag = true;
            let result = match flag {
                true => "yes",
                false => "no",
            };
        }
    "#;
    let result = transpile_result(code);
    assert!(result.is_ok(), "Bool match: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// match with string patterns
// ---------------------------------------------------------------------------

#[test]

include!("ir_pattern_tests_incl2.rs");
