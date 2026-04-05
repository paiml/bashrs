
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

include!("codegen_tests_extracted_transpile_transpile_transpile.rs");
