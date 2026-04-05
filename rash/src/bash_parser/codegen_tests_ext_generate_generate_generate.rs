
    #[test]
    fn test_generate_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "x".to_string(),
            message: Box::new(BashExpr::Literal("not set".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${x:?"));
    }

    #[test]
    fn test_generate_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "x".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${x:+alt}\"");
    }

    #[test]
    fn test_generate_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "str".to_string(),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${#str}\"");
    }

    #[test]
    fn test_generate_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "file".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${file%.txt}\"");
    }

    #[test]
    fn test_generate_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${path#"));
    }

    #[test]
    fn test_generate_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${path##"));
    }

    #[test]
    fn test_generate_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "file".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${file%%"));
    }

    #[test]
    fn test_generate_expr_command_condition() {
        let cmd = Box::new(BashStmt::Command {
            name: "test".to_string(),
            args: vec![
                BashExpr::Literal("-f".to_string()),
                BashExpr::Literal("file".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        });
        let expr = BashExpr::CommandCondition(cmd);
        let output = generate_expr(&expr);
        assert!(output.contains("test") && output.contains("-f"));
    }

    // ============================================================================
    // Statement Generation Coverage
    // ============================================================================

    #[test]
    fn test_generate_statement_return_without_code() {
        let stmt = BashStmt::Return {
            code: None,
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "return");
    }

    #[test]
    fn test_generate_statement_coproc_with_name() {
        let stmt = BashStmt::Coproc {
            name: Some("MY_PROC".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert!(output.contains("coproc MY_PROC"));
    }

    #[test]
    fn test_generate_statement_coproc_without_name() {
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
        let output = generate_statement(&stmt);
        assert!(output.starts_with("coproc {"));
    }

    #[test]
    fn test_generate_statement_until_loop() {
        let stmt = BashStmt::Until {
            condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                BashExpr::Variable("i".to_string()),
                BashExpr::Literal("5".to_string()),
            ))),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        // until loop converts to while with negated condition
        assert!(output.contains("while") && output.contains("done"));
    }

    #[test]
    fn test_generate_statement_for_c_style() {
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
        let output = generate_statement(&stmt);
        // C-style for loop converts to POSIX while loop
        assert!(output.contains("i=0"));
        assert!(output.contains("while"));
        assert!(output.contains("-lt"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_statement_for_c_style_empty_init() {
        let stmt = BashStmt::ForCStyle {
            init: "".to_string(),
            condition: "i<10".to_string(),
            increment: "".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert!(output.contains("while"));
        // No init line, no increment at end
    }

    // ============================================================================
    // negate_condition Coverage
    // ============================================================================

    #[test]
    fn test_negate_condition_test_expr() {
        let condition = BashExpr::Test(Box::new(TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        )));
        let output = negate_condition(&condition);
        assert!(output.contains("! ") || output.contains("[ !"));
    }

    #[test]
    fn test_negate_condition_non_test() {
        let condition = BashExpr::Literal("true".to_string());
        let output = negate_condition(&condition);
        assert!(output.starts_with("! "));
    }

    // ============================================================================
    // generate_test_condition Coverage
    // ============================================================================

    #[test]
    fn test_generate_test_condition_int_ne() {
        let expr = TestExpr::IntNe(
            BashExpr::Variable("a".to_string()),
            BashExpr::Literal("0".to_string()),
        );
        let output = generate_test_condition(&expr);
        assert_eq!(output, "\"$a\" -ne 0");
    }

    #[test]
    fn test_generate_test_condition_int_le() {
        let expr = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("100".to_string()),
        );
        let output = generate_test_condition(&expr);
        assert_eq!(output, "\"$x\" -le 100");
    }

    #[test]
    fn test_generate_test_condition_int_ge() {
        let expr = TestExpr::IntGe(
            BashExpr::Variable("y".to_string()),
            BashExpr::Literal("1".to_string()),
        );
        let output = generate_test_condition(&expr);
        assert_eq!(output, "\"$y\" -ge 1");
    }

    #[test]
    fn test_generate_test_condition_file_exists() {
        let expr = TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-e /tmp");
    }

    #[test]
    fn test_generate_test_condition_file_readable() {
        let expr = TestExpr::FileReadable(BashExpr::Literal("file".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-r file");
    }

    #[test]
    fn test_generate_test_condition_file_writable() {
        let expr = TestExpr::FileWritable(BashExpr::Literal("file".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-w file");
    }

    #[test]
    fn test_generate_test_condition_file_executable() {
        let expr = TestExpr::FileExecutable(BashExpr::Literal("script".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-x script");
    }

    #[test]
    fn test_generate_test_condition_string_empty() {
        let expr = TestExpr::StringEmpty(BashExpr::Variable("s".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-z \"$s\"");
    }

    #[test]
    fn test_generate_test_condition_string_non_empty() {
        let expr = TestExpr::StringNonEmpty(BashExpr::Variable("s".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-n \"$s\"");
    }

    #[test]
    fn test_generate_test_condition_and() {
        let expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileDirectory(BashExpr::Literal("a".to_string()))),
        );
        let output = generate_test_condition(&expr);
        assert!(output.contains("&&"));
    }

    #[test]
    fn test_generate_test_condition_or() {
        let expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_condition(&expr);
        assert!(output.contains("||"));
    }

    #[test]
    fn test_generate_test_condition_not() {
        let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "x".to_string(),
        ))));
        let output = generate_test_condition(&expr);
        assert!(output.starts_with("! "));
    }

    // ============================================================================
    // C-style for loop conversion helpers
    // ============================================================================

    #[test]
    fn test_convert_c_init_to_posix() {
        assert_eq!(convert_c_init_to_posix("i=0"), "i=0");
        assert_eq!(convert_c_init_to_posix("x=10"), "x=10");
    }

    #[test]
    fn test_convert_c_condition_less_equal() {
        let output = convert_c_condition_to_posix("i<=10");
        assert!(output.contains("-le") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_greater_equal() {
        let output = convert_c_condition_to_posix("i>=0");
        assert!(output.contains("-ge") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_not_equal() {
        let output = convert_c_condition_to_posix("i!=5");
        assert!(output.contains("-ne") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_equal() {
        let output = convert_c_condition_to_posix("i==0");
        assert!(output.contains("-eq") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_greater() {
        let output = convert_c_condition_to_posix("i>5");
        assert!(output.contains("-gt") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_fallback() {
        let output = convert_c_condition_to_posix("some_expr");
        assert_eq!(output, "[ some_expr ]");
    }

    #[test]
    fn test_convert_c_increment_postfix_increment() {
        let output = convert_c_increment_to_posix("i++");
        assert_eq!(output, "i=$((i+1))");
    }

    #[test]
    fn test_convert_c_increment_prefix_increment() {
        let output = convert_c_increment_to_posix("++i");
        assert_eq!(output, "i=$((i+1))");
    }

    #[test]
    fn test_convert_c_increment_postfix_decrement() {
        let output = convert_c_increment_to_posix("i--");
        assert_eq!(output, "i=$((i-1))");
    }

    #[test]
    fn test_convert_c_increment_prefix_decrement() {
        let output = convert_c_increment_to_posix("--i");
        assert_eq!(output, "i=$((i-1))");
    }

    #[test]
    fn test_convert_c_increment_plus_equals() {
        let output = convert_c_increment_to_posix("i+=2");
        assert_eq!(output, "i=$((i+2))");
    }

    #[test]
    fn test_convert_c_increment_minus_equals() {
        let output = convert_c_increment_to_posix("i-=3");
        assert_eq!(output, "i=$((i-3))");
    }

    #[test]
    fn test_convert_c_increment_assignment() {
        let output = convert_c_increment_to_posix("i=i+1");
        assert_eq!(output, "i=i+1");
    }

    #[test]
    fn test_convert_c_increment_fallback() {
        let output = convert_c_increment_to_posix("something_else");
        assert_eq!(output, ":something_else");
    }

    // ============================================================================
    // extract_var_name Coverage
    // ============================================================================

    #[test]
    fn test_extract_var_name_with_dollar() {
        assert_eq!(extract_var_name("$i"), "i");
        assert_eq!(extract_var_name("$var"), "var");
    }

    #[test]
    fn test_extract_var_name_without_dollar() {
        assert_eq!(extract_var_name("i"), "i");
        assert_eq!(extract_var_name("count"), "count");
    }

    // ============================================================================
    // strip_quotes Coverage
    // ============================================================================

    #[test]
    fn test_strip_quotes_double() {
        assert_eq!(strip_quotes("\"value\""), "value");
    }

    #[test]
    fn test_strip_quotes_single() {
        assert_eq!(strip_quotes("'value'"), "value");
    }

    #[test]
    fn test_strip_quotes_mixed() {
        assert_eq!(strip_quotes("\"value'"), "value");
    }

    #[test]
    fn test_strip_quotes_none() {
        assert_eq!(strip_quotes("value"), "value");
    }

    // ============================================================================
    // generate_condition Coverage
    // ============================================================================

    #[test]
    fn test_generate_condition_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "f".to_string(),
        ))));
        let output = generate_condition(&expr);
        assert!(output.contains("-e"));
    }

    #[test]
    fn test_generate_condition_non_test() {
        let expr = BashExpr::Literal("true".to_string());
        let output = generate_condition(&expr);
        assert_eq!(output, "true");
    }

    // ============================================================================
    // Comment shebang filtering
    // ============================================================================

    #[test]
    fn test_generate_comment_shebang_filtered() {
        let stmt = BashStmt::Comment {
            text: "!/bin/bash".to_string(),
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "");
    }

    #[test]
    fn test_generate_comment_shebang_with_space_filtered() {
        let stmt = BashStmt::Comment {
            text: " !/bin/sh".to_string(),
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "");
    }

    #[test]
    fn test_generate_comment_normal() {
        let stmt = BashStmt::Comment {
            text: "This is a normal comment".to_string(),
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "# This is a normal comment");
    }
}

#[cfg(test)]
mod test_issue_64 {
    use crate::bash_parser::codegen::generate_purified_bash;
    use crate::bash_parser::BashParser;

    #[test]
    fn test_ISSUE_64_single_quoted_ansi_codes() {
        // RED phase: Test single-quoted ANSI escape sequences
        let input = r#"RED='\033[0;31m'"#;
        let mut parser = BashParser::new(input).expect("Failed to parse");
        let ast = parser.parse().expect("Failed to parse");
        let output = generate_purified_bash(&ast);

        // Single quotes should be preserved for escape sequences
        assert!(
            output.contains("RED='\\033[0;31m'"),
            "Output should preserve single quotes around escape sequences: {}",
            output
        );
    }

    #[test]
    fn test_ISSUE_64_single_quoted_literal() {
        let input = "echo 'Hello World'";
        let mut parser = BashParser::new(input).expect("Failed to parse");
        let ast = parser.parse().expect("Failed to parse");
        let output = generate_purified_bash(&ast);

        // Single quotes should be preserved
        assert!(
            output.contains("'Hello World'"),
            "Output should preserve single quotes: {}",
            output
        );
    }

    #[test]
    fn test_ISSUE_64_assignment_with_single_quotes() {
        let input = "x='value'";
        let mut parser = BashParser::new(input).expect("Failed to parse");
        let ast = parser.parse().expect("Failed to parse");
        let output = generate_purified_bash(&ast);

        // For simple alphanumeric strings, quotes are optional in purified output
        // Both x=value and x='value' are correct POSIX shell
        // The important thing is it parses without error
        assert!(
            output.contains("x=value") || output.contains("x='value'"),
            "Output should contain valid assignment: {}",
            output
        );
    }

    #[test]
    fn test_ELIF_001_basic_elif_preserved() {
        let input = r#"if [ "$1" = "a" ]; then
    echo alpha
elif [ "$1" = "b" ]; then
    echo beta
else
    echo unknown
fi"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(
            output.contains("elif"),
            "elif should be preserved in output: {output}"
        );
        assert!(
            output.contains("echo alpha"),
            "then branch preserved: {output}"
        );
        assert!(
            output.contains("echo beta"),
            "elif branch preserved: {output}"
        );
        assert!(
            output.contains("echo unknown"),
            "else branch preserved: {output}"
        );
    }

    #[test]
    fn test_ELIF_002_multiple_elif_preserved() {
        let input = r#"if [ "$1" = "a" ]; then
    echo alpha
elif [ "$1" = "b" ]; then
    echo beta
elif [ "$1" = "c" ]; then
    echo gamma
else
    echo unknown
fi"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        let elif_count = output.matches("elif").count();
        assert_eq!(
            elif_count, 2,
            "should have 2 elif branches, got {elif_count}: {output}"
        );
    }

    #[test]
    fn test_ELIF_003_elif_no_else() {
        let input = r#"if [ "$1" = "a" ]; then
    echo alpha
elif [ "$1" = "b" ]; then
    echo beta
fi"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("elif"), "elif preserved: {output}");
        assert!(!output.contains("else"), "no else block: {output}");
    }
}
