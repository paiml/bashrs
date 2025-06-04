use super::restricted::{MatchArm, Pattern};
use super::*;
use proptest::prelude::*;
use rstest::*;

#[test]
fn test_restricted_ast_validation() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(restricted::Literal::U32(42)),
            }],
        }],
        entry_point: "main".to_string(),
    };

    assert!(ast.validate().is_ok());
}

#[test]
fn test_missing_entry_point() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(restricted::Literal::U32(1)),
            }],
        }],
        entry_point: "main".to_string(),
    };

    assert!(ast.validate().is_err());
    assert!(ast
        .validate()
        .unwrap_err()
        .contains("Entry point function 'main' not found"));
}

#[test]
fn test_function_validation() {
    let func = Function {
        name: "test".to_string(),
        params: vec![],
        return_type: Type::Str,
        body: vec![],
    };

    // Empty function bodies are now allowed
    assert!(func.validate().is_ok());
}

#[test]
fn test_recursion_detection() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "recursive".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "recursive".to_string(),
                args: vec![],
            })],
        }],
        entry_point: "recursive".to_string(),
    };

    assert!(ast.validate().is_err());
    assert!(ast.validate().unwrap_err().contains("Recursion detected"));
}

#[test]
fn test_indirect_recursion_detection() {
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "a".to_string(),
                params: vec![],
                return_type: Type::Str,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "b".to_string(),
                    args: vec![],
                })],
            },
            Function {
                name: "b".to_string(),
                params: vec![],
                return_type: Type::Str,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "a".to_string(),
                    args: vec![],
                })],
            },
        ],
        entry_point: "a".to_string(),
    };

    assert!(ast.validate().is_err());
    assert!(ast.validate().unwrap_err().contains("Recursion detected"));
}

#[rstest]
#[case(Type::Bool)]
#[case(Type::U32)]
#[case(Type::Str)]
fn test_allowed_types(#[case] typ: Type) {
    assert!(typ.is_allowed());
}

#[test]
fn test_complex_types_allowed() {
    let result_type = Type::Result {
        ok_type: Box::new(Type::Str),
        err_type: Box::new(Type::Str),
    };
    assert!(result_type.is_allowed());

    let option_type = Type::Option {
        inner_type: Box::new(Type::U32),
    };
    assert!(option_type.is_allowed());
}

#[test]
fn test_expression_validation() {
    let valid_expr = Expr::Binary {
        op: restricted::BinaryOp::Add,
        left: Box::new(Expr::Literal(restricted::Literal::U32(1))),
        right: Box::new(Expr::Literal(restricted::Literal::U32(2))),
    };
    assert!(valid_expr.validate().is_ok());

    let function_call = Expr::FunctionCall {
        name: "test".to_string(),
        args: vec![
            Expr::Literal(restricted::Literal::Str("hello".to_string())),
            Expr::Variable("x".to_string()),
        ],
    };
    assert!(function_call.validate().is_ok());
}

#[test]
fn test_statement_validation() {
    let let_stmt = Stmt::Let {
        name: "x".to_string(),
        value: Expr::Literal(restricted::Literal::U32(42)),
    };
    assert!(let_stmt.validate().is_ok());

    let if_stmt = Stmt::If {
        condition: Expr::Literal(restricted::Literal::Bool(true)),
        then_block: vec![Stmt::Expr(Expr::Literal(restricted::Literal::Str(
            "then".to_string(),
        )))],
        else_block: Some(vec![Stmt::Expr(Expr::Literal(restricted::Literal::Str(
            "else".to_string(),
        )))]),
    };
    assert!(if_stmt.validate().is_ok());
}

#[test]
fn test_function_call_collection() {
    let func = Function {
        name: "main".to_string(),
        params: vec![],
        return_type: Type::Str,
        body: vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "helper1".to_string(),
                args: vec![],
            }),
            Stmt::Let {
                name: "x".to_string(),
                value: Expr::FunctionCall {
                    name: "helper2".to_string(),
                    args: vec![],
                },
            },
        ],
    };

    let mut calls = Vec::new();
    func.collect_function_calls(&mut calls);

    assert_eq!(calls.len(), 2);
    assert!(calls.contains(&"helper1".to_string()));
    assert!(calls.contains(&"helper2".to_string()));
}

// Property-based tests using proptest
proptest! {
    #[test]
    fn test_bool_literal_validation(value in prop::bool::ANY) {
        let expr = Expr::Literal(restricted::Literal::Bool(value));
        assert!(expr.validate().is_ok());
    }

    #[test]
    fn test_u32_literal_validation(value in 0u32..1000u32) {
        let expr = Expr::Literal(restricted::Literal::U32(value));
        assert!(expr.validate().is_ok());
    }

    #[test]
    fn test_string_literal_validation(value in "[^\0]*") {
        // Test with strings that don't contain null characters
        let expr = Expr::Literal(restricted::Literal::Str(value));
        assert!(expr.validate().is_ok());
    }

    #[test]
    fn test_variable_names_are_valid_identifiers(name in "[a-zA-Z_][a-zA-Z0-9_]*") {
        let expr = Expr::Variable(name);
        assert!(expr.validate().is_ok());
    }
}

#[test]
fn test_pattern_binds_variable() {
    // Test Pattern::Variable
    let pattern = Pattern::Variable("x".to_string());
    assert!(pattern.binds_variable("x"));
    assert!(!pattern.binds_variable("y"));

    // Test Pattern::Tuple
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Variable("a".to_string()),
        Pattern::Variable("b".to_string()),
    ]);
    assert!(tuple_pattern.binds_variable("a"));
    assert!(tuple_pattern.binds_variable("b"));
    assert!(!tuple_pattern.binds_variable("c"));

    // Test Pattern::Struct
    let struct_pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), Pattern::Variable("x_val".to_string())),
            ("y".to_string(), Pattern::Variable("y_val".to_string())),
        ],
    };
    assert!(struct_pattern.binds_variable("x_val"));
    assert!(struct_pattern.binds_variable("y_val"));
    assert!(!struct_pattern.binds_variable("z_val"));

    // Test Pattern::Literal and Pattern::Wildcard
    let literal_pattern = Pattern::Literal(restricted::Literal::U32(42));
    assert!(!literal_pattern.binds_variable("x"));

    let wildcard_pattern = Pattern::Wildcard;
    assert!(!wildcard_pattern.binds_variable("x"));
}

#[test]
fn test_pattern_validation() {
    // Test literal pattern
    let literal_pattern = Pattern::Literal(restricted::Literal::Bool(true));
    assert!(literal_pattern.validate().is_ok());

    // Test variable pattern
    let var_pattern = Pattern::Variable("x".to_string());
    assert!(var_pattern.validate().is_ok());

    // Test wildcard pattern
    let wildcard_pattern = Pattern::Wildcard;
    assert!(wildcard_pattern.validate().is_ok());

    // Test tuple pattern with nested patterns
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Variable("a".to_string()),
        Pattern::Literal(restricted::Literal::U32(42)),
        Pattern::Wildcard,
    ]);
    assert!(tuple_pattern.validate().is_ok());

    // Test struct pattern with nested patterns
    let struct_pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), Pattern::Variable("x_val".to_string())),
            ("y".to_string(), Pattern::Wildcard),
        ],
    };
    assert!(struct_pattern.validate().is_ok());
}

#[test]
fn test_type_is_allowed() {
    // Test basic allowed types
    assert!(Type::Void.is_allowed());
    assert!(Type::Bool.is_allowed());
    assert!(Type::U32.is_allowed());
    assert!(Type::Str.is_allowed());

    // Test Result type with allowed inner types
    let result_type = Type::Result {
        ok_type: Box::new(Type::U32),
        err_type: Box::new(Type::Str),
    };
    assert!(result_type.is_allowed());

    // Test Option type with allowed inner type
    let option_type = Type::Option {
        inner_type: Box::new(Type::Bool),
    };
    assert!(option_type.is_allowed());

    // Test nested allowed types
    let nested_result = Type::Result {
        ok_type: Box::new(Type::Option {
            inner_type: Box::new(Type::U32),
        }),
        err_type: Box::new(Type::Str),
    };
    assert!(nested_result.is_allowed());
}

#[test]
fn test_expr_array_try_block_handling() {
    // Test Array expression
    let array_expr = Expr::Array(vec![
        Expr::Literal(restricted::Literal::U32(1)),
        Expr::Literal(restricted::Literal::U32(2)),
    ]);
    assert!(array_expr.validate().is_ok());

    // Test Try expression
    let try_expr = Expr::Try {
        expr: Box::new(Expr::Literal(restricted::Literal::U32(42))),
    };
    assert!(try_expr.validate().is_ok());

    // Test Block expression
    let block_expr = Expr::Block(vec![Stmt::Let {
        name: "x".to_string(),
        value: Expr::Literal(restricted::Literal::U32(42)),
    }]);
    assert!(block_expr.validate().is_ok());
}

#[test]
fn test_expr_collect_function_calls_comprehensive() {
    let mut calls = Vec::new();

    // Test Array expression
    let array_expr = Expr::Array(vec![
        Expr::FunctionCall {
            name: "array_func".to_string(),
            args: vec![],
        },
        Expr::Literal(restricted::Literal::U32(1)),
    ]);
    array_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"array_func".to_string()));

    calls.clear();

    // Test Index expression
    let index_expr = Expr::Index {
        object: Box::new(Expr::FunctionCall {
            name: "get_array".to_string(),
            args: vec![],
        }),
        index: Box::new(Expr::FunctionCall {
            name: "get_index".to_string(),
            args: vec![],
        }),
    };
    index_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"get_array".to_string()));
    assert!(calls.contains(&"get_index".to_string()));

    calls.clear();

    // Test Try expression
    let try_expr = Expr::Try {
        expr: Box::new(Expr::FunctionCall {
            name: "fallible_func".to_string(),
            args: vec![],
        }),
    };
    try_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"fallible_func".to_string()));

    calls.clear();

    // Test Block expression
    let block_expr = Expr::Block(vec![Stmt::Expr(Expr::FunctionCall {
        name: "block_func".to_string(),
        args: vec![],
    })]);
    block_expr.collect_function_calls(&mut calls);
    assert!(calls.contains(&"block_func".to_string()));
}

#[test]
fn test_stmt_match_validation_and_collection() {
    let match_stmt = Stmt::Match {
        scrutinee: Expr::Variable("x".to_string()),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(restricted::Literal::U32(1)),
                guard: Some(Expr::FunctionCall {
                    name: "guard_func".to_string(),
                    args: vec![],
                }),
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "arm_func".to_string(),
                    args: vec![],
                })],
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: vec![Stmt::Return(None)],
            },
        ],
    };

    // Test validation
    assert!(match_stmt.validate().is_ok());

    // Test function call collection
    let mut calls = Vec::new();
    match_stmt.collect_function_calls(&mut calls);
    assert!(calls.contains(&"guard_func".to_string()));
    assert!(calls.contains(&"arm_func".to_string()));
}

#[test]
fn test_stmt_for_while_bounded_validation() {
    // Test For loop with bounded iterations (should pass)
    let for_stmt = Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Variable("collection".to_string()),
        body: vec![Stmt::Return(None)],
        max_iterations: Some(100),
    };
    assert!(for_stmt.validate().is_ok());

    // Test For loop without bounded iterations (should fail)
    let unbounded_for = Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::Variable("collection".to_string()),
        body: vec![],
        max_iterations: None,
    };
    assert!(unbounded_for.validate().is_err());

    // Test While loop with bounded iterations (should pass)
    let while_stmt = Stmt::While {
        condition: Expr::Literal(restricted::Literal::Bool(true)),
        body: vec![Stmt::Break],
        max_iterations: Some(50),
    };
    assert!(while_stmt.validate().is_ok());

    // Test While loop without bounded iterations (should fail)
    let unbounded_while = Stmt::While {
        condition: Expr::Literal(restricted::Literal::Bool(true)),
        body: vec![],
        max_iterations: None,
    };
    assert!(unbounded_while.validate().is_err());
}

#[test]
fn test_stmt_for_while_function_call_collection() {
    let mut calls = Vec::new();

    // Test For loop
    let for_stmt = Stmt::For {
        pattern: Pattern::Variable("i".to_string()),
        iter: Expr::FunctionCall {
            name: "get_iter".to_string(),
            args: vec![],
        },
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "process".to_string(),
            args: vec![],
        })],
        max_iterations: Some(100),
    };
    for_stmt.collect_function_calls(&mut calls);
    assert!(calls.contains(&"get_iter".to_string()));
    assert!(calls.contains(&"process".to_string()));

    calls.clear();

    // Test While loop
    let while_stmt = Stmt::While {
        condition: Expr::FunctionCall {
            name: "check_condition".to_string(),
            args: vec![],
        },
        body: vec![Stmt::Expr(Expr::FunctionCall {
            name: "loop_body".to_string(),
            args: vec![],
        })],
        max_iterations: Some(50),
    };
    while_stmt.collect_function_calls(&mut calls);
    assert!(calls.contains(&"check_condition".to_string()));
    assert!(calls.contains(&"loop_body".to_string()));
}

#[test]
fn test_stmt_break_continue() {
    // Test Break and Continue validation
    assert!(Stmt::Break.validate().is_ok());
    assert!(Stmt::Continue.validate().is_ok());

    // Test Break and Continue function call collection (should be empty)
    let mut calls = Vec::new();
    Stmt::Break.collect_function_calls(&mut calls);
    assert!(calls.is_empty());

    Stmt::Continue.collect_function_calls(&mut calls);
    assert!(calls.is_empty());
}

#[test]
fn test_validate_public_api() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Literal(restricted::Literal::U32(42)),
            }],
        }],
        entry_point: "main".to_string(),
    };

    // Test the public validate function
    assert!(validate(&ast).is_ok());
}

#[test]
fn test_invalid_ast_returns_validation_error() {
    let ast = RestrictedAst {
        functions: vec![],
        entry_point: "main".to_string(),
    };

    match validate(&ast) {
        Err(crate::models::Error::Validation(_)) => (), // Expected
        _ => panic!("Expected validation error"),
    }
}
