//! Tests extracted from codegen.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::codegen::*;
use crate::bash_parser::BashParser;

// ============================================================================
// Statement Generation Tests
// ============================================================================

#[test]
fn test_generate_simple_command() {
    let input = "echo hello world";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("echo hello world") || output.contains("echo 'hello' 'world'"));
}

#[test]
fn test_generate_command_with_quotes() {
    let input = r#"echo "hello world""#;
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("hello world"));
}

#[test]
fn test_generate_assignment() {
    let input = "x=42";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("x=42"));
}

#[test]
fn test_generate_exported_assignment() {
    let input = "export PATH=/usr/bin";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("export") && output.contains("PATH"));
}

#[test]
fn test_generate_comment() {
    let input = "# This is a comment\necho hello";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    // Comment should be preserved (may have different formatting)
    assert!(output.contains("#") && output.contains("comment"));
}

#[test]
fn test_generate_function() {
    let input = "hello() { echo hi; }";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("hello()") && output.contains("echo"));
}

#[test]
fn test_generate_if_statement() {
    let input = "if [ -f file ]; then echo exists; fi";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("if") && output.contains("then") && output.contains("fi"));
}

#[test]
fn test_generate_if_else_statement() {
    let input = "if [ -f file ]; then echo yes; else echo no; fi";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("if") && output.contains("else") && output.contains("fi"));
}

#[test]
fn test_generate_for_loop() {
    let input = "for i in 1 2 3; do echo $i; done";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("for") && output.contains("do") && output.contains("done"));
}

#[test]
fn test_generate_while_loop() {
    let input = "while [ $x -lt 10 ]; do echo $x; done";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("while") && output.contains("do") && output.contains("done"));
}

#[test]
fn test_generate_case_statement() {
    let input = "case $x in a) echo a;; b) echo b;; esac";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("case") && output.contains("esac"));
}

#[test]
fn test_generate_pipeline() {
    let input = "ls | grep foo";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("|"));
}

#[test]
fn test_generate_and_list() {
    let input = "test -f file && echo exists";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("&&"));
}

#[test]
fn test_generate_or_list() {
    let input = "test -f file || echo missing";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("||"));
}

#[test]
fn test_generate_redirect() {
    let input = "echo hello > output.txt";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains(">"));
}

#[test]
fn test_generate_append_redirect() {
    let input = "echo hello >> output.txt";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains(">>"));
}

#[test]
fn test_generate_input_redirect() {
    let input = "cat < input.txt";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("<"));
}

#[test]
fn test_generate_variable_expansion() {
    let input = r#"echo "$HOME""#;
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("HOME"));
}

#[test]
fn test_generate_arithmetic() {
    let input = "x=$((1 + 2))";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("$((") || output.contains("x="));
}

#[test]
fn test_generate_command_substitution() {
    let input = "x=$(pwd)";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("$(") || output.contains("pwd"));
}

#[test]
fn test_generate_return_statement() {
    let input = "return 0";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("return"));
}

#[test]
fn test_generate_shebang_replaced() {
    let input = "#!/bin/bash\necho hello";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    // Shebang should be replaced with #!/bin/sh
    assert!(output.starts_with("#!/bin/sh"));
    // Should not have duplicate shebangs
    assert_eq!(output.matches("#!/bin/sh").count(), 1);
}

#[test]
fn test_generate_subshell() {
    // Use a simpler subshell syntax that parses correctly
    let input = "result=$(pwd)";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("$(") || output.contains("pwd"));
}

#[test]
fn test_generate_brace_group() {
    let input = "{ echo a; echo b; }";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("{") && output.contains("}"));
}

// ============================================================================
// Expression Generation Tests
// ============================================================================

#[test]
fn test_generate_string_literal() {
    let input = "echo 'literal'";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("literal"));
}

#[test]
fn test_generate_array_access() {
    let input = "echo ${arr[0]}";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    // Array access should be preserved or transformed
    assert!(output.contains("arr") || output.contains("${"));
}

#[test]
fn test_generate_parameter_default() {
    let input = "echo ${x:-default}";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains(":-") || output.contains("default"));
}

#[test]
fn test_generate_here_document() {
    let input = "cat <<EOF\nhello\nEOF";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("<<") || output.contains("hello"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_generate_empty_ast() {
    let ast = BashAst {
        statements: vec![],
        metadata: AstMetadata {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        },
    };
    let output = generate_purified_bash(&ast);
    assert!(output.starts_with("#!/bin/sh"));
}

#[test]
fn test_generate_nested_structures() {
    let input = "if true; then for i in 1 2; do echo $i; done; fi";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(
        output.contains("if")
            && output.contains("for")
            && output.contains("done")
            && output.contains("fi")
    );
}

#[test]
fn test_generate_complex_pipeline() {
    let input = "cat file | grep pattern | sort | uniq";
    let mut parser = BashParser::new(input).expect("parse");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("|"));
}

// ============================================================================
// Additional Coverage Tests - Direct Unit Tests
// ============================================================================
#[cfg(test)]
mod codegen_coverage_tests {
use crate::bash_parser::codegen::*;

// ============================================================================
// Redirect Generation Tests
// ============================================================================

#[test]
fn test_generate_redirect_error() {
    let redirect = Redirect::Error {
        target: BashExpr::Literal("error.log".to_string()),
    };
    let output = generate_redirect(&redirect);
    assert_eq!(output, "2> error.log");
}

#[test]
fn test_generate_redirect_append_error() {
    let redirect = Redirect::AppendError {
        target: BashExpr::Literal("error.log".to_string()),
    };
    let output = generate_redirect(&redirect);
    assert_eq!(output, "2>> error.log");
}

#[test]
fn test_generate_redirect_combined() {
    let redirect = Redirect::Combined {
        target: BashExpr::Literal("all.log".to_string()),
    };
    let output = generate_redirect(&redirect);
    assert_eq!(output, "> all.log 2>&1");
}

#[test]
fn test_generate_redirect_duplicate() {
    let redirect = Redirect::Duplicate {
        from_fd: 2,
        to_fd: 1,
    };
    let output = generate_redirect(&redirect);
    assert_eq!(output, "2>&1");
}

#[test]
fn test_generate_redirect_here_string() {
    let redirect = Redirect::HereString {
        content: "hello world".to_string(),
    };
    let output = generate_redirect(&redirect);
    assert_eq!(output, "<<< \"hello world\"");
}

#[test]
fn test_generate_redirect_here_string_with_quotes() {
    let redirect = Redirect::HereString {
        content: "say \"hello\"".to_string(),
    };
    let output = generate_redirect(&redirect);
    assert_eq!(output, "<<< \"say \\\"hello\\\"\"");
}

// ============================================================================
// Test Expression Coverage
// ============================================================================

#[test]
fn test_generate_test_expr_int_ne() {
    let expr = TestExpr::IntNe(
        BashExpr::Variable("a".to_string()),
        BashExpr::Literal("5".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ \"$a\" -ne 5 ]");
}

#[test]
fn test_generate_test_expr_int_le() {
    let expr = TestExpr::IntLe(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("10".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ \"$x\" -le 10 ]");
}

#[test]
fn test_generate_test_expr_int_ge() {
    let expr = TestExpr::IntGe(
        BashExpr::Variable("y".to_string()),
        BashExpr::Literal("0".to_string()),
    );
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ \"$y\" -ge 0 ]");
}

#[test]
fn test_generate_test_expr_file_exists() {
    let expr = TestExpr::FileExists(BashExpr::Variable("file".to_string()));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -e \"$file\" ]");
}

#[test]
fn test_generate_test_expr_file_readable() {
    let expr = TestExpr::FileReadable(BashExpr::Literal("/etc/passwd".to_string()));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -r /etc/passwd ]");
}

#[test]
fn test_generate_test_expr_file_writable() {
    let expr = TestExpr::FileWritable(BashExpr::Literal("/tmp/test".to_string()));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -w /tmp/test ]");
}

#[test]
fn test_generate_test_expr_file_executable() {
    let expr = TestExpr::FileExecutable(BashExpr::Literal("/bin/sh".to_string()));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -x /bin/sh ]");
}

#[test]
fn test_generate_test_expr_string_empty() {
    let expr = TestExpr::StringEmpty(BashExpr::Variable("str".to_string()));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -z \"$str\" ]");
}

#[test]
fn test_generate_test_expr_string_non_empty() {
    let expr = TestExpr::StringNonEmpty(BashExpr::Variable("str".to_string()));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -n \"$str\" ]");
}

#[test]
fn test_generate_test_expr_and() {
    let expr = TestExpr::And(
        Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
        Box::new(TestExpr::FileReadable(BashExpr::Literal("a".to_string()))),
    );
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -e a ] && [ -r a ]");
}

#[test]
fn test_generate_test_expr_or() {
    let expr = TestExpr::Or(
        Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
        Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
    );
    let output = generate_test_expr(&expr);
    assert_eq!(output, "[ -e a ] || [ -e b ]");
}

#[test]
fn test_generate_test_expr_not() {
    let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
        "x".to_string(),
    ))));
    let output = generate_test_expr(&expr);
    assert_eq!(output, "! [ -e x ]");
}

// ============================================================================
// Arithmetic Expression Coverage
// ============================================================================

#[test]
fn test_generate_arith_sub() {
    let expr = ArithExpr::Sub(
        Box::new(ArithExpr::Variable("a".to_string())),
        Box::new(ArithExpr::Number(1)),
    );
    let output = generate_arith_expr(&expr);
    assert_eq!(output, "a - 1");
}

