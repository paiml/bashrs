//! Coverage tests for ir/expr.rs — bitwise and shift binary operations.
//!
//! These operations are tested through the public `from_ast()` API.
//! Targets: BitAnd, BitOr, BitXor, Shl, Shr branches in convert_binary_to_value.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::ast::restricted::{BinaryOp, Literal};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};

/// Helper: wrap a single let statement in a main function and convert to IR
fn convert_let_stmt(name: &str, value: Expr) -> ShellIR {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Let {
                name: name.to_string(),
                value,
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };
    from_ast(&ast).expect("IR conversion should succeed")
}

/// Helper: extract the ShellValue from a single Let in a Sequence
fn extract_let_value(ir: &ShellIR) -> &ShellValue {
    match ir {
        ShellIR::Sequence(stmts) => match &stmts[0] {
            ShellIR::Let { value, .. } => value,
            other => panic!("Expected Let, got {:?}", other),
        },
        other => panic!("Expected Sequence, got {:?}", other),
    }
}

// ===== Bitwise AND =====

#[test]
fn test_binary_bitand() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::BitAnd,
            left: Box::new(Expr::Literal(Literal::U32(0xFF))),
            right: Box::new(Expr::Literal(Literal::U32(0x0F))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::BitAnd,
            ..
        }
    ));
}

// ===== Bitwise OR =====

#[test]
fn test_binary_bitor() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::BitOr,
            left: Box::new(Expr::Literal(Literal::U32(0xF0))),
            right: Box::new(Expr::Literal(Literal::U32(0x0F))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::BitOr,
            ..
        }
    ));
}

// ===== Bitwise XOR =====

#[test]
fn test_binary_bitxor() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::BitXor,
            left: Box::new(Expr::Literal(Literal::U32(0xFF))),
            right: Box::new(Expr::Literal(Literal::U32(0xF0))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::BitXor,
            ..
        }
    ));
}

// ===== Shift Left =====

#[test]
fn test_binary_shl() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Shl,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Literal(Literal::U32(4))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Shl,
            ..
        }
    ));
}

// ===== Shift Right =====

#[test]
fn test_binary_shr() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Shr,
            left: Box::new(Expr::Literal(Literal::U32(256))),
            right: Box::new(Expr::Literal(Literal::U32(4))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Shr,
            ..
        }
    ));
}

// ===== Bitwise ops with variable operands =====

#[test]
fn test_binary_bitand_with_variables() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::BitAnd,
            left: Box::new(Expr::Variable("flags".to_string())),
            right: Box::new(Expr::Literal(Literal::U32(0x01))),
        },
    );
    match extract_let_value(&ir) {
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::BitAnd,
            left,
            right,
        } => {
            assert!(matches!(**left, ShellValue::Variable(ref v) if v == "flags"));
            assert!(matches!(**right, ShellValue::String(ref s) if s == "1"));
        }
        other => panic!("Expected Arithmetic(BitAnd), got {:?}", other),
    }
}

#[test]
fn test_binary_shl_with_expressions() {
    let ir = convert_let_stmt(
        "mask",
        Expr::Binary {
            op: BinaryOp::Shl,
            left: Box::new(Expr::Literal(Literal::U32(1))),
            right: Box::new(Expr::Variable("shift_amount".to_string())),
        },
    );
    match extract_let_value(&ir) {
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Shl,
            left,
            right,
        } => {
            assert!(matches!(**left, ShellValue::String(ref s) if s == "1"));
            assert!(matches!(**right, ShellValue::Variable(ref v) if v == "shift_amount"));
        }
        other => panic!("Expected Arithmetic(Shl), got {:?}", other),
    }
}

// ===== Compound bitwise expressions =====

#[test]
fn test_binary_compound_bitwise_and_or() {
    // (a & 0xFF) | 0x100
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::BitOr,
            left: Box::new(Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Literal(Literal::U32(0xFF))),
            }),
            right: Box::new(Expr::Literal(Literal::U32(0x100))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::BitOr,
            ..
        }
    ));
}

#[test]
fn test_binary_xor_with_same_values() {
    // x ^ x (should be 0 but IR doesn't fold)
    let ir = convert_let_stmt(
        "zero",
        Expr::Binary {
            op: BinaryOp::BitXor,
            left: Box::new(Expr::Variable("x".to_string())),
            right: Box::new(Expr::Variable("x".to_string())),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::BitXor,
            ..
        }
    ));
}

#[test]
fn test_binary_shr_with_large_shift() {
    let ir = convert_let_stmt(
        "r",
        Expr::Binary {
            op: BinaryOp::Shr,
            left: Box::new(Expr::Literal(Literal::U32(0xFFFFFFFF))),
            right: Box::new(Expr::Literal(Literal::U32(16))),
        },
    );
    assert!(matches!(
        extract_let_value(&ir),
        ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Shr,
            ..
        }
    ));
}
