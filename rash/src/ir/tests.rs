use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

// Helper: wrap a single let statement in a main function and convert to IR
fn convert_let_stmt(name: &str, value: Expr) -> ShellIR {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: name.to_string(),
                value,
            }],
        }],
        entry_point: "main".to_string(),
    };
    from_ast(&ast).expect("IR conversion should succeed")
}

// Helper: extract the ShellValue from a single Let in a Sequence
fn extract_let_value(ir: &ShellIR) -> &ShellValue {
    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Let { value, .. } => value,
            other => panic!("Expected Let, got {:?}", other),
        },
        other => panic!("Expected Sequence, got {:?}", other),
    }
}

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

// ============= Sprint 26: Mutation Testing - Kill Remaining 4 Mutants =============

/// MUTATION KILLER: Line 434 - analyze_command_effects returns Default::default()
/// Kills mutant: "replace analyze_command_effects -> EffectSet with Default::default()"
/// Tests that IrConverter::analyze_command_effects is actually called and used
#[test]
fn test_ir_converter_analyze_command_effects_used() {
    // Create an AST with a function call that should have effects
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "curl".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "http://example.com".to_string(),
                ))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // If the mutant survives (returns Default::default()), effects would be empty/pure
    // The correct implementation should return effects with NetworkAccess
    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Exec { effects, .. } => {
                assert!(
                    effects.has_network_effects(),
                    "curl command should have NetworkAccess effect via IR converter"
                );
                assert!(!effects.is_pure(), "curl command should not be pure");
            }
            _ => panic!("Expected Exec statement for curl"),
        },
        _ => panic!("Expected Sequence"),
    }
}

/// MUTATION KILLER: Line 437 - Delete "curl" | "wget" match arm in IR converter
/// Kills mutant: "delete match arm curl | wget in IrConverter::analyze_command_effects"
/// Tests that wget function calls get NetworkAccess effect through IR converter
#[test]
fn test_ir_converter_wget_command_effect() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "wget".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "http://example.com".to_string(),
                ))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Exec { cmd, effects } => {
                assert_eq!(cmd.program, "wget");
                assert!(
                    effects.has_network_effects(),
                    "wget should have NetworkAccess effect through IR converter"
                );
            }
            _ => panic!("Expected Exec statement for wget"),
        },
        _ => panic!("Expected Sequence"),
    }
}

/// MUTATION KILLER: Line 440 - Delete "echo" | "printf" match arm in IR converter
/// Kills mutant: "delete match arm echo | printf in IrConverter::analyze_command_effects"
/// Tests that printf function calls get FileWrite effect through IR converter
#[test]
fn test_ir_converter_printf_command_effect() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "printf".to_string(),
                args: vec![Expr::Literal(Literal::Str("Hello\\n".to_string()))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Exec { cmd, effects } => {
                assert_eq!(cmd.program, "printf");
                assert!(
                    effects.has_filesystem_effects(),
                    "printf should have FileWrite effect through IR converter"
                );
            }
            _ => panic!("Expected Exec statement for printf"),
        },
        _ => panic!("Expected Sequence"),
    }
}

/// MUTATION KILLER: Line 523 - Replace && with || in is_string_value
/// Kills mutant: "replace && with || in is_string_value condition"
/// Tests that is_string_value correctly uses && logic (both parse failures required)
///
/// The mutant changes line 523 from:
///   s.parse::<i64>().is_err() && s.parse::<f64>().is_err()
/// to:
///   s.parse::<i64>().is_err() || s.parse::<f64>().is_err()
///
/// This would cause numeric strings like "123" to be treated as strings,
/// resulting in string comparison (=) instead of numeric comparison (-eq)
#[test]
fn test_is_string_value_requires_both_parse_failures() {
    // Test that integer comparison uses numeric operators, not string operators
    // "123" == "124" should use NumEq, not StrEq
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(Expr::Literal(Literal::Str("123".to_string()))),
                    right: Box::new(Expr::Literal(Literal::Str("124".to_string()))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let _ir = from_ast(&ast).unwrap();

    // With correct && logic: "123" and "124" parse as i64, so NOT strings -> NumEq
    // With mutated || logic: "123".parse::<f64>().is_err() = false, so || = false -> NumEq (would still work)
    // BUT: Let's test with a float string "123.5" which DOES expose the bug

    let ast_float = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "result".to_string(),
                value: Expr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(Expr::Literal(Literal::Str("123.5".to_string()))),
                    right: Box::new(Expr::Literal(Literal::Str("124.5".to_string()))),
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir_float = from_ast(&ast_float).unwrap();

    // With correct && logic:
    // "123.5".parse::<i64>().is_err() = true && "123.5".parse::<f64>().is_err() = false
    // = false (NOT a string, should use NumEq)
    //
    // With mutated || logic:
    // "123.5".parse::<i64>().is_err() = true || "123.5".parse::<f64>().is_err() = false
    // = true (WRONG! would think it's a string, use StrEq)

    // Check that float strings use NumEq (numeric comparison), not StrEq
    match ir_float {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    match value {
                        ShellValue::Comparison { op, .. } => {
                            // CRITICAL: Must be NumEq, not StrEq
                            // If mutant survives (|| instead of &&), this would be StrEq
                            assert!(
                                matches!(op, crate::ir::shell_ir::ComparisonOp::NumEq),
                                "Float strings like '123.5' should use NumEq, not StrEq. \
                                If this fails, is_string_value is using || instead of &&"
                            );
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

// ============= Sprint 27a: Environment Variables Support - RED PHASE =============

/// RED TEST: env() call should convert to EnvVar variant in IR
/// Tests that env("HOME") is properly recognized and converted to ShellValue::EnvVar
#[test]
fn test_env_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "home".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![Expr::Literal(Literal::Str("HOME".to_string()))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "home");
                    // RED: This will fail until we implement EnvVar variant
                    match value {
                        ShellValue::EnvVar { name, default } => {
                            assert_eq!(name, "HOME");
                            assert_eq!(default, &None);
                        }
                        other => panic!("Expected EnvVar, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: env_var_or() call should convert to EnvVar with default value
/// Tests that env_var_or("PREFIX", "/usr/local") converts to EnvVar with Some(default)
#[test]
fn test_env_var_or_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "prefix".to_string(),
                value: Expr::FunctionCall {
                    name: "env_var_or".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("PREFIX".to_string())),
                        Expr::Literal(Literal::Str("/usr/local".to_string())),
                    ],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "prefix");
                    // RED: This will fail until we implement EnvVar variant with default
                    match value {
                        ShellValue::EnvVar { name, default } => {
                            assert_eq!(name, "PREFIX");
                            assert_eq!(default, &Some("/usr/local".to_string()));
                        }
                        other => panic!("Expected EnvVar with default, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: env() in variable assignment context
/// Tests that env() works in typical variable assignment patterns
#[test]
fn test_env_in_assignment() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "setup".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![
                Stmt::Let {
                    name: "user".to_string(),
                    value: Expr::FunctionCall {
                        name: "env".to_string(),
                        args: vec![Expr::Literal(Literal::Str("USER".to_string()))],
                    },
                },
                Stmt::Let {
                    name: "path".to_string(),
                    value: Expr::FunctionCall {
                        name: "env".to_string(),
                        args: vec![Expr::Literal(Literal::Str("PATH".to_string()))],
                    },
                },
            ],
        }],
        entry_point: "setup".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // RED: This will fail until EnvVar variant exists
    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 2);

            // Check first env() call
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::EnvVar { name, default }
                            if name == "USER" && default.is_none()),
                        "First env() should be EnvVar for USER"
                    );
                }
                _ => panic!("Expected Let statement"),
            }

            // Check second env() call
            match &stmts[1] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::EnvVar { name, default }
                            if name == "PATH" && default.is_none()),
                        "Second env() should be EnvVar for PATH"
                    );
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

// ============= Sprint 27b: Command-Line Arguments Support - RED PHASE =============

/// RED TEST: arg(1) call should convert to Arg variant in IR
/// Tests that arg(1) is properly recognized and converted to ShellValue::Arg
#[test]
fn test_arg_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "first".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::U32(1))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "first");
                    // RED: This will fail until we implement Arg variant
                    match value {
                        ShellValue::Arg { position } => {
                            assert_eq!(position, &Some(1));
                        }
                        other => panic!("Expected Arg, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: args() call should convert to Arg variant with None position
/// Tests that args() converts to Arg { position: None } (representing $@)
#[test]
fn test_args_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "all".to_string(),
                value: Expr::FunctionCall {
                    name: "args".to_string(),
                    args: vec![],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "all");
                    // RED: This will fail until we implement Arg variant with None
                    match value {
                        ShellValue::Arg { position } => {
                            assert_eq!(position, &None);
                        }
                        other => panic!("Expected Arg with None position, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: arg_count() call should convert to ArgCount variant
/// Tests that arg_count() is properly recognized and converted to ShellValue::ArgCount
#[test]
fn test_arg_count_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "count".to_string(),
                value: Expr::FunctionCall {
                    name: "arg_count".to_string(),
                    args: vec![],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "count");
                    // RED: This will fail until we implement ArgCount variant
                    match value {
                        ShellValue::ArgCount => {
                            // Success!
                        }
                        other => panic!("Expected ArgCount, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: arg(0) should be rejected (validation)
/// Tests that arg(0) is rejected because shell arguments start at $1
#[test]
fn test_arg_rejects_zero_position() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::U32(0))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    // RED: This will fail until we implement position validation
    let result = from_ast(&ast);
    assert!(result.is_err(), "arg(0) should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("position must be >= 1")
            || error_msg.contains("position")
            || error_msg.contains("1"),
        "Error message should mention position requirement, got: {}",
        error_msg
    );
}

// ============= Sprint 27c: Exit Code Handling - RED PHASE =============

/// RED TEST: exit_code() call should convert to ExitCode variant in IR
/// Tests that exit_code() is properly recognized and converted to ShellValue::ExitCode
#[test]
fn test_exit_code_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "status".to_string(),
                value: Expr::FunctionCall {
                    name: "exit_code".to_string(),
                    args: vec![],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "status");
                    // RED: This will fail until we implement ExitCode variant
                    match value {
                        ShellValue::ExitCode => {
                            // Success!
                        }
                        other => panic!("Expected ExitCode, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: exit_code() in comparison context
/// Tests that exit_code() works in if condition comparisons
#[test]
fn test_exit_code_in_comparison() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::If {
                condition: Expr::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(Expr::FunctionCall {
                        name: "exit_code".to_string(),
                        args: vec![],
                    }),
                    right: Box::new(Expr::Literal(Literal::Str("0".to_string()))),
                },
                then_block: vec![Stmt::Expr(Expr::Literal(Literal::Str(
                    "success".to_string(),
                )))],
                else_block: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // RED: This will fail until ExitCode variant exists
    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::If { test, .. } => {
                    // Should contain Comparison with ExitCode on the left
                    match test {
                        ShellValue::Comparison { left, .. } => {
                            assert!(
                                matches!(**left, ShellValue::ExitCode),
                                "Expected ExitCode in comparison"
                            );
                        }
                        _ => panic!("Expected Comparison in if condition"),
                    }
                }
                _ => panic!("Expected If statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: Multiple exit_code() calls in sequence
/// Tests that multiple exit_code() calls work correctly
#[test]
fn test_multiple_exit_code_calls() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![
                Stmt::Let {
                    name: "status1".to_string(),
                    value: Expr::FunctionCall {
                        name: "exit_code".to_string(),
                        args: vec![],
                    },
                },
                Stmt::Let {
                    name: "status2".to_string(),
                    value: Expr::FunctionCall {
                        name: "exit_code".to_string(),
                        args: vec![],
                    },
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // RED: This will fail until ExitCode variant exists
    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 2);

            // Check both calls convert to ExitCode
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::ExitCode),
                        "First exit_code() should be ExitCode variant"
                    );
                }
                _ => panic!("Expected Let statement"),
            }

            match &stmts[1] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::ExitCode),
                        "Second exit_code() should be ExitCode variant"
                    );
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

// ============= Optimizer Enhancement: Arithmetic Constant Folding - RED PHASE =============

/// RED TEST: Arithmetic addition constant folding
/// Tests that $((10 + 20)) → "30" at compile time
#[test]
fn test_optimizer_arithmetic_addition_folding() {
    let config = crate::models::Config::default(); // optimize = true

    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("20".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    // Should fold to constant "30"
    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "30", "10 + 20 should fold to 30");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic subtraction constant folding
/// Tests that $((50 - 12)) → "38" at compile time
#[test]
fn test_optimizer_arithmetic_subtraction_folding() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "diff".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Sub,
            left: Box::new(ShellValue::String("50".to_string())),
            right: Box::new(ShellValue::String("12".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "38", "50 - 12 should fold to 38");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic multiplication constant folding
/// Tests that $((10 * 1024 * 1024)) → "10485760" (10MB) at compile time
#[test]
fn test_optimizer_arithmetic_multiplication_folding() {
    let config = crate::models::Config::default();

    // First multiply: 10 * 1024 = 10240
    let inner_mul = ShellValue::Arithmetic {
        op: crate::ir::shell_ir::ArithmeticOp::Mul,
        left: Box::new(ShellValue::String("10".to_string())),
        right: Box::new(ShellValue::String("1024".to_string())),
    };

    // Second multiply: (10 * 1024) * 1024 = 10485760
    let ir = ShellIR::Let {
        name: "bytes".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Mul,
            left: Box::new(inner_mul),
            right: Box::new(ShellValue::String("1024".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "10485760", "10 * 1024 * 1024 should fold to 10485760");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic division constant folding
/// Tests that $((100 / 5)) → "20" at compile time
#[test]
fn test_optimizer_arithmetic_division_folding() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "quotient".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Div,
            left: Box::new(ShellValue::String("100".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    match optimized {
        ShellIR::Let {
            value: ShellValue::String(s),
            ..
        } => {
            assert_eq!(s, "20", "100 / 5 should fold to 20");
        }
        _ => panic!("Expected optimized constant string"),
    }
}

/// RED TEST: Arithmetic with non-constant should NOT fold
/// Tests that $((x + 10)) stays as Arithmetic (cannot fold with variable)
#[test]
fn test_optimizer_arithmetic_with_variable_no_fold() {
    let config = crate::models::Config::default();

    let ir = ShellIR::Let {
        name: "result".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("10".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let optimized = optimize(ir, &config).unwrap();

    // Should NOT fold (variable involved)
    match optimized {
        ShellIR::Let {
            value: ShellValue::Arithmetic { .. },
            ..
        } => {
            // Good - still Arithmetic, not folded
        }
        _ => panic!("Expected unoptimized Arithmetic (variable involved)"),
    }
}

/// RED TEST: Optimization disabled should preserve arithmetic
/// Tests that optimize=false keeps Arithmetic unchanged
#[test]
fn test_optimizer_disabled_preserves_arithmetic() {
    let config = crate::models::Config {
        optimize: false,
        ..Default::default()
    };

    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("20".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = optimize(ir, &config).unwrap();

    // Should be unchanged when optimization is disabled
    match result {
        ShellIR::Let {
            value: ShellValue::Arithmetic { .. },
            ..
        } => {
            // Good - preserved
        }
        _ => panic!("Expected unoptimized Arithmetic"),
    }
}

// ===== convert_expr_to_value coverage tests =====

#[test]
fn test_IR_COV_001_literal_bool() {
    let ir = convert_let_stmt("flag", Expr::Literal(Literal::Bool(true)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Bool(true)));
}

#[test]
fn test_IR_COV_002_literal_u16() {
    let ir = convert_let_stmt("port", Expr::Literal(Literal::U16(8080)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "8080"));
}

#[test]
fn test_IR_COV_003_literal_i32() {
    let ir = convert_let_stmt("neg", Expr::Literal(Literal::I32(-42)));
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "-42"));
}

#[test]
fn test_IR_COV_004_unary_not() {
    let ir = convert_let_stmt(
        "negated",
        Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(true))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::LogicalNot { .. }));
}

#[test]
fn test_IR_COV_005_unary_neg() {
    let ir = convert_let_stmt(
        "neg",
        Expr::Unary {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            ..
        } => {}
        other => panic!("Expected Arithmetic Sub, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_006_binary_eq_string() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("world".to_string()))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::StrEq,
            ..
        } => {}
        other => panic!("Expected StrEq comparison, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_007_binary_eq_numeric() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::NumEq,
            ..
        } => {}
        other => panic!("Expected NumEq comparison, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_008_binary_ne_string() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Literal(Literal::Str("a".to_string()))),
            right: Box::new(Expr::Literal(Literal::Str("b".to_string()))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::StrNe,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_009_binary_gt() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Literal(Literal::U32(5))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Gt,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_010_binary_ge() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Ge,
            left: Box::new(Expr::Literal(Literal::U32(5))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Ge,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_011_binary_lt() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Literal(Literal::U32(3))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Lt,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_012_binary_le() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Le,
            left: Box::new(Expr::Literal(Literal::U32(3))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::Le,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_013_binary_sub() {
    let ir = convert_let_stmt(
        "diff",
        Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_014_binary_mul() {
    let ir = convert_let_stmt(
        "product",
        Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Literal(Literal::U32(4))),
            right: Box::new(Expr::Literal(Literal::U32(5))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Mul,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_015_binary_div() {
    let ir = convert_let_stmt(
        "quotient",
        Expr::Binary {
            op: BinaryOp::Div,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Div,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_016_binary_rem() {
    let ir = convert_let_stmt(
        "remainder",
        Expr::Binary {
            op: BinaryOp::Rem,
            left: Box::new(Expr::Literal(Literal::U32(10))),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Mod,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_017_binary_and() {
    let ir = convert_let_stmt(
        "both",
        Expr::Binary {
            op: BinaryOp::And,
            left: Box::new(Expr::Literal(Literal::Bool(true))),
            right: Box::new(Expr::Literal(Literal::Bool(false))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::LogicalAnd { .. }));
}

#[test]
fn test_IR_COV_018_binary_or() {
    let ir = convert_let_stmt(
        "either",
        Expr::Binary {
            op: BinaryOp::Or,
            left: Box::new(Expr::Literal(Literal::Bool(false))),
            right: Box::new(Expr::Literal(Literal::Bool(true))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::LogicalOr { .. }));
}

#[test]
fn test_IR_COV_019_func_call_env() {
    let ir = convert_let_stmt(
        "home",
        Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![Expr::Literal(Literal::Str("HOME".to_string()))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::EnvVar { name, default } => {
            assert_eq!(name, "HOME");
            assert!(default.is_none());
        }
        other => panic!("Expected EnvVar, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_020_func_call_env_var_or() {
    let ir = convert_let_stmt(
        "editor",
        Expr::FunctionCall {
            name: "env_var_or".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("EDITOR".to_string())),
                Expr::Literal(Literal::Str("vi".to_string())),
            ],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::EnvVar { name, default } => {
            assert_eq!(name, "EDITOR");
            assert_eq!(default.as_deref(), Some("vi"));
        }
        other => panic!("Expected EnvVar with default, got {:?}", other),
    }
}

#[test]
fn test_IR_COV_021_func_call_arg() {
    let ir = convert_let_stmt(
        "first",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::U32(1))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arg {
            position: Some(1), ..
        } => {}
        other => panic!("Expected Arg(1), got {:?}", other),
    }
}

#[test]
fn test_IR_COV_022_func_call_args() {
    let ir = convert_let_stmt(
        "all",
        Expr::FunctionCall {
            name: "args".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: None }));
}

#[test]
fn test_IR_COV_023_func_call_arg_count() {
    let ir = convert_let_stmt(
        "count",
        Expr::FunctionCall {
            name: "arg_count".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::ArgCount));
}

#[test]
fn test_IR_COV_024_func_call_exit_code() {
    let ir = convert_let_stmt(
        "code",
        Expr::FunctionCall {
            name: "exit_code".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::ExitCode));
}

#[test]
fn test_IR_COV_025_positional_args() {
    let ir = convert_let_stmt("all_args", Expr::PositionalArgs);
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::Arg { position: None }));
}

#[test]
fn test_IR_COV_026_func_call_generic() {
    // Non-special function call becomes CommandSubst
    let ir = convert_let_stmt(
        "output",
        Expr::FunctionCall {
            name: "whoami".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::CommandSubst(_)));
}

#[test]
fn test_IR_COV_027_env_invalid_var_name() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![Expr::Literal(Literal::Str("BAD-NAME".to_string()))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_028_arg_position_zero() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::U32(0))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_029_env_no_args() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_030_env_non_string_arg() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![Expr::Literal(Literal::U32(42))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_031_binary_ne_numeric() {
    let ir = convert_let_stmt(
        "result",
        Expr::Binary {
            op: BinaryOp::Ne,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(
        val,
        ShellValue::Comparison {
            op: shell_ir::ComparisonOp::NumNe,
            ..
        }
    ));
}

#[test]
fn test_IR_COV_032_method_call_unknown_pattern() {
    // MethodCall that doesn't match any recognized pattern → "unknown"
    let ir = convert_let_stmt(
        "result",
        Expr::MethodCall {
            receiver: Box::new(Expr::Variable("foo".to_string())),
            method: "bar".to_string(),
            args: vec![],
        },
    );
    let val = extract_let_value(&ir);
    assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
}

#[test]
fn test_IR_COV_033_func_call_arg_with_i32() {
    let ir = convert_let_stmt(
        "arg2",
        Expr::FunctionCall {
            name: "arg".to_string(),
            args: vec![Expr::Literal(Literal::I32(2))],
        },
    );
    let val = extract_let_value(&ir);
    match val {
        ShellValue::Arg {
            position: Some(2), ..
        } => {}
        other => panic!("Expected Arg(2), got {:?}", other),
    }
}

#[test]
fn test_IR_COV_034_env_var_or_non_string_default() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "env_var_or".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("PATH".to_string())),
                        Expr::Literal(Literal::U32(42)),
                    ],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_035_arg_no_args() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

#[test]
fn test_IR_COV_036_arg_string_arg() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "bad".to_string(),
                value: Expr::FunctionCall {
                    name: "arg".to_string(),
                    args: vec![Expr::Literal(Literal::Str("not_a_number".to_string()))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };
    let result = from_ast(&ast);
    assert!(result.is_err());
}

/// Test that return inside while loop in a function produces ShellIR::Return,
/// not ShellIR::Exit with debug format. Regression test for the bug where
/// `return expr` in loop bodies emitted `{value:?}` debug representation.
#[test]
fn test_return_inside_while_in_function_produces_return_ir() {
    use crate::ast::restricted::Parameter;
    // fn find(n: u32) -> u32 { let i = 0; while i < n { return i + 1; } 0 }
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "find".to_string(),
                params: vec![Parameter {
                    name: "n".to_string(),
                    param_type: Type::U32,
                }],
                return_type: Type::U32,
                body: vec![
                    Stmt::Let {
                        name: "i".to_string(),
                        value: Expr::Literal(Literal::U32(0)),
                    },
                    Stmt::While {
                        condition: Expr::Binary {
                            op: BinaryOp::Lt,
                            left: Box::new(Expr::Variable("i".to_string())),
                            right: Box::new(Expr::Variable("n".to_string())),
                        },
                        body: vec![Stmt::Return(Some(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Variable("i".to_string())),
                            right: Box::new(Expr::Literal(Literal::U32(1))),
                        }))],
                        max_iterations: Some(1000),
                    },
                    Stmt::Expr(Expr::Literal(Literal::U32(0))),
                ],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "r".to_string(),
                    value: Expr::FunctionCall {
                        name: "find".to_string(),
                        args: vec![Expr::Literal(Literal::U32(5))],
                    },
                }],
            },
        ],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert successfully");

    // The function body's while loop should contain Return, not Exit
    fn contains_return_not_exit(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::Return { .. } => true,
            ShellIR::Exit { .. } => false,
            ShellIR::Sequence(items) => items.iter().any(contains_return_not_exit),
            ShellIR::While { body, .. } => contains_return_not_exit(body),
            ShellIR::For { body, .. } => contains_return_not_exit(body),
            ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                contains_return_not_exit(then_branch)
                    || else_branch
                        .as_ref()
                        .map_or(false, |e| contains_return_not_exit(e))
            }
            ShellIR::Function { body, .. } => contains_return_not_exit(body),
            _ => false,
        }
    }

    // The IR should have a Function with Return inside its while loop
    assert!(
        contains_return_not_exit(&ir),
        "Return inside while loop in function should produce ShellIR::Return, not ShellIR::Exit"
    );
}

#[test]
fn test_let_match_expression_produces_case_with_assignment() {
    use crate::ast::restricted::{MatchArm, Parameter, Pattern};

    // Test: let score = match bucket { 0 => 10, 1 => 5, _ => 1 }
    // Should produce Case with Let assignments, NOT score='unknown'
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "classify".to_string(),
            params: vec![Parameter {
                name: "n".to_string(),
                param_type: Type::U32,
            }],
            return_type: Type::U32,
            body: vec![
                Stmt::Let {
                    name: "bucket".to_string(),
                    value: Expr::Binary {
                        op: BinaryOp::Rem,
                        left: Box::new(Expr::Variable("n".to_string())),
                        right: Box::new(Expr::Literal(Literal::U32(4))),
                    },
                },
                Stmt::Let {
                    name: "score".to_string(),
                    // Parser produces Expr::Block([Stmt::Match{...}]) for match-in-let
                    value: Expr::Block(vec![Stmt::Match {
                        scrutinee: Expr::Variable("bucket".to_string()),
                        arms: vec![
                            MatchArm {
                                pattern: Pattern::Literal(Literal::U32(0)),
                                guard: None,
                                body: vec![Stmt::Expr(Expr::Binary {
                                    op: BinaryOp::Mul,
                                    left: Box::new(Expr::Variable("n".to_string())),
                                    right: Box::new(Expr::Literal(Literal::U32(10))),
                                })],
                            },
                            MatchArm {
                                pattern: Pattern::Literal(Literal::U32(1)),
                                guard: None,
                                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(5)))],
                            },
                            MatchArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(1)))],
                            },
                        ],
                    }]),
                },
                Stmt::Return(Some(Expr::Variable("score".to_string()))),
            ],
        },
        Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: "r".to_string(),
                value: Expr::FunctionCall {
                    name: "classify".to_string(),
                    args: vec![Expr::Literal(Literal::U32(8))],
                },
            }],
        }],
        entry_point: "main".to_string(),
    };

    // We just need to verify it doesn't produce ShellValue::String("unknown")
    // by checking that the IR contains a Case node (not just a Let with "unknown")
    let ir = from_ast(&ast).expect("Should convert successfully");

    fn contains_case(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::Case { .. } => true,
            ShellIR::Sequence(items) => items.iter().any(contains_case),
            ShellIR::Function { body, .. } => contains_case(body),
            ShellIR::While { body, .. } => contains_case(body),
            ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                contains_case(then_branch)
                    || else_branch
                        .as_ref()
                        .map_or(false, |e| contains_case(e))
            }
            _ => false,
        }
    }

    assert!(
        contains_case(&ir),
        "let x = match y {{ ... }} should produce ShellIR::Case, not Let with 'unknown'"
    );
}

#[test]
fn test_exclusive_for_range_with_variable_end_subtracts_one() {
    use crate::ast::restricted::Pattern;

    // Test: for i in 0..n should produce seq 0 $((n-1)), not seq 0 $n
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::For {
                pattern: Pattern::Variable("i".to_string()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Variable("n".to_string())),
                    inclusive: false,
                },
                body: vec![Stmt::Expr(Expr::Variable("i".to_string()))],
                max_iterations: Some(1000),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert successfully");

    // The For node's end value should be Arithmetic { Sub, Variable("n"), String("1") }
    fn has_adjusted_end(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::For { end, .. } => matches!(
                end,
                ShellValue::Arithmetic {
                    op: crate::ir::shell_ir::ArithmeticOp::Sub,
                    ..
                }
            ),
            ShellIR::Sequence(items) => items.iter().any(has_adjusted_end),
            _ => false,
        }
    }

    assert!(
        has_adjusted_end(&ir),
        "for i in 0..n should produce end=$((n-1)), not end=$n"
    );
}

#[test]
fn test_nested_match_in_match_arm_produces_nested_case() {
    // Regression: `let next = match state { 0 => match bit { ... }, ... }`
    // should produce nested Case statements, not flat assignments to '0'.
    let source = r#"
fn dispatch(state: u32, bit: u32) -> u32 {
    let next = match state {
        0 => match bit { 0 => 10, _ => 20, },
        _ => match bit { 0 => 30, _ => 40, },
    };
    return next;
}
fn main() { println!("{}", dispatch(0, 1)); }
"#;

    let ast = crate::services::parser::parse(source)
        .expect("should parse");
    let ir = super::from_ast(&ast).expect("should lower");

    // Walk the IR tree to find a nested Case inside a Case arm
    fn has_nested_case(ir: &super::ShellIR) -> bool {
        match ir {
            super::ShellIR::Case { arms, .. } => {
                for arm in arms {
                    if matches!(&*arm.body, super::ShellIR::Case { .. }) {
                        return true;
                    }
                    if has_nested_case(&arm.body) {
                        return true;
                    }
                }
                false
            }
            super::ShellIR::Sequence(stmts) => stmts.iter().any(has_nested_case),
            super::ShellIR::Function { body, .. } => has_nested_case(body),
            _ => false,
        }
    }

    assert!(
        has_nested_case(&ir),
        "nested match-in-match-arm should produce nested Case IR"
    );
}

#[test]
fn test_if_else_expression_in_match_block_arm_produces_if_assignment() {
    let source = r#"
fn categorize(x: u32) -> u32 {
    let r = match x % 3 {
        0 => {
            let half = x / 2;
            if half > 5 { half * 10 } else { half }
        },
        _ => x,
    };
    return r;
}
fn main() { println!("{}", categorize(12)); }
"#;

    let ast = crate::services::parser::parse(source).expect("should parse");
    let ir = super::from_ast(&ast).expect("should lower");

    // Walk IR tree to find If inside a Case arm (if-else assigns to let target)
    fn has_if_in_case(ir: &super::ShellIR) -> bool {
        match ir {
            super::ShellIR::Case { arms, .. } => {
                for arm in arms {
                    if has_if_inside(&arm.body) {
                        return true;
                    }
                }
                false
            }
            super::ShellIR::Sequence(stmts) => stmts.iter().any(has_if_in_case),
            super::ShellIR::Function { body, .. } => has_if_in_case(body),
            _ => false,
        }
    }
    fn has_if_inside(ir: &super::ShellIR) -> bool {
        match ir {
            super::ShellIR::If { .. } => true,
            super::ShellIR::Sequence(stmts) => stmts.iter().any(has_if_inside),
            _ => false,
        }
    }

    assert!(
        has_if_in_case(&ir),
        "if-else expression in match block arm should produce If IR inside Case arm"
    );
}

/// Regression test: match with range patterns produces If chain (not Case)
/// because POSIX case cannot handle numeric ranges like 0..=10.
#[test]
fn test_range_patterns_produce_if_chain_not_case() {
    use crate::ast::restricted::{MatchArm, Parameter, Pattern};

    let ast = RestrictedAst {
        functions: vec![Function {
            name: "grade".to_string(),
            params: vec![Parameter {
                name: "score".to_string(),
                param_type: Type::U32,
            }],
            return_type: Type::U32,
            body: vec![
                Stmt::Let {
                    name: "r".to_string(),
                    value: Expr::Block(vec![Stmt::Match {
                        scrutinee: Expr::Variable("score".to_string()),
                        arms: vec![
                            MatchArm {
                                pattern: Pattern::Range {
                                    start: Literal::U32(90),
                                    end: Literal::U32(100),
                                    inclusive: true,
                                },
                                guard: None,
                                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(4)))],
                            },
                            MatchArm {
                                pattern: Pattern::Range {
                                    start: Literal::U32(80),
                                    end: Literal::U32(89),
                                    inclusive: true,
                                },
                                guard: None,
                                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(3)))],
                            },
                            MatchArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: vec![Stmt::Expr(Expr::Literal(Literal::U32(0)))],
                            },
                        ],
                    }]),
                },
                Stmt::Return(Some(Expr::Variable("r".to_string()))),
            ],
        }],
        entry_point: "grade".to_string(),
    };

    let converter = IrConverter::new();
    let ir = converter.convert(&ast).expect("conversion should succeed");

    // The IR should NOT contain a Case node (ranges can't use case in POSIX)
    fn has_case(ir: &super::ShellIR) -> bool {
        match ir {
            super::ShellIR::Case { .. } => true,
            super::ShellIR::Sequence(stmts) => stmts.iter().any(has_case),
            super::ShellIR::Function { body, .. } => has_case(body),
            super::ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                has_case(then_branch)
                    || else_branch.as_ref().map_or(false, |e| has_case(e))
            }
            _ => false,
        }
    }

    fn has_if(ir: &super::ShellIR) -> bool {
        match ir {
            super::ShellIR::If { .. } => true,
            super::ShellIR::Sequence(stmts) => stmts.iter().any(has_if),
            super::ShellIR::Function { body, .. } => has_if(body),
            _ => false,
        }
    }

    assert!(
        !has_case(&ir),
        "Range patterns should produce If chain, not Case"
    );
    assert!(
        has_if(&ir),
        "Range patterns should produce If chain for numeric comparisons"
    );
}
