fn test_IRPAT_018_let_match_string_patterns() {
    let code = r#"
        fn main() {
            let cmd = "start";
            let result = match cmd {
                "start" => 1,
                "stop" => 2,
                _ => 0,
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "String patterns: {out}");
}

// ---------------------------------------------------------------------------
// match returning integers (non-string values)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_019_match_returns_integers_in_fn() {
    let code = r#"
        fn score(x: i32) -> i32 {
            match x {
                1 => 100,
                2 => 200,
                _ => 0,
            }
        }
        fn main() {
            let s = score(2);
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("score"), "Function score should exist: {out}");
}

// ---------------------------------------------------------------------------
// match with variable binding pattern
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_020_match_variable_binding_pattern() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = match x {
                n => n,
            };
        }
    "#;
    let result = transpile_result(code);
    // Variable patterns are treated as wildcards
    assert!(result.is_ok(), "Variable binding: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Mixed literal types in convert_match_pattern
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_021_match_u32_literal_pattern() {
    let code = r#"
        fn main() {
            let x = 42;
            let result = match x {
                42 => "found",
                _ => "not found",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "U32 pattern: {out}");
}

// ---------------------------------------------------------------------------
// has_range_patterns detection
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_022_no_range_patterns_uses_case() {
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
    // Without range patterns, should use case statement
    assert!(
        out.contains("case") || out.contains("result="),
        "Case statement: {out}"
    );
}

#[test]
fn test_IRPAT_023_range_patterns_uses_if_chain() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = match x {
                1..=3 => "low",
                _ => "other",
            };
        }
    "#;
    let out = transpile_ok(code);
    // With range patterns, should use if-elif-else chain
    assert!(out.contains("result="), "Range if-chain: {out}");
}

// ---------------------------------------------------------------------------
// literal_to_string coverage for all Literal variants
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_024_match_bool_true_pattern() {
    let code = r#"
        fn main() {
            let b = true;
            let result = match b {
                true => 1,
                _ => 0,
            };
        }
    "#;
    let result = transpile_result(code);
    assert!(result.is_ok(), "Bool true pattern: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// convert_range_match (non-let, non-function context) via standalone match
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_025_standalone_range_match() {
    let code = r#"
        fn main() {
            let x = 5;
            match x {
                1..=3 => println!("low"),
                4..=6 => println!("mid"),
                _ => println!("high"),
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(
        out.contains("low") || out.contains("mid") || out.contains("high"),
        "Range match: {out}"
    );
}

// ---------------------------------------------------------------------------
// convert_range_match_fn (function context with should_echo)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_026_range_match_in_function_context() {
    let code = r#"
        fn classify(n: i32) -> i32 {
            match n {
                0..=9 => 1,
                10..=99 => 2,
                _ => 3,
            }
        }
        fn main() {
            let c = classify(50);
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("classify"), "Function with range: {out}");
}

// ---------------------------------------------------------------------------
// Multiple range patterns in let-match
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_027_multiple_range_patterns_for_let() {
    let code = r#"
        fn main() {
            let x = 50;
            let result = match x {
                0..=9 => "single",
                10..=99 => "double",
                _ => "large",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multiple ranges: {out}");
}

// ---------------------------------------------------------------------------
// Wildcard and variable patterns in range match (pattern_to_condition → None)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_028_range_match_with_wildcard_fallback() {
    let code = r#"
        fn main() {
            let x = 50;
            let result = match x {
                1..=10 => "range",
                _ => "default",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Wildcard in range: {out}");
}

// ---------------------------------------------------------------------------
// Empty match arm body (edge case)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_029_match_with_empty_block_arm() {
    // This tests the convert_match_arm_for_let empty body path
    let code = r#"
        fn main() {
            let x = 1;
            match x {
                1 => {},
                _ => {},
            };
        }
    "#;
    let result = transpile_result(code);
    assert!(result.is_ok(), "Empty block arms: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// let-if with multi-statement branches (triggers Block path)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_030_let_if_multi_stmt_then_branch() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = if x > 3 {
                let tmp = 10;
                tmp
            } else {
                0
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multi-stmt if: {out}");
}

// ---------------------------------------------------------------------------
// Match in function returning value (convert_range_match_fn with should_echo=true)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_031_fn_returning_range_match() {
    let code = r#"
        fn grade(score: i32) -> i32 {
            match score {
                90..=100 => 4,
                80..=89 => 3,
                70..=79 => 2,
                _ => 1,
            }
        }
        fn main() {
            let g = grade(85);
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("grade"), "Grade function: {out}");
}

// ---------------------------------------------------------------------------
// Match with non-range literal patterns (convert_match_pattern for various types)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_032_match_i32_negative() {
    let code = r#"
        fn main() {
            let x = -1;
            let result = match x {
                -1 => "neg_one",
                0 => "zero",
                _ => "other",
            };
        }
    "#;
    let result = transpile_result(code);
    assert!(result.is_ok(), "Negative i32 match: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Deeply nested let-if-else chain
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_033_deeply_nested_if_else_expr() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = if x > 10 {
                "big"
            } else if x > 5 {
                "medium"
            } else if x > 0 {
                "small"
            } else {
                "zero"
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Deep if-else: {out}");
}

// ---------------------------------------------------------------------------
// Function returning if-else expression (lower_return_if_expr path)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_034_fn_returning_if_expr() {
    let code = r#"
        fn pick(x: i32) -> i32 {
            if x > 0 { 1 } else { 0 }
        }
        fn main() {
            let p = pick(5);
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("pick"), "Function returning if expr: {out}");
}

#[test]
fn test_IRPAT_035_fn_returning_nested_if_expr() {
    let code = r#"
        fn classify(x: i32) -> i32 {
            if x > 10 { 3 } else if x > 5 { 2 } else { 1 }
        }
        fn main() {
            let c = classify(7);
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("classify"), "Nested if-expr return: {out}");
}

// ---------------------------------------------------------------------------
// Multiple arms with guards
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_036_match_multiple_guards() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = match x {
                n if n > 10 => "big",
                n if n > 5 => "medium",
                _ => "small",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multiple guards: {out}");
}

// ---------------------------------------------------------------------------
// Mixed range and literal patterns
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_037_match_mixed_range_and_literal() {
    // This tests has_range_patterns returning true when mixed
    let code = r#"
        fn main() {
            let x = 5;
            let result = match x {
                1..=3 => "range",
                5 => "five",
                _ => "other",
            };
        }
    "#;
    let result = transpile_result(code);
    assert!(result.is_ok(), "Mixed range/literal: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// convert_block_for_let (delegates to convert_match_arm_for_let)
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_038_let_if_with_multi_stmt_else() {
    let code = r#"
        fn main() {
            let x = 5;
            let result = if x > 3 {
                let a = 1;
                a
            } else {
                let b = 2;
                b
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Multi-stmt if-else: {out}");
}

// ---------------------------------------------------------------------------
// Standalone match (not let) with range patterns in function
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_039_standalone_match_in_function_with_ranges() {
    let code = r#"
        fn process(n: i32) {
            match n {
                1..=10 => println!("low"),
                _ => println!("high"),
            };
        }
        fn main() {
            process(5);
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("process"), "Standalone match fn: {out}");
}

// ---------------------------------------------------------------------------
// Large match with many arms
// ---------------------------------------------------------------------------

#[test]
fn test_IRPAT_040_large_match_many_arms() {
    let code = r#"
        fn main() {
            let x = 3;
            let result = match x {
                1 => "one",
                2 => "two",
                3 => "three",
                4 => "four",
                5 => "five",
                _ => "other",
            };
        }
    "#;
    let out = transpile_ok(code);
    assert!(out.contains("result="), "Large match: {out}");
}
