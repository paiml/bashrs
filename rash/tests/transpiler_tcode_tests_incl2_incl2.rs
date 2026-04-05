fn test_t057_multiple_patterns() {
    // STMT: match x { 1 | 2 => {}, _ => {} } - multiple patterns
    let (ok, output) = transpile_stmt("match x { 1 | 2 => {}, _ => {} }");
    if !ok {
        println!("T057: Multiple patterns unsupported: {}", output);
    }
}

#[test]
fn test_t058_catch_all() {
    // STMT: match x { _ => {} } - catch-all pattern
    let (ok, output) = transpile_stmt("match x { _ => {} }");
    if !ok {
        println!("T058: Catch-all pattern unsupported: {}", output);
    } else if !output.contains("*)") && !output.contains("*") {
        println!("T058: WARNING - Catch-all should produce *)");
    }
}

#[test]
fn test_t059_match_guards() {
    // STMT: match x { y if y > 0 => {}, _ => {} } - match guards
    let (ok, output) = transpile_stmt("match x { y if y > 0 => {}, _ => {} }");
    if !ok {
        println!("T059: Match guards unsupported: {}", output);
    }
}

#[test]
fn test_t060_tuple_destructure() {
    // STMT: let (a, b) = (1, 2); - tuple destructuring
    let (ok, output) = transpile_stmt("let (a, b) = (1, 2);");
    if !ok {
        println!("T060: Tuple destructuring unsupported: {}", output);
    } else if !output.contains("a=") && !output.contains("b=") {
        println!("T060: WARNING - Destructuring may not produce assignments");
    }
}

#[test]
fn test_t062_option_match() {
    // STMT: match opt { Some(_) => {}, None => {} } - Option matching
    let (ok, output) = transpile_stmt("let opt = Some(1); match opt { Some(_) => {}, None => {} }");
    if !ok {
        println!("T062: Option matching unsupported: {}", output);
    }
}

#[test]
fn test_t063_result_match() {
    // STMT: match res { Ok(_) => {}, Err(_) => {} } - Result matching
    let code = "let res: Result<i32, &str> = Ok(1); match res { Ok(_) => {}, Err(_) => {} }";
    let (ok, output) = transpile_stmt(code);
    if !ok {
        println!("T063: Result matching unsupported: {}", output);
    }
}

#[test]
fn test_t064_tuple_match() {
    // STMT: match (1, 2) { (1, 2) => {}, _ => {} } - tuple matching
    let (ok, output) = transpile_stmt("match (1, 2) { (1, 2) => {}, _ => {} }");
    if !ok {
        println!("T064: Tuple matching unsupported: {}", output);
    }
}

#[test]
fn test_t061_struct_destructure() {
    // STMT: struct P {x:i32} let p = P{x:1}; let P{x} = p; - struct destructuring
    let code = "struct P { x: i32 } fn main() { let p = P { x: 1 }; let P { x } = p; let _ = x; }";
    let (ok, output) = transpile_prog(code);
    if !ok {
        println!("T061: Struct destructuring unsupported: {}", output);
    }
}

#[test]
fn test_t065_array_destructure() {
    // STMT: let [a, b, c] = arr; - array destructuring
    let (ok, output) = transpile_stmt("let arr = [1, 2, 3]; let [a, b, c] = arr;");
    if !ok {
        println!("T065: Array destructuring unsupported: {}", output);
    }
}

#[test]
fn test_t066_matches_macro() {
    // STMT: if matches!(x, 1..=5) {} - matches! macro
    let (ok, output) = transpile_stmt("if matches!(x, 1..=5) {}");
    if !ok {
        println!("T066: matches! macro unsupported: {}", output);
    }
}

#[test]
fn test_t067_ref_patterns() {
    // STMT: match x { ref y => {}, _ => {} } - ref patterns
    let (ok, output) = transpile_stmt("match x { ref y => { let _ = y; }, _ => {} }");
    if !ok {
        println!("T067: Ref patterns unsupported: {}", output);
    }
}

#[test]
fn test_t068_mut_patterns() {
    // STMT: match x { mut y => {}, _ => {} } - mut patterns
    let (ok, output) = transpile_stmt("match x { mut y => { y = 1; let _ = y; }, _ => {} }");
    if !ok {
        println!("T068: Mut patterns unsupported: {}", output);
    }
}

#[test]
fn test_t069_match_expression() {
    // STMT: let _ = match x { 1 => 10, _ => 0 }; - match as expression
    let (ok, output) = transpile_stmt("let result = match x { 1 => 10, _ => 0 };");
    if !ok {
        println!("T069: Match expression unsupported: {}", output);
    }
}

#[test]
fn test_t070_match_assignment() {
    // STMT: let a = match x { _ => 1 }; - match assignment
    let (ok, output) = transpile_stmt("let a = match x { _ => 1 };");
    if !ok {
        println!("T070: Match assignment unsupported: {}", output);
    }
}

// ============================================================================
// SECTION 4.5: Functions & Params (T071-T090)
// ============================================================================

#[test]
fn test_t071_function_definition() {
    // PROG: fn foo() {} fn main() { foo(); } - KNOWN BUG TB-001
    let code = "fn foo() {} fn main() { foo(); }";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("foo()") {
        println!("T071: KNOWN BUG TB-001 - User functions not transpiled");
        println!("      Output does not contain foo()");
    }
}

#[test]
fn test_t072_function_params() {
    // PROG: fn foo(x: i32) {} fn main() { foo(1); } - KNOWN BUG TB-002
    let code = "fn foo(x: i32) { let _ = x; } fn main() { foo(1); }";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("$1") && !output.contains("foo ") {
        println!("T072: KNOWN BUG TB-002 - Function params not passed");
    }
}

#[test]
fn test_t074_return_value() {
    // PROG: fn foo() -> i32 { 1 } fn main() {} - KNOWN BUG TB-006
    let code = "fn foo() -> i32 { 1 } fn main() {}";
    let (ok, output) = transpile_prog(code);
    if ok {
        // Return values in shell are tricky
        if !output.contains("return") && !output.contains("echo") && !output.contains("printf") {
            println!("T074: KNOWN BUG TB-006 - Return values not handled");
        }
    }
}

#[test]
fn test_t078_multi_param() {
    // PROG: fn foo(x:i32, y:i32){} fn main(){foo(1,2);} - KNOWN BUG TB-002
    let code = "fn foo(x: i32, y: i32) { let _ = (x, y); } fn main() { foo(1, 2); }";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("$2") {
        println!("T078: KNOWN BUG TB-002 - Multiple params not handled");
    }
}

#[test]
fn test_t082_multiple_functions() {
    // PROG: fn main() {} fn foo() {} - KNOWN BUG TB-003
    let code = "fn main() {} fn foo() {}";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("foo()") {
        println!("T082: KNOWN BUG TB-003 - Multiple functions fail");
    }
}

#[test]
fn test_t073_function_call() {
    // STMT: foo(1); - function application
    // Note: Requires foo to be defined, using PROG instead
    let code = "fn foo(x: i32) { let _ = x; } fn main() { foo(1); }";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("foo") {
        println!("T073: WARNING - Function call missing");
    }
}

#[test]
fn test_t075_capture_return() {
    // STMT: let _ = foo(1); - capture return value
    let code = "fn foo(x: i32) -> i32 { x + 1 } fn main() { let r = foo(1); let _ = r; }";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("$(") && !output.contains("r=") {
        println!("T075: WARNING - Return capture may not work");
    }
}

#[test]
fn test_t076_string_ref_param() {
    // PROG: fn foo(x: &str) {} - string reference parameter
    let code = r#"fn foo(x: &str) { let _ = x; } fn main() { foo("test"); }"#;
    let (ok, output) = transpile_prog(code);
    if !ok {
        println!("T076: String ref param unsupported: {}", output);
    }
}

#[test]
fn test_t077_pub_function() {
    // PROG: pub fn foo() {} - public function
    let code = "pub fn foo() {} fn main() { foo(); }";
    let (ok, output) = transpile_prog(code);
    if !ok {
        println!("T077: pub functions unsupported: {}", output);
    }
}

#[test]
fn test_t079_quoted_args() {
    // PROG: fn foo(s:&str){} fn main(){foo("a b");} - quoted string args
    let code = r#"fn foo(s: &str) { let _ = s; } fn main() { foo("a b"); }"#;
    let (ok, output) = transpile_prog(code);
    if ok {
        // Quoted args should preserve the space
        if !output.contains("a b") && !output.contains("\"a b\"") && !output.contains("'a b'") {
            println!("T079: WARNING - Quoted args may have word splitting issues");
        }
    }
}

#[test]
fn test_t080_recursion() {
    // PROG: fn f(n:i32){if n>0{f(n-1)}} - recursive function
    let code = "fn f(n: i32) { if n > 0 { f(n - 1); } } fn main() { f(5); }";
    let (ok, output) = transpile_prog(code);
    if ok && !output.contains("f()") && !output.contains("f ") {
        println!("T080: WARNING - Recursion may not work");
    }
}

#[test]
fn test_t081_attribute() {
    // PROG: #[bashrs::main] fn main() {} - attribute annotation
    let code = "#[bashrs::main] fn main() {}";
    let (ok, output) = transpile_prog(code);
    // Attributes may or may not affect output
    if ok && !output.contains("main") {
        println!("T081: WARNING - main should still be generated");
    }
}

#[test]
fn test_t083_inlining() {
    // STMT: /* inline hint? */ - inline functions
    // This is a comment test - comments should be preserved or removed cleanly
    let (ok, _output) = transpile_stmt("/* inline hint */ let x = 1;");
    if !ok {
        println!("T083: Comment handling may have issues");
    }
}

#[test]
fn test_t084_closures() {
    // STMT: let _ = |x:i32| x + 1; - closures
    let (ok, output) = transpile_stmt("let _ = |x: i32| x + 1;");
    if !ok {
        println!("T084: Closures unsupported (expected): {}", output);
    }
}

#[test]
fn test_t085_generics() {
    // PROG: fn foo<T>(x: T) {} - generic functions
    let code = "fn foo<T>(x: T) { let _ = x; } fn main() {}";
    let (ok, output) = transpile_prog(code);
    if !ok {
        println!("T085: Generics unsupported (expected): {}", output);
    }
}

#[test]
fn test_t086_result_return() {
    // PROG: fn foo() -> Result<(),()> {Ok(())} - Result return
    let code = "fn foo() -> Result<(), ()> { Ok(()) } fn main() {}";
    let (ok, output) = transpile_prog(code);
    if !ok {
        println!("T086: Result return unsupported: {}", output);
    }
}

#[test]
fn test_t087_nested_calls() {
    // STMT: foo(foo(1)); - nested function calls
    let code = "fn foo(x: i32) -> i32 { x } fn main() { let _ = foo(foo(1)); }";
    let (ok, output) = transpile_prog(code);
    if ok {
        // Should have nested call structure
        if !output.contains("foo") {
            println!("T087: WARNING - Nested calls may not work");
        }
    }
}

#[test]
fn test_t088_expr_as_arg() {
    // STMT: foo(1 + 2); - expression as argument
    let code = "fn foo(x: i32) { let _ = x; } fn main() { foo(1 + 2); }";
    let (ok, output) = transpile_prog(code);
    if ok {
        // Should evaluate 1+2=3 or pass arithmetic
        if !output.contains("3") && !output.contains("$((") {
            println!("T088: WARNING - Expression argument may not evaluate");
        }
    }
}

#[test]
fn test_t089_println_macro() {
    // STMT: println!("{}", x); - should produce echo
    let (ok, output) = transpile_stmt(r#"println!("{}", x);"#);
    if ok {
        let has_print =
            output.contains("echo") || output.contains("printf") || output.contains("rash_println");
        if !has_print {
            println!("T089: WARNING - println! should produce echo/printf");
        }
    }
}

#[test]
fn test_t090_eprintln_macro() {
    // STMT: eprintln!("{}", x); - should have >&2
    let (ok, output) = transpile_stmt(r#"eprintln!("{}", x);"#);
    if ok && !output.contains(">&2") && !output.contains("1>&2") {
        println!("T090: WARNING - eprintln! should redirect to stderr");
    }
}

// ============================================================================
// SECTION 4.6: Standard Library & OS (T091-T105)
// ============================================================================

#[test]

include!("transpiler_tcode_tests_incl2_incl2_incl2.rs");
