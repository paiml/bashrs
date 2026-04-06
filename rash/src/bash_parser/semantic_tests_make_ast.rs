#[cfg(test)]
mod tests {
    use super::*;

    fn make_ast(statements: Vec<BashStmt>) -> BashAst {
        BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        }
    }

    #[test]
    fn test_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "FOO".to_string(),
            index: None,
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("FOO"));
    }

    #[test]
    fn test_exported_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "PATH".to_string(),
            index: None,
            value: BashExpr::Literal("/usr/bin".to_string()),
            exported: true,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.get("PATH").unwrap().exported);
        assert!(report.effects.env_modifications.contains("PATH"));
    }

    #[test]
    fn test_effect_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "curl".to_string(),
            args: vec![BashExpr::Literal("http://example.com".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.network_access);
    }

    #[test]
    fn test_effect_tracker_is_pure() {
        let tracker = EffectTracker::new();
        assert!(tracker.is_pure());

        let mut impure = EffectTracker::new();
        impure.network_access = true;
        assert!(!impure.is_pure());
    }

    #[test]
    fn test_effect_tracker_default() {
        let tracker = EffectTracker::default();
        assert!(tracker.is_pure());
    }

    #[test]
    fn test_file_read_commands() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "cat".to_string(),
            args: vec![BashExpr::Literal("file.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
    }

    #[test]
    fn test_file_write_commands() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "rm".to_string(),
            args: vec![BashExpr::Literal("file.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_writes.contains("rm"));
    }

    #[test]
    fn test_network_commands() {
        for cmd in &["wget", "nc", "telnet", "ssh"] {
            let mut analyzer = SemanticAnalyzer::new();
            let ast = make_ast(vec![BashStmt::Command {
                name: cmd.to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }]);

            let report = analyzer.analyze(&ast).unwrap();
            assert!(
                report.effects.network_access,
                "Command {} should enable network_access",
                cmd
            );
        }
    }

    #[test]
    fn test_if_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                "VAR".to_string(),
            )))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("yes".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![(
                BashExpr::Test(Box::new(TestExpr::StringEmpty(BashExpr::Literal(
                    "".to_string(),
                )))),
                vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("elif".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            )],
            else_block: Some(vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("no".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }]),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_while_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::While {
            condition: BashExpr::Literal("true".to_string()),
            body: vec![BashStmt::Command {
                name: "sleep".to_string(),
                args: vec![BashExpr::Literal("1".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("sleep"));
    }

    #[test]
    fn test_until_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Until {
            condition: BashExpr::Literal("false".to_string()),
            body: vec![BashStmt::Command {
                name: "wait".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("wait"));
    }

    #[test]
    fn test_for_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::For {
            variable: "i".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("1".to_string()),
                BashExpr::Literal("2".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }
    include!("semantic_tests_extracted_for.rs");
}
