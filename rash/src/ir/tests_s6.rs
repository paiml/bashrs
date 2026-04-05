#![allow(clippy::expect_used)]
use super::*;
use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use proptest::prelude::*;
use rstest::*;

// Helper: wrap a single let statement in a main function and convert to IR

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
                    declaration: true,
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
            } => has_case(then_branch) || else_branch.as_ref().is_some_and(|e| has_case(e)),
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
