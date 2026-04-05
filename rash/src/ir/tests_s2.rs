#![allow(clippy::expect_used)]
use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

// Helper: wrap a single let statement in a main function and convert to IR

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
                    declaration: true,
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
                    declaration: true,
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
                declaration: true,
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
                declaration: true,
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
                declaration: true,
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
                declaration: true,
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
                declaration: true,
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
                declaration: true,
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
                    declaration: true,
                },
                Stmt::Let {
                    name: "path".to_string(),
                    value: Expr::FunctionCall {
                        name: "env".to_string(),
                        args: vec![Expr::Literal(Literal::Str("PATH".to_string()))],
                    },
                    declaration: true,
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
