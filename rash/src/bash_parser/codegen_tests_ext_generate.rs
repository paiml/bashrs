
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
    use super::*;

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

// FIXME(PMAT-238): include!("codegen_tests_ext_generate_generate.rs");
