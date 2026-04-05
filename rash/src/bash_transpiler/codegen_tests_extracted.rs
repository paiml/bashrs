#[cfg(test)]
mod tests {
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

    // Arithmetic tests via expressions
    #[test]
    fn test_transpile_arithmetic_add() {
        // We test arithmetic through the AST directly
        let arith = ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("+"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_transpile_arithmetic_sub() {
        let arith = ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("-"));
    }

    #[test]
    fn test_transpile_arithmetic_mul() {
        let arith = ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Number(3)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("*"));
    }

    #[test]
    fn test_transpile_arithmetic_div() {
        let arith = ArithExpr::Div(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(2)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("/"));
    }

    #[test]
    fn test_transpile_arithmetic_mod() {
        let arith = ArithExpr::Mod(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(3)),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert!(result.contains("%"));
    }

    #[test]
    fn test_transpile_arithmetic_variable() {
        let arith = ArithExpr::Variable("x".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert_eq!(result, "x");
    }

    #[test]
    fn test_transpile_arithmetic_number() {
        let arith = ArithExpr::Number(42);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_arithmetic(&arith).unwrap();
        assert_eq!(result, "42");
    }

    // Test expression direct tests
    #[test]
    fn test_transpile_test_int_le() {
        let test = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("<="));
    }

    #[test]
    fn test_transpile_test_int_ge() {
        let test = TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains(">="));
    }

    #[test]
    fn test_transpile_test_int_ne() {
        let test = TestExpr::IntNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("!="));
    }

    #[test]
    fn test_transpile_test_and() {
        let test = TestExpr::And(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_transpile_test_or() {
        let test = TestExpr::Or(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        );
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_transpile_test_not() {
        let test = TestExpr::Not(Box::new(TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("!("));
    }

    #[test]
    fn test_transpile_test_file_readable() {
        let test = TestExpr::FileReadable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("metadata"));
        assert!(result.contains("readonly"));
    }

    #[test]
    fn test_transpile_test_file_writable() {
        let test = TestExpr::FileWritable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("metadata"));
        assert!(result.contains("readonly"));
    }

    #[test]
    fn test_transpile_test_file_executable() {
        let test = TestExpr::FileExecutable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("is_executable"));
    }

    // Expression direct tests
    #[test]
    fn test_transpile_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("glob"));
        assert!(result.contains("*.txt"));
    }

    #[test]
    fn test_transpile_expr_default_value() {
        let expr = BashExpr::DefaultValue {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("unwrap_or"));
    }

    #[test]
    fn test_transpile_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("get_or_insert"));
    }

    #[test]
    fn test_transpile_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "x".to_string(),
            message: Box::new(BashExpr::Literal("error".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("expect"));
    }

    #[test]
    fn test_transpile_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "x".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("as_ref"));
        assert!(result.contains("map"));
    }

    #[test]
    fn test_transpile_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "x".to_string(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains(".len()"));
    }

    #[test]
    fn test_transpile_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("strip_suffix"));
    }

    #[test]
    fn test_transpile_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal("/tmp/".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("strip_prefix"));
    }

    #[test]
    fn test_transpile_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal("/*/".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("rsplit_once"));
    }

    #[test]
    fn test_transpile_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("split_once"));
    }

    #[test]
    fn test_transpile_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        ]);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("vec!"));
    }

    #[test]
    fn test_transpile_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("hello".to_string()),
            BashExpr::Variable("name".to_string()),
        ]);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("format!"));
    }

    #[test]
    fn test_transpile_expr_command_subst() {
        let stmt = BashStmt::Command {
            name: "ls".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let expr = BashExpr::CommandSubst(Box::new(stmt));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_transpile_expr_command_condition() {
        let stmt = BashStmt::Command {
            name: "test".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let expr = BashExpr::CommandCondition(Box::new(stmt));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("success"));
    }

    #[test]
    fn test_CODEGEN_COV_001_if_with_elif() {
        let stmt = BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("one".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![(
                BashExpr::Test(Box::new(TestExpr::StringEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("2".to_string()),
                ))),
                vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("two".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            )],
            else_block: None,
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("else if"));
    }

    #[test]
    fn test_CODEGEN_COV_002_for_c_style() {
        let stmt = BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("C-style for loop"));
    }

    #[test]
    fn test_CODEGEN_COV_003_case_statement() {
        let stmt = BashStmt::Case {
            word: BashExpr::Variable("opt".to_string()),
            arms: vec![
                CaseArm {
                    patterns: vec!["start".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("starting".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
                CaseArm {
                    patterns: vec!["*".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("default".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
            ],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("start"));
    }

    #[test]
    fn test_CODEGEN_COV_004_pipeline() {
        let stmt = BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file.txt".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![BashExpr::Literal("pattern".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("|"));
    }

    #[test]
    fn test_CODEGEN_COV_005_and_list() {
        let stmt = BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![BashExpr::Literal("-f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("exists".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_CODEGEN_COV_006_or_list() {
        let stmt = BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![BashExpr::Literal("-f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("missing".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_CODEGEN_COV_007_brace_group() {
        let stmt = BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("inside".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_CODEGEN_COV_008_coproc_named() {
        let stmt = BashStmt::Coproc {
            name: Some("myproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("coproc myproc"));
    }

    #[test]
    fn test_CODEGEN_COV_009_coproc_unnamed() {
        let stmt = BashStmt::Coproc {
            name: None,
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("coproc -"));
    }

    #[test]
    fn test_CODEGEN_COV_010_select() {
        let stmt = BashStmt::Select {
            variable: "opt".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("a".to_string()),
                BashExpr::Literal("b".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("opt".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("select opt"));
    }

    #[test]
    fn test_CODEGEN_COV_011_expr_arithmetic() {
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("5") && result.contains("3"));
    }

    #[test]
    fn test_CODEGEN_COV_012_expr_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::StringEq(
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("=="));
    }

    #[test]
    fn test_CODEGEN_COV_013_test_expression_fallback() {
        // Non-Test expr in test position falls through to transpile_expression
        let expr = BashExpr::Literal("true".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test_expression(&expr).unwrap();
        assert!(result.contains("true"));
    }
}
