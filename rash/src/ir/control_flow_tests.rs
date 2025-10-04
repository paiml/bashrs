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
                },
                Stmt::If {
                    condition: Expr::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(Expr::Variable("env".to_string())),
                        right: Box::new(Expr::Literal(Literal::Str("production".to_string()))),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Running in production".to_string()))],
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
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // BUG: Currently emits [ "$env" -eq production ] which fails
    // SHOULD emit: [ "$env" = "production" ] for string comparison
    println!("Generated shell code:\n{}", shell_code);

    // This test will FAIL until we fix string comparison
    assert!(shell_code.contains("[ \"$env\" = ") || shell_code.contains("[ \"$env\" = \"production\" ]"),
        "String comparison should use = not -eq. Got:\n{}", shell_code);
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
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Integer comparison should use -eq
    assert!(shell_code.contains("-eq"),
        "Integer comparison should use -eq. Got:\n{}", shell_code);
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
                },
                Stmt::Let {
                    name: "y".to_string(),
                    value: Expr::Literal(Literal::I32(20)),
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
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: [ "$x" -gt 5 ] && [ "$y" -gt 15 ]
    // Currently FAILS with "Comparison expression cannot be used in string concatenation"
    println!("AND test generated:\n{}", shell_code);
    assert!(shell_code.contains("&&") || shell_code.contains("-a"),
        "Logical AND should generate && or -a. Got:\n{}", shell_code);
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
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: [ "$x" -lt 0 ] || [ "$x" -gt 100 ]
    println!("OR test generated:\n{}", shell_code);
    assert!(shell_code.contains("||") || shell_code.contains("-o"),
        "Logical OR should generate || or -o. Got:\n{}", shell_code);
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
                },
                Stmt::If {
                    condition: Expr::Unary {
                        op: UnaryOp::Not,
                        operand: Box::new(Expr::Variable("enabled".to_string())),
                    },
                    then_block: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("Feature is disabled".to_string()))],
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
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: if ! "$enabled"; then or if ! $enabled; then or if ! false; then
    println!("NOT test generated:\n{}", shell_code);
    assert!(shell_code.contains("if ! \"$enabled\"") || shell_code.contains("if ! $enabled") || shell_code.contains("if ! false"),
        "NOT operator should be in if condition. Got:\n{}", shell_code);
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
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Should generate: [ "$env" != "production" ]
    // Currently generates: [ "$env" -ne production ] (wrong)
    println!("String inequality test generated:\n{}", shell_code);
    assert!(shell_code.contains("!=") || shell_code.contains("[ \"$env\" != "),
        "String inequality should use != not -ne. Got:\n{}", shell_code);
}
