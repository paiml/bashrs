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

include!("parser_coverage_tests5_counter.rs");
