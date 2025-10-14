#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restricted_ast_validation() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Let {
                            name: "x".to_string(),
                            value: Expr::Literal(Literal::U32(42)),
                        }
                    ],
                }
            ],
            entry_point: "main".to_string(),
        };

        assert!(ast.validate().is_ok());
    }

    #[test]
    fn test_missing_entry_point() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "helper".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![],
                }
            ],
            entry_point: "main".to_string(),
        };

        assert!(ast.validate().is_err());
        assert!(ast.validate().unwrap_err().contains("Entry point function 'main' not found"));
    }

    #[test]
    fn test_function_validation() {
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![],
        };

        assert!(func.validate().is_err());
        assert!(func.validate().unwrap_err().contains("empty body"));
    }

    #[test]
    fn test_recursion_detection() {
        let ast = RestrictedAst {
            functions: vec![
                Function {
                    name: "recursive".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Expr(Expr::FunctionCall {
                            name: "recursive".to_string(),
                            args: vec![],
                        })
                    ],
                },
            ],
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
                    body: vec![
                        Stmt::Expr(Expr::FunctionCall {
                            name: "b".to_string(),
                            args: vec![],
                        })
                    ],
                },
                Function {
                    name: "b".to_string(),
                    params: vec![],
                    return_type: Type::Str,
                    body: vec![
                        Stmt::Expr(Expr::FunctionCall {
                            name: "a".to_string(),
                            args: vec![],
                        })
                    ],
                },
            ],
            entry_point: "a".to_string(),
        };

        assert!(ast.validate().is_err());
        assert!(ast.validate().unwrap_err().contains("Recursion detected"));
    }

    #[test]
    fn test_type_validation() {
        assert!(Type::Bool.is_allowed());
        assert!(Type::U32.is_allowed());
        assert!(Type::Str.is_allowed());
        
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
            op: BinaryOp::Add,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        };
        assert!(valid_expr.validate().is_ok());

        let function_call = Expr::FunctionCall {
            name: "test".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("hello".to_string())),
                Expr::Variable("x".to_string()),
            ],
        };
        assert!(function_call.validate().is_ok());
    }

    #[test]
    fn test_statement_validation() {
        let let_stmt = Stmt::Let {
            name: "x".to_string(),
            value: Expr::Literal(Literal::U32(42)),
        };
        assert!(let_stmt.validate().is_ok());

        let if_stmt = Stmt::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_block: vec![
                Stmt::Expr(Expr::Literal(Literal::Str("then".to_string()))),
            ],
            else_block: Some(vec![
                Stmt::Expr(Expr::Literal(Literal::Str("else".to_string()))),
            ]),
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

    // Sprint 29 Mutation Testing - Nesting Depth Validation
    // These tests kill mutants #6, #7, #8: boundary condition tests for depth > 30

    #[test]
    fn test_expr_nesting_depth_at_limit() {
        // Create expression with EXACTLY depth=30 (at the limit)
        let mut expr = Expr::Literal(Literal::U32(1));
        for _ in 0..30 {
            expr = Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(expr),
            };
        }

        let result = expr.validate();
        assert!(result.is_ok(), "Depth=30 should be allowed");
    }

    #[test]
    fn test_expr_nesting_depth_exceeds_limit() {
        // Create expression with depth=31 (exceeds limit by 1)
        let mut expr = Expr::Literal(Literal::U32(1));
        for _ in 0..31 {
            expr = Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(expr),
            };
        }

        let result = expr.validate();
        assert!(result.is_err(), "Depth=31 should be rejected");
        assert!(
            result.unwrap_err().contains("nesting too deep"),
            "Error message should mention nesting"
        );
        // Verify it reports depth=31 specifically
        let mut expr2 = Expr::Literal(Literal::U32(1));
        for _ in 0..31 {
            expr2 = Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(expr2),
            };
        }
        let err = expr2.validate().unwrap_err();
        assert!(err.contains("31"), "Error should report depth=31");
    }

    #[test]
    fn test_expr_nesting_depth_way_exceeds_limit() {
        // Create expression with depth=50 (way over limit)
        let mut expr = Expr::Literal(Literal::U32(1));
        for _ in 0..50 {
            expr = Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(expr),
                right: Box::new(Expr::Literal(Literal::U32(1))),
            };
        }

        let result = expr.validate();
        assert!(result.is_err(), "Depth=50 should be rejected");
        let err = result.unwrap_err();
        assert!(err.contains("nesting too deep"));
        assert!(err.contains("50"));
    }

    // Sprint 29 Mutation Testing - String Literal Validation
    // This test kills potential mutants related to null character checking

    #[test]
    fn test_string_literal_rejects_null_characters() {
        let expr = Expr::Literal(Literal::Str("hello\0world".to_string()));
        let result = expr.validate();
        assert!(result.is_err(), "Strings with null characters should be rejected");
        assert!(
            result.unwrap_err().contains("Null characters not allowed"),
            "Error should mention null characters"
        );
    }

    #[test]
    fn test_string_literal_allows_valid_strings() {
        let expr = Expr::Literal(Literal::Str("hello world".to_string()));
        assert!(expr.validate().is_ok(), "Valid strings should pass");
    }

    // Sprint 29 Mutation Testing - Priority 1: Validation Function Tests
    // These tests kill validation bypass mutants and verify negative test cases

    #[test]
    fn test_type_is_allowed_nested_result_both_sides_required() {
        // Verify && logic (not ||) in Result type validation
        // Kills mutant: replace && with || in Type::is_allowed
        let valid = Type::Result {
            ok_type: Box::new(Type::U32),
            err_type: Box::new(Type::Str),
        };
        assert!(valid.is_allowed(), "Result with both valid types should be allowed");

        // Test deep nesting - all nested types must be allowed
        let nested = Type::Option {
            inner_type: Box::new(Type::Result {
                ok_type: Box::new(Type::U32),
                err_type: Box::new(Type::Str),
            }),
        };
        assert!(nested.is_allowed(), "Deeply nested valid types should be allowed");

        // Test multiple levels of nesting
        let deeply_nested = Type::Result {
            ok_type: Box::new(Type::Option {
                inner_type: Box::new(Type::U32),
            }),
            err_type: Box::new(Type::Option {
                inner_type: Box::new(Type::Str),
            }),
        };
        assert!(deeply_nested.is_allowed(), "Complex nested types should be allowed");
    }

    #[test]
    fn test_validate_if_stmt_rejects_invalid_condition() {
        // Verify validate_if_stmt actually validates condition
        // Kills mutant: replace validate_if_stmt -> Ok(())
        let invalid_if = Stmt::If {
            condition: Expr::Literal(Literal::Str("hello\0world".to_string())),
            then_block: vec![],
            else_block: None,
        };

        let result = invalid_if.validate();
        assert!(result.is_err(), "If statement with null char in condition should be rejected");
        assert!(
            result.unwrap_err().contains("Null characters not allowed"),
            "Error should mention null characters"
        );
    }

    #[test]
    fn test_validate_if_stmt_rejects_deeply_nested_condition() {
        // Verify validate_if_stmt checks nesting depth in condition
        // Kills mutant: replace validate_if_stmt -> Ok(())
        let mut deep_expr = Expr::Literal(Literal::U32(1));
        for _ in 0..35 {
            deep_expr = Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(deep_expr),
            };
        }

        let invalid_if = Stmt::If {
            condition: deep_expr,
            then_block: vec![],
            else_block: None,
        };

        let result = invalid_if.validate();
        assert!(result.is_err(), "If statement with deeply nested condition should be rejected");
        assert!(
            result.unwrap_err().contains("nesting too deep"),
            "Error should mention nesting depth"
        );
    }

    #[test]
    fn test_validate_if_stmt_rejects_invalid_then_block() {
        // Verify validate_if_stmt validates then_block statements
        let invalid_if = Stmt::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_block: vec![
                Stmt::Expr(Expr::Literal(Literal::Str("invalid\0string".to_string())))
            ],
            else_block: None,
        };

        let result = invalid_if.validate();
        assert!(result.is_err(), "If statement with invalid then_block should be rejected");
    }

    #[test]
    fn test_validate_if_stmt_rejects_invalid_else_block() {
        // Verify validate_if_stmt validates else_block statements
        let invalid_if = Stmt::If {
            condition: Expr::Literal(Literal::Bool(true)),
            then_block: vec![
                Stmt::Expr(Expr::Literal(Literal::Str("valid".to_string())))
            ],
            else_block: Some(vec![
                Stmt::Expr(Expr::Literal(Literal::Str("\0invalid".to_string())))
            ]),
        };

        let result = invalid_if.validate();
        assert!(result.is_err(), "If statement with invalid else_block should be rejected");
    }

    #[test]
    fn test_validate_match_stmt_rejects_invalid_arm_body() {
        // Verify validate_match_stmt validates match arm bodies
        // Kills mutant: replace validate_match_stmt -> Ok(())
        let invalid_match = Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::U32(1)),
                    body: vec![
                        Stmt::Expr(Expr::Literal(Literal::Str("\0invalid".to_string())))
                    ],
                }
            ],
        };

        let result = invalid_match.validate();
        assert!(result.is_err(), "Match statement with invalid arm body should be rejected");
    }

    #[test]
    fn test_validate_match_stmt_rejects_deeply_nested_scrutinee() {
        // Verify validate_match_stmt validates scrutinee expression
        let mut deep_expr = Expr::Literal(Literal::U32(1));
        for _ in 0..40 {
            deep_expr = Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(deep_expr),
            };
        }

        let invalid_match = Stmt::Match {
            scrutinee: deep_expr,
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::U32(1)),
                    body: vec![
                        Stmt::Expr(Expr::Literal(Literal::Str("body".to_string())))
                    ],
                }
            ],
        };

        let result = invalid_match.validate();
        assert!(result.is_err(), "Match with deeply nested scrutinee should be rejected");
    }

    #[test]
    fn test_validate_stmt_block_rejects_invalid_nested_stmt() {
        // Verify validate_stmt_block checks all statements in block
        // Kills mutant: replace validate_stmt_block -> Ok(())
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![
                Stmt::Let {
                    name: "x".to_string(),
                    value: Expr::Literal(Literal::Str("valid".to_string())),
                },
                Stmt::Let {
                    name: "y".to_string(),
                    value: Expr::Literal(Literal::Str("in\0valid".to_string())),
                },
            ],
        };

        let result = func.validate();
        assert!(result.is_err(), "Function with invalid statement should be rejected");
    }

    #[test]
    fn test_expr_nesting_depth_calculation_accuracy() {
        // Verify nesting_depth returns correct values (not always 0 or 1)
        // Kills mutants: replace nesting_depth -> 0, replace -> 1, arithmetic mutations

        // Depth = 0 (literal)
        let lit = Expr::Literal(Literal::U32(1));
        assert_eq!(lit.nesting_depth(), 0, "Literal should have depth 0");

        // Depth = 1 (unary)
        let unary = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Literal(Literal::Bool(true))),
        };
        assert_eq!(unary.nesting_depth(), 1, "Unary should have depth 1");

        // Depth = 2 (binary with unary left)
        let binary = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(unary.clone()),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        };
        assert_eq!(binary.nesting_depth(), 2, "Binary with nested unary should have depth 2");

        // Depth = 3 (nested binary)
        let nested = Expr::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(binary),
            right: Box::new(Expr::Literal(Literal::U32(3))),
        };
        assert_eq!(nested.nesting_depth(), 3, "Nested binary should have depth 3");

        // Depth with function call
        let func_call = Expr::FunctionCall {
            name: "test".to_string(),
            args: vec![
                Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Literal::U32(1))),
                    right: Box::new(Expr::Literal(Literal::U32(2))),
                }
            ],
        };
        assert_eq!(func_call.nesting_depth(), 2, "FunctionCall with nested binary should have depth 2");
    }

    #[test]
    fn test_pattern_validate_rejects_invalid_literal() {
        // Verify Pattern::validate checks string literals for null characters
        // Kills mutant: replace Pattern::validate -> Ok(())
        let invalid_pattern = Pattern::Literal(Literal::Str("\0invalid".to_string()));
        let result = invalid_pattern.validate();
        assert!(result.is_err(), "Pattern with null character should be rejected");
    }

    #[test]
    fn test_pattern_validate_accepts_valid_patterns() {
        // Verify Pattern::validate accepts valid patterns (positive control)
        let valid_patterns = vec![
            Pattern::Literal(Literal::U32(42)),
            Pattern::Literal(Literal::Bool(true)),
            Pattern::Literal(Literal::Str("valid".to_string())),
            Pattern::Variable("x".to_string()),
        ];

        for pattern in valid_patterns {
            assert!(pattern.validate().is_ok(), "Valid patterns should pass validation");
        }
    }

    // Sprint 29 Mutation Testing - Priority 2: Match Arm Coverage Tests
    // These tests kill match arm deletion mutants by exercising all variants

    #[test]
    fn test_expr_validate_all_variants_comprehensive() {
        // Comprehensive test covering all Expr variants in validate()
        // Kills: 6 match arm deletion mutants in Expr::validate (lines 383-403)

        // Test all expression types validate successfully
        let expressions = vec![
            // Literal variants
            Expr::Literal(Literal::U32(42)),
            Expr::Literal(Literal::Bool(true)),
            Expr::Literal(Literal::Str("valid string".to_string())),

            // Variable
            Expr::Variable("x".to_string()),

            // FunctionCall with nested args
            Expr::FunctionCall {
                name: "test_func".to_string(),
                args: vec![
                    Expr::Literal(Literal::U32(1)),
                    Expr::Variable("y".to_string()),
                ],
            },

            // Binary with nested expressions
            Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::U32(1))),
                right: Box::new(Expr::Literal(Literal::U32(2))),
            },

            // Unary
            Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Literal(Literal::Bool(true))),
            },

            // MethodCall with receiver and args
            Expr::MethodCall {
                receiver: Box::new(Expr::Variable("obj".to_string())),
                method: "process".to_string(),
                args: vec![Expr::Literal(Literal::U32(10))],
            },

            // Range (inclusive and exclusive)
            Expr::Range {
                start: Box::new(Expr::Literal(Literal::U32(1))),
                end: Box::new(Expr::Literal(Literal::U32(10))),
                inclusive: false,
            },
            Expr::Range {
                start: Box::new(Expr::Literal(Literal::U32(1))),
                end: Box::new(Expr::Literal(Literal::U32(10))),
                inclusive: true,
            },
        ];

        // All expressions should validate successfully
        for (i, expr) in expressions.iter().enumerate() {
            assert!(
                expr.validate().is_ok(),
                "Expression variant {} should validate successfully",
                i
            );
        }

        // Test nested combinations to ensure recursive validation
        let complex = Expr::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(Expr::FunctionCall {
                name: "compute".to_string(),
                args: vec![
                    Expr::Unary {
                        op: UnaryOp::Negate,
                        operand: Box::new(Expr::Literal(Literal::U32(5))),
                    }
                ],
            }),
            right: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("data".to_string())),
                method: "len".to_string(),
                args: vec![],
            }),
        };
        assert!(complex.validate().is_ok(), "Complex nested expression should validate");
    }

    #[test]
    fn test_expr_nesting_depth_all_variants_accurate() {
        // Test that nesting_depth correctly calculates depth for all Expr variants
        // Kills: 5 match arm deletion mutants in Expr::nesting_depth (lines 414-424)
        // Kills: 9 arithmetic operator mutations (+ to - or *)

        // Test Binary expression depth calculation
        let binary_depth_2 = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Literal(Literal::Bool(true))),
            }),
            right: Box::new(Expr::Literal(Literal::U32(2))),
        };
        assert_eq!(
            binary_depth_2.nesting_depth(),
            2,
            "Binary with unary left should have depth 2"
        );

        // Test Unary expression depth
        let unary_depth_2 = Expr::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Literal(Literal::Bool(true))),
            }),
        };
        assert_eq!(
            unary_depth_2.nesting_depth(),
            2,
            "Nested unary should have depth 2"
        );

        // Test FunctionCall with nested args
        let func_call_depth_2 = Expr::FunctionCall {
            name: "test".to_string(),
            args: vec![
                Expr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(Expr::Literal(Literal::U32(3))),
                    right: Box::new(Expr::Literal(Literal::U32(4))),
                }
            ],
        };
        assert_eq!(
            func_call_depth_2.nesting_depth(),
            2,
            "FunctionCall with binary arg should have depth 2"
        );

        // Test MethodCall with nested receiver
        let method_depth_3 = Expr::MethodCall {
            receiver: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(Expr::Literal(Literal::Bool(false))),
                }),
                right: Box::new(Expr::Literal(Literal::U32(1))),
            }),
            method: "to_string".to_string(),
            args: vec![],
        };
        assert!(
            method_depth_3.nesting_depth() >= 2,
            "MethodCall with nested receiver should have depth >= 2"
        );

        // Test MethodCall with nested args
        let method_with_args = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("x".to_string())),
            method: "process".to_string(),
            args: vec![
                Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Literal(Literal::U32(1))),
                    right: Box::new(Expr::Literal(Literal::U32(2))),
                }
            ],
        };
        assert!(
            method_with_args.nesting_depth() >= 2,
            "MethodCall with nested args should have depth >= 2"
        );

        // Test Range with nested start/end
        let range_depth_2 = Expr::Range {
            start: Box::new(Expr::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::Literal(Literal::U32(1))),
            }),
            end: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Literal(Literal::Bool(true))),
            }),
            inclusive: true,
        };
        assert_eq!(
            range_depth_2.nesting_depth(),
            2,
            "Range with unary start/end should have depth 2"
        );

        // Test that depth increases correctly with nesting
        let deeply_nested = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Subtract,
                    left: Box::new(Expr::Literal(Literal::U32(1))),
                    right: Box::new(Expr::Literal(Literal::U32(2))),
                }),
                right: Box::new(Expr::Literal(Literal::U32(3))),
            }),
            right: Box::new(Expr::Literal(Literal::U32(4))),
        };
        assert_eq!(
            deeply_nested.nesting_depth(),
            3,
            "Triple-nested binary should have depth 3"
        );
    }

    #[test]
    fn test_collect_function_calls_all_expr_types() {
        // Test collect_function_calls covers all expression types
        // Kills: 4 match arm deletion mutants in collect_function_calls (lines 437-467)

        // Test Binary expression with function calls in both branches
        let binary_with_calls = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::FunctionCall {
                name: "left_func".to_string(),
                args: vec![],
            }),
            right: Box::new(Expr::FunctionCall {
                name: "right_func".to_string(),
                args: vec![],
            }),
        };
        let mut calls = Vec::new();
        binary_with_calls.collect_function_calls(&mut calls);
        assert_eq!(calls.len(), 2, "Should find 2 function calls in binary");
        assert!(calls.contains(&"left_func".to_string()), "Should find left_func");
        assert!(calls.contains(&"right_func".to_string()), "Should find right_func");

        // Test Unary expression with nested function call
        let unary_with_call = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::FunctionCall {
                name: "inner_func".to_string(),
                args: vec![],
            }),
        };
        let mut calls2 = Vec::new();
        unary_with_call.collect_function_calls(&mut calls2);
        assert_eq!(calls2.len(), 1, "Should find 1 function call in unary");
        assert!(calls2.contains(&"inner_func".to_string()), "Should find inner_func");

        // Test MethodCall with function calls in receiver and args
        let method_with_calls = Expr::MethodCall {
            receiver: Box::new(Expr::FunctionCall {
                name: "receiver_func".to_string(),
                args: vec![],
            }),
            method: "process".to_string(),
            args: vec![
                Expr::FunctionCall {
                    name: "arg_func".to_string(),
                    args: vec![],
                }
            ],
        };
        let mut calls3 = Vec::new();
        method_with_calls.collect_function_calls(&mut calls3);
        assert_eq!(calls3.len(), 2, "Should find 2 function calls in method call");
        assert!(calls3.contains(&"receiver_func".to_string()), "Should find receiver_func");
        assert!(calls3.contains(&"arg_func".to_string()), "Should find arg_func");

        // Test Range with function calls in start and end
        let range_with_calls = Expr::Range {
            start: Box::new(Expr::FunctionCall {
                name: "start_func".to_string(),
                args: vec![],
            }),
            end: Box::new(Expr::FunctionCall {
                name: "end_func".to_string(),
                args: vec![],
            }),
            inclusive: false,
        };
        let mut calls4 = Vec::new();
        range_with_calls.collect_function_calls(&mut calls4);
        assert_eq!(calls4.len(), 2, "Should find 2 function calls in range");
        assert!(calls4.contains(&"start_func".to_string()), "Should find start_func");
        assert!(calls4.contains(&"end_func".to_string()), "Should find end_func");

        // Test complex nested expression with multiple calls
        let complex = Expr::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "outer".to_string(),
                    args: vec![],
                }),
                method: "map".to_string(),
                args: vec![
                    Expr::FunctionCall {
                        name: "mapper".to_string(),
                        args: vec![],
                    }
                ],
            }),
            right: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::FunctionCall {
                    name: "checker".to_string(),
                    args: vec![],
                }),
            }),
        };
        let mut calls5 = Vec::new();
        complex.collect_function_calls(&mut calls5);
        assert_eq!(calls5.len(), 3, "Should find 3 function calls in complex expression");
        assert!(calls5.contains(&"outer".to_string()), "Should find outer");
        assert!(calls5.contains(&"mapper".to_string()), "Should find mapper");
        assert!(calls5.contains(&"checker".to_string()), "Should find checker");
    }
}