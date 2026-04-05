
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

include!("ast_tests_extracted_bracegroup_orlist.rs");
