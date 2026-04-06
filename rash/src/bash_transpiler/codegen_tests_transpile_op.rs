
use super::*;
use crate::bash_parser::ast::AstMetadata;
use crate::bash_parser::parser::BashParser;

// TranspileOptions tests
#[test]
fn test_transpile_options_default() {
    let opts = TranspileOptions::default();
    assert!(opts.add_safety_checks);
    assert!(opts.preserve_comments);
    assert_eq!(opts.indent_size, 4);
}

#[test]
fn test_transpile_options_custom() {
    let opts = TranspileOptions {
        add_safety_checks: false,
        preserve_comments: false,
        indent_size: 2,
    };
    assert!(!opts.add_safety_checks);
    assert!(!opts.preserve_comments);
    assert_eq!(opts.indent_size, 2);
}

#[test]
fn test_transpiler_new() {
    let opts = TranspileOptions::default();
    let transpiler = BashToRashTranspiler::new(opts);
    assert_eq!(transpiler.current_indent, 0);
}

// Assignment tests
#[test]
fn test_transpile_simple_assignment() {
    let bash_code = "FOO=bar";
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("let FOO"));
    assert!(rash_code.contains("bar"));
}

#[test]
fn test_transpile_exported_assignment() {
    let bash_code = "export FOO=bar";
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("env::set_var"));
}

#[test]
fn test_transpile_numeric_assignment() {
    let bash_code = "COUNT=42";
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("42"));
}

// Function tests
#[test]
fn test_transpile_function() {
    let bash_code = r#"
function greet() {
echo "hello"
}
"#;
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("fn greet()"));
}

#[test]
fn test_transpile_function_with_body() {
    let bash_code = r#"
foo() {
x=1
echo $x
}
"#;
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("fn foo()"));
}

// If statement tests
#[test]
fn test_transpile_if_statement() {
    let bash_code = r#"
if [ $x == 1 ]; then
echo "one"
fi
"#;
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("if x == 1"));
}

#[test]
fn test_transpile_if_else() {
    let bash_code = r#"
if [ $x -eq 1 ]; then
echo "one"
else
echo "other"
fi
"#;
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("if"));
    assert!(rash_code.contains("else"));
}

// While loop tests
#[test]
fn test_transpile_while_loop() {
    let bash_code = r#"
while [ $x -lt 10 ]; do
echo $x
done
"#;
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("while"));
}

// Until loop tests - test using AST directly since parser may not support all operators
#[test]
fn test_transpile_until_loop() {
    // Build until loop AST directly
    let until_stmt = BashStmt::Until {
        condition: BashExpr::Test(Box::new(TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ))),
        body: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("x".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    };

    let ast = BashAst {
        statements: vec![until_stmt],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // Until becomes while with negated condition
    assert!(rash_code.contains("while"));
    assert!(rash_code.contains("!"));
}

// For loop tests
#[test]
fn test_transpile_for_loop() {
    let bash_code = r#"
for i in 1 2 3; do
echo $i
done
"#;
    let mut parser = BashParser::new(bash_code).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    assert!(rash_code.contains("for"));
}

// Comment tests

include!("codegen_tests_extracted_transpile.rs");
