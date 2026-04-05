
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

    #[test]
    fn test_generate_arith_mul() {
        let expr = ArithExpr::Mul(
            Box::new(ArithExpr::Number(3)),
            Box::new(ArithExpr::Number(4)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "3 * 4");
    }

    #[test]
    fn test_generate_arith_div() {
        let expr = ArithExpr::Div(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "10 / 2");
    }

    #[test]
    fn test_generate_arith_mod() {
        let expr = ArithExpr::Mod(
            Box::new(ArithExpr::Number(7)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "7 % 3");
    }

    // ============================================================================
    // Expression Generation Coverage
    // ============================================================================

    #[test]
    fn test_generate_expr_literal_with_spaces() {
        let expr = BashExpr::Literal("hello world".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'hello world'");
    }

    #[test]
    fn test_generate_expr_literal_with_single_quote() {
        let expr = BashExpr::Literal("don't".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'don'\\''t'");
    }

    #[test]
    fn test_generate_expr_literal_with_command_subst() {
        let expr = BashExpr::Literal("$(pwd)".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"$(pwd)\"");
    }

    #[test]
    fn test_generate_expr_literal_with_variable() {
        let expr = BashExpr::Literal("$HOME".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"$HOME\"");
    }

    #[test]
    fn test_generate_expr_literal_with_brace_expansion() {
        let expr = BashExpr::Literal("${HOME}".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${HOME}\"");
    }

    #[test]
    fn test_generate_expr_literal_with_double_quote() {
        let expr = BashExpr::Literal("say \"hi\"".to_string());
        let output = generate_expr(&expr);
        // Contains embedded quotes but no expansion - uses single quotes
        assert_eq!(output, "'say \"hi\"'");
    }

    #[test]
    fn test_generate_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
            BashExpr::Literal("c".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert_eq!(output, "a b c");
    }

    #[test]
    fn test_generate_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "*.txt");
    }

    #[test]
    fn test_generate_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("prefix_".to_string()),
            BashExpr::Variable("var".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert!(output.contains("prefix_") && output.contains("$var"));
    }

    #[test]
    fn test_generate_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${x:=default}\"");
    }

include!("codegen_tests_ext_generate_generate_generate.rs");
