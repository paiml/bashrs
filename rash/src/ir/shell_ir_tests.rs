//! Tests for shell_ir.rs - improving coverage from 70% to 85%+

use super::effects::EffectSet;
use super::shell_ir::*;

// ============================================================================
// ShellIR Tests
// ============================================================================

#[test]
fn test_shell_ir_let_effects() {
    let ir = ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::String("test".to_string()),
        effects: EffectSet::pure(),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_if_effects() {
    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: Some(Box::new(ShellIR::Noop)),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_if_no_else() {
    let ir = ShellIR::If {
        test: ShellValue::Bool(false),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_sequence() {
    let ir = ShellIR::Sequence(vec![
        ShellIR::Noop,
        ShellIR::Echo {
            value: ShellValue::String("test".to_string()),
        },
    ]);

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_exit() {
    let ir = ShellIR::Exit {
        code: 0,
        message: Some("done".to_string()),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_exit_no_message() {
    let ir = ShellIR::Exit {
        code: 1,
        message: None,
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_function() {
    let ir = ShellIR::Function {
        name: "foo".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        body: Box::new(ShellIR::Noop),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_echo() {
    let ir = ShellIR::Echo {
        value: ShellValue::Variable("result".to_string()),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_for_loop() {
    let ir = ShellIR::For {
        var: "i".to_string(),
        start: ShellValue::String("0".to_string()),
        end: ShellValue::String("10".to_string()),
        body: Box::new(ShellIR::Noop),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_while_loop() {
    let ir = ShellIR::While {
        condition: ShellValue::Bool(true),
        body: Box::new(ShellIR::Break),
    };

    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_break() {
    let ir = ShellIR::Break;
    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_continue() {
    let ir = ShellIR::Continue;
    assert!(ir.is_pure());
}

#[test]
fn test_shell_ir_case() {
    let arms = vec![
        CaseArm {
            pattern: CasePattern::Literal("1".to_string()),
            guard: None,
            body: Box::new(ShellIR::Noop),
        },
        CaseArm {
            pattern: CasePattern::Wildcard,
            guard: Some(ShellValue::Bool(true)),
            body: Box::new(ShellIR::Noop),
        },
    ];

    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("x".to_string()),
        arms,
    };

    assert!(ir.is_pure());
}

// ============================================================================
// ShellValue Tests
// ============================================================================

#[test]
fn test_shell_value_string_constant() {
    let val = ShellValue::String("hello".to_string());
    assert!(val.is_constant());
    assert_eq!(val.as_constant_string(), Some("hello".to_string()));
}

#[test]
fn test_shell_value_bool_true() {
    let val = ShellValue::Bool(true);
    assert!(val.is_constant());
    assert_eq!(val.as_constant_string(), Some("true".to_string()));
}

#[test]
fn test_shell_value_bool_false() {
    let val = ShellValue::Bool(false);
    assert!(val.is_constant());
    assert_eq!(val.as_constant_string(), Some("false".to_string()));
}

#[test]
fn test_shell_value_variable_not_constant() {
    let val = ShellValue::Variable("x".to_string());
    assert!(!val.is_constant());
    assert_eq!(val.as_constant_string(), None);
}

#[test]
fn test_shell_value_command_subst_not_constant() {
    let val =
        ShellValue::CommandSubst(Command::new("echo").arg(ShellValue::String("test".to_string())));
    assert!(!val.is_constant());
    assert_eq!(val.as_constant_string(), None);
}

#[test]
fn test_shell_value_concat_constant() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::String(" ".to_string()),
        ShellValue::String("world".to_string()),
    ]);

    assert!(val.is_constant());
    assert_eq!(val.as_constant_string(), Some("hello world".to_string()));
}

#[test]
fn test_shell_value_concat_with_variable() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello ".to_string()),
        ShellValue::Variable("name".to_string()),
    ]);

    assert!(!val.is_constant());
    assert_eq!(val.as_constant_string(), None);
}

#[test]
fn test_shell_value_comparison() {
    let val = ShellValue::Comparison {
        op: ComparisonOp::Gt,
        left: Box::new(ShellValue::String("5".to_string())),
        right: Box::new(ShellValue::String("3".to_string())),
    };

    assert!(val.is_constant());
}

#[test]
fn test_shell_value_comparison_with_variable() {
    let val = ShellValue::Comparison {
        op: ComparisonOp::NumEq,
        left: Box::new(ShellValue::Variable("x".to_string())),
        right: Box::new(ShellValue::String("10".to_string())),
    };

    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_arithmetic() {
    let val = ShellValue::Arithmetic {
        op: ArithmeticOp::Add,
        left: Box::new(ShellValue::String("10".to_string())),
        right: Box::new(ShellValue::String("5".to_string())),
    };

    assert!(val.is_constant());
}

#[test]
fn test_shell_value_arithmetic_with_variable() {
    let val = ShellValue::Arithmetic {
        op: ArithmeticOp::Sub,
        left: Box::new(ShellValue::Variable("x".to_string())),
        right: Box::new(ShellValue::String("1".to_string())),
    };

    assert!(!val.is_constant());
}

// ============================================================================
// Command Tests
// ============================================================================

#[test]
fn test_command_new() {
    let cmd = Command::new("echo");
    assert_eq!(cmd.program, "echo");
    assert!(cmd.args.is_empty());
}

#[test]
fn test_command_with_single_arg() {
    let cmd = Command::new("echo").arg(ShellValue::String("hello".to_string()));
    assert_eq!(cmd.program, "echo");
    assert_eq!(cmd.args.len(), 1);
}

#[test]
fn test_command_with_multiple_args() {
    let cmd = Command::new("printf")
        .arg(ShellValue::String("%s\n".to_string()))
        .arg(ShellValue::String("test".to_string()));

    assert_eq!(cmd.program, "printf");
    assert_eq!(cmd.args.len(), 2);
}

#[test]
fn test_command_args_method() {
    let cmd = Command::new("seq").args(vec![
        ShellValue::String("1".to_string()),
        ShellValue::String("10".to_string()),
    ]);

    assert_eq!(cmd.program, "seq");
    assert_eq!(cmd.args.len(), 2);
}

// ============================================================================
// ComparisonOp Tests
// ============================================================================

#[test]
fn test_comparison_ops_equality() {
    assert_eq!(ComparisonOp::NumEq, ComparisonOp::NumEq);
    assert_ne!(ComparisonOp::NumEq, ComparisonOp::NumNe);
}

#[test]
fn test_comparison_ops_variants() {
    let _eq = ComparisonOp::NumEq;
    let _ne = ComparisonOp::NumNe;
    let _gt = ComparisonOp::Gt;
    let _ge = ComparisonOp::Ge;
    let _lt = ComparisonOp::Lt;
    let _le = ComparisonOp::Le;
}

// ============================================================================
// ArithmeticOp Tests
// ============================================================================

#[test]
fn test_arithmetic_ops_equality() {
    assert_eq!(ArithmeticOp::Add, ArithmeticOp::Add);
    assert_ne!(ArithmeticOp::Add, ArithmeticOp::Sub);
}

#[test]
fn test_arithmetic_ops_variants() {
    let _add = ArithmeticOp::Add;
    let _sub = ArithmeticOp::Sub;
    let _mul = ArithmeticOp::Mul;
    let _div = ArithmeticOp::Div;
    let _mod = ArithmeticOp::Mod;
}

// ============================================================================
// CasePattern Tests
// ============================================================================

#[test]
fn test_case_pattern_literal() {
    let pattern = CasePattern::Literal("value".to_string());
    match pattern {
        CasePattern::Literal(s) => assert_eq!(s, "value"),
        _ => panic!("Expected Literal pattern"),
    }
}

#[test]
fn test_case_pattern_wildcard() {
    let pattern = CasePattern::Wildcard;
    match pattern {
        CasePattern::Wildcard => {}
        _ => panic!("Expected Wildcard pattern"),
    }
}

// ============================================================================
// ShellExpression Tests
// ============================================================================

#[test]
fn test_shell_expression_string_quoted() {
    let expr = ShellExpression::String("\"hello\"".to_string());
    assert!(expr.is_quoted());
}

#[test]
fn test_shell_expression_string_unquoted() {
    let expr = ShellExpression::String("hello".to_string());
    assert!(!expr.is_quoted());
}

#[test]
fn test_shell_expression_variable_quoted() {
    let expr = ShellExpression::Variable("x".to_string(), true);
    assert!(expr.is_quoted());
}

#[test]
fn test_shell_expression_variable_unquoted() {
    let expr = ShellExpression::Variable("x".to_string(), false);
    assert!(!expr.is_quoted());
}

#[test]
fn test_shell_expression_command_not_quoted() {
    let expr = ShellExpression::Command("whoami".to_string());
    assert!(!expr.is_quoted());
}

#[test]
fn test_shell_expression_arithmetic_quoted() {
    let expr = ShellExpression::Arithmetic("1 + 2".to_string());
    assert!(expr.is_quoted());
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_shell_ir_serialization() {
    let ir = ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::String("test".to_string()),
        effects: EffectSet::pure(),
    };

    let json = serde_json::to_string(&ir).expect("Failed to serialize");
    let _deserialized: ShellIR = serde_json::from_str(&json).expect("Failed to deserialize");
}

#[test]
fn test_shell_value_serialization() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::Variable("world".to_string()),
    ]);

    let json = serde_json::to_string(&val).expect("Failed to serialize");
    let _deserialized: ShellValue = serde_json::from_str(&json).expect("Failed to deserialize");
}

#[test]
fn test_case_arm_serialization() {
    let arm = CaseArm {
        pattern: CasePattern::Literal("test".to_string()),
        guard: Some(ShellValue::Bool(true)),
        body: Box::new(ShellIR::Noop),
    };

    let json = serde_json::to_string(&arm).expect("Failed to serialize");
    let _deserialized: CaseArm = serde_json::from_str(&json).expect("Failed to deserialize");
}
