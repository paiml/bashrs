fn test_IR_EXPR_023_exec_function_call() {
    let out = transpile_main(r#"exec("ls -la");"#);
    // exec() maps to eval in shell
    assert!(out.contains("eval"), "Expected eval for exec() in:\n{out}");
}

// ====================================================================
// convert_expr: FunctionCall with stdlib function name
// ====================================================================

#[test]
fn test_IR_EXPR_024_stdlib_function_call() {
    let out = transpile_full(
        r#"
fn main() {
    string_trim("  hello  ");
}
"#,
    );
    // stdlib functions get rash_ prefix
    assert!(
        out.contains("rash_string_trim"),
        "Expected rash_string_trim in:\n{out}"
    );
}

// ====================================================================
// convert_expr: FunctionCall with non-stdlib function name
// ====================================================================

#[test]
fn test_IR_EXPR_025_non_stdlib_function_call() {
    let out = transpile_full(
        r#"
fn main() {
    my_custom_fn("arg1");
}
fn my_custom_fn(s: &str) {}
"#,
    );
    assert!(
        out.contains("my_custom_fn"),
        "Expected my_custom_fn in:\n{out}"
    );
}

// ====================================================================
// convert_expr: Variable expression
// ====================================================================

#[test]
fn test_IR_EXPR_026_variable_expr() {
    let out = transpile_full(
        r#"
fn main() {
    let greeting = "hi";
    echo(greeting);
}
fn echo(msg: &str) {}
"#,
    );
    // Variable reference should appear as $greeting or "$greeting"
    assert!(
        out.contains("$greeting") || out.contains("\"$greeting\""),
        "Expected $greeting reference in:\n{out}"
    );
}

// ====================================================================
// convert_expr: Literal expression (string)
// ====================================================================

#[test]
fn test_IR_EXPR_027_literal_string() {
    let out = transpile_main(r#"let name = "Alice";"#);
    assert!(out.contains("Alice"), "Expected Alice in output:\n{out}");
}

// ====================================================================
// convert_expr: Literal expression (integer)
// ====================================================================

#[test]
fn test_IR_EXPR_028_literal_integer() {
    let out = transpile_main("let count = 99;");
    assert!(out.contains("99"), "Expected 99 in output:\n{out}");
}

// ====================================================================
// convert_expr: Literal expression (boolean)
// ====================================================================

#[test]
fn test_IR_EXPR_029_literal_bool() {
    let out = transpile_main("let flag = true;");
    // true is emitted as 1 in shell
    assert!(
        out.contains("1") || out.contains("true"),
        "Expected boolean true representation in:\n{out}"
    );
}

// ====================================================================
// convert_expr: If expression used in condition
// ====================================================================

#[test]
fn test_IR_EXPR_030_if_expr_as_condition() {
    let out = transpile_full(
        r#"
fn main() {
    let a = 5;
    let b = 10;
    if a < b {
        echo("less");
    } else {
        echo("not less");
    }
}
fn echo(msg: &str) {}
"#,
    );
    assert!(out.contains("if"), "Expected if in:\n{out}");
    assert!(out.contains("then"), "Expected then in:\n{out}");
    assert!(out.contains("else"), "Expected else in:\n{out}");
    assert!(out.contains("fi"), "Expected fi in:\n{out}");
}

// ====================================================================
// convert_expr: Index expression (array access)
// ====================================================================

#[test]
fn test_IR_EXPR_031_index_expr() {
    let out = transpile_full(
        r#"
fn main() {
    let arr = [10, 20, 30];
    let first = arr[0];
}
"#,
    );
    // Array indexing: arr[0] becomes arr_0 in POSIX shell
    assert!(
        out.contains("arr_0"),
        "Expected arr_0 for array index in:\n{out}"
    );
}

// ====================================================================
// convert_expr: FunctionCall with __format_concat (println! desugaring)
// ====================================================================

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
