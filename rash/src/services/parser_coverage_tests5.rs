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
    assert!(result.is_ok(), "Expected parse OK for code, got: {:?}", result.err());
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
fn test_SPCOV_027_println_expr_position() {
    // Test println! as the last expression in a block (expr position)
    let code = r#"
        fn main() {
            let x = {
                println!("hello");
                42
            };
            println!("{}", x);
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// for loop with wildcard pattern
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_028_for_wildcard_pattern() {
    let code = r#"
        fn main() {
            for _ in 0..3 {
                println!("tick");
            }
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// while loop
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_029_while_loop() {
    let code = r#"
        fn main() {
            let x = 0;
            while x < 5 {
                x += 1;
            }
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// loop (infinite)
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_030_infinite_loop() {
    let code = r#"
        fn main() {
            loop {
                break;
            }
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// match with guards
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_031_match_with_guard() {
    let code = r#"
        fn main() {
            let x = 5;
            match x {
                n if n > 3 => println!("big"),
                _ => println!("small"),
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// match with block body
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_032_match_block_body() {
    let code = r#"
        fn main() {
            let x = 1;
            match x {
                1 => {
                    let y = 10;
                    println!("{}", y);
                },
                _ => println!("other"),
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Pattern: TupleStruct (Some, Ok, Err, None)
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_033_pattern_some() {
    let code = r#"
        fn main() {
            let x = "hello";
            match x {
                Some(v) => println!("got"),
                None => println!("none"),
            };
        }
    "#;
    // Some/None patterns handled in convert_pattern
    parse_ok(code);
}

#[test]
fn test_SPCOV_034_pattern_ok_err() {
    let code = r#"
        fn main() {
            let x = "hello";
            match x {
                Ok(v) => println!("ok"),
                Err(e) => println!("err"),
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Range patterns in match
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_035_range_pattern_inclusive() {
    let code = r#"
        fn main() {
            let x = 5;
            match x {
                1..=10 => println!("in range"),
                _ => println!("out"),
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Negative int in range pattern
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_036_range_pattern_negative() {
    let code = r#"
        fn main() {
            let x = -5;
            match x {
                -10..=-1 => println!("negative"),
                _ => println!("other"),
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// convert_block_expr (block in expression position)
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_037_block_expr() {
    let code = r#"
        fn main() {
            let x = {
                let a = 1;
                a
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// convert_repeat_expr
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_038_repeat_expr() {
    let code = r#"
        fn main() {
            let arr = [0; 5];
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// convert_tuple_expr
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_039_tuple_expr() {
    let code = r#"
        fn main() {
            let t = (1, 2, 3);
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// convert_struct_expr
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_040_struct_expr() {
    let code = r#"
        struct Point { x: i32, y: i32 }
        fn main() {
            let p = Point { x: 1, y: 2 };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Field access expression
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_041_field_access() {
    let code = r#"
        fn main() {
            let s = "hello";
            let x = s.len;
        }
    "#;
    // Field access converts to Index with literal 0
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Cast expression
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_042_cast_expr() {
    let code = r#"
        fn main() {
            let x = 5 as u32;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Closure expression
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_043_closure_expr() {
    let code = r#"
        fn main() {
            let f = |x| x;
            let r = f(5);
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// match in expression position
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_044_match_in_expr_position() {
    let code = r#"
        fn main() {
            let x = 1;
            let y = match x {
                1 => "one",
                _ => "other",
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// process_item: non-function items skipped
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_045_skip_struct_item() {
    let code = r#"
        struct Foo { x: i32 }
        fn main() { let f = 1; }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_046_skip_enum_item() {
    let code = r#"
        enum Color { Red, Green, Blue }
        fn main() { let c = 1; }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_047_skip_use_item() {
    let code = r#"
        use std::io;
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_048_skip_const_item() {
    let code = r#"
        const MAX: i32 = 100;
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_049_skip_static_item() {
    let code = r#"
        static COUNT: i32 = 0;
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_050_skip_type_alias() {
    let code = r#"
        type Num = i32;
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

#[test]
fn test_SPCOV_051_skip_trait_item() {
    let code = r#"
        trait Greet { fn hello(&self); }
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// process_item: impl block → methods extracted as functions
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_052_impl_block_methods() {
    let code = r#"
        struct Counter { value: i32 }
        impl Counter {
            fn increment(&self) {
                self.value += 1;
            }
        }
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Self receiver parameter skipped
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_053_self_receiver_skipped() {
    let code = r#"
        struct Foo {}
        impl Foo {
            fn bar(&self) {
                println!("bar");
            }
        }
        fn main() { let x = 1; }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// return without expression
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_054_return_void() {
    let code = r#"
        fn do_nothing() {
            return;
        }
        fn main() { do_nothing(); }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// eprintln! macro
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_055_eprintln_macro() {
    let code = r#"
        fn main() {
            eprintln!("error: {}", "bad");
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// print! macro (no newline)
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_056_print_macro() {
    let code = r#"
        fn main() {
            print!("no newline");
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Unsupported macro error
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_057_unsupported_macro_error() {
    let code = r#"
        fn main() {
            dbg!("test");
        }
    "#;
    parse_err(code);
}

// ---------------------------------------------------------------------------
// let-if expression: single-stmt branches → __if_expr
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_058_let_if_expr_simple() {
    let code = r#"
        fn main() {
            let x = 5;
            let r = if x > 3 { "big" } else { "small" };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// let-if expression: multi-stmt branches → Block path
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_059_let_if_expr_multi_stmt() {
    let code = r#"
        fn main() {
            let x = 5;
            let r = if x > 3 {
                let tmp = "big";
                tmp
            } else {
                "small"
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Index expression
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_060_index_expression() {
    let code = r#"
        fn main() {
            let arr = [1, 2, 3];
            let first = arr[0];
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Reference expression
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_061_reference_expression() {
    let code = r#"
        fn main() {
            let arr = [1, 2, 3];
            let r = &arr;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Deref expression in convert_unary_expr
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_062_deref_expression() {
    let code = r#"
        fn main() {
            let x = 5;
            let y = *x;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Negation of int literal → negative literal
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_063_neg_int_literal() {
    let code = r#"
        fn main() {
            let x = -42;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Negation of i32::MIN special case
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_064_neg_i32_min() {
    let code = r#"
        fn main() {
            let x = -2147483648;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Method call: std::env::args().collect()
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_065_positional_args() {
    let code = r#"
        fn main() {
            let args: Vec<String> = std::env::args().collect();
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Bool literal
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_066_bool_literal() {
    let code = r#"
        fn main() {
            let t = true;
            let f = false;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// u16 literal suffix
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_067_u16_suffix() {
    let code = r#"
        fn main() {
            let port = 8080u16;
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// Inclusive range
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_068_inclusive_range() {
    let code = r#"
        fn main() {
            for i in 0..=5 {
                println!("{}", i);
            }
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// let-if with nested else-if → convert_if_expr recursion
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_069_let_if_nested_else_if() {
    let code = r#"
        fn main() {
            let x = 5;
            let r = if x > 10 {
                "big"
            } else if x > 5 {
                "medium"
            } else {
                "small"
            };
        }
    "#;
    parse_ok(code);
}

// ---------------------------------------------------------------------------
// format! with format specifier (e.g., {:02x})
// ---------------------------------------------------------------------------

#[test]
fn test_SPCOV_070_format_specifier() {
    let code = r#"
        fn main() {
            let val = 42;
            let msg = format!("{:02x}", val);
        }
    "#;
    parse_ok(code);
}
