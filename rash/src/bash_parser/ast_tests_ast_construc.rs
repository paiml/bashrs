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
}

#[cfg(test)]
mod ast_bracegroup_tests {
    use super::*;
    include!("ast_tests_extracted_bracegroup.rs");
}
