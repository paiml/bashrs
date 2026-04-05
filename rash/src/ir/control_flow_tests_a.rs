#![allow(clippy::expect_used)]
// Control flow IR generation and emission tests
// Testing string comparisons, logical operators, and NOT operator
// Following TDD: Write failing tests first, then fix implementation

use crate::ast::restricted::*;
use crate::emitter::PosixEmitter;
use crate::ir::from_ast;
use crate::models::Config;

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
    assert_eq!(
        echo_count, 1,
        "Only the last statement should be wrapped in Echo. Found {} Echo nodes. Got:\n{}",
        echo_count, ir_str
    );
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
    let emitter = PosixEmitter::new();
    let shell_code = emitter.emit(&ir).expect("Should emit shell code");

    // Verify the range end is 2 (3 - 1), not 3 or 4
    println!("Exclusive range test generated:\n{}", shell_code);
    assert!(
        shell_code.contains("seq 0 2") || shell_code.contains("0 2"),
        "Exclusive range 0..3 should generate 'seq 0 2' (end adjusted to 2). Got:\n{}",
        shell_code
    );
}
