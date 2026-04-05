#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        let stmt = BashStmt::Assignment {
            name: "FOO".to_string(),
            index: None,
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        };

        assert!(matches!(stmt, BashStmt::Assignment { .. }));
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_col, 10);
    }

    #[test]
    fn test_span_dummy() {
        let span = Span::dummy();
        // dummy() returns all zeros
        assert_eq!(span.start_line, 0);
        assert_eq!(span.start_col, 0);
        assert_eq!(span.end_line, 0);
        assert_eq!(span.end_col, 0);
    }

    #[test]
    fn test_span_zero() {
        // Span doesn't implement Default, test with explicit zeros
        let span = Span::new(0, 0, 0, 0);
        assert_eq!(span.start_line, 0);
        assert_eq!(span.start_col, 0);
        assert_eq!(span.end_line, 0);
        assert_eq!(span.end_col, 0);
    }

    // BashStmt construction tests
    #[test]
    fn test_assignment_construction() {
        let stmt = BashStmt::Assignment {
            name: "x".to_string(),
            index: None,
            value: BashExpr::Literal("1".to_string()),
            exported: false,
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Assignment { .. }));
    }

    #[test]
    fn test_command_construction() {
        let stmt = BashStmt::Command {
            name: "echo".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Command { .. }));
    }

    #[test]
    fn test_function_construction() {
        let stmt = BashStmt::Function {
            name: "func".to_string(),
            body: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Function { .. }));
    }

    #[test]
    fn test_if_construction() {
        let stmt = BashStmt::If {
            condition: BashExpr::Literal("true".to_string()),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::If { .. }));
    }

    #[test]
    fn test_while_construction() {
        let stmt = BashStmt::While {
            condition: BashExpr::Literal("true".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::While { .. }));
    }

    #[test]
    fn test_until_construction() {
        let stmt = BashStmt::Until {
            condition: BashExpr::Literal("false".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Until { .. }));
    }

    #[test]
    fn test_for_construction() {
        let stmt = BashStmt::For {
            variable: "i".to_string(),
            items: BashExpr::Literal("1 2 3".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::For { .. }));
    }

    #[test]
    fn test_for_cstyle_construction() {
        let stmt = BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::ForCStyle { .. }));
    }

    #[test]
    fn test_case_construction() {
        let stmt = BashStmt::Case {
            word: BashExpr::Variable("x".to_string()),
            arms: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Case { .. }));
    }

    #[test]
    fn test_return_construction() {
        let stmt = BashStmt::Return {
            code: Some(BashExpr::Literal("0".to_string())),
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Return { .. }));
    }

    #[test]
    fn test_comment_construction() {
        let stmt = BashStmt::Comment {
            text: "# comment".to_string(),
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Comment { .. }));
    }

    #[test]
    fn test_pipeline_construction() {
        let stmt = BashStmt::Pipeline {
            commands: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Pipeline { .. }));
    }

    #[test]
    fn test_andlist_construction() {
        let cmd = BashStmt::Command {
            name: "true".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let stmt = BashStmt::AndList {
            left: Box::new(cmd.clone()),
            right: Box::new(cmd),
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::AndList { .. }));
    }

    #[test]
    fn test_orlist_construction() {
        let cmd = BashStmt::Command {
            name: "false".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let stmt = BashStmt::OrList {
            left: Box::new(cmd.clone()),
            right: Box::new(cmd),
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::OrList { .. }));
    }

    #[test]
    fn test_bracegroup_construction() {
        let stmt = BashStmt::BraceGroup {
            body: vec![],
            subshell: false,
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::BraceGroup { .. }));
    }

    #[test]
    fn test_coproc_construction() {
        let stmt = BashStmt::Coproc {
            name: Some("mycoproc".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert!(matches!(stmt, BashStmt::Coproc { .. }));
    }

    // BashStmt span() tests
    #[test]
    fn test_assignment_span() {
        let span = Span::new(1, 0, 1, 10);
        let stmt = BashStmt::Assignment {
            name: "x".to_string(),
            index: None,
            value: BashExpr::Literal("1".to_string()),
            exported: false,
            span,
        };
        let retrieved_span = stmt.span();
        // Verify the span was converted properly
        assert_eq!(retrieved_span.line_start, 1);
        assert_eq!(retrieved_span.col_end, 10);
    }

    #[test]
    fn test_command_span() {
        let span = Span::new(2, 0, 2, 15);
        let stmt = BashStmt::Command {
            name: "echo".to_string(),
            args: vec![],
            redirects: vec![],
            span,
        };
        let retrieved_span = stmt.span();
        // Verify the span was converted properly
        assert_eq!(retrieved_span.line_start, 2);
        assert_eq!(retrieved_span.col_end, 15);
    }

    // BashStmt Display tests
    #[test]
    fn test_assignment_display() {
        let stmt = BashStmt::Assignment {
            name: "FOO".to_string(),
            index: None,
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Assignment(FOO)");
    }

    #[test]
    fn test_command_display() {
        let stmt = BashStmt::Command {
            name: "echo".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Command(echo)");
    }

    #[test]
    fn test_function_display() {
        let stmt = BashStmt::Function {
            name: "my_func".to_string(),
            body: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Function(my_func)");
    }

    #[test]
    fn test_if_display() {
        let stmt = BashStmt::If {
            condition: BashExpr::Literal("true".to_string()),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "If");
    }

    #[test]
    fn test_while_display() {
        let stmt = BashStmt::While {
            condition: BashExpr::Literal("true".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "While");
    }

    #[test]
    fn test_until_display() {
        let stmt = BashStmt::Until {
            condition: BashExpr::Literal("false".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Until");
    }

    #[test]
    fn test_for_display() {
        let stmt = BashStmt::For {
            variable: "i".to_string(),
            items: BashExpr::Literal("1 2 3".to_string()),
            body: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "For(i)");
    }

    #[test]
    fn test_for_cstyle_display() {
        let stmt = BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "ForCStyle");
    }

    #[test]
    fn test_case_display() {
        let stmt = BashStmt::Case {
            word: BashExpr::Variable("x".to_string()),
            arms: vec![],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Case");
    }

    #[test]
    fn test_return_display() {
        let stmt = BashStmt::Return {
            code: Some(BashExpr::Literal("0".to_string())),
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Return");
    }

    #[test]
    fn test_comment_display() {
        let stmt = BashStmt::Comment {
            text: "comment".to_string(),
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Comment");
    }

    #[test]
    fn test_pipeline_display() {
        let stmt = BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "ls".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Pipeline(2 cmds)");
    }

    #[test]
    fn test_andlist_display() {
        let cmd = BashStmt::Command {
            name: "true".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let stmt = BashStmt::AndList {
            left: Box::new(cmd.clone()),
            right: Box::new(cmd),
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "AndList");
    }

    #[test]
    fn test_orlist_display() {
        let cmd = BashStmt::Command {
            name: "false".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let stmt = BashStmt::OrList {
            left: Box::new(cmd.clone()),
            right: Box::new(cmd),
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "OrList");
    }

    #[test]
    fn test_bracegroup_display() {
        let stmt = BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "BraceGroup(1 stmts)");
    }

    #[test]
    fn test_coproc_display_with_name() {
        let stmt = BashStmt::Coproc {
            name: Some("mycoproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        assert_eq!(format!("{}", stmt), "Coproc(mycoproc, 1 stmts)");
    }

    #[test]
    fn test_coproc_display_without_name() {
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
        assert_eq!(format!("{}", stmt), "Coproc(1 stmts)");
    }

    // BashExpr tests
    #[test]
    fn test_literal_expr() {
        let expr = BashExpr::Literal("hello".to_string());
        assert!(matches!(expr, BashExpr::Literal(_)));
    }

    #[test]
    fn test_variable_expr() {
        let expr = BashExpr::Variable("HOME".to_string());
        assert!(matches!(expr, BashExpr::Variable(_)));
    }

    #[test]
    fn test_array_expr() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        ]);
        if let BashExpr::Array(items) = expr {
            assert_eq!(items.len(), 2);
        }
    }

    #[test]
    fn test_concat_expr() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("hello".to_string()),
            BashExpr::Variable("NAME".to_string()),
        ]);
        if let BashExpr::Concat(parts) = expr {
            assert_eq!(parts.len(), 2);
        }
    }

    #[test]
    fn test_glob_expr() {
        let expr = BashExpr::Glob("*.txt".to_string());
        assert!(matches!(expr, BashExpr::Glob(_)));
    }

    #[test]
    fn test_default_value_expr() {
        let expr = BashExpr::DefaultValue {
            variable: "VAR".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        assert!(matches!(expr, BashExpr::DefaultValue { .. }));
    }

    #[test]
    fn test_assign_default_expr() {
        let expr = BashExpr::AssignDefault {
            variable: "VAR".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        assert!(matches!(expr, BashExpr::AssignDefault { .. }));
    }

    #[test]
    fn test_error_if_unset_expr() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "VAR".to_string(),
            message: Box::new(BashExpr::Literal("not set!".to_string())),
        };
        assert!(matches!(expr, BashExpr::ErrorIfUnset { .. }));
    }

    #[test]
    fn test_alternative_value_expr() {
        let expr = BashExpr::AlternativeValue {
            variable: "VAR".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        assert!(matches!(expr, BashExpr::AlternativeValue { .. }));
    }

    #[test]
    fn test_string_length_expr() {
        let expr = BashExpr::StringLength {
            variable: "VAR".to_string(),
        };
        assert!(matches!(expr, BashExpr::StringLength { .. }));
    }

    #[test]
    fn test_remove_prefix_expr() {
        let expr = BashExpr::RemovePrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        assert!(matches!(expr, BashExpr::RemovePrefix { .. }));
    }

    #[test]
    fn test_remove_suffix_expr() {
        let expr = BashExpr::RemoveSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        assert!(matches!(expr, BashExpr::RemoveSuffix { .. }));
    }

    // TestExpr tests
    #[test]
    fn test_file_exists_test_expr() {
        let expr = TestExpr::FileExists(BashExpr::Literal("/tmp/file".to_string()));
        assert!(matches!(expr, TestExpr::FileExists(_)));
    }

    #[test]
    fn test_file_directory_test_expr() {
        let expr = TestExpr::FileDirectory(BashExpr::Literal("/tmp".to_string()));
        assert!(matches!(expr, TestExpr::FileDirectory(_)));
    }

    #[test]
    fn test_file_readable_test_expr() {
        let expr = TestExpr::FileReadable(BashExpr::Literal("/tmp".to_string()));
        assert!(matches!(expr, TestExpr::FileReadable(_)));
    }

    #[test]
    fn test_file_writable_test_expr() {
        let expr = TestExpr::FileWritable(BashExpr::Literal("/tmp".to_string()));
        assert!(matches!(expr, TestExpr::FileWritable(_)));
    }

    #[test]
    fn test_file_executable_test_expr() {
        let expr = TestExpr::FileExecutable(BashExpr::Literal("/bin/sh".to_string()));
        assert!(matches!(expr, TestExpr::FileExecutable(_)));
    }

    #[test]
    fn test_string_empty_test_expr() {
        let expr = TestExpr::StringEmpty(BashExpr::Literal("".to_string()));
        assert!(matches!(expr, TestExpr::StringEmpty(_)));
    }

    #[test]
    fn test_string_non_empty_test_expr() {
        let expr = TestExpr::StringNonEmpty(BashExpr::Literal("hello".to_string()));
        assert!(matches!(expr, TestExpr::StringNonEmpty(_)));
    }

    #[test]
    fn test_string_eq_test_expr() {
        let expr = TestExpr::StringEq(
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        );
        assert!(matches!(expr, TestExpr::StringEq(_, _)));
    }

    #[test]
    fn test_string_ne_test_expr() {
        let expr = TestExpr::StringNe(
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        );
        assert!(matches!(expr, TestExpr::StringNe(_, _)));
    }

    #[test]
    fn test_int_eq_test_expr() {
        let expr = TestExpr::IntEq(
            BashExpr::Literal("1".to_string()),
            BashExpr::Literal("1".to_string()),
        );
        assert!(matches!(expr, TestExpr::IntEq(_, _)));
    }

    #[test]
    fn test_int_ne_test_expr() {
        let expr = TestExpr::IntNe(
            BashExpr::Literal("1".to_string()),
            BashExpr::Literal("2".to_string()),
        );
        assert!(matches!(expr, TestExpr::IntNe(_, _)));
    }

    #[test]
    fn test_int_lt_test_expr() {
        let expr = TestExpr::IntLt(
            BashExpr::Literal("1".to_string()),
            BashExpr::Literal("2".to_string()),
        );
        assert!(matches!(expr, TestExpr::IntLt(_, _)));
    }

    #[test]
    fn test_int_le_test_expr() {
        let expr = TestExpr::IntLe(
            BashExpr::Literal("1".to_string()),
            BashExpr::Literal("2".to_string()),
        );
        assert!(matches!(expr, TestExpr::IntLe(_, _)));
    }

    #[test]
    fn test_int_gt_test_expr() {
        let expr = TestExpr::IntGt(
            BashExpr::Literal("2".to_string()),
            BashExpr::Literal("1".to_string()),
        );
        assert!(matches!(expr, TestExpr::IntGt(_, _)));
    }

    #[test]
    fn test_int_ge_test_expr() {
        let expr = TestExpr::IntGe(
            BashExpr::Literal("2".to_string()),
            BashExpr::Literal("1".to_string()),
        );
        assert!(matches!(expr, TestExpr::IntGe(_, _)));
    }

    #[test]
    fn test_and_test_expr() {
        let expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()))),
            Box::new(TestExpr::FileDirectory(BashExpr::Literal(
                "/tmp".to_string(),
            ))),
        );
        assert!(matches!(expr, TestExpr::And(_, _)));
    }

    #[test]
    fn test_or_test_expr() {
        let expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()))),
            Box::new(TestExpr::FileDirectory(BashExpr::Literal(
                "/var".to_string(),
            ))),
        );
        assert!(matches!(expr, TestExpr::Or(_, _)));
    }

    #[test]
    fn test_not_test_expr() {
        let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "/nonexistent".to_string(),
        ))));
        assert!(matches!(expr, TestExpr::Not(_)));
    }

    // ArithExpr tests
    #[test]
    fn test_arith_number() {
        let expr = ArithExpr::Number(42);
        assert!(matches!(expr, ArithExpr::Number(42)));
    }

    #[test]
    fn test_arith_variable() {
        let expr = ArithExpr::Variable("count".to_string());
        assert!(matches!(expr, ArithExpr::Variable(_)));
    }

    #[test]
    fn test_arith_add() {
        let expr = ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        );
        assert!(matches!(expr, ArithExpr::Add(_, _)));
    }

    #[test]
    fn test_arith_sub() {
        let expr = ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        );
        assert!(matches!(expr, ArithExpr::Sub(_, _)));
    }

    #[test]
    fn test_arith_mul() {
        let expr = ArithExpr::Mul(
            Box::new(ArithExpr::Number(3)),
            Box::new(ArithExpr::Number(4)),
        );
        assert!(matches!(expr, ArithExpr::Mul(_, _)));
    }

    #[test]
    fn test_arith_div() {
        let expr = ArithExpr::Div(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(2)),
        );
        assert!(matches!(expr, ArithExpr::Div(_, _)));
    }

    #[test]
    fn test_arith_mod() {
        let expr = ArithExpr::Mod(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(3)),
        );
        assert!(matches!(expr, ArithExpr::Mod(_, _)));
    }

    // Redirect tests
    #[test]
    fn test_redirect_output() {
        let redirect = Redirect::Output {
            target: BashExpr::Literal("output.txt".to_string()),
        };
        assert!(matches!(redirect, Redirect::Output { .. }));
    }

    #[test]
    fn test_redirect_append() {
        let redirect = Redirect::Append {
            target: BashExpr::Literal("output.txt".to_string()),
        };
        assert!(matches!(redirect, Redirect::Append { .. }));
    }

    #[test]
    fn test_redirect_input() {
        let redirect = Redirect::Input {
            target: BashExpr::Literal("input.txt".to_string()),
        };
        assert!(matches!(redirect, Redirect::Input { .. }));
    }

    #[test]
    fn test_redirect_error() {
        let redirect = Redirect::Error {
            target: BashExpr::Literal("error.txt".to_string()),
        };
        assert!(matches!(redirect, Redirect::Error { .. }));
    }

    #[test]
    fn test_redirect_append_error() {
        let redirect = Redirect::AppendError {
            target: BashExpr::Literal("error.txt".to_string()),
        };
        assert!(matches!(redirect, Redirect::AppendError { .. }));
    }

    #[test]
    fn test_redirect_combined() {
        let redirect = Redirect::Combined {
            target: BashExpr::Literal("combined.txt".to_string()),
        };
        assert!(matches!(redirect, Redirect::Combined { .. }));
    }

    #[test]
    fn test_redirect_duplicate() {
        let redirect = Redirect::Duplicate {
            from_fd: 2,
            to_fd: 1,
        };
        assert!(matches!(redirect, Redirect::Duplicate { .. }));
    }

    #[test]
    fn test_redirect_herestring() {
        let redirect = Redirect::HereString {
            content: "test string".to_string(),
        };
        assert!(matches!(redirect, Redirect::HereString { .. }));
    }

    // CaseArm tests
    #[test]
    fn test_case_arm() {
        let arm = CaseArm {
            patterns: vec!["*.txt".to_string(), "*.md".to_string()],
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("text file".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
        };
        assert_eq!(arm.patterns.len(), 2);
        assert_eq!(arm.body.len(), 1);
    }

    // BashAst tests
    #[test]
    fn test_bash_ast_construction() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: Some("test.sh".to_string()),
                line_count: 1,
                parse_time_ms: 10,
            },
        };
        assert_eq!(ast.statements.len(), 1);
        assert_eq!(ast.metadata.source_file, Some("test.sh".to_string()));
    }

    // BashNode tests
    #[test]
    fn test_bash_node_creation() {
        let span = Span::new(1, 0, 1, 10);
        let node = BashNode::new("test value", span);
        assert_eq!(node.node, "test value");
        assert_eq!(node.span, span);
    }

    // Span comprehensive test
    #[test]
    fn test_span_comprehensive() {
        let span = Span::new(5, 10, 8, 20);
        assert_eq!(span.start_line, 5);
        assert_eq!(span.start_col, 10);
        assert_eq!(span.end_line, 8);
        assert_eq!(span.end_col, 20);
    }
}
