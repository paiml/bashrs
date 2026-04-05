#![allow(clippy::expect_used)]
// Control flow IR generation and emission tests
// Testing string comparisons, logical operators, and NOT operator
// Following TDD: Write failing tests first, then fix implementation

use crate::ast::restricted::*;
use crate::emitter::PosixEmitter;
use crate::ir::from_ast;
use crate::models::Config;

#[test]
fn test_string_comparison_equality() {
    // Bug: String equality should generate = not -eq
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "env".to_string(),
                    value: Expr::Literal(Literal::Str("production".to_string())),
                    declaration: true,
                },
                Stmt::If {
                    condition: Expr::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(Expr::Variable("env".to_string())),
                        right: Box::new(Expr::Literal(Literal::Str("production".to_string()))),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str(
                            "Running in production".to_string(),
                        ))],
                    })],
                    else_block: None,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert to IR");

    // Emit the shell code
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // BUG: Currently emits [ "$env" -eq production ] which fails
    // SHOULD emit: [ "$env" = "production" ] for string comparison
    println!("Generated shell code:\n{}", shell_code);

    // This test will FAIL until we fix string comparison
    assert!(
        shell_code.contains("[ \"$env\" = ")
            || shell_code.contains("[ \"$env\" = \"production\" ]"),
        "String comparison should use = not -eq. Got:\n{}",
        shell_code
    );
}

#[test]
fn test_integer_comparison_equality() {
    // Integer equality should generate -eq
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::I32(10)),
                    declaration: true,
                },
                Stmt::If {
                    condition: Expr::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Literal(Literal::I32(10))),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Equal".to_string()))],
                    })],
                    else_block: None,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert to IR");

    // Emit and verify it uses -eq for integer comparison
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Integer comparison should use -eq
    assert!(
        shell_code.contains("-eq"),
        "Integer comparison should use -eq. Got:\n{}",
        shell_code
    );
}

#[test]
fn test_logical_and_operator() {
    // Bug: && operator causes "Comparison expression cannot be used in string concatenation"
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::I32(10)),
                    declaration: true,
                },
                Stmt::Let {
                    name: "y".to_string(),
                    value: Expr::Literal(Literal::I32(20)),
                    declaration: true,
                },
                Stmt::If {
                    condition: Expr::Binary {
                        op: BinaryOp::And,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Gt,
                            left: Box::new(Expr::Variable("x".to_string())),
                            right: Box::new(Expr::Literal(Literal::I32(5))),
                        }),
                        right: Box::new(Expr::Binary {
                            op: BinaryOp::Gt,
                            left: Box::new(Expr::Variable("y".to_string())),
                            right: Box::new(Expr::Literal(Literal::I32(15))),
                        }),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Both true".to_string()))],
                    })],
                    else_block: None,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert to IR");

    // Emit and verify it generates correct logical AND
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: [ "$x" -gt 5 ] && [ "$y" -gt 15 ]
    // Currently FAILS with "Comparison expression cannot be used in string concatenation"
    println!("AND test generated:\n{}", shell_code);
    assert!(
        shell_code.contains("&&") || shell_code.contains("-a"),
        "Logical AND should generate && or -a. Got:\n{}",
        shell_code
    );
}

#[test]
fn test_logical_or_operator() {
    // Bug: || operator causes "Comparison expression cannot be used in string concatenation"
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::I32(10)),
                    declaration: true,
                },
                Stmt::If {
                    condition: Expr::Binary {
                        op: BinaryOp::Or,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Lt,
                            left: Box::new(Expr::Variable("x".to_string())),
                            right: Box::new(Expr::Literal(Literal::I32(0))),
                        }),
                        right: Box::new(Expr::Binary {
                            op: BinaryOp::Gt,
                            left: Box::new(Expr::Variable("x".to_string())),
                            right: Box::new(Expr::Literal(Literal::I32(100))),
                        }),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Out of range".to_string()))],
                    })],
                    else_block: None,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert to IR");

    // Emit and verify it generates correct logical OR
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: [ "$x" -lt 0 ] || [ "$x" -gt 100 ]
    println!("OR test generated:\n{}", shell_code);
    assert!(
        shell_code.contains("||") || shell_code.contains("-o"),
        "Logical OR should generate || or -o. Got:\n{}",
        shell_code
    );
}

#[test]
fn test_not_operator() {
    // Bug: ! operator is not transpiled
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "enabled".to_string(),
                    value: Expr::Literal(Literal::Bool(false)),
                    declaration: true,
                },
                Stmt::If {
                    condition: Expr::Unary {
                        op: UnaryOp::Not,
                        operand: Box::new(Expr::Variable("enabled".to_string())),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str(
                            "Feature is disabled".to_string(),
                        ))],
                    })],
                    else_block: None,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert to IR");

    // Emit and verify it generates correct NOT
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: if ! "$enabled"; then or if ! $enabled; then or if ! false; then
    println!("NOT test generated:\n{}", shell_code);
    assert!(
        shell_code.contains("if ! \"$enabled\"")
            || shell_code.contains("if ! $enabled")
            || shell_code.contains("if ! false"),
        "NOT operator should be in if condition. Got:\n{}",
        shell_code
    );
}

#[test]
fn test_string_inequality() {
    // String inequality should use != not -ne
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Let {
                    name: "env".to_string(),
                    value: Expr::Literal(Literal::Str("development".to_string())),
                    declaration: true,
                },
                Stmt::If {
                    condition: Expr::Binary {
                        op: BinaryOp::Ne,
                        left: Box::new(Expr::Variable("env".to_string())),
                        right: Box::new(Expr::Literal(Literal::Str("production".to_string()))),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Not production".to_string()))],
                    })],
                    else_block: None,
                },
            ],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert to IR");

    // Emit and verify it uses != for string inequality
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: [ "$env" != "production" ]
    // Currently generates: [ "$env" -ne production ] (wrong)
    println!("String inequality test generated:\n{}", shell_code);
    assert!(
        shell_code.contains("!=") || shell_code.contains("[ \"$env\" != "),
        "String inequality should use != not -ne. Got:\n{}",
        shell_code
    );
}

// ============================================================================
// MUTATION KILLER TESTS
// Added to kill surviving mutants from mutation testing
// ============================================================================

/// MUTATION KILLER: Line 306 - delete match arm Expr::FunctionCall in convert_expr_to_value
/// Tests that function calls can be used as values in assignments
#[test]
fn test_function_call_as_value() {
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "get_value".to_string(),
                params: vec![],
                return_type: Type::Str,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Literal(Literal::Str("result".to_string()))],
                })],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::FunctionCall {
                        name: "get_value".to_string(),
                        args: vec![],
                    },
                    declaration: true,
                }],
            },
        ],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert AST to IR");

    // Verify the Let statement contains a CommandSubst value
    let ir_str = format!("{:?}", ir);
    assert!(
        ir_str.contains("CommandSubst"),
        "Function call should generate CommandSubst value. Got:\n{}",
        ir_str
    );
}

include!("control_flow_tests_tests_for_loop.rs");
