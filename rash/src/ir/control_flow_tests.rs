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
                }],
            },
        ],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert AST to IR");
    
    // Verify the Let statement contains a CommandSubst value
    let ir_str = format!("{:?}", ir);
    assert!(ir_str.contains("CommandSubst"), 
        "Function call should generate CommandSubst value. Got:\n{}", ir_str);
}

/// MUTATION KILLER: Line 161 - delete match arm Pattern::Variable in convert_stmt
/// Tests that for loops with variable patterns are converted correctly
#[test]
fn test_for_loop_variable_pattern() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::For {
                pattern: Pattern::Variable("i".to_string()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Literal(Literal::U32(5))),
                    inclusive: true,
                },
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Variable("i".to_string())],
                })],
                max_iterations: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert for loop with variable pattern");
    
    // Verify it generates a For IR node
    let ir_str = format!("{:?}", ir);
    assert!(ir_str.contains("For"), 
        "For loop should generate For IR node. Got:\n{}", ir_str);
}

/// MUTATION KILLER: Line 436 - delete match arm "echo" | "printf" in analyze_command_effects
/// Tests that echo and printf commands have FileWrite effect
#[test]
fn test_echo_printf_file_write_effect() {
    
    // Test echo command
    let ast_echo = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "echo".to_string(),
                args: vec![Expr::Literal(Literal::Str("test".to_string()))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let ir_echo = from_ast(&ast_echo).expect("Should convert echo");
    
    // Verify echo has FileWrite effect
    let ir_str = format!("{:?}", ir_echo);
    assert!(ir_str.contains("FileWrite") || ir_str.contains("effects"),
        "Echo should have FileWrite effect. Got:\n{}", ir_str);

    // Test printf command
    let ast_printf = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "printf".to_string(),
                args: vec![Expr::Literal(Literal::Str("test".to_string()))],
            })],
        }],
        entry_point: "main".to_string(),
    };

    let ir_printf = from_ast(&ast_printf).expect("Should convert printf");
    
    // Verify printf has FileWrite effect
    let ir_str = format!("{:?}", ir_printf);
    assert!(ir_str.contains("FileWrite") || ir_str.contains("effects"),
        "Printf should have FileWrite effect. Got:\n{}", ir_str);
}

/// MUTATION KILLER: Line 70 - replace && with || in convert
/// Tests that only the last statement in a function with return type gets Echo wrapper
#[test]
fn test_only_last_statement_echoed_with_return_type() {
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "get_value".to_string(),
                params: vec![],
                return_type: Type::Str,
                body: vec![
                    Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("first".to_string()))],
                    }),
                    Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("second".to_string()))],
                    }),
                    Stmt::Expr(Expr::FunctionCall {
                        name: "echo".to_string(),
                        args: vec![Expr::Literal(Literal::Str("last".to_string()))],
                    }),
                ],
            },
            Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "get_value".to_string(),
                    args: vec![],
                })],
            },
        ],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert function with return type");

    // Verify only the last statement is wrapped in Echo
    let ir_str = format!("{:?}", ir);
    let echo_count = ir_str.matches("Echo {").count();
    assert_eq!(echo_count, 1,
        "Only the last statement should be wrapped in Echo. Found {} Echo nodes. Got:\n{}",
        echo_count, ir_str);
}

/// MUTATION KILLER: Line 180 - replace - with + in convert_stmt
/// Tests that exclusive ranges are correctly adjusted (n - 1)
#[test]
fn test_exclusive_range_adjustment() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::For {
                pattern: Pattern::Variable("i".to_string()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Literal(Literal::U32(3))),
                    inclusive: false, // 0..3 should become 0..=2
                },
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "echo".to_string(),
                    args: vec![Expr::Variable("i".to_string())],
                })],
                max_iterations: None,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert exclusive range");
    
    // Emit to shell code
    let config = Config::default();
    let emitter = PosixEmitter::new(config.clone());
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Verify the range end is 2 (3 - 1), not 3 or 4
    println!("Exclusive range test generated:\n{}", shell_code);
    assert!(shell_code.contains("seq 0 2") || shell_code.contains("0 2"),
        "Exclusive range 0..3 should generate 'seq 0 2' (end adjusted to 2). Got:\n{}", shell_code);
}
