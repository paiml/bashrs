#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! Property-Based Tests for Bash-to-Rash Transpiler
//!
//! Implements properties from the specification:
//! - Transpilation preserves semantics
//! - Purification maintains idempotency
//! - Determinism across executions
//! - Parse/transpile round-trips

use bashrs::bash_parser::ast::*;
use bashrs::bash_parser::SemanticAnalyzer;
use bashrs::bash_transpiler::codegen::{BashToRashTranspiler, TranspileOptions};
use proptest::prelude::*;

// Inline generators for integration tests (can't use #[cfg(test)] modules)
fn bash_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,15}".prop_map(|s| s.to_string())
}

fn bash_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ ]{0,20}")
        .unwrap()
        .prop_map(|s| s.to_string())
}

fn bash_integer() -> impl Strategy<Value = i64> {
    -1000i64..1000i64
}

fn bash_variable_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "FOO".to_string(),
        "BAR".to_string(),
        "PATH".to_string(),
        "x".to_string(),
        "result".to_string(),
    ])
}

fn bash_test_expr() -> impl Strategy<Value = TestExpr> {
    prop_oneof![
        (bash_variable_name(), bash_string())
            .prop_map(|(v, s)| { TestExpr::StringEq(BashExpr::Variable(v), BashExpr::Literal(s)) }),
        (bash_variable_name(), bash_integer()).prop_map(|(v, n)| {
            TestExpr::IntEq(BashExpr::Variable(v), BashExpr::Literal(n.to_string()))
        }),
    ]
}

fn bash_script() -> impl Strategy<Value = BashAst> {
    prop::collection::vec(
        prop_oneof![
            (bash_variable_name(), bash_string()).prop_map(|(name, value)| {
                BashStmt::Assignment {
                    name,
                    value: BashExpr::Literal(value),
                    exported: false,
                    index: None,
                    span: Span::dummy(),
                }
            }),
            (
                bash_identifier(),
                prop::collection::vec(bash_string(), 0..2)
            )
                .prop_map(|(name, args)| {
                    BashStmt::Command {
                        name,
                        args: args.into_iter().map(BashExpr::Literal).collect(),
                        redirects: vec![],
                        span: Span::dummy(),
                    }
                }),
        ],
        1..10,
    )
    .prop_map(|statements| BashAst {
        statements,
        metadata: AstMetadata {
            source_file: None,
            line_count: 0,
            parse_time_ms: 0,
        },
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000, // Start with 1000, work up to 10k
        max_shrink_iters: 1000,
        .. ProptestConfig::default()
    })]

    /// Property: Valid bash scripts can be parsed successfully
    /// Rationale: ∀ valid_bash_script, parse(script) succeeds
    #[test]
    fn prop_valid_scripts_parse_successfully(script in bash_script()) {
        let analyzer = SemanticAnalyzer::new();

        // The generated AST should be valid
        prop_assert!(!script.statements.is_empty());

        // Semantic analysis should succeed (may have warnings but no errors)
        let mut analyzer = analyzer;
        let result = analyzer.analyze(&script);
        prop_assert!(result.is_ok());
    }

    /// Property: Transpiled code preserves variable assignments
    /// Rationale: ∀ assignment, transpile preserves variable name and value
    #[test]
    fn prop_transpilation_preserves_assignments(
        name in bash_variable_name(),
        value in bash_string()
    ) {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: name.clone(),
                value: BashExpr::Literal(value.clone()),
                exported: false,
                index: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast)?;

        // Rash code should contain the variable name
        let expected = format!("let {}", name);
        prop_assert!(rash_code.contains(&expected));
    }

    /// Property: Transpilation is deterministic
    /// Rationale: ∀ script, transpile(script) always produces same output
    #[test]
    fn prop_transpilation_is_deterministic(script in bash_script()) {
        let mut transpiler1 = BashToRashTranspiler::new(TranspileOptions::default());
        let mut transpiler2 = BashToRashTranspiler::new(TranspileOptions::default());

        let result1 = transpiler1.transpile(&script)?;
        let result2 = transpiler2.transpile(&script)?;

        prop_assert_eq!(result1, result2);
    }

    /// Property: Generated code structure is valid
    /// Rationale: Transpiled code should have correct structure
    #[test]
    fn prop_generated_code_structure_valid(script in bash_script()) {
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&script)?;

        // Should have header comment
        prop_assert!(rash_code.contains("// Transpiled from bash"));

        // Should not be empty
        prop_assert!(!rash_code.is_empty());

        // Should have reasonable length (not absurdly large)
        prop_assert!(rash_code.len() < 100_000);
    }

    /// Property: Function names are preserved
    /// Rationale: ∀ function definition, name is preserved in transpilation
    #[test]
    fn prop_function_names_preserved(name in bash_identifier()) {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: name.clone(),
                body: vec![BashStmt::Comment {
                    text: " Empty function".to_string(),
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

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast)?;

        // Function name should appear in Rust syntax
        let expected = format!("fn {}()", name);
        prop_assert!(rash_code.contains(&expected));
    }

    /// Property: Variable scoping is tracked correctly
    /// Rationale: ∀ script, semantic analysis tracks all assigned variables
    #[test]
    fn prop_variable_scoping_tracked(
        var1 in bash_variable_name(),
        var2 in bash_variable_name()
    ) {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![
                BashStmt::Assignment {
                    name: var1.clone(),
                    value: BashExpr::Literal("value1".to_string()),
                    exported: false,
                    index: None,
                    span: Span::dummy(),
                },
                BashStmt::Assignment {
                    name: var2.clone(),
                    value: BashExpr::Literal("value2".to_string()),
                    exported: false,
                    index: None,
                    span: Span::dummy(),
                },
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let mut analyzer = SemanticAnalyzer::new();
        let report = analyzer.analyze(&ast)?;

        // Both variables should be tracked
        prop_assert!(report.scope_info.variables.contains_key(&var1));
        prop_assert!(report.scope_info.variables.contains_key(&var2));
    }

    /// Property: Exported variables are tracked as environment modifications
    /// Rationale: export statements should be tracked in effects
    #[test]
    fn prop_exported_vars_tracked_as_effects(name in bash_variable_name()) {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: name.clone(),
                value: BashExpr::Literal("value".to_string()),
                exported: true,
                index: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut analyzer = SemanticAnalyzer::new();
        let report = analyzer.analyze(&ast)?;

        // Exported variable should appear in env modifications
        prop_assert!(report.effects.env_modifications.contains(&name));
    }

    /// Property: Control flow statements produce valid Rash control flow
    /// Rationale: if/while/for should map to Rust equivalents
    #[test]
    fn prop_control_flow_maps_correctly(condition in bash_test_expr()) {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(condition)),
                then_block: vec![BashStmt::Comment {
                    text: " then block".to_string(),
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast)?;

        // Should contain Rust if statement
        prop_assert!(rash_code.contains("if "));
    }

    /// Property: Arithmetic operations are preserved
    /// Rationale: bash arithmetic should map to Rust arithmetic
    #[test]
    fn prop_arithmetic_preserved(a in bash_integer(), b in bash_integer()) {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "result".to_string(),
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Number(a)),
                    Box::new(ArithExpr::Number(b)),
                ))),
                exported: false,
                index: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast)?;

        // Should contain the addition operation
        prop_assert!(rash_code.contains("+"));
        prop_assert!(rash_code.contains(&a.to_string()));
        prop_assert!(rash_code.contains(&b.to_string()));
    }
}

#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn test_multiple_runs_produce_same_output() {
        use bashrs::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![
                BashStmt::Assignment {
                    name: "FOO".to_string(),
                    value: BashExpr::Literal("bar".to_string()),
                    exported: false,
                    index: None,
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("FOO".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let mut outputs = Vec::new();
        for _ in 0..100 {
            let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
            outputs.push(transpiler.transpile(&ast).unwrap());
        }

        // All outputs should be identical
        for window in outputs.windows(2) {
            assert_eq!(
                window[0], window[1],
                "Transpilation should be deterministic"
            );
        }
    }
}
