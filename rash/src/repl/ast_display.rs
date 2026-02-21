// REPL AST Display Module
//
// Task: REPL-004-002 - AST display mode
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 6+ scenarios
// - Integration tests: 2+ CLI tests
// - Complexity: <10 per function

use crate::bash_parser::{BashAst, BashStmt};

/// Format AST for display in REPL
///
/// # Examples
///
/// ```
/// use bashrs::repl::ast_display::format_ast;
/// use bashrs::bash_parser::{BashAst, ast::AstMetadata};
///
/// let ast = BashAst {
///     statements: vec![],
///     metadata: AstMetadata {
///         source_file: None,
///         line_count: 0,
///         parse_time_ms: 0,
///     },
/// };
/// let output = format_ast(&ast);
/// assert!(output.contains("AST"));
/// ```
pub fn format_ast(ast: &BashAst) -> String {
    let mut output = String::new();
    output.push_str("=== AST ===\n");
    output.push_str(&format!("Statements: {}\n", ast.statements.len()));

    for (i, stmt) in ast.statements.iter().enumerate() {
        output.push_str(&format!("\n[{}] {}\n", i, format_statement(stmt, 0)));
    }

    output
}

/// Format a single statement with indentation
fn format_statement(stmt: &BashStmt, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);

    match stmt {
        BashStmt::Command { name, args, .. } => {
            if args.is_empty() {
                format!("{}Command: {}", indent_str, name)
            } else {
                format!("{}Command: {} (args: {})", indent_str, name, args.len())
            }
        }
        BashStmt::Assignment { name, .. } => {
            format!("{}Assignment: {}", indent_str, name)
        }
        BashStmt::If {
            then_block,
            elif_blocks,
            else_block,
            ..
        } => {
            let mut s = format!("{}If statement", indent_str);
            s.push_str(&format!(
                "\n{}  then: {} statements",
                indent_str,
                then_block.len()
            ));
            if !elif_blocks.is_empty() {
                s.push_str(&format!(
                    "\n{}  elif: {} branches",
                    indent_str,
                    elif_blocks.len()
                ));
            }
            if else_block.is_some() {
                s.push_str(&format!("\n{}  else: present", indent_str));
            }
            s
        }
        BashStmt::While { body, .. } => {
            format!("{}While loop ({} statements)", indent_str, body.len())
        }
        BashStmt::For { variable, body, .. } => {
            format!(
                "{}For loop: {} ({} statements)",
                indent_str,
                variable,
                body.len()
            )
        }
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => {
            format!(
                "{}C-style for loop: {}; {}; {} ({} statements)",
                indent_str,
                init,
                condition,
                increment,
                body.len()
            )
        }
        BashStmt::Function { name, body, .. } => {
            format!(
                "{}Function: {} ({} statements)",
                indent_str,
                name,
                body.len()
            )
        }
        BashStmt::Case { arms, .. } => {
            format!("{}Case statement ({} arms)", indent_str, arms.len())
        }
        BashStmt::Until { body, .. } => {
            format!("{}Until loop ({} statements)", indent_str, body.len())
        }
        BashStmt::Return { .. } => {
            format!("{}Return statement", indent_str)
        }
        BashStmt::Comment { text, .. } => {
            format!(
                "{}Comment: {}",
                indent_str,
                text.lines().next().unwrap_or("")
            )
        }
        BashStmt::Pipeline { commands, .. } => {
            format!("{}Pipeline ({} commands)", indent_str, commands.len())
        }
        BashStmt::AndList { .. } => {
            format!("{}AndList (&&)", indent_str)
        }
        BashStmt::OrList { .. } => {
            format!("{}OrList (||)", indent_str)
        }
        BashStmt::BraceGroup { body, .. } => {
            format!("{}BraceGroup ({} statements)", indent_str, body.len())
        }
        BashStmt::Coproc { name, body, .. } => {
            if let Some(n) = name {
                format!("{}Coproc: {} ({} statements)", indent_str, n, body.len())
            } else {
                format!("{}Coproc ({} statements)", indent_str, body.len())
            }
        }
        BashStmt::Select { variable, body, .. } => {
            format!(
                "{}Select: {} ({} statements)",
                indent_str,
                variable,
                body.len()
            )
        }

        BashStmt::Negated { command, .. } => {
            format!(
                "{}Negated: {}",
                indent_str,
                format_statement(command, indent + 1)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bash_parser::ast::{AstMetadata, BashExpr, Span};

    fn dummy_span() -> Span {
        Span::new(1, 1, 1, 10)
    }

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-004-002-001 - Display simple command AST
    #[test]
    fn test_REPL_004_002_display_simple_command() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);

        assert!(output.contains("=== AST ==="), "Should have AST header");
        assert!(
            output.contains("Statements: 1"),
            "Should show statement count"
        );
        assert!(output.contains("Command: echo"), "Should show command name");
    }

    /// Test: REPL-004-002-002 - Display assignment AST
    #[test]
    fn test_REPL_004_002_display_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("value".to_string()),
                exported: false,
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);

        assert!(output.contains("Assignment: VAR"), "Should show assignment");
    }

    /// Test: REPL-004-002-003 - Display if statement AST
    #[test]
    fn test_REPL_004_002_display_if_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                elif_blocks: vec![],
                else_block: None,
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);

        assert!(output.contains("If statement"), "Should show if statement");
        assert!(
            output.contains("then: 1 statements"),
            "Should show then block"
        );
    }

    /// Test: REPL-004-002-004 - Display empty AST
    #[test]
    fn test_REPL_004_002_display_empty_ast() {
        let ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);

        assert!(
            output.contains("Statements: 0"),
            "Should show zero statements"
        );
    }

    /// Test: REPL-004-002-005 - Display multiple statements
    #[test]
    fn test_REPL_004_002_display_multiple_statements() {
        let ast = BashAst {
            statements: vec![
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                },
                BashStmt::Assignment {
                    name: "X".to_string(),
                    index: None,
                    value: BashExpr::Literal("5".to_string()),
                    exported: false,
                    span: dummy_span(),
                },
                BashStmt::Command {
                    name: "ls".to_string(),
                    args: vec![BashExpr::Literal("-la".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                },
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);

        assert!(output.contains("Statements: 3"), "Should show 3 statements");
        assert!(output.contains("[0]"), "Should have index 0");
        assert!(output.contains("[1]"), "Should have index 1");
        assert!(output.contains("[2]"), "Should have index 2");
    }

    /// Test: REPL-004-002-006 - Display for loop AST
    #[test]
    fn test_REPL_004_002_display_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "i".to_string(),
                items: BashExpr::Array(vec![
                    BashExpr::Literal("1".to_string()),
                    BashExpr::Literal("2".to_string()),
                    BashExpr::Literal("3".to_string()),
                ]),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);

        assert!(
            output.contains("For loop: i"),
            "Should show for loop variable"
        );
        assert!(output.contains("1 statements"), "Should show body size");
    }

    /// Test: REPL-004-002-007 - Display while loop AST
    #[test]
    fn test_REPL_004_002_display_while_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::While {
                condition: BashExpr::Literal("true".to_string()),
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "sleep".to_string(),
                        args: vec![BashExpr::Literal("1".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("While loop (2 statements)"));
    }

    /// Test: REPL-004-002-008 - Display case statement AST
    #[test]
    fn test_REPL_004_002_display_case_statement() {
        use crate::bash_parser::ast::CaseArm;

        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("choice".to_string()),
                arms: vec![
                    CaseArm {
                        patterns: vec!["yes".to_string(), "y".to_string()],
                        body: vec![],
                    },
                    CaseArm {
                        patterns: vec!["no".to_string(), "n".to_string()],
                        body: vec![],
                    },
                ],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 5,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Case statement (2 arms)"));
    }

    /// Test: REPL-004-002-009 - Display until loop AST
    #[test]
    fn test_REPL_004_002_display_until_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Literal("false".to_string()),
                body: vec![BashStmt::Command {
                    name: "work".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Until loop (1 statements)"));
    }

    /// Test: REPL-004-002-010 - Display return statement AST
    #[test]
    fn test_REPL_004_002_display_return_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Return statement"));
    }

    /// Test: REPL-004-002-011 - Display comment AST
    #[test]
    fn test_REPL_004_002_display_comment() {
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: "This is a test comment\nwith multiple lines".to_string(),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Comment: This is a test comment"));
    }

    /// Test: REPL-004-002-012 - Display pipeline AST
    #[test]
    fn test_REPL_004_002_display_pipeline() {
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "cat".to_string(),
                        args: vec![BashExpr::Literal("file.txt".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("pattern".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "wc".to_string(),
                        args: vec![BashExpr::Literal("-l".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Pipeline (3 commands)"));
    }

    /// Test: REPL-004-002-013 - Display AndList AST
    #[test]
    fn test_REPL_004_002_display_and_list() {
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "true".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("success".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("AndList (&&)"));
    }

    /// Test: REPL-004-002-014 - Display OrList AST
    #[test]
    fn test_REPL_004_002_display_or_list() {
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "false".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("fallback".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("OrList (||)"));
    }

    /// Test: REPL-004-002-015 - Display BraceGroup AST
    #[test]
    fn test_REPL_004_002_display_brace_group() {
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("inside brace".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "pwd".to_string(),
                        args: vec![],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                ],
                subshell: false,
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("BraceGroup (2 statements)"));
    }

    /// Test: REPL-004-002-016 - Display function AST
    #[test]
    fn test_REPL_004_002_display_function() {
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "my_function".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("Hello".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Function: my_function (1 statements)"));
    }

    /// Test: REPL-004-002-017 - Display C-style for loop AST
    #[test]
    fn test_REPL_004_002_display_c_style_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
                init: "i=0".to_string(),
                condition: "i<10".to_string(),
                increment: "i++".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("C-style for loop: i=0; i<10; i++ (1 statements)"));
    }

    /// Test: REPL-004-002-018 - Display if with elif and else
    #[test]
    fn test_REPL_004_002_display_if_with_elif_else() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("test1".to_string()),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("first".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                elif_blocks: vec![
                    (BashExpr::Literal("test2".to_string()), vec![]),
                    (BashExpr::Literal("test3".to_string()), vec![]),
                ],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("default".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }]),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 7,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("If statement"));
        assert!(output.contains("then: 1 statements"));
        assert!(output.contains("elif: 2 branches"));
        assert!(output.contains("else: present"));
    }

    /// Test: REPL-004-002-019 - Display command with no args
    #[test]
    fn test_REPL_004_002_display_command_no_args() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "pwd".to_string(),
                args: vec![],
                redirects: vec![],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        // Command with no args should not have "(args: X)"
        assert!(output.contains("Command: pwd"));
        assert!(!output.contains("(args: 0)"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::bash_parser::ast::{AstMetadata, BashExpr, Span};
    use proptest::prelude::*;

    fn dummy_span() -> Span {
        Span::new(1, 1, 1, 10)
    }

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    // Property: format_ast never panics on valid AST
    proptest! {
        #[test]
        fn prop_format_ast_never_panics(stmt_count in 0usize..100) {
            // Generate AST with variable number of simple commands
            let statements: Vec<BashStmt> = (0..stmt_count)
                .map(|i| BashStmt::Command {
                    name: format!("cmd{}", i),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                })
                .collect();

            let ast = BashAst {
                statements,
                metadata: AstMetadata {
                    source_file: None,
                    line_count: stmt_count,
                    parse_time_ms: 0,
                },
            };

            // Should never panic
            let _ = format_ast(&ast);
        }
    }

    // Property: Output always contains header and statement count
    proptest! {
        #[test]
        fn prop_output_has_header_and_count(stmt_count in 0usize..50) {
            let statements: Vec<BashStmt> = (0..stmt_count)
                .map(|i| BashStmt::Command {
                    name: format!("echo{}", i),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                })
                .collect();

            let ast = BashAst {
                statements,
                metadata: AstMetadata {
                    source_file: None,
                    line_count: stmt_count,
                    parse_time_ms: 0,
                },
            };

            let output = format_ast(&ast);

            // Verify header present
            prop_assert!(output.contains("=== AST ==="), "Should have header");

            // Verify statement count matches
            let expected_count = format!("Statements: {}", stmt_count);
            prop_assert!(output.contains(&expected_count), "Should show correct count");
        }
    }

    // Property: Every statement produces non-empty formatted output
    proptest! {
        #[test]
        fn prop_statements_produce_output(
            cmd_name in "[a-z]{1,20}",
            var_name in "[A-Z]{1,20}",
            loop_var in "[a-z]{1,10}"
        ) {
            // Test various statement types
            let statements = vec![
                BashStmt::Command {
                    name: cmd_name.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                },
                BashStmt::Assignment {
                    name: var_name.clone(),
            index: None,
                    value: BashExpr::Literal("test".to_string()),
                    exported: false,
                    span: dummy_span(),
                },
                BashStmt::For {
                    variable: loop_var.clone(),
                    items: BashExpr::Array(vec![]),
                    body: vec![],
                    span: dummy_span(),
                },
            ];

            let ast = BashAst {
                statements,
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 3,
                    parse_time_ms: 0,
                },
            };

            let output = format_ast(&ast);

            // Verify each statement type appears in output
            prop_assert!(output.contains(&cmd_name), "Command should appear");
            prop_assert!(output.contains(&var_name), "Variable should appear");
            prop_assert!(output.contains(&loop_var), "Loop variable should appear");
        }
    }

    // Property: format_statement is deterministic
    proptest! {
        #[test]
        fn prop_format_statement_deterministic(
            cmd in "[a-z]{1,15}",
            indent in 0usize..10
        ) {
            let stmt = BashStmt::Command {
                name: cmd,
                args: vec![],
                redirects: vec![],
                span: dummy_span(),
            };

            // Format twice with same inputs
            let output1 = format_statement(&stmt, indent);
            let output2 = format_statement(&stmt, indent);

            // Should be identical
            prop_assert_eq!(output1, output2, "Same input should produce same output");
        }
    }

    // Property: Indentation depth affects output correctly
    proptest! {
        #[test]
        fn prop_indentation_works(indent in 0usize..20) {
            let stmt = BashStmt::Command {
                name: "test".to_string(),
                args: vec![],
                redirects: vec![],
                span: dummy_span(),
            };

            let output = format_statement(&stmt, indent);

            // Expected indent string
            let expected_indent = "  ".repeat(indent);

            // Output should start with correct indentation
            prop_assert!(
                output.starts_with(&expected_indent),
                "Should have correct indentation: expected '{}', got '{}'",
                expected_indent,
                &output[..std::cmp::min(indent * 2, output.len())]
            );
        }
    }

    // Property: Empty AST produces minimal valid output
    proptest! {
        #[test]
        fn prop_empty_ast_valid(line_count in 0usize..10) {
            let ast = BashAst {
                statements: vec![],
                metadata: AstMetadata {
                    source_file: None,
                    line_count,
                    parse_time_ms: 0,
                },
            };

            let output = format_ast(&ast);

            // Should have header and zero count
            prop_assert!(output.contains("=== AST ==="), "Should have header");
            prop_assert!(output.contains("Statements: 0"), "Should show zero statements");
            prop_assert!(!output.is_empty(), "Should not be empty");
        }
    }
}
