#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::bash_parser::ast::*;
    use proptest::prelude::*;

    // Property: All Until loops must be transformed to While loops
    // This verifies the core transformation rule
    proptest! {
        #[test]
        fn prop_until_always_becomes_while(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..100i64
        ) {
            // Create an until loop: until [ $var -gt threshold ]; do ...; done
            let ast = BashAst {
                statements: vec![BashStmt::Until {
                    condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                        BashExpr::Variable(var_name.clone()),
                        BashExpr::Literal(threshold.to_string()),
                    ))),
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Variable(var_name)],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain "while"
            prop_assert!(
                purified.contains("while"),
                "Until loop must be transformed to while, got: {}",
                purified
            );

            // PROPERTY: Must NOT contain "until"
            prop_assert!(
                !purified.contains("until"),
                "Purified output must not contain 'until', got: {}",
                purified
            );

            // PROPERTY: Must contain negation "!"
            prop_assert!(
                purified.contains("!"),
                "Until condition must be negated in while loop, got: {}",
                purified
            );
        }
    }

    // Property: Until transformation must be deterministic
    // Same input must always produce same output
    proptest! {
        #[test]
        fn prop_until_transformation_is_deterministic(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..100i64
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Until {
                    condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                        BashExpr::Variable(var_name.clone()),
                        BashExpr::Literal(threshold.to_string()),
                    ))),
                    body: vec![BashStmt::Assignment {
                        name: var_name.clone(),
            index: None,
                        value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                            Box::new(ArithExpr::Variable(var_name)),
                            Box::new(ArithExpr::Number(1)),
                        ))),
                        exported: false,
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Until transformation must be deterministic"
            );
        }
    }

    // Property: Until loops with different test expressions all transform correctly
    proptest! {
        #[test]
        fn prop_until_handles_all_test_types(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..10i64
        ) {
            // Test with different comparison operators
            for test_expr in [
                TestExpr::IntEq(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntNe(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntLt(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntGt(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
            ] {
                let ast = BashAst {
                    statements: vec![BashStmt::Until {
                        condition: BashExpr::Test(Box::new(test_expr)),
                        body: vec![BashStmt::Comment {
                            text: "loop body".to_string(),
                            span: Span::dummy(),
                        }],
                        span: Span::dummy(),
                    }],
                    metadata: AstMetadata {
                        source_file: None,
                        line_count: 1,
                        parse_time_ms: 0,
                    },
                };

                let purified = generators::generate_purified_bash(&ast);

                // PROPERTY: All test types must be transformed
                prop_assert!(
                    purified.contains("while") && !purified.contains("until"),
                    "All until test types must transform to while, got: {}",
                    purified
                );
            }
        }
    }

    // Property: Default value expansion preserves variable name
    proptest! {
        #[test]
        fn prop_default_value_preserves_variable_name(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::DefaultValue {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must contain :- operator
            prop_assert!(
                purified.contains(":-"),
                "Purified output must contain :- operator, got: {}",
                purified
            );
        }
    }

    // Property: Default value expansion is deterministic
    proptest! {
        #[test]
        fn prop_default_value_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
            index: None,
                    value: BashExpr::DefaultValue {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Default value expansion must be deterministic"
            );
        }
    }

    // Property: Nested default values are handled correctly
    proptest! {
        #[test]
        fn prop_nested_default_values(
            var1 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            var2 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            // ${VAR1:-${VAR2:-default}}
            let nested_default = BashExpr::DefaultValue {
                variable: var1.clone(),
                default: Box::new(BashExpr::DefaultValue {
                    variable: var2.clone(),
                    default: Box::new(BashExpr::Literal(default_val.clone())),
                }),
            };

            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![nested_default],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain both variable names
            prop_assert!(
                purified.contains(&var1),
                "Purified output must contain first variable '{}', got: {}",
                var1,
                purified
            );
            prop_assert!(
                purified.contains(&var2),
                "Purified output must contain second variable '{}', got: {}",
                var2,
                purified
            );

            // PROPERTY: Must contain default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must have two :- operators (for nesting)
            let count = purified.matches(":-").count();
            prop_assert!(
                count == 2,
                "Nested default should have 2 :- operators, got {} in: {}",
                count,
                purified
            );
        }
    }

    // Property: Assign default expansion preserves variable name
    proptest! {
        #[test]
        fn prop_assign_default_preserves_variable_name(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::AssignDefault {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must contain := operator (not :-)
            prop_assert!(
                purified.contains(":="),
                "Purified output must contain := operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT contain :- operator
            prop_assert!(
                !purified.contains(":-"),
                "Purified output must not contain :- operator (should be :=), got: {}",
                purified
            );
        }
    }

    // Property: Assign default expansion is deterministic
    proptest! {
        #[test]
        fn prop_assign_default_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
            index: None,
                    value: BashExpr::AssignDefault {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Assign default expansion must be deterministic"
            );
        }
    }

    // Property: Nested assign defaults are handled correctly
    proptest! {
        #[test]
        fn prop_nested_assign_defaults(
            var1 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            var2 in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            // ${VAR1:=${VAR2:=default}}
            let nested_assign = BashExpr::AssignDefault {
                variable: var1.clone(),
                default: Box::new(BashExpr::AssignDefault {
                    variable: var2.clone(),
                    default: Box::new(BashExpr::Literal(default_val.clone())),
                }),
            };

            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![nested_assign],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain both variable names
            prop_assert!(
                purified.contains(&var1),
                "Purified output must contain first variable '{}', got: {}",
                var1,
                purified
            );
            prop_assert!(
                purified.contains(&var2),
                "Purified output must contain second variable '{}', got: {}",
                var2,
                purified
            );

            // PROPERTY: Must contain default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must have two := operators (for nesting)
            let count = purified.matches(":=").count();
            prop_assert!(
                count == 2,
                "Nested assign default should have 2 := operators, got {} in: {}",
                count,
                purified
            );
        }
    }
}
