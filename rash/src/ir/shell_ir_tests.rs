use super::*;

#[test]
fn test_shell_ir_noop_is_pure() {
    assert!(ShellIR::Noop.is_pure());
}

#[test]
fn test_shell_ir_exit_effects() {
    let exit = ShellIR::Exit {
        code: 0,
        message: None,
    };
    assert!(exit.effects().is_pure());
}

#[test]
fn test_shell_ir_echo_effects() {
    let echo = ShellIR::Echo {
        value: ShellValue::String("hello".to_string()),
    };
    assert!(echo.effects().is_pure());
}

#[test]
fn test_shell_ir_break_continue_effects() {
    assert!(ShellIR::Break.effects().is_pure());
    assert!(ShellIR::Continue.effects().is_pure());
}

#[test]
fn test_shell_ir_sequence_effects() {
    let seq = ShellIR::Sequence(vec![ShellIR::Noop, ShellIR::Noop]);
    assert!(seq.effects().is_pure());
}

#[test]
fn test_shell_ir_if_effects() {
    let if_ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: Some(Box::new(ShellIR::Noop)),
    };
    assert!(if_ir.effects().is_pure());
}

#[test]
fn test_shell_ir_if_no_else() {
    let if_ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    assert!(if_ir.effects().is_pure());
}

#[test]
fn test_shell_ir_function_effects() {
    let func = ShellIR::Function {
        name: "test".to_string(),
        params: vec!["arg".to_string()],
        body: Box::new(ShellIR::Noop),
    };
    assert!(func.effects().is_pure());
}

#[test]
fn test_shell_ir_for_effects() {
    let for_ir = ShellIR::For {
        var: "i".to_string(),
        start: ShellValue::String("1".to_string()),
        end: ShellValue::String("10".to_string()),
        body: Box::new(ShellIR::Noop),
    };
    assert!(for_ir.effects().is_pure());
}

#[test]
fn test_shell_ir_while_effects() {
    let while_ir = ShellIR::While {
        condition: ShellValue::Bool(true),
        body: Box::new(ShellIR::Noop),
    };
    assert!(while_ir.effects().is_pure());
}

#[test]
fn test_shell_ir_case_effects() {
    let case_ir = ShellIR::Case {
        scrutinee: ShellValue::String("x".to_string()),
        arms: vec![CaseArm {
            pattern: CasePattern::Wildcard,
            guard: None,
            body: Box::new(ShellIR::Noop),
        }],
    };
    assert!(case_ir.effects().is_pure());
}

#[test]
fn test_shell_ir_let_with_pure_effects() {
    let let_ir = ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::String("hello".to_string()),
        effects: EffectSet::pure(),
    };
    assert!(let_ir.is_pure());
}

#[test]
fn test_shell_ir_exec_with_pure_effects() {
    let exec_ir = ShellIR::Exec {
        cmd: Command::new("echo"),
        effects: EffectSet::pure(),
    };
    assert!(exec_ir.is_pure());
}

// ===== Command tests =====

#[test]
fn test_command_new() {
    let cmd = Command::new("ls");
    assert_eq!(cmd.program, "ls");
    assert!(cmd.args.is_empty());
}

#[test]
fn test_command_arg() {
    let cmd = Command::new("echo").arg(ShellValue::String("hello".to_string()));
    assert_eq!(cmd.args.len(), 1);
}

#[test]
fn test_command_args() {
    let cmd = Command::new("cp").args(vec![
        ShellValue::String("src".to_string()),
        ShellValue::String("dst".to_string()),
    ]);
    assert_eq!(cmd.args.len(), 2);
}

#[test]
fn test_command_chained() {
    let cmd = Command::new("grep")
        .arg(ShellValue::String("-r".to_string()))
        .arg(ShellValue::String("pattern".to_string()));
    assert_eq!(cmd.args.len(), 2);
}

// ===== ShellValue tests =====

#[test]
fn test_shell_value_string_is_constant() {
    let val = ShellValue::String("hello".to_string());
    assert!(val.is_constant());
}

#[test]
fn test_shell_value_bool_is_constant() {
    assert!(ShellValue::Bool(true).is_constant());
    assert!(ShellValue::Bool(false).is_constant());
}

#[test]
fn test_shell_value_variable_not_constant() {
    let val = ShellValue::Variable("x".to_string());
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_command_subst_not_constant() {
    let val = ShellValue::CommandSubst(Command::new("date"));
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_env_var_not_constant() {
    let val = ShellValue::EnvVar {
        name: "HOME".to_string(),
        default: None,
    };
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_arg_not_constant() {
    let val = ShellValue::Arg { position: Some(1) };
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_arg_with_default_not_constant() {
    let val = ShellValue::ArgWithDefault {
        position: 1,
        default: "default".to_string(),
    };
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_arg_count_not_constant() {
    assert!(!ShellValue::ArgCount.is_constant());
}

#[test]
fn test_shell_value_exit_code_not_constant() {
    assert!(!ShellValue::ExitCode.is_constant());
}

#[test]
fn test_shell_value_concat_constant() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::String(" world".to_string()),
    ]);
    assert!(val.is_constant());
}

#[test]
fn test_shell_value_concat_not_constant() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::Variable("x".to_string()),
    ]);
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_comparison_constant() {
    let val = ShellValue::Comparison {
        op: ComparisonOp::NumEq,
        left: Box::new(ShellValue::String("1".to_string())),
        right: Box::new(ShellValue::String("1".to_string())),
    };
    assert!(val.is_constant());
}

#[test]
fn test_shell_value_comparison_not_constant() {
    let val = ShellValue::Comparison {
        op: ComparisonOp::NumEq,
        left: Box::new(ShellValue::Variable("x".to_string())),
        right: Box::new(ShellValue::String("1".to_string())),
    };
    assert!(!val.is_constant());
}

#[test]
fn test_shell_value_arithmetic_constant() {
    let val = ShellValue::Arithmetic {
        op: ArithmeticOp::Add,
        left: Box::new(ShellValue::String("1".to_string())),
        right: Box::new(ShellValue::String("2".to_string())),
    };
    assert!(val.is_constant());
}

#[test]
fn test_shell_value_logical_and_constant() {
    let val = ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Bool(true)),
        right: Box::new(ShellValue::Bool(false)),
    };
    assert!(val.is_constant());
}

#[test]
fn test_shell_value_logical_or_constant() {
    let val = ShellValue::LogicalOr {
        left: Box::new(ShellValue::Bool(true)),
        right: Box::new(ShellValue::Bool(false)),
    };
    assert!(val.is_constant());
}

#[test]
fn test_shell_value_logical_not_constant() {
    let val = ShellValue::LogicalNot {
        operand: Box::new(ShellValue::Bool(true)),
    };
    assert!(val.is_constant());
}

// ===== as_constant_string tests =====

#[test]
fn test_as_constant_string_string() {
    let val = ShellValue::String("hello".to_string());
    assert_eq!(val.as_constant_string(), Some("hello".to_string()));
}

#[test]
fn test_as_constant_string_bool_true() {
    let val = ShellValue::Bool(true);
    assert_eq!(val.as_constant_string(), Some("true".to_string()));
}

#[test]
fn test_as_constant_string_bool_false() {
    let val = ShellValue::Bool(false);
    assert_eq!(val.as_constant_string(), Some("false".to_string()));
}

#[test]
fn test_as_constant_string_concat() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::String(" world".to_string()),
    ]);
    assert_eq!(val.as_constant_string(), Some("hello world".to_string()));
}

#[test]
fn test_as_constant_string_concat_with_variable() {
    let val = ShellValue::Concat(vec![
        ShellValue::String("hello".to_string()),
        ShellValue::Variable("x".to_string()),
    ]);
    assert_eq!(val.as_constant_string(), None);
}

#[test]
fn test_as_constant_string_variable() {
    let val = ShellValue::Variable("x".to_string());
    assert_eq!(val.as_constant_string(), None);
}

// ===== ShellExpression tests =====

#[test]
fn test_shell_expression_string_quoted() {
    let expr = ShellExpression::String("\"hello\"".to_string());
    assert!(expr.is_quoted());
}

#[test]
fn test_shell_expression_string_not_quoted() {
    let expr = ShellExpression::String("hello".to_string());
    assert!(!expr.is_quoted());
}

#[test]
fn test_shell_expression_variable_quoted() {
    let expr = ShellExpression::Variable("x".to_string(), true);
    assert!(expr.is_quoted());
}

#[test]
fn test_shell_expression_variable_not_quoted() {
    let expr = ShellExpression::Variable("x".to_string(), false);
    assert!(!expr.is_quoted());
}

#[test]
fn test_shell_expression_command_not_quoted() {
    let expr = ShellExpression::Command("date".to_string());
    assert!(!expr.is_quoted());
}

#[test]
fn test_shell_expression_arithmetic_is_quoted() {
    let expr = ShellExpression::Arithmetic("1 + 2".to_string());
    assert!(expr.is_quoted());
}

// ===== ComparisonOp tests =====

#[test]
fn test_comparison_op_eq() {
    assert_eq!(ComparisonOp::NumEq, ComparisonOp::NumEq);
    assert_ne!(ComparisonOp::NumEq, ComparisonOp::NumNe);
}

#[test]
fn test_comparison_op_clone() {
    let ops = [
        ComparisonOp::NumEq,
        ComparisonOp::NumNe,
        ComparisonOp::Gt,
        ComparisonOp::Ge,
        ComparisonOp::Lt,
        ComparisonOp::Le,
        ComparisonOp::StrEq,
        ComparisonOp::StrNe,
    ];
    for op in ops {
        let _ = op.clone();
    }
}

// ===== ArithmeticOp tests =====

#[test]
fn test_arithmetic_op_eq() {
    assert_eq!(ArithmeticOp::Add, ArithmeticOp::Add);
    assert_ne!(ArithmeticOp::Add, ArithmeticOp::Sub);
}

#[test]
fn test_arithmetic_op_clone() {
    let ops = [
        ArithmeticOp::Add,
        ArithmeticOp::Sub,
        ArithmeticOp::Mul,
        ArithmeticOp::Div,
        ArithmeticOp::Mod,
    ];
    for op in ops {
        let _ = op.clone();
    }
}

// ===== CasePattern tests =====

#[test]
fn test_case_pattern_literal() {
    let pattern = CasePattern::Literal("hello".to_string());
    let cloned = pattern.clone();
    matches!(cloned, CasePattern::Literal(_));
}

#[test]
fn test_case_pattern_wildcard() {
    let pattern = CasePattern::Wildcard;
    let cloned = pattern.clone();
    matches!(cloned, CasePattern::Wildcard);
}

// ===== CaseArm tests =====

#[test]
fn test_case_arm_clone() {
    let arm = CaseArm {
        pattern: CasePattern::Wildcard,
        guard: Some(ShellValue::Bool(true)),
        body: Box::new(ShellIR::Noop),
    };
    let cloned = arm.clone();
    assert!(cloned.guard.is_some());
}
