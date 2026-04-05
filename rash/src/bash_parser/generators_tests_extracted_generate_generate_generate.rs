
    #[test]
    fn test_generate_expr_arithmetic() {
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        )));
        let output = generate_expr(&expr);
        assert_eq!(output, "$((1 + 2))");
    }

    #[test]
    fn test_generate_expr_command_subst() {
        let expr = BashExpr::CommandSubst(Box::new(BashStmt::Command {
            name: "date".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        }));
        let output = generate_expr(&expr);
        assert_eq!(output, "$(date)");
    }

    #[test]
    fn test_generate_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("prefix_".to_string()),
            BashExpr::Variable("VAR".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert!(output.contains("prefix_"));
        assert!(output.contains("\"$VAR\""));
    }

    #[test]
    fn test_generate_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "*.txt");
    }

    #[test]
    fn test_generate_expr_default_value() {
        let expr = BashExpr::DefaultValue {
            variable: "FOO".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:-default}"));
    }

    #[test]
    fn test_generate_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "FOO".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:=default}"));
    }

    #[test]
    fn test_generate_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "FOO".to_string(),
            message: Box::new(BashExpr::Literal("error".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:?error}"));
    }

    #[test]
    fn test_generate_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "FOO".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:+alt}"));
    }

    #[test]
    fn test_generate_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "FOO".to_string(),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${#FOO}"));
    }

    #[test]
    fn test_generate_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FILE%.txt}"));
    }

    #[test]
    fn test_generate_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${PATH#*/}"));
    }

    #[test]
    fn test_generate_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${PATH##*/}"));
    }

    #[test]
    fn test_generate_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FILE%%.*}"));
    }

    #[test]
    fn test_generate_expr_command_condition() {
        let expr = BashExpr::CommandCondition(Box::new(BashStmt::Command {
            name: "test".to_string(),
            args: vec![
                BashExpr::Literal("-f".to_string()),
                BashExpr::Literal("file".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        }));
        let output = generate_expr(&expr);
        assert!(output.contains("test -f file"));
    }

    // ============== generate_arith_expr tests ==============

    #[test]
    fn test_generate_arith_expr_number() {
        let expr = ArithExpr::Number(42);
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "42");
    }

    #[test]
    fn test_generate_arith_expr_variable() {
        let expr = ArithExpr::Variable("x".to_string());
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "x");
    }

    #[test]
    fn test_generate_arith_expr_add() {
        let expr = ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "1 + 2");
    }

    #[test]
    fn test_generate_arith_expr_sub() {
        let expr = ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "5 - 3");
    }

    #[test]
    fn test_generate_arith_expr_mul() {
        let expr = ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "2 * 3");
    }

    #[test]
    fn test_generate_arith_expr_div() {
        let expr = ArithExpr::Div(
            Box::new(ArithExpr::Number(6)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "6 / 2");
    }

include!("generators_tests_extracted_generate_generate_generate_generate.rs");
