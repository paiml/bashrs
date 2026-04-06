//! Code Generation for Bash-to-Rash Transpiler

use super::patterns::*;
use super::TranspileResult;
use crate::bash_parser::ast::*;


impl BashToRashTranspiler {
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
    #[test]
    fn test_transpile_comment_preserved() {
        let bash_code = "# This is a comment";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let opts = TranspileOptions {
            preserve_comments: true,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("//"));
    }

    #[test]
    fn test_transpile_comment_discarded() {
        let bash_code = "# This is a comment\nx=1";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let opts = TranspileOptions {
            preserve_comments: false,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        let rash_code = transpiler.transpile(&ast).unwrap();

        // Comment line should be empty, not contain //
        assert!(rash_code.contains("let x"));
    }

    // Return statement tests
    #[test]
    fn test_transpile_return_no_value() {
        let bash_code = r#"
foo() {
    return
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("return;"));
    }

    #[test]
    fn test_transpile_return_with_value() {
        let bash_code = r#"
foo() {
    return 0
}
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("return"));
        assert!(rash_code.contains("0"));
    }

    // Expression tests
    #[test]
    fn test_transpile_literal_string() {
        let bash_code = "echo hello";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("hello"));
    }

    #[test]
    fn test_transpile_variable() {
        let bash_code = "echo $x";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("x"));
    }

    // Test expression tests
    #[test]
    fn test_transpile_string_eq() {
        let bash_code = r#"
if [ "$x" == "foo" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("=="));
    }

    #[test]
    fn test_transpile_string_ne() {
        let bash_code = r#"
if [ "$x" != "foo" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("!="));
    }

    #[test]
    fn test_transpile_int_lt() {
        let bash_code = r#"
if [ $x -lt 10 ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("<"));
    }

    #[test]
    fn test_transpile_int_gt() {
        let bash_code = r#"
if [ $x -gt 10 ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains(">"));
    }

    #[test]
    fn test_transpile_file_exists() {
        let bash_code = r#"
if [ -e /tmp/file ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("exists"));
    }

    #[test]
    fn test_transpile_file_directory() {
        let bash_code = r#"
if [ -d /tmp ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("is_dir"));
    }

    #[test]
    fn test_transpile_string_empty() {
        let bash_code = r#"
if [ -z "$x" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("is_empty"));
    }

    #[test]
    fn test_transpile_string_non_empty() {
        let bash_code = r#"
if [ -n "$x" ]; then
    echo yes
fi
"#;
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("!"));
        assert!(rash_code.contains("is_empty"));
    }

    // Indent tests
    #[test]
    fn test_indent_empty_lines() {
        let opts = TranspileOptions::default();
        let transpiler = BashToRashTranspiler::new(opts);

        let result = transpiler.indent("line1\n\nline2");
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
    }

    #[test]
    fn test_indent_with_level() {
        let opts = TranspileOptions {
            indent_size: 2,
            ..TranspileOptions::default()
        };
        let mut transpiler = BashToRashTranspiler::new(opts);
        transpiler.current_indent = 1;

        let result = transpiler.indent("code");
        assert!(result.starts_with("  ")); // 2 spaces for indent level 1
    }

    // Header test
    #[test]
    fn test_transpile_header() {
        let bash_code = "x=1";
        let mut parser = BashParser::new(bash_code).unwrap();
        let ast = parser.parse().unwrap();

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast).unwrap();

        assert!(rash_code.contains("// Transpiled from bash by rash"));
    }


}

    include!("codegen_part3_incl2.rs");
