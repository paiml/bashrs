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
