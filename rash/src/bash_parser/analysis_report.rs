//! Semantic Analysis for Bash AST
//!
//! Performs semantic analysis including:
//! - Variable scope resolution
//! - Command effect tracking
//! - Type inference (basic)

use super::ast::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;


impl SemanticAnalyzer {

    fn infer_type(&self, expr: &BashExpr) -> InferredType {
        match expr {
            BashExpr::Literal(s) => {
                if s.parse::<i64>().is_ok() {
                    InferredType::Integer
                } else {
                    InferredType::String
                }
            }
            BashExpr::Array(_) => InferredType::Array,
            BashExpr::Arithmetic(_) => InferredType::Integer,
            _ => InferredType::Unknown,
        }
    }

    fn track_command_effects(&mut self, command: &str) {
        // Track known commands with side effects
        match command {
            "curl" | "wget" | "nc" | "telnet" | "ssh" => {
                self.effects.network_access = true;
            }
            "rm" | "mv" | "cp" | "touch" | "mkdir" | "rmdir" => {
                // File modification commands
                self.effects.file_writes.insert(command.to_string());
            }
            "cat" | "less" | "more" | "head" | "tail" | "grep" => {
                // File reading commands
                self.effects.file_reads.insert(command.to_string());
            }
            _ => {}
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub scope_info: ScopeInfo,
    pub effects: EffectTracker,
    pub warnings: Vec<String>,
}

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

    #[test]
    fn test_for_cstyle() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("loop".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_case_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Case {
            word: BashExpr::Variable("opt".to_string()),
            arms: vec![CaseArm {
                patterns: vec!["a".to_string()],
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("option a".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_pipeline() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file".to_string())],
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
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
        assert!(report.effects.file_reads.contains("grep"));
    }

    #[test]
    fn test_and_list() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("test"));
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_or_list() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "false".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "true".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("false"));
        assert!(report.effects.process_spawns.contains("true"));
    }

    #[test]
    fn test_brace_group() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "pwd".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("pwd"));
    }

    #[test]
    fn test_coproc() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Coproc {
            name: Some("mycoproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
    }

    #[test]
    fn test_function_definition() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Function {
            name: "myfunc".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.functions.contains_key("myfunc"));
        let func = report.scope_info.functions.get("myfunc").unwrap();
        assert!(func.calls_detected.contains("echo"));
    }

    #[test]
    fn test_function_redefinition_error() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Function {
                name: "myfunc".to_string(),
                body: vec![],
                span: Span::dummy(),
            },
            BashStmt::Function {
                name: "myfunc".to_string(),
                body: vec![],
                span: Span::dummy(),
            },
        ]);

        let result = analyzer.analyze(&ast);
        assert!(matches!(
            result,
            Err(SemanticError::FunctionRedefinition(_))
        ));
    }


}

    include!("semantic_part3_incl2.rs");
