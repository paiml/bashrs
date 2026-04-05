//! Coverage tests for services/parser.rs — targeting uncovered branches
//!
//! Focuses on:
//! - convert_type: Reference, Tuple, Array types
//! - convert_path_type: Result<>, Option<>, unknown paths
//! - convert_let_stmt: tuple destructuring, type-annotated patterns
//! - convert_assign_stmt: array index, field, deref, nested index
//! - convert_compound_assign_stmt: all operators, array index target, field target, deref target
//! - convert_macro_expr: format!, vec!, println! in expr position
//! - split_macro_args: nested parens, brackets, braces, strings
//! - parse_format_string: escaped braces, format specifiers
//! - convert_for_loop: wildcard pattern
//! - convert_match_stmt: block body, guard expressions
//! - convert_pattern: TupleStruct (Some, Ok, Err, None), Path (None)
//! - extract_pattern_literal: negative int
//! - convert_range_pattern: inclusive/exclusive
//! - convert_block_expr / convert_let_expr / convert_struct_expr / convert_tuple_expr
//! - convert_repeat_expr
//! - process_item: impl blocks, struct/enum/use/const/static/type/trait skipped

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::services::parser::parse;

fn parse_ok(code: &str) {
    let result = parse(code);
    assert!(
        result.is_ok(),
        "Expected parse OK for code, got: {:?}",
        result.err()
    );
}

fn parse_err(code: &str) {
    let result = parse(code);
    assert!(result.is_err(), "Expected parse error for code");
}

// ---------------------------------------------------------------------------
// Type conversion: various types
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_001_type_reference_str() {
    let code = r#"
        fn greet(msg: &str) {
            println!("{}", msg);
        }
        fn main() { greet("hi"); }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_002_type_result() {
    let code = r#"
        fn try_it() -> Result<String, String> {
            return "ok";
        }
        fn main() { let r = try_it(); }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_003_type_option() {
    let code = r#"
        fn maybe() -> Option<String> {
            return "val";
        }
        fn main() { let o = maybe(); }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_004_type_tuple() {
    let code = r#"
        fn pair() -> (i32, i32) {
            return (1, 2);
        }
        fn main() { let p = pair(); }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_005_type_array() {
    let code = r#"
        fn get_arr() -> [i32; 3] {
            return [1, 2, 3];
        }
        fn main() { let a = get_arr(); }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_006_type_u16() {
    let code = r#"
        fn port() -> u16 { 8080 }
        fn main() { let p = port(); }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Tuple destructuring in let
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_007_tuple_destructuring() {
    let code = r#"
        fn main() {
            let (a, b) = (1, 2);
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Type-annotated let binding
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_008_type_annotated_let() {
    let code = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Assignment targets: array index, field, deref
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_009_assign_array_index() {
    let code = r#"
        fn main() {
            let arr = [0, 1, 2];
            arr[0] = 99;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_010_assign_field() {
    let code = r#"
        fn main() {
            let s = "hello";
            self.value = 42;
        }
    "#;
    // self.value assignment strips receiver
    parse_ok(code);
}

#[test]
fn test_SPCOV_011_assign_deref() {
    let code = r#"
        fn main() {
            let x = 5;
            *x = 10;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Compound assignment operators
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_012_compound_add_assign() {
    let code = r#"
        fn main() {
            let x = 5;
            x += 3;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_013_compound_sub_assign() {
    let code = r#"
        fn main() {
            let x = 10;
            x -= 3;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_014_compound_mul_assign() {
    let code = r#"
        fn main() {
            let x = 5;
            x *= 2;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_015_compound_div_assign() {
    let code = r#"
        fn main() {
            let x = 10;
            x /= 2;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_016_compound_rem_assign() {
    let code = r#"
        fn main() {
            let x = 10;
            x %= 3;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_017_compound_bitand_assign() {
    let code = r#"
        fn main() {
            let x = 0xFF;
            x &= 0x0F;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_018_compound_bitor_assign() {
    let code = r#"
        fn main() {
            let x = 0x0F;
            x |= 0xF0;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_019_compound_bitxor_assign() {
    let code = r#"
        fn main() {
            let x = 0xFF;
            x ^= 0x0F;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_020_compound_shl_assign() {
    let code = r#"
        fn main() {
            let x = 1;
            x <<= 4;
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_021_compound_shr_assign() {
    let code = r#"
        fn main() {
            let x = 16;
            x >>= 2;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Compound assignment on array index
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_022_compound_assign_array_index() {
    let code = r#"
        fn main() {
            let arr = [1, 2, 3];
            arr[0] += 10;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Compound assignment on field
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_023_compound_assign_field() {
    let code = r#"
        fn main() {
            let s = "x";
            self.count += 1;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// format! macro in expression position
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_024_format_macro_expr() {
    let code = r#"
        fn main() {
            let name = "world";
            let msg = format!("hello {}", name);
        }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_025_format_macro_escaped_braces() {
    let code = r#"
        fn main() {
            let msg = format!("{{escaped}} and {}", "arg");
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// vec! macro
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_026_vec_macro() {
    let code = r#"
        fn main() {
            let v = vec![1, 2, 3];
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// println! in expression position
// ---------------------------------------------------------------------------

#[test]

include!("parser_coverage_tests5_incl2.rs");
