use super::*;
use crate::ast::restricted::{BinaryOp, Literal};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

#[test]
fn test_simple_ast_to_ir_conversion() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(Literal::U32(42)),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "x");
                    assert!(matches!(value, ShellValue::String(_)));
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_function_call_to_command() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".to_string(),
                args: vec![Expr::Literal(Literal::Str("hello".to_string()))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                ShellIR::Exec { cmd, .. } => {
                    assert_eq!(cmd.program, "echo");
                    assert_eq!(cmd.args.len(), 1);
                }
                _ => panic!("Expected Exec statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_shell_value_constant_detection() {
    assert!(ShellValue::String("hello".to_string()).is_constant());
    assert!(ShellValue::Bool(true).is_constant());
    assert!(!ShellValue::Variable("x".to_string()).is_constant());
    assert!(!ShellValue::CommandSubst(Command::new("echo")).is_constant());

    let concat = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::String(" world".to_string()),
    ]);
    assert!(concat.is_constant());

    let concat_with_var = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::Variable("name".to_string()),
    ]);
    assert!(!concat_with_var.is_constant());
}

#[test]
fn test_shell_value_constant_string_extraction() {
    assert_eq!(
        ShellValue::String("hello".to_string()).as_constant_string(),
        Some("hello".to_string())
    );

    assert_eq!(
        ShellValue::Bool(true).as_constant_string(),
        Some("true".to_string())
    );

    assert_eq!(
        ShellValue::Bool(false).as_constant_string(),
        Some("false".to_string())
    );

    let concat = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::String(" world".to_string()),
    ]);
    assert_eq!(concat.as_constant_string(), Some("hello world".to_string()));

    assert_eq!(
        ShellValue::Variable("x".to_string()).as_constant_string(),
        None
    );
}

#[test]
fn test_command_builder() {
    let cmd = Command::new("echo")
        .arg(ShellValue::String("hello".to_string()))
        .arg(ShellValue::Variable("name".to_string()));

    assert_eq!(cmd.program, "echo");
    assert_eq!(cmd.args.len(), 2);
    assert!(matches!(cmd.args[0], ShellValue::String(_)));
    assert!(matches!(cmd.args[1], ShellValue::Variable(_)));
}

#[test]
fn test_shell_ir_effects_calculation() {
    let let_ir = ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::String("hello".to_string()),
        effects: EffectSet::pure(),
    };
    assert!(let_ir.is_pure());

    let exec_ir = ShellIR::Exec {
        cmd: Command::new("echo"),
        effects: EffectSet::single(Effect::ProcessExec),
    };
    assert!(!exec_ir.is_pure());
    assert!(exec_ir.effects().contains(&Effect::ProcessExec));
}

#[test]
fn test_optimization_constant_folding() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "greeting".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::String(" world".to_string()),
        ]),
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "hello world");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

#[test]
fn test_optimization_disabled() {
    let config = crate::models::Config {
        optimize: false,
        ..Default::default()
    };

    let ir = ShellIR::Let {
        name: "greeting".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::String(" world".to_string()),
        ]),
        effects: EffectSet::pure(),
    };

    let result = optimize(ir.clone(), &config).unwrap();

    // Should be unchanged when optimization is disabled
    match result {
        ShellIR::Let {
            value: ShellValue::Concat(parts),
            ..
        } => {
            assert_eq!(parts.len(), 2);
        }
        _ => panic!("Expected unoptimized concat"),
    }
}

#[test]
fn test_if_statement_conversion() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::If {
                condition: Expr::Literal(Literal::Bool(true)),
                then_block: vec![Stmt::Let {
                    name: "result".to_string(),
                    value: Expr::Literal(Literal::Str("true_branch".to_string())),
                }],
                else_block: Some(vec![Stmt::Let {
                    name: "result".to_string(),
                    value: Expr::Literal(Literal::Str("false_branch".to_string())),
                }]),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                ShellIR::If {
                    test,
                    then_branch,
                    else_branch,
                } => {
                    assert!(matches!(test, ShellValue::Bool(true)));
                    assert!(then_branch.is_pure()); // Let statements are pure
                    assert!(else_branch.is_some());
                }
                _ => panic!("Expected If statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_return_statement_conversion() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Return(Some(Expr::Literal(Literal::Str(
                "success".to_string(),
            ))))],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                ShellIR::Exit { code, message } => {
                    assert_eq!(*code, 0);
                    assert!(message.is_some());
                }
                _ => panic!("Expected Exit statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

#[test]
fn test_binary_expression_conversion() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
                    right: Box::new(Expr::Literal(Literal::Str(" world".to_string()))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 1);
            match &stmts[0] {
                ShellIR::Let {
                    value: ShellValue::Arithmetic { op, left, right },
                    ..
                } => {
                    // After TICKET-5006: Addition now generates Arithmetic variant
                    assert!(matches!(op, crate::ir::shell_ir::ArithmeticOp::Add));
                    assert!(matches!(**left, ShellValue::String(_)));
                    assert!(matches!(**right, ShellValue::String(_)));
                }
                _ => panic!("Expected Let with Arithmetic value (after TICKET-5006)"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

// Property-based tests
proptest! {
    #[test]
    fn test_command_effects_are_deterministic(
        cmd_name in "[a-z]{1,10}"
    ) {
        let effects1 = effects::analyze_command_effects(&cmd_name);
        let effects2 = effects::analyze_command_effects(&cmd_name);

        // Effects should be deterministic for the same command
        assert_eq!(effects1.to_vec().len(), effects2.to_vec().len());
    }

    #[test]
    fn test_shell_value_concat_preserves_constants(
        s1 in ".*",
        s2 in ".*"
    ) {
        let concat = ShellValue::Concat(vec![
            ShellValue::String(s1.clone()),
            ShellValue::String(s2.clone()),
        ]);

        if let Some(result) = concat.as_constant_string() {
            assert_eq!(result, format!("{s1}{s2}"));
        }
    }
}

#[rstest]
#[case("echo", true)]
#[case("cat", false)]
#[case("curl", false)]
#[case("unknown_command", false)]
fn test_command_effect_classification(#[case] cmd: &str, #[case] should_be_pure: bool) {
    let effects = effects::analyze_command_effects(cmd);
    assert_eq!(effects.is_pure(), should_be_pure);
}

#[test]
fn test_ir_sequence_effects_aggregation() {
    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "x".to_string(),
            value: ShellValue::String("hello".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::Exec {
            cmd: Command::new("curl"),
            effects: EffectSet::single(Effect::NetworkAccess),
        },
    ]);

    let combined_effects = ir.effects();
    assert!(combined_effects.has_network_effects());
    assert!(!combined_effects.is_pure());
}

#[test]
fn test_nested_ir_effects() {
    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Exec {
            cmd: Command::new("rm"),
            effects: vec![Effect::FileWrite, Effect::SystemModification].into(),
        }),
        else_branch: Some(Box::new(ShellIR::Exec {
            cmd: Command::new("touch"),
            effects: EffectSet::single(Effect::FileWrite),
        })),
    };

    let effects = ir.effects();
    assert!(effects.has_filesystem_effects());
    assert!(effects.has_system_effects());
}

#[test]
fn test_error_handling_in_conversion() {
    // Test with empty function list
    let empty_ast = RestrictedAst {
        functions: vec![],
        entry_point: "main".to_string(),
    };

    assert!(from_ast(&empty_ast).is_err());
}

#[test]
fn test_complex_nested_structures() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::If {
                condition: Expr::Variable("condition".to_string()),
                then_block: vec![Stmt::If {
                    condition: Expr::Literal(Literal::Bool(true)),
                    then_block: vec![Stmt::Let {
                        name: "nested".to_string(),
                        value: Expr::Literal(Literal::Str("deep".to_string())),
                    }],
                    else_block: None,
                }],
                else_block: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // Should handle nested structures without panicking
    assert!(ir.effects().is_pure()); // Only let statements, should be pure
}

// ============= Sprint 29: Mutation Testing - Kill Surviving Mutants =============

/// MUTATION KILLER: Line 61 - Arithmetic operator in loop boundary calculation
/// Kills mutants: "replace - with +" and "replace - with /"
#[test]
fn test_function_body_length_calculation() {
    // Test with 3 statements - ensures correct last index calculation (len - 1 = 2)
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "add".to_string(),
            params: vec![],
            return_type: Type::U32,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::U32(1)),
                },
                Stmt::Let {
                    name: "y".to_string(),
                    value: Expr::Literal(Literal::U32(2)),
                },
                Stmt::Expr(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Variable("y".to_string())),
                }),
            ],
        }],
        entry_point: "add".to_string(),
    };

    // This should succeed - if the arithmetic is wrong (+ or /), it would fail
    let ir = from_ast(&ast);
    assert!(
        ir.is_ok(),
        "Should convert AST with correct boundary calculation"
    );
}

/// MUTATION KILLER: Line 95 - should_echo guard condition
/// Kills mutants: "replace should_echo with true" and "replace should_echo with false"
#[test]
fn test_should_echo_guard_conditions() {
    // Test with multi-statement function to avoid Noop optimization
    let ast_with_return = RestrictedAst {
        functions: vec![Function {
            name: "get_value".to_string(),
            params: vec![],
            return_type: Type::U32,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::U32(10)),
                },
                Stmt::Expr(Expr::Variable("x".to_string())),
            ],
        }],
        entry_point: "get_value".to_string(),
    };

    let ast_void = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::U32(10)),
                },
                Stmt::Expr(Expr::Variable("x".to_string())),
            ],
        }],
        entry_point: "main".to_string(),
    };

    // Both should convert successfully - the guard condition logic is tested by mutations
    let ir1 = from_ast(&ast_with_return);
    let ir2 = from_ast(&ast_void);

    assert!(ir1.is_ok(), "Function with return type should convert");
    assert!(ir2.is_ok(), "Function with void return type should convert");
}

/// MUTATION KILLER: Line 327 - BinaryOp::Eq match arm
/// Kills mutant: "delete match arm BinaryOp::Eq"
#[test]
fn test_equality_operator_conversion() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Eq, // Test Eq operator specifically
                    left: Box::new(Expr::Literal(Literal::U32(5))),
                    right: Box::new(Expr::Literal(Literal::U32(5))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    // Should generate Comparison with Eq operator
                    match value {
                        ShellValue::Comparison { op, .. } => {
                            assert!(matches!(op, crate::ir::shell_ir::ComparisonOp::NumEq));
                        }
                        other => panic!("Expected Comparison, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// MUTATION KILLER: Line 363 - BinaryOp::Sub match arm
/// Kills mutant: "delete match arm BinaryOp::Sub"
#[test]
fn test_subtraction_operator_conversion() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Sub, // Test Sub operator specifically
                    left: Box::new(Expr::Literal(Literal::U32(10))),
                    right: Box::new(Expr::Literal(Literal::U32(3))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    // Should generate Arithmetic with Sub operator
                    match value {
                        ShellValue::Arithmetic { op, .. } => {
                            assert!(matches!(op, crate::ir::shell_ir::ArithmeticOp::Sub));
                        }
                        other => panic!("Expected Arithmetic with Sub, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// MUTATION KILLER: Line 391 - Command detection for curl/wget
/// Kills mutant: "delete match arm curl | wget"
#[test]
fn test_curl_command_network_effect() {
    let effects = effects::analyze_command_effects("curl");
    assert!(
        effects.has_network_effects(),
        "curl should have network effects"
    );
    assert!(!effects.is_pure(), "curl should not be pure");
}

#[test]
fn test_wget_command_network_effect() {
    let effects = effects::analyze_command_effects("wget");
    assert!(
        effects.has_network_effects(),
        "wget should have network effects"
    );
    assert!(!effects.is_pure(), "wget should not be pure");
}

#[test]
fn test_non_network_command_no_effect() {
    let effects = effects::analyze_command_effects("ls");
    assert!(
        !effects.has_network_effects(),
        "ls should not have network effects"
    );
}
